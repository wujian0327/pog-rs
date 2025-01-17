use crate::blockchain::block::{Block, BlockError};
use crate::blockchain::transaction::Transaction;
use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Path {
    pub to: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionPaths {
    pub transaction: Transaction,
    pub paths: Vec<Path>,
}

impl TransactionPaths {
    pub fn new(transaction: Transaction) -> TransactionPaths {
        TransactionPaths {
            transaction,
            paths: Vec::new(),
        }
    }

    pub fn new_with_paths(transaction: Transaction, paths: Vec<Path>) -> TransactionPaths {
        TransactionPaths { transaction, paths }
    }

    pub fn add_path(&mut self, to: String, wallet: Wallet) {
        let sign = wallet.sign(to.clone().into_bytes());
        self.paths.push(Path {
            to,
            signature: sign.clone(),
        });
    }

    pub fn verify(&self, current_address: String) -> bool {
        if !self.transaction.clone().verify() {
            return false;
        }
        if self.paths.is_empty() && current_address == self.transaction.from {
            return true;
        }
        let mut from = self.transaction.from.clone();
        let mut to = "".to_string();
        for path in &self.paths {
            to = path.to.clone();
            let signature = path.signature.clone();
            let result = Wallet::verify_by_address(to.clone().into_bytes(), signature, from);
            if !result {
                return false;
            }
            from = path.to.clone();
        }
        if to != current_address {
            return false;
        }
        true
    }

    pub fn from_json(json: Vec<u8>) -> Result<TransactionPaths, PathError> {
        let transaction_paths: TransactionPaths = serde_json::from_slice(json.as_slice())?;
        Ok(transaction_paths)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug)]
pub enum PathError {
    JSONError,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathError::JSONError => {
                write!(f, "Invalid Json Error")
            }
        }
    }
}

impl From<serde_json::error::Error> for PathError {
    fn from(_: serde_json::error::Error) -> Self {
        PathError::JSONError
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_paths() {
        let wallet = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();
        let miner = Wallet::new();
        let transaction = Transaction::new("123".to_string(), 32, wallet.clone());
        let mut transaction_paths = TransactionPaths::new(transaction);
        transaction_paths.add_path(wallet2.address.clone(), wallet);
        transaction_paths.add_path(wallet3.address.clone(), wallet2);
        transaction_paths.add_path(miner.address.clone(), wallet3);
        println!("{:#?}", transaction_paths);
        assert!(transaction_paths.verify(miner.address.clone()));
    }
}
