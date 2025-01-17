use crate::blockchain::block::{Block, BlockError, Body};
use crate::blockchain::blockchain::{BlockChainError, Blockchain};
use crate::blockchain::path::{Path, PathError, TransactionPaths};
use crate::blockchain::transaction::Transaction;
use crate::network::message::{Message, MessageType};
use crate::wallet::Wallet;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

///Node 主要负责与其他节点交互
///     通过Tokio的mpsc通道进行交互
struct Node {
    index: u32,
    wallet: Wallet,
    blockchain: Arc<RwLock<Blockchain>>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    neighbors: Vec<Neighbor>,
    transaction_paths_cache: RwLock<Vec<TransactionPaths>>,
}

#[derive(Clone)]
struct Neighbor {
    index: u32,
    address: String,
    sender: Sender<Message>,
}

impl Node {
    pub fn new(index: u32, blockchain: Blockchain) -> Self {
        let wallet = Wallet::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        Node {
            index,
            wallet,
            blockchain: Arc::new(RwLock::new(blockchain)),
            sender,
            receiver,
            transaction_paths_cache: RwLock::new(Vec::new()),
            neighbors: Vec::new(),
        }
    }

    pub fn new_with_wallet(index: u32, blockchain: Blockchain, wallet: Wallet) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        Node {
            index,
            wallet,
            blockchain: Arc::new(RwLock::new(blockchain)),
            sender,
            receiver,
            transaction_paths_cache: RwLock::new(Vec::new()),
            neighbors: Vec::new(),
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
        let mut paths: Vec<Vec<Path>> = Vec::with_capacity(transaction_paths.len());
        for x in transaction_paths {
            transactions.push(x.transaction.clone());
            paths.push(x.paths.clone());
        }
        let body = Body::new(transactions, paths);
        let new_block = {
            let blockchain = self.blockchain.clone().read().await.clone();

            let new_block = Block::new(
                blockchain.get_lash_index() + 1,
                epoch,
                slot,
                blockchain.get_last_hash(),
                body,
                self.wallet.clone(),
            )?;
            new_block
        };
        {
            if let Err(e) = self
                .blockchain
                .clone()
                .write()
                .await
                .add_block(new_block.clone())
            {
                println!("{:?}", e);
                return Err(BlockError::InvalidBlock);
            };
        }

        Ok(new_block)
    }
    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            println!("Node[{}] received msg type: {}", self.index, msg.msg_type);
            match msg.msg_type {
                MessageType::SEND_BLOCK => {
                    let block = match Block::from_json(msg.data) {
                        Ok(b) => b,
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    };
                    {
                        let mut blockchain = self.blockchain.write().await;
                        if let Err(e) = blockchain.add_block(block.clone()) {
                            println!("{}", e);
                        }
                        println!("Node[{}] add block successfully", self.index);
                        block.simple_print();
                    }
                    //广播到其他邻居
                    for neighbor_sender in self.neighbors.clone() {
                        let block = block.clone();
                        tokio::spawn(async move {
                            neighbor_sender
                                .sender
                                .send(Message::new_block_msg(block))
                                .await
                                .unwrap();
                        });
                    }
                }
                MessageType::SEND_TRANSACTION_PATHS => {
                    let transaction_paths = match TransactionPaths::from_json(msg.data) {
                        Ok(t) => t,
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    };
                    if !transaction_paths.verify(self.wallet.address.clone()) {
                        println!("Node[{}] invalid transaction paths", self.index);
                        continue;
                    }
                    //判断交易是否已经收到了,判断交易的paths是否最短
                    {
                        let transactions_cache = self.transaction_paths_cache.read().await;
                        let mut skip = false;
                        for cache in transactions_cache.iter() {
                            if cache.transaction.hash == transaction_paths.transaction.hash {
                                if cache.paths.len() <= transaction_paths.paths.len() {
                                    skip = true;
                                    break;
                                }
                            }
                        }
                        if skip {
                            continue;
                        }
                    }
                    //收到交易，存储
                    {
                        let mut transactions_cache = self.transaction_paths_cache.write().await;
                        //先删除，再添加
                        transactions_cache
                            .retain(|t| t.transaction.hash == transaction_paths.transaction.hash);
                        transactions_cache.push(transaction_paths.clone())
                    }
                    //并广播到邻居
                    for neighbor_sender in self.neighbors.clone() {
                        println!(
                            "Node[{}] send transaction paths to Node[{}]",
                            self.index, neighbor_sender.index
                        );
                        let mut new_trans_paths = transaction_paths.clone();
                        new_trans_paths.add_path(neighbor_sender.address, self.wallet.clone());
                        tokio::spawn(async move {
                            neighbor_sender
                                .sender
                                .send(Message::new_transaction_paths_msg(new_trans_paths))
                                .await
                                .unwrap();
                        });
                    }
                }

                MessageType::GENERATE_BLOCK => {
                    self.generate_block(0, 1).await.unwrap();
                }
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
        let body = Body::new(vec![transaction], vec![transaction_paths.paths.clone()]);
        let block = Block::new(
            blockchain.get_lash_index() + 1,
            0,
            1,
            blockchain.get_last_hash(),
            body,
            miner,
        )
        .unwrap();

        let mut node = Node::new(0, blockchain);
        let node_sender = node.sender.clone();
        let handle1 = tokio::spawn(async move {
            node.run().await;
        });

        let msg = Message::new_block_msg(block);
        let handle2 = tokio::spawn(async move {
            println!("send msg:{:?}", msg);
            node_sender.send(msg).await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(1)).await;

        handle1.abort();
        handle2.abort();
    }

    #[tokio::test]
    async fn test_send_transaction() {
        let blockchain = Blockchain::new(Block::gen_genesis_block());
        let wallet0 = Wallet::new();
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();
        let mut node0 = Node::new_with_wallet(0, blockchain.clone(), wallet0.clone());
        let mut node1 = Node::new_with_wallet(1, blockchain.clone(), wallet1.clone());
        let mut node2 = Node::new_with_wallet(2, blockchain.clone(), wallet2.clone());
        let mut node3 = Node::new_with_wallet(3, blockchain.clone(), wallet3.clone());

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

        let node0_sender = node0.sender.clone();
        let handle0 = tokio::spawn(async move {
            node0.run().await;
        });
        let handle1 = tokio::spawn(async move {
            node1.run().await;
        });
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
            .send(Message::new_transaction_paths_msg(transaction_paths))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        node3_sender
            .send(Message::new_generate_block_msg())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        {
            node3_bc.read().await.simple_print_last_five_block();
        }
        handle0.abort();
        handle1.abort();
        handle2.abort();
        handle3.abort();
    }
}
