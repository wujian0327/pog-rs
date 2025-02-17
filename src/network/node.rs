use crate::blockchain::block::{Block, BlockError, Body};
use crate::blockchain::blockchain::{BlockChainError, Blockchain};
use crate::blockchain::path::{AggregatedSignedPaths, TransactionPaths};
use crate::blockchain::transaction::Transaction;
use crate::network::message::{Message, MessageType};
use crate::network::validator::{RandaoSeed, Validator};
use crate::network::world_state::SlotManager;
use crate::wallet::Wallet;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

///通过Tokio的mpsc通道与其他节点交互
///负责出块、发送交易、发送seed
pub(crate) struct Node {
    pub index: u32,
    pub epoch: u64,
    pub slot: u64,
    pub wallet: Wallet,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
    pub neighbors: Vec<Neighbor>,
    pub world_state_sender: Sender<Message>,
    pub transaction_paths_cache: Arc<RwLock<Vec<TransactionPaths>>>,
}

#[derive(Clone)]
pub struct Neighbor {
    index: u32,
    address: String,
    sender: Sender<Message>,
}

impl Node {
    pub fn new(
        index: u32,
        epoch: u64,
        slot: u64,
        blockchain: Blockchain,
        world_state_sender: Sender<Message>,
    ) -> Self {
        let wallet = Wallet::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        Node {
            index,
            epoch,
            slot,
            wallet,
            blockchain: Arc::new(RwLock::new(blockchain)),
            sender,
            receiver,
            transaction_paths_cache: Arc::new(RwLock::new(Vec::new())),
            neighbors: Vec::new(),
            world_state_sender,
        }
    }

