use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::pog::Pog;
use crate::consensus::pos::Pos;
use crate::consensus::{ConsensusType, RandaoSeed, Validator};
use crate::network::message::{Message, MessageType};
use crate::tools::get_timestamp;
use crate::{consensus, tools};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::Instant;
use tokio::{task, time};

/// 全局状态，用于管理时隙、vdf投票，余额等等
/// 也可以用于与所有的节点进行通信
pub struct WorldState {
    pub consensus_type: ConsensusType,
    pub current_slot: Arc<RwLock<SlotManager>>,
    // pub slots: Vec<SlotManager>,
    pub validators: Arc<RwLock<Vec<Validator>>>,
    // sender和receiver要和WorldState解耦，独立返回
    // pub sender: Sender<Message>,
    // pub receiver: Receiver<Message>,
    // pub nodes_balance: HashMap<String, u64>,
    pub nodes_sender: HashMap<String, Sender<Message>>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub ntd: usize,
}

static SLOT_DURATION: Duration = Duration::from_secs(5);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotManager {
    pub randao_seeds: Vec<RandaoSeed>,
    pub slot_duration: Duration,
    pub current_epoch: u64,
    pub current_slot: u64,
    pub next_seed: [u8; 32],
    pub start_timestamp: u64,
}

impl WorldState {
    pub fn new(
        genesis_block: Block,
        consensus_type: ConsensusType,
        blockchain: Blockchain,
    ) -> (Self, Sender<Message>, Receiver<Message>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let nodes_sender: HashMap<String, Sender<Message>> = HashMap::new();
        (
            WorldState {
                current_slot: Arc::new(RwLock::new(SlotManager {
                    randao_seeds: vec![],
                    slot_duration: SLOT_DURATION,
                    current_epoch: 0,
                    current_slot: 0,
                    next_seed: [0; 32],
                    start_timestamp: genesis_block.header.timestamp,
                })),
                validators: Arc::new(RwLock::new(vec![])),
                consensus_type,
                nodes_sender,
                blockchain: Arc::new(RwLock::new(blockchain)),
                ntd: 0,
            },
            sender,
            receiver,
        )
    }

    pub async fn next_slot(&mut self) {
        let current_slot = self.current_slot.read().await.clone();
        //计算randao seed
        let validators = self.validators.read().await.clone();
        let next_seed = consensus::combine_seed(validators.clone(), current_slot.randao_seeds);

        if current_slot.current_slot >= 10 {
            //更新epoch
            self.next_epoch().await;
        } else {
            self.current_slot = Arc::new(RwLock::new(SlotManager {
                randao_seeds: vec![],
                slot_duration: SLOT_DURATION,
                current_epoch: current_slot.current_epoch,
                current_slot: current_slot.current_slot + 1,
                next_seed,
                start_timestamp: get_timestamp(),
            }));
        }
        let current_slot = self.get_current_slot().await;
        info!(
            "World State change slot to: epoch[{}] slot[{}] NTD[{}] seed{:?}",
            current_slot.current_epoch, current_slot.current_slot, self.ntd, next_seed
        );

        let nodes_sender: Vec<Sender<Message>> = self.nodes_sender.values().cloned().collect();

        //通知所有节点更新slot
        for sender in nodes_sender {
            if let Err(e) = sender
                .send(Message::new_update_slot_msg(current_slot.clone()))
                .await
            {
                error!("World State error: send update slot msg failed {:?}", e);
            }
        }

        //通知所有的validator可以开始新一轮的发送seed
        for v in validators.clone() {
            if let Err(e) = self.nodes_sender[&v.address]
                .send(Message::new_send_randao_seed_msg())
                .await
            {
                error!("World State error: send new randao seed msg failed {:?}", e);
            }
        }

        //获得出块节点
        let bc = self.blockchain.read().await.clone();
        let miner_validator = match self.consensus_type {
            ConsensusType::POS => Pos::select(validators.clone(), next_seed.clone(), bc).unwrap(),
            ConsensusType::POG => {
                let k = self.ntd;
                Pog::select(validators.clone(), next_seed.clone(), bc, self.ntd, k).unwrap()
            }
        };

        //这里简化成通知miner出块，实际上应该是每个节点自己算
        match self.nodes_sender.get(&miner_validator.address) {
            Some(sender) => {
                debug!(
                    "World State find miner: {}",
                    miner_validator.address.clone()
                );
                sender
                    .send(Message::new_generate_block_msg())
                    .await
                    .unwrap();
            }
            None => {
                warn!("World State error: failed to find miner");
            }
        }
    }

