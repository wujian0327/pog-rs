use crate::blockchain::block::Block;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new(genesis_block: Block) -> Blockchain {
        Blockchain {
            blocks: vec![genesis_block],
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
        if self.get_last_block().header.epoch == block.header.epoch && self.get_last_block().header.slot > block.header.slot {
            return Err(BlockChainError::SlotError);
        }
        self.blocks.push(block);
        Ok(())
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
            x.simple_print();
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

    #[test]
    fn test_blockchain() {
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
        blockchain.add_block(block).unwrap();
        blockchain.simple_print_last_five_block();
    }
}