    pub fn new_with_wallet(
        index: u32,
        epoch: u64,
        slot: u64,
        blockchain: Blockchain,
        wallet: Wallet,
        world_state_sender: Sender<Message>,
    ) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        Node {
            index,
            epoch,
            slot,
            wallet,
            blockchain: Arc::new(RwLock::new(blockchain)),
            sender,
            receiver,
            transaction_paths_cache: Arc::new(RwLock::new(Vec::new())),
            neighbors: Vec::new(),
            world_state_sender,
        }
    }

    pub async fn generate_block(&self, epoch: u64, slot: u64) -> Result<Block, BlockError> {
        let transaction_paths = {
            let mut transaction_paths = self.transaction_paths_cache.write().await;
            let transaction_paths_clone = transaction_paths.clone();
            transaction_paths.clear();
            transaction_paths_clone
        };
        let mut transactions: Vec<Transaction> = Vec::with_capacity(transaction_paths.len());
        let mut paths: Vec<AggregatedSignedPaths> = Vec::with_capacity(transaction_paths.len());
        for x in transaction_paths {
            transactions.push(x.transaction.clone());
            paths.push(x.to_aggregated_signed_paths());
        }
        let body = Body::new(transactions, paths);
        let new_block = {
            let blockchain = self.blockchain.clone().read().await.clone();

            Block::new(
                blockchain.get_lash_index() + 1,
                epoch,
                slot,
                blockchain.get_last_hash(),
                body,
                self.wallet.clone(),
            )?
        };
        {
            if let Err(e) = self
                .blockchain
                .clone()
                .write()
                .await
                .add_block(new_block.clone())
            {
                error!("Node[{}] error :{}", self.index, e);
                return Err(BlockError::InvalidBlock);
            };
        }

        Ok(new_block)
    }

    pub fn get_address(&self) -> String {
        self.wallet.address.clone()
    }

    pub fn short_address(&self) -> String {
        self.wallet.address.clone()[0..5].to_string()
    }

    pub fn short_address_with_index(&self) -> String {
        self.index.to_string() + "-" + self.short_address().as_str()
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg.msg_type {
                MessageType::SendBlock => {
                    let block = match Block::from_json(msg.data) {
                        Ok(b) => b,
                        Err(e) => {
                            error!("Node[{}] error: {}", self.index, e);
                            continue;
                        }
                    };
                    debug!(
                        "Node[{}] received msg[{}]: block hash[{}]",
                        self.index, msg.msg_type, block.header.hash
                    );
                    {
                        //添加到自己的区块链
                        let mut blockchain = self.blockchain.write().await;
                        if let Err(e) = blockchain.add_block(block.clone()) {
                            match e {
                                BlockChainError::DuplicateBlocksReceived => {
                                    debug!("Node[{}] error: {}", self.index, e);
                                }
                                _ => {
                                    error!("Node[{}] error: {}", self.index, e);
                                }
                            }
                            continue;
                        }
                        debug!("Node[{}] add block successfully", self.index);
                    }
                    {
                        //清除交易缓存
                        let tx_hashs: Vec<String> = block
                            .body
                            .transactions
                            .iter()
                            .map(|t| t.hash.to_string())
                            .collect();
                        let mut transaction_paths_cache =
                            self.transaction_paths_cache.write().await;
                        transaction_paths_cache.retain(|x| !tx_hashs.contains(&x.transaction.hash));
                    }
                    //广播到其他邻居
                    for neighbor_sender in self.neighbors.clone() {
                        if msg.from == neighbor_sender.address {
                            continue;
                        }
                        let block = block.clone();
                        debug!(
                            "Node[{}] send block to Node[{}]",
                            self.index, neighbor_sender.index
                        );
                        let self_address = self.get_address();
                        tokio::spawn(async move {
                            neighbor_sender
                                .sender
                                .send(Message::new_block_msg(block, self_address))
                                .await
                                .unwrap();
                        });
                    }
                }
                MessageType::SendTransactionPaths => {
                    let transaction_paths = match TransactionPaths::from_json(msg.data) {
                        Ok(t) => t,
                        Err(e) => {
                            error!("Node[{}] error: {}", self.index, e);
                            continue;
                        }
                    };

                    // if !transaction_paths.verify_last(self.wallet.address.clone()) {
                    //     error!("Node[{}] invalid transaction paths", self.index);
                    //     continue;
                    // }
                    {
                        let bc = self.blockchain.read().await;
                        if bc.exist_transaction(transaction_paths.transaction.hash.clone()) {
                            debug!(
                                "Node[{}] received transaction[{}] already in blockchain",
                                self.index, transaction_paths.transaction.hash
                            );
                            continue;
                        }
                    }
                    //判断交易是否已经收到了,判断交易的paths是否最短
                    {
                        let transactions_cache = self.transaction_paths_cache.read().await;
                        let mut skip = false;
                        for cache in transactions_cache.iter() {
                            if cache.transaction.hash == transaction_paths.transaction.hash
                            // && cache.paths.len() <= transaction_paths.paths.len()
                            {
                                skip = true;
                                break;
                            }
                        }
                        if skip {
                            continue;
                        }
                    }
                    debug!(
                        "Node[{}] received msg[{}]: transaction hash[{}],path[{}]",
                        self.short_address_with_index(),
                        msg.msg_type,
                        transaction_paths.transaction.hash,
                        transaction_paths.to_paths_string(),
                    );
                    //收到交易，存储
                    {
                        let mut transactions_cache = self.transaction_paths_cache.write().await;
                        //先删除，再添加
                        transactions_cache
                            .retain(|t| t.transaction.hash != transaction_paths.transaction.hash);
                        transactions_cache.push(transaction_paths.clone())
                    }
                    //并广播到邻居
                    for neighbor_sender in self.neighbors.clone() {
                        if msg.from == neighbor_sender.address {
                            continue;
                        }
                        let mut new_trans_paths = transaction_paths.clone();
                        new_trans_paths
                            .add_path(neighbor_sender.address.clone(), self.wallet.clone());
                        debug!(
                            "Node[{}] send transaction[{}] paths[{}] to Node[{}]",
                            self.short_address_with_index(),
                            new_trans_paths.transaction.hash,
                            new_trans_paths.to_paths_string(),
                            neighbor_sender.short_address_with_index()
                        );
                        let self_address = self.get_address();
                        tokio::spawn(async move {
                            neighbor_sender
                                .sender
                                .send(Message::new_transaction_paths_msg(
                                    new_trans_paths,
                                    self_address,
                                ))
                                .await
                                .unwrap();
                        });
                    }
                }

                MessageType::GenerateBlock => {
                    let last_block_time = {
                        self.blockchain
                            .read()
                            .await
                            .get_last_block()
                            .header
                            .timestamp
                    };
                    //出块
                    let block = match self.generate_block(self.epoch, self.slot).await {
                        Ok(b) => b,
                        Err(e) => {
                            error!("Node[{}] generate block failed:{}", self.index, e);
                            continue;
                        }
                    };
                    info!(
                        "Node[{}] is the miner: block hash[{}]",
                        self.index, block.header.hash
                    );
                    block.simple_print();
                    let during = block.header.timestamp - last_block_time;
                    info!(
                        "Current {:.2}TX/s",
                        block.body.transactions.len() as f64 / during as f64
                    );

                    //广播区块
                    for neighbor_sender in self.neighbors.clone() {
                        let block = block.clone();
                        let self_address = self.get_address();
                        tokio::spawn(async move {
                            neighbor_sender
                                .sender
                                .send(Message::new_block_msg(block, self_address))
                                .await
                                .unwrap();
                        });
                    }
                }
                MessageType::GenerateTransactionPaths => {
                    let to = match String::from_utf8(msg.data) {
                        Ok(to) => to,
                        Err(e) => {
                            error!(
                                "Node[{}] generate transaction paths failed:{}",
                                self.index, e
                            );
                            continue;
                        }
                    };
                    let transaction = Transaction::new(to, 0, self.wallet.clone());
                    let transaction_paths = TransactionPaths::new(transaction);
                    debug!(
                        "Node[{}] received msg[{}]: transaction hash[{}],path[{}]",
                        self.short_address_with_index(),
                        msg.msg_type,
                        transaction_paths.transaction.hash,
                        transaction_paths.to_paths_string()
                    );
                    //缓存交易
                    {
                        let mut transactions_cache = self.transaction_paths_cache.write().await;
                        transactions_cache.push(transaction_paths.clone())
                    }
                    //广播交易
                    for neighbor_sender in self.neighbors.clone() {
                        let mut new_trans_paths = transaction_paths.clone();
                        new_trans_paths
                            .add_path(neighbor_sender.address.clone(), self.wallet.clone());
                        debug!(
                            "Node[{}] send transaction[{}] paths[{}] to Node[{}]",
                            self.short_address_with_index(),
                            new_trans_paths.transaction.hash,
                            new_trans_paths.to_paths_string(),
                            neighbor_sender.short_address_with_index()
                        );
                        let self_address = self.get_address();
                        tokio::spawn(async move {
                            neighbor_sender
                                .sender
                                .send(Message::new_transaction_paths_msg(
                                    new_trans_paths,
                                    self_address,
                                ))
                                .await
                                .unwrap();
                        });
                    }
                }
                MessageType::SendRandaoSeed => {
                    let seed = RandaoSeed::generate_seed();
                    let signature = self.wallet.sign(Vec::from(seed));
                    let randao_seed = RandaoSeed {
                        address: self.wallet.address.clone(),
                        seed,
                        signature,
                    };
                    debug!(
                        "Node[{}] received msg[{}]: seed[{:?}]",
                        self.index, msg.msg_type, seed
                    );
                    self.world_state_sender
                        .send(Message::new_receive_random_seed_msg(randao_seed))
                        .await
                        .unwrap();
                }
                MessageType::BecomeValidator => {
                    debug!("Node[{}] received msg[{}]", self.index, msg.msg_type);
                    self.world_state_sender
                        .send(Message::new_receive_become_validator_msg(Validator::new(
                            self.wallet.address.clone(),
                            32,
                        )))
                        .await
                        .unwrap();
                }
                MessageType::UpdateSlot => {
                    let slot = match SlotManager::from_json(msg.data) {
                        Ok(t) => t,
                        Err(e) => {
                            error!("Node[{}] error: {}", self.index, e);
                            continue;
                        }
                    };
                    debug!("Node[{}] received msg[{}]", self.index, msg.msg_type);
                    self.slot = slot.current_slot;
                    self.epoch = slot.current_epoch;
                }
                MessageType::PrintBlockchain => {
                    info!("Node[{}] received msg[{}]", self.index, msg.msg_type);
                    self.blockchain
                        .read()
                        .await
                        .write_to_file_last_block_simple()
                        .await;
                }
                _ => {}
            }
        }
    }
}