    pub async fn next_epoch(&mut self) {
        let current_slot = self.current_slot.read().await.clone();
        let _current_epoch = current_slot.current_epoch;
        //更新NTD
        let blocks = self.blockchain.read().await.get_last_epoch_block();
        let paths: Vec<Vec<String>> = blocks.iter().flat_map(|b| b.get_all_paths()).collect();
        if !paths.is_empty() {
            let p_ave =
                //出块节点不算，要减一
                paths.iter().map(|v| v.len() - 1).sum::<usize>() as f64 / paths.len() as f64;
            if self.ntd > p_ave.ceil() as usize {
                self.ntd = self.ntd - 1;
            } else if self.ntd < p_ave.ceil() as usize {
                self.ntd = self.ntd + 1;
            }
        }

        let validators = self.validators.read().await.clone();
        let next_seed = consensus::combine_seed(validators.clone(), current_slot.randao_seeds);
        self.current_slot = Arc::new(RwLock::new(SlotManager {
            randao_seeds: vec![],
            slot_duration: SLOT_DURATION,
            current_epoch: current_slot.current_epoch + 1,
            current_slot: 0,
            next_seed,
            start_timestamp: get_timestamp(),
        }));
    }

    pub async fn get_current_slot(&self) -> SlotManager {
        self.current_slot.read().await.clone()
    }

    pub async fn run(self, mut receiver: Receiver<Message>) {
        let shared_self = Arc::new(RwLock::new(self));

        let receiver_task = {
            let shared_self = Arc::clone(&shared_self);
            task::spawn(async move {
                while let Some(msg) = receiver.recv().await {
                    debug!("World State received msg type: {}", msg.msg_type);
                    match msg.msg_type {
                        MessageType::ReceiveRandaoSeed => {
                            let randao_seed = match RandaoSeed::from_json(msg.data) {
                                Ok(t) => t,
                                Err(e) => {
                                    error!("World State error: {}", e);
                                    continue;
                                }
                            };
                            {
                                let shared_self = shared_self.write().await;
                                let mut current_slot = shared_self.current_slot.write().await;
                                current_slot.randao_seeds.push(randao_seed.clone());
                            }
                        }
                        MessageType::ReceiveBecomeValidator => {
                            let validator = match Validator::from_json(msg.data) {
                                Ok(t) => t,
                                Err(e) => {
                                    error!("World State error: {}", e);
                                    continue;
                                }
                            };
                            {
                                let shared_self = shared_self.write().await;
                                let mut validators = shared_self.validators.write().await;
                                validators.retain(|v| v.address != validator.address);
                                validators.push(validator.clone());
                            }
                        }
                        MessageType::SendBlock => {
                            let block = match Block::from_json(msg.data) {
                                Ok(b) => b,
                                Err(e) => {
                                    error!("Error: {}", e);
                                    continue;
                                }
                            };

                            let shared_self = shared_self.write().await;
                            if let Err(e) = shared_self.blockchain.write().await.add_block(block) {
                                match e {
                                    _ => {
                                        error!("World State Error: {}", e);
                                    }
                                }
                                continue;
                            }
                            debug!("World State add block successfully");
                        }
                        _ => {}
                    }
                }
            })
        };
        let timer_task = task::spawn(async move {
            loop {
                let time_interval = {
                    let shared_self = shared_self.read().await;
                    let current_slot = shared_self.get_current_slot().await;
                    let time_interval = current_slot.start_timestamp
                        + current_slot.slot_duration.as_secs()
                        - get_timestamp();
                    Duration::from_secs(time_interval)
                };
                let deadline = Instant::now() + time_interval;
                time::sleep_until(deadline).await;
                debug!("World State time trigger: {}", tools::get_time_string());
                {
                    let mut shared_self = shared_self.write().await;
                    shared_self.next_slot().await;
                }
            }
        });

        let _ = tokio::join!(timer_task, receiver_task);
    }
}

impl SlotManager {
    pub fn from_json(json: Vec<u8>) -> Result<SlotManager, WorldStateError> {
        let slot: SlotManager = serde_json::from_slice(json.as_slice())?;
        Ok(slot)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug)]
