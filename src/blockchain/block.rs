use crate::blockchain::path::{AggregatedSignedPaths, TransactionPaths};
use crate::blockchain::transaction::Transaction;
use crate::tools;
use crate::wallet::Wallet;
use hex::{decode, encode};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub index: u64,
    pub epoch: u64,
    pub slot: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub merkle_root: String,
    pub miner: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body {
    pub transactions: Vec<Transaction>,
    pub paths: Vec<AggregatedSignedPaths>,
}

impl Header {
    pub fn new(
        index: u64,
        epoch: u64,
        slot: u64,
        merkle_root: String,
        miner: String,
        parent_hash: String,
    ) -> Header {
        let mut header = Header {
            index,
            epoch,
            slot,
            hash: "".to_string(),
            parent_hash,
            timestamp: tools::get_timestamp(),
            merkle_root,
            miner,
        };
        header.hash = header.get_hash();
        header
    }

    pub fn get_hash(&self) -> String {
        let mut header = self.clone();
        header.hash = "".to_string();
        let t_json = serde_json::to_string(&header).unwrap();
        let hash = tools::Hasher::hash(t_json.as_bytes().to_vec());
        encode(hash)
    }
}

impl Block {
    pub fn new(
        index: u64,
        epoch: u64,
        slot: u64,
        parent_hash: String,
        body: Body,
        wallet: Wallet,
    ) -> Result<Block, BlockError> {
        if body.transactions.len() != body.paths.len() {
            return Err(BlockError::InvalidBlock);
        }
        for (i, transaction) in body.transactions.iter().enumerate() {
            if !transaction.verify() {
                return Err(BlockError::InvalidBlockTransactions);
            }
            if !body.paths[i].verify(transaction.clone(), wallet.address.clone()) {
                return Err(BlockError::InvalidBlockPath);
            }
        }
        let hash_vec = body.transactions.iter().map(|t| t.hash.clone()).collect();
        let merkle_root = Block::cal_merkle_root(hash_vec);
        let header = Header::new(index, epoch, slot, merkle_root, wallet.address, parent_hash);
        Ok(Block { header, body })
    }

    pub fn verify(&self) -> bool {
        if self.body.transactions.len() != self.body.paths.len() {
            error!("{}", BlockError::InvalidBlock);
            return false;
        }
        for (i, transaction) in self.body.transactions.iter().enumerate() {
            if !transaction.verify() {
                error!("{}", BlockError::InvalidBlockTransactions);
                return false;
            }
            if !self.body.paths[i].verify(transaction.clone(), self.header.miner.clone()) {
                error!("{}", BlockError::InvalidBlockPath);
                return false;
            }
        }
        true
    }

    fn cal_merkle_root(mut leaves: Vec<String>) -> String {
        if leaves.len() == 1 {
            return leaves[0].clone();
        }

        if leaves.len() % 2 != 0 {
            leaves.push(leaves.last().unwrap().clone());
        }

        let mut next_level = Vec::new();
        for pair in leaves.chunks(2) {
            let mut combined = decode(pair[0].clone()).unwrap();
            combined.append(&mut decode(pair[1].clone()).unwrap());
            let hash = encode(combined);
            next_level.push(hash);
        }
        Block::cal_merkle_root(next_level)
    }

    pub fn gen_genesis_block() -> Block {
        let miner = Wallet::new();
        let transaction = Transaction::new("000".to_string(), 50, miner.clone());
        let transaction_paths = TransactionPaths::new(transaction.clone());
        let paths = AggregatedSignedPaths::from_transaction_paths(transaction_paths);
        let body = Body::new(vec![transaction], vec![paths]);
        Block::new(0, 0, 0, "".to_string(), body, miner).unwrap()
    }

    pub fn from_json(json: Vec<u8>) -> Result<Block, BlockError> {
        let block: Block = serde_json::from_slice(json.as_slice())?;
        Ok(block)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn simple_print(&self) {
        info!("Block[{}]:", self.header.index);
        info!("\t epoch:{}:", self.header.epoch);
        info!("\t slot:{}:", self.header.slot);
        info!("\t miner:{}:", self.header.miner);
        info!("\t timestamp:{}:", self.header.timestamp);
        for (i, x) in self.body.transactions.iter().enumerate() {
            info!("\t transactions[{i}]:");
            info!("\t\t from:{}:", x.from);
            info!("\t\t to:{}:", x.to);
            info!("\t\t paths:");
            let mut s = String::from("");
            for (j, p) in self.body.paths[i].paths.clone().iter().enumerate() {
                if j == 0 {
                    s.push_str("\t\t\t");
                }
                s.push_str(format!("->{}", p).as_str());
            }
            info!("{}", s);
        }
    }

    pub fn simple_print_no_transaction_string(&self) -> String {
        let mut s = format!("Block[{}]: \n", self.header.index);
        s.push_str(format!("\t epoch:{} \n", self.header.epoch).as_str());
        s.push_str(format!("\t slot:{} \n", self.header.slot).as_str());
        s.push_str(format!("\t miner:{} \n", self.header.miner).as_str());
        s.push_str(format!("\t timestamp:{} \n", self.header.timestamp).as_str());
        let trans_hash: Vec<String> = self
            .body
            .transactions
            .iter()
            .map(|x| x.hash.to_string())
            .collect();
        s.push_str(format!("\t transactions[{}]\n", trans_hash.join(",")).as_str());
        let paths: Vec<String> = self.body.paths.iter().map(|p| p.paths.join("->")).collect();
        s.push_str(format!("\t paths[{}]\n", paths.join(",")).as_str());
        s
    }

    pub fn simple_print_no_transaction_detail(&self) {
        info!("{}", self.simple_print_no_transaction_string());
    }
}

impl Body {
    pub(crate) fn new(transactions: Vec<Transaction>, paths: Vec<AggregatedSignedPaths>) -> Body {
        Body {
            transactions,
            paths,
        }
    }
}

#[derive(Debug)]
pub enum BlockError {
    InvalidBlock,
    InvalidBlockPath,
    InvalidBlockTransactions,
    JSONError,
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlockError::InvalidBlock => {
                write!(f, "Invalid Block Error")
            }

            BlockError::InvalidBlockPath => {
                write!(f, "Invalid Block Path Error")
            }
            BlockError::InvalidBlockTransactions => {
                write!(f, "Invalid Block Transactions Error")
            }
            BlockError::JSONError => {
                write!(f, "Invalid Block Json Error")
            }
        }
    }
}

impl From<serde_json::error::Error> for BlockError {
    fn from(_: serde_json::error::Error) -> Self {
        BlockError::JSONError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet;

    #[test]
    fn test_block() {
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
        let block = match Block::new(0, 0, 0, String::from(""), body, miner) {
            Ok(block) => block,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        println!("{:#?}", block);
        block.simple_print();
    }

    #[test]
    fn test_gen_genesis_block() {
        println!("{:#?}", Block::gen_genesis_block());
    }
}