impl Neighbor {
    pub fn new(index: u32, address: String, sender: Sender<Message>) -> Self {
        Neighbor {
            index,
            address,
            sender,
        }
    }

    pub fn short_address(&self) -> String {
        self.address.clone()[0..5].to_string()
    }

    pub fn short_address_with_index(&self) -> String {
        self.index.to_string() + "-" + self.short_address().as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::block::Body;
    use crate::blockchain::path::TransactionPaths;
    use crate::blockchain::transaction::Transaction;
    use crate::wallet::Wallet;
    use std::time::Duration;

    #[tokio::test]
    async fn test_send_block() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        let (world_sender, _) = tokio::sync::mpsc::channel(8);
        let blockchain = Blockchain::new(Block::gen_genesis_block());
        let wallet = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();
        let miner = Wallet::new();
        let transaction = Transaction::new("123".to_string(), 32, wallet.clone());
        let mut transaction_paths = TransactionPaths::new(transaction.clone());
        transaction_paths.add_path(wallet2.address.clone(), wallet);
        transaction_paths.add_path(wallet3.address.clone(), wallet2);
        transaction_paths.add_path(miner.address.clone(), wallet3);

        let body = Body::new(
            vec![transaction],
            vec![transaction_paths.to_aggregated_signed_paths()],
        );
        let block = Block::new(
            blockchain.get_lash_index() + 1,
            0,
            1,
            blockchain.get_last_hash(),
            body,
            miner,
        )
        .unwrap();

        let mut node = Node::new(0, 0, 0, blockchain, world_sender);
        let node_sender = node.sender.clone();
        let handle1 = tokio::spawn(async move {
            node.run().await;
        });

        let msg = Message::new_block_msg(block, "".to_string());
        let handle2 = tokio::spawn(async move {
            info!("send msg:{:?}", msg);
            node_sender.send(msg).await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(1)).await;

        handle1.abort();
        handle2.abort();
    }

    #[tokio::test]
    async fn test_send_transaction_and_block() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        let (world_sender, _) = tokio::sync::mpsc::channel(8);
        let blockchain = Blockchain::new(Block::gen_genesis_block());
        let wallet0 = Wallet::new();
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();
        let mut node0 = Node::new_with_wallet(
            0,
            0,
            1,
            blockchain.clone(),
            wallet0.clone(),
            world_sender.clone(),
        );
        let mut node1 = Node::new_with_wallet(
            1,
            0,
            1,
            blockchain.clone(),
            wallet1.clone(),
            world_sender.clone(),
        );
        let mut node2 = Node::new_with_wallet(
            2,
            0,
            1,
            blockchain.clone(),
            wallet2.clone(),
            world_sender.clone(),
        );
        let mut node3 = Node::new_with_wallet(
            3,
            0,
            1,
            blockchain.clone(),
            wallet3.clone(),
            world_sender.clone(),
        );

        node0.neighbors.push(Neighbor::new(
            node1.index,
            node1.wallet.address.clone(),
            node1.sender.clone(),
        ));
        node1.neighbors.push(Neighbor::new(
            node2.index,
            node2.wallet.address.clone(),
            node2.sender.clone(),
        ));
        node2.neighbors.push(Neighbor::new(
            node3.index,
            node3.wallet.address.clone(),
            node3.sender.clone(),
        ));

        node3.neighbors.push(Neighbor::new(
            node2.index,
            node2.wallet.address.clone(),
            node2.sender.clone(),
        ));

        node2.neighbors.push(Neighbor::new(
            node1.index,
            node1.wallet.address.clone(),
            node1.sender.clone(),
        ));

        node1.neighbors.push(Neighbor::new(
            node0.index,
            node0.wallet.address.clone(),
            node0.sender.clone(),
        ));
        let node0_bc = node0.blockchain.clone();
        let node0_sender = node0.sender.clone();
        let handle0 = tokio::spawn(async move {
            node0.run().await;
        });
        let node1_bc = node1.blockchain.clone();
        let handle1 = tokio::spawn(async move {
            node1.run().await;
        });
        let node2_bc = node2.blockchain.clone();
        let handle2 = tokio::spawn(async move {
            node2.run().await;
        });
        let node3_bc = node3.blockchain.clone();
        let node3_sender = node3.sender.clone();
        let handle3 = tokio::spawn(async move {
            node3.run().await;
        });

        //node0发送交易
        let transaction = Transaction::new(wallet3.address.clone(), 32, wallet0.clone());
        let transaction_paths = TransactionPaths::new(transaction);
        node0_sender
            .send(Message::new_transaction_paths_msg(
                transaction_paths,
                "".to_string(),
            ))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        node3_sender
            .send(Message::new_generate_block_msg())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

        assert_eq!(
            node0_bc.read().await.get_last_hash(),
            node1_bc.read().await.get_last_hash()
        );
        assert_eq!(
            node1_bc.read().await.get_last_hash(),
            node2_bc.read().await.get_last_hash()
        );
        assert_eq!(
            node2_bc.read().await.get_last_hash(),
            node3_bc.read().await.get_last_hash()
        );
        {
            node3_bc.read().await.simple_print_last_five_block();
        }
        handle0.abort();
        handle1.abort();
        handle2.abort();
        handle3.abort();
    }
}