pub enum WorldStateError {
    JSONError,
}
impl fmt::Display for WorldStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WorldStateError::JSONError => {
                write!(f, "Invalid Json Error")
            }
        }
    }
}
impl From<serde_json::error::Error> for WorldStateError {
    fn from(_: serde_json::error::Error) -> Self {
        WorldStateError::JSONError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::block::Block;
    use crate::blockchain::path::TransactionPaths;
    use crate::blockchain::transaction::Transaction;
    use crate::blockchain::Blockchain;
    use crate::network::node::{Neighbor, Node};
    use log::info;

    #[tokio::test]
    async fn timer_trigger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        let blockchain = Blockchain::new(Block::gen_genesis_block());
        let (world, _world_sender, world_receiver) = WorldState::new(
            blockchain.get_last_block().clone(),
            ConsensusType::POS,
            Blockchain::new(Block::gen_genesis_block()),
        );
        tokio::spawn(async move {
            world.run(world_receiver).await;
        });
        tokio::time::sleep(Duration::from_secs(11)).await;
    }

    #[tokio::test]
    async fn collect_seeds() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        let blockchain = Blockchain::new(Block::gen_genesis_block());
        let (mut world, world_sender, world_receiver) = WorldState::new(
            blockchain.get_last_block().clone(),
            ConsensusType::POS,
            Blockchain::new(Block::gen_genesis_block()),
        );

        let validators = world.validators.clone();
        let current_slot = world.current_slot.clone();

        let mut node0 = Node::new(0, 0, 0, blockchain.clone(), world_sender.clone());
        let mut node1 = Node::new(1, 0, 0, blockchain.clone(), world_sender.clone());
        let node0_sender = node0.sender.clone();
        let node1_sender = node1.sender.clone();
        let node0_wallet = node0.wallet.clone();
        let node1_wallet = node1.wallet.clone();
        let node0_bc = node0.blockchain.clone();
        let node0_tx_cache = node0.transaction_paths_cache.clone();

        world
            .nodes_sender
            .insert(node0_wallet.address.clone(), node0_sender.clone());
        world
            .nodes_sender
            .insert(node1_wallet.address.clone(), node1_sender.clone());

        node0.neighbors.push(Neighbor::new(
            node1.index,
            node1.wallet.address.clone(),
            node1.sender.clone(),
        ));
        node1.neighbors.push(Neighbor::new(
            node0.index,
            node0.wallet.address.clone(),
            node0.sender.clone(),
        ));

        let handle_world = tokio::spawn(async move {
            world.run(world_receiver).await;
        });
        let handle0 = tokio::spawn(async move {
            node0.run().await;
        });
        let handle1 = tokio::spawn(async move {
            node1.run().await;
        });
        //become validator
        node0_sender
            .send(Message::new_become_validator_msg())
            .await
            .unwrap();
        node1_sender
            .send(Message::new_become_validator_msg())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        {
            let validators = validators.read().await.clone();
            info!("validators:{:?}", validators);
        }

        //send seed
        node0_sender
            .send(Message::new_send_randao_seed_msg())
            .await
            .unwrap();
        node1_sender
            .send(Message::new_send_randao_seed_msg())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        {
            let current_slot = current_slot.read().await.clone();
            info!("current_slot:{:?}", current_slot);
        }

        //node0发送交易
        let transaction = Transaction::new(node1_wallet.address.clone(), 0, node0_wallet.clone());
        let transaction_paths = TransactionPaths::new(transaction);
        node0_sender
            .send(Message::new_transaction_paths_msg(
                transaction_paths,
                "".to_string(),
            ))
            .await
            .unwrap();

        //wait for next slot
        tokio::time::sleep(Duration::from_secs(5)).await;
        {
            node0_bc.read().await.simple_print_last_five_block();
        }
        {
            let txs_cache = node0_tx_cache.read().await;
            info!("txs_cache:{:?}", txs_cache);
        }

        //node1发送交易
        let transaction = Transaction::new(node0_wallet.address, 0, node1_wallet);
        let transaction_paths = TransactionPaths::new(transaction);
        node1_sender
            .send(Message::new_transaction_paths_msg(
                transaction_paths,
                "".to_string(),
            ))
            .await
            .unwrap();

        //wait for next slot
        tokio::time::sleep(Duration::from_secs(5)).await;
        {
            node0_bc.read().await.simple_print_last_five_block();
        }
        {
            let txs_cache = node0_tx_cache.read().await;
            info!("txs_cache:{:?}", txs_cache);
        }
    }

    #[tokio::test]
    async fn test_flat_map() {
        let a = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let b = vec![vec![7, 8, 9], vec![10, 11, 12], vec![13, 14, 15]];
        let c = vec![a, b];
        let d: Vec<Vec<i32>> = c.iter().flat_map(|v| v.clone()).collect();
        println!("{:?}", d);
    }
}
