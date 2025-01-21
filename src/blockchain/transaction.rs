use crate::tools;
use crate::tools::get_timestamp;
use crate::wallet::Wallet;
use hex::encode;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub hash: String,
    pub signature: String,
    pub timestamp: u64,
}

impl Transaction {
    pub(crate) fn new(to: String, amount: i64, wallet: Wallet) -> Transaction {
        let from = wallet.address.clone();

        let mut t = Transaction {
            from: from.clone(),
            to: to.clone(),
            amount,
            hash: "".to_string(),
            signature: "".to_string(),
            timestamp: get_timestamp(),
        };
        let t_json = serde_json::to_string(&t).unwrap();
        let hash = tools::Hasher::hash(t_json.as_bytes().to_vec());
        let signature = wallet.sign(hash.to_vec());
        let hash = encode(&hash);
        t.hash = hash;
        t.signature = signature;
        t
    }

    pub(crate) fn verify(&self) -> bool {
        let from = self.from.clone();
        let to = self.to.clone();
        let t = Transaction {
            from: from.clone(),
            to: to.clone(),
            amount: self.amount,
            hash: "".to_string(),
            signature: "".to_string(),
            timestamp: self.timestamp,
        };
        let t_json = serde_json::to_string(&t).unwrap();
        let hash = tools::Hasher::hash(t_json.as_bytes().to_vec());
        if self.hash != encode(&hash) {
            return false;
        }
        Wallet::verify_by_address(Vec::from(hash), self.signature.clone(), from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction() {
        let wallet = Wallet::new();
        let transaction = Transaction::new("123".to_string(), 32, wallet);
        info!("{:#?}", transaction);
        assert!(transaction.verify());
    }
}
