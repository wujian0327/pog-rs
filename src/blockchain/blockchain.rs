use crate::blockchain::block::Block;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    blocks: Vec<Block>,
    transactions_hash_set: HashSet<String>,
}

impl Blockchain {
    pub fn new(genesis_block: Block) -> Blockchain {
        let mut set = HashSet::new();
        for x in genesis_block.clone().body.transactions {
            set.insert(x.hash.to_string());
        }
        Blockchain {
            blocks: vec![genesis_block],
            transactions_hash_set: set,
        }
    }

    pub fn get_block(&self, height: u64) -> Block {
        self.blocks[height as usize - 1].clone()
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockChainError> {
        if !block.verify() {
            return Err(BlockChainError::InvalidBlock);
        }
        if self.get_last_hash() == block.header.hash {
            //重复收到
            return Err(BlockChainError::DuplicateBlocksReceived);
        }
        if self.get_last_hash() != block.header.parent_hash {
            return Err(BlockChainError::ParentHashMismatch);
        }
        if self.get_last_block().header.index + 1 != block.header.index {
            return Err(BlockChainError::IndexMismatch);
        }
        if self.get_last_block().header.epoch > block.header.epoch {
            return Err(BlockChainError::EpochError);
        }
        if self.get_last_block().header.epoch == block.header.epoch
            && self.get_last_block().header.slot > block.header.slot
        {
            return Err(BlockChainError::SlotError);
        }
        //check transaction if exists
        for x in block.clone().body.transactions {
            if self.exist_transaction(x.hash.to_string()) {
                return Err(BlockChainError::TransactionExists);
            }
        }
        self.blocks.push(block.clone());
        for x in block.body.transactions {
            self.transactions_hash_set.insert(x.hash.to_string());
        }
        Ok(())
    }

    pub fn exist_transaction(&self, hash: String) -> bool {
        self.transactions_hash_set.contains(&hash)
    }

    pub fn get_last_block(&self) -> Block {
        self.blocks.last().unwrap().clone()
    }

    pub fn get_last_hash(&self) -> String {
        self.blocks.last().unwrap().header.hash.clone()
    }
    pub fn get_lash_index(&self) -> u64 {
        self.get_last_block().header.index
    }

    pub fn simple_print_last_five_block(&self) {
        let last_five = &self.blocks[self.blocks.len().saturating_sub(5)..];
        for x in last_five {
            x.simple_print_no_transaction_detail();
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockChainError {
    InvalidBlock,
    ParentHashMismatch,
    IndexMismatch,
    EpochError,
    SlotError,
    DuplicateBlocksReceived,
    TransactionExists,
}

impl fmt::Display for BlockChainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlockChainError::InvalidBlock => {
                write!(f, "Invalid Block Error")
            }
            BlockChainError::ParentHashMismatch => {
                write!(f, "Parent Hash Mismatch Error")
            }
            BlockChainError::IndexMismatch => {
                write!(f, "Parent Index Mismatch Error")
            }
            BlockChainError::EpochError => {
                write!(f, "Epoch Error")
            }
            BlockChainError::SlotError => {
                write!(f, "Slot Error")
            }

            BlockChainError::DuplicateBlocksReceived => {
                write!(f, "Duplicate Block Received")
            }

            BlockChainError::TransactionExists => {
                write!(f, "Transaction exists")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::block::Body;
    use crate::blockchain::path::{AggregatedSignedPaths, TransactionPaths};
    use crate::blockchain::transaction::Transaction;
    use crate::wallet::Wallet;

    #[test]
    fn test_blockchain() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        let mut blockchain = Blockchain::new(Block::gen_genesis_block());
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
            vec![AggregatedSignedPaths::from_transaction_paths(
                transaction_paths,
            )],
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
        blockchain.add_block(block).unwrap();
        blockchain.simple_print_last_five_block();
    }
}
