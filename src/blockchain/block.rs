use crate::blockchain::transaction::Transaction;

pub struct Block {
    pub header: Header,
    pub body: Body,
}

pub struct Header {
    pub index: u64,
    pub hash: String,
    pub parent: String,
    pub timestamp: u128,
    pub merkle_root: String,
    pub miner: String,
}

pub struct Body {
    pub transaction: Vec<Transaction>,
    pub vdf_seeds: Vec<Transaction>,
}
