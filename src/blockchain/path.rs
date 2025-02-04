use crate::blockchain::transaction::Transaction;
use crate::wallet::Wallet;
use crate::{tools, wallet};
use bls_signatures::{PublicKey, Signature};
use hex::decode;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Path {
    pub to: String,
    //此处使用bls的签名
    pub signature: String,
}

/// 传播交易时使用
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionPaths {
    pub transaction: Transaction,
    pub paths: Vec<Path>,
}

/// 打包到区块时使用
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregatedSignedPaths {
    pub signature: String,
    pub paths: Vec<String>,
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

    // pub fn add_path(&mut self, to: String, wallet: Wallet) {
    //     let sign = wallet.sign(to.clone().into_bytes());
    //     self.paths.push(Path {
    //         to,
    //         signature: sign.clone(),
    //     });
    // }

    pub fn add_path(&mut self, to: String, wallet: Wallet) {
        // data-> H(tx) || H(to)
        let hash = self.concat_tx_hash_with_to_hash(to.clone());
        let sign = wallet.sign_by_bls(hash);
        self.paths.push(Path {
            to,
            signature: sign.clone(),
        });
    }

    fn concat_tx_hash_with_to_hash(&self, to: String) -> Vec<u8> {
        concat_tx_hash_with_to_hash_static(self.transaction.hash.clone(), to)
    }

    // pub fn verify(&self, current_address: String) -> bool {
    //     if !self.transaction.clone().verify() {
    //         return false;
    //     }
    //     if self.paths.is_empty() && current_address == self.transaction.from {
    //         return true;
    //     }
    //     let mut from = self.transaction.from.clone();
    //     let mut to = "".to_string();
    //     for path in &self.paths {
    //         to = path.to.clone();
    //         let signature = path.signature.clone();
    //         let result = Wallet::verify_by_address(to.clone().into_bytes(), signature, from);
    //         if !result {
    //             return false;
    //         }
    //         from = path.to.clone();
    //     }
    //     if to != current_address {
    //         return false;
    //     }
    //     true
    // }

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
            let pk = match wallet::get_bls_pub_key(from) {
                Some(pk) => pk,
                None => {
                    return false;
                }
            };
            let hash = self.concat_tx_hash_with_to_hash(to.clone());
            let result = Wallet::verify_bls_with_pk(hash, signature, pk);
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

    //只需要验证上一个节点的签名就行，出块时才需要全部验证
    pub fn verify_last(&self, current_address: String) -> bool {
        if !self.transaction.clone().verify() {
            return false;
        }
        //自己发起的
        if self.paths.is_empty() && current_address == self.transaction.from {
            return true;
        }
        if self.paths.is_empty() {
            return false;
        }
        let mut from = self.transaction.from.clone();
        if self.paths.len() > 1 {
            from = self.paths[self.paths.len() - 2].clone().to;
        }
        let path = self.paths.last().unwrap();
        let to = path.to.clone();
        if to != current_address {
            return false;
        }
        let signature = path.signature.clone();
        let pk = match wallet::get_bls_pub_key(from) {
            Some(pk) => pk,
            None => {
                return false;
            }
        };
        let hash = self.concat_tx_hash_with_to_hash(to.clone());
        let result = Wallet::verify_bls_with_pk(hash, signature, pk);
        if !result {
            return false;
        }
        true
    }

    pub fn to_aggregated_signed_paths(&self) -> AggregatedSignedPaths {
        AggregatedSignedPaths::from_transaction_paths(self.clone())
    }

    pub fn from_json(json: Vec<u8>) -> Result<TransactionPaths, PathError> {
        let transaction_paths: TransactionPaths = serde_json::from_slice(json.as_slice())?;
        Ok(transaction_paths)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn to_paths_string(&self) -> String {
        self.paths
            .iter()
            .map(|x| (&x.to.clone()[0..5]).to_string())
            .collect::<Vec<String>>()
            .join("->")
    }
}

pub fn concat_tx_hash_with_to_hash_static(tx_hash: String, to: String) -> Vec<u8> {
    let mut tx_hash = decode(tx_hash).unwrap();
    let to_hash = tools::Hasher::hash(to.as_bytes().to_vec()).to_vec();
    tx_hash.append(to_hash.clone().as_mut());
    tx_hash
}

impl AggregatedSignedPaths {
    pub fn from_transaction_paths(paths: TransactionPaths) -> AggregatedSignedPaths {
        let from = paths.transaction.from.clone();
        let mut path_string_vec: Vec<String> = paths.paths.iter().map(|p| p.to.clone()).collect();
        path_string_vec.insert(0, from);
        //聚合签名
        let signatures: Vec<Signature> = paths
            .paths
            .iter()
            .map(|p| Wallet::bls_signature_from_string(p.signature.clone()).unwrap())
            .collect();
        let aggregated_sign = Wallet::bls_aggregated_sign(signatures);
        AggregatedSignedPaths {
            signature: aggregated_sign,
            paths: path_string_vec,
        }
    }

    pub fn verify(&self, transaction: Transaction, miner: String) -> bool {
        if self.paths.is_empty() {
            return false;
        }
        //miner和发起是一个节点
        if transaction.from == miner && self.paths.first().unwrap().to_string() == miner {
            return true;
        }

        //miner必须是最后一个path
        if self.paths.last().unwrap().to_string() != miner {
            return false;
        }
        //聚合签名验证
        //先还原message
        let mut messages: Vec<Vec<u8>> = vec![];
        for (i, p) in self.paths.iter().enumerate() {
            //发起者是对下一个节点进行的签名
            if i == 0 {
                continue;
            }
            let hash = concat_tx_hash_with_to_hash_static(transaction.hash.clone(), p.clone());
            messages.push(hash.to_vec());
        }

        //再去找公钥
        let mut pks: Vec<PublicKey> = self
            .paths
            .iter()
            .map(|p| wallet::get_bls_pub_key(p.clone()).unwrap())
            .collect();
        //miner并没有传播交易，所以去掉
        pks.remove(pks.len() - 1);
        Wallet::bls_aggregated_verify(messages, pks, self.signature.clone())
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
    use crate::wallet;

    #[test]
    fn test_transaction_paths_bls() {
        let wallet = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();
        let miner = Wallet::new();
        let transaction = Transaction::new("123".to_string(), 32, wallet.clone());
        let mut transaction_paths = TransactionPaths::new(transaction.clone());
        transaction_paths.add_path(wallet2.address.clone(), wallet.clone());
        transaction_paths.add_path(wallet3.address.clone(), wallet2.clone());
        transaction_paths.add_path(miner.address.clone(), wallet3.clone());
        println!("{:#?}", transaction_paths);
        wallet::insert_bls_pub_key(wallet.address.clone(), wallet.bls_public_key.clone());
        wallet::insert_bls_pub_key(wallet2.address.clone(), wallet2.bls_public_key.clone());
        wallet::insert_bls_pub_key(wallet3.address.clone(), wallet3.bls_public_key.clone());
        wallet::insert_bls_pub_key(miner.address.clone(), miner.bls_public_key.clone());
        assert!(transaction_paths.verify(miner.address.clone()));

        //check aggregated_signed_paths
        let aggregated_signed_paths =
            AggregatedSignedPaths::from_transaction_paths(transaction_paths);
        assert!(aggregated_signed_paths.verify(transaction.clone(), miner.address.clone()));
        println!("{:#?}", aggregated_signed_paths);
    }
}
