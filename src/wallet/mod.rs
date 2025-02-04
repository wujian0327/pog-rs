use crate::tools::Hasher;
use bls_signatures::{aggregate, verify_messages, Error, PrivateKey, Serialize, Signature};
use dashmap::DashMap;
use hex::{decode, encode, FromHexError};
use lazy_static::lazy_static;
use log::info;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

// 设置一个全局的bls的公钥管理对象
// 一般来说，这个功能在以太坊2.0由验证者注册合约实现
// 我们简化成一个全局变量来使用
// 我们希望愿意参与网络贡献的节点，都注册bls公钥
// 这样可以大大减少签名带来的存储开销
lazy_static! {
    static ref BLS_PUB_KEY_MAP: DashMap<String, bls_signatures::PublicKey> = DashMap::new();
}

// 定义一个 OnceCell 用于存储 Arc<HashMap<String, String>>
// static HASHMAP_CELL: OnceCell<Arc<HashMap<String, bls_signatures::PublicKey>>> = OnceCell::new();
//
// fn init_pk_map(
//     addresses: Vec<String>,
//     pks: Vec<bls_signatures::PublicKey>,
// ) -> &'static Arc<HashMap<String, bls_signatures::PublicKey>> {
//     HASHMAP_CELL.get_or_init(|| {
//         let mut map = HashMap::new();
//         for (i, address) in addresses.into_iter().enumerate() {
//             map.insert(address.to_string(), pks[i].clone());
//         }
//         Arc::new(map)
//     })
// }

pub fn get_bls_pub_key(address: String) -> Option<bls_signatures::PublicKey> {
    BLS_PUB_KEY_MAP
        .get(&address)
        .map(|entry| *entry.value())
}
pub fn insert_bls_pub_key(address: String, public_key: bls_signatures::PublicKey) {
    BLS_PUB_KEY_MAP.insert(address, public_key);
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    // blsKey用于对网络贡献度和pos投票进行签名
    pub bls_private_key: PrivateKey,
    pub bls_public_key: bls_signatures::PublicKey,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();

        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let address = Wallet::public_key_to_address(public_key);
        let bls_private_key = PrivateKey::new(secret_key.secret_bytes());
        let bls_public_key = bls_private_key.public_key();
        insert_bls_pub_key(address.clone(), bls_public_key);
        Wallet {
            secret_key,
            public_key,
            bls_private_key,
            bls_public_key,
            address,
        }
    }

    fn from_secret_key_string(mut secret_key: String) -> Result<Wallet, WalletError> {
        if secret_key.len() == 66 {
            secret_key = secret_key[2..].to_string();
        }
        if secret_key.len() != 64 {
            return Err(WalletError::InvalidPrivateKeyString);
        }
        let secret_key = match SecretKey::from_str(secret_key.as_str()) {
            Ok(sk) => sk,
            Err(e) => {
                return Err(WalletError::InvalidPrivateKeyString);
            }
        };
        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        let address = Wallet::public_key_to_address(public_key);
        let bls_private_key = PrivateKey::new(secret_key.secret_bytes());
        let bls_public_key = bls_private_key.public_key();
        Ok(Wallet {
            secret_key,
            public_key,
            bls_private_key,
            bls_public_key,
            address,
        })
    }

    fn public_key_to_address(public_key: PublicKey) -> String {
        // 忽略第一个字节（表示前缀）
        let public_key_bytes = &public_key.serialize_uncompressed()[1..];

        //sha3
        let hash_result = Hasher::hash(public_key_bytes.to_vec());

        // 以太坊地址（最后 20 字节）
        let address = &hash_result[12..];
        format!("0x{}", encode(address))
    }

    pub fn sign(&self, msg: Vec<u8>) -> String {
        //hash first
        let hash_result = Hasher::hash(msg);
        let message = Message::from_digest(hash_result);

        //sign recoverable
        let secp = Secp256k1::new();
        let recoverable_signature = secp.sign_ecdsa_recoverable(&message, &self.secret_key);

        //以太坊签名(r, s, v), v = 27 + RecoveryId
        let (recovery_id, signature_bytes) = recoverable_signature.serialize_compact();
        let v = 27 + recovery_id as i32;
        format!("0x{}{:02x}", encode(signature_bytes), v)
    }

    pub fn sign_by_bls(&self, msg: Vec<u8>) -> String {
        let sign = self.bls_private_key.sign(msg);
        format!("0x{}", encode(sign.as_bytes()))
    }

    fn recover_pubkey(msg: Vec<u8>, mut signature: String) -> Result<PublicKey, WalletError> {
        //使用签名和消息恢复公钥
        if signature.starts_with("0x") {
            signature = signature[2..].to_string();
        }
        let hash_result = Hasher::hash(msg);
        let message = Message::from_digest(hash_result);

        // 分解签名为 r, s 和 v
        let signature_bytes = decode(&signature[0..128])?;

        let v = u8::from_str_radix(&signature[128..130], 16)?;

        // 生成可恢复签名对象
        let recovery_id = RecoveryId::try_from((v - 27) as i32).expect("Valid RecoveryId");
        let recoverable_signature =
            RecoverableSignature::from_compact(&signature_bytes, recovery_id)
                .expect("Valid signature");

        // 从签名恢复公钥
        let secp = Secp256k1::new();
        let recovered_public_key = secp
            .recover_ecdsa(&message, &recoverable_signature)
            .expect("Recovered public key");
        Ok(recovered_public_key)
    }

    fn verify(&self, msg: Vec<u8>, signature: String) -> bool {
        //使用签名和消息恢复公钥，再判断公钥是否一致
        match Wallet::recover_pubkey(msg, signature) {
            Ok(recovered_public_key) => {
                // 验证公钥匹配
                recovered_public_key == self.public_key
            }
            Err(_) => false,
        }
    }

    fn verify_bls(&self, msg: Vec<u8>, signature: String) -> bool {
        let signature = match Wallet::bls_signature_from_string(signature) {
            Ok(signature) => signature,
            Err(e) => {
                return false;
            }
        };
        self.bls_public_key.verify(signature, msg)
    }

    pub fn verify_by_address(msg: Vec<u8>, signature: String, address: String) -> bool {
        //使用签名和消息恢复公钥
        //再使用公钥生成地址，判断地址是否一致
        let pk = match Wallet::recover_pubkey(msg, signature) {
            Ok(pk) => pk,
            Err(_) => {
                return false;
            }
        };
        let recovery_address = Wallet::public_key_to_address(pk);
        recovery_address == address
    }

    pub fn verify_bls_with_pk(
        msg: Vec<u8>,
        signature: String,
        public_key: bls_signatures::PublicKey,
    ) -> bool {
        let signature = match Wallet::bls_signature_from_string(signature) {
            Ok(signature) => signature,
            Err(e) => {
                return false;
            }
        };
        public_key.verify(signature, msg)
    }

    pub fn bls_signature_from_string(mut signature: String) -> Result<Signature, WalletError> {
        if signature.starts_with("0x") {
            signature = signature[2..].to_string();
        }
        let signature_bytes = decode(&signature)?;
        let signature = Signature::from_bytes(signature_bytes.as_slice())?;
        Ok(signature)
    }

    pub fn bls_aggregated_sign(signatures: Vec<Signature>) -> String {
        if signatures.is_empty() {
            return String::new();
        }
        let aggregated_signature = aggregate(&signatures).unwrap();
        format!("0x{}", encode(aggregated_signature.as_bytes()))
    }

    pub fn bls_aggregated_verify(
        messages: Vec<Vec<u8>>,
        public_keys: Vec<bls_signatures::PublicKey>,
        signature: String,
    ) -> bool {
        let signature = match Wallet::bls_signature_from_string(signature) {
            Ok(signature) => signature,
            Err(_) => {
                return false;
            }
        };
        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let result = verify_messages(&signature, messages.as_slice(), public_keys.as_slice());
        result
    }

    fn print(&self) {
        info!("Secret Key: 0x{}", encode(self.secret_key.secret_bytes()));
        let public_key_bytes = &self.public_key.serialize_uncompressed()[1..];
        info!("Public Key: 0x{}", encode(public_key_bytes));
        info!("Address: {}", &self.address);
    }
}

#[derive(Debug)]
pub enum WalletError {
    InvalidPrivateKeyString,
    InvalidSignature,
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WalletError::InvalidPrivateKeyString => write!(f, "Invalid Private Key String Error"),
            WalletError::InvalidSignature => write!(f, "Invalid Signature Error"),
        }
    }
}

impl From<FromHexError> for WalletError {
    fn from(_: FromHexError) -> Self {
        WalletError::InvalidSignature
    }
}

impl From<ParseIntError> for WalletError {
    fn from(_: ParseIntError) -> Self {
        WalletError::InvalidSignature
    }
}

impl From<Error> for WalletError {
    fn from(_: Error) -> Self {
        WalletError::InvalidSignature
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bls_signatures::{aggregate, verify, verify_messages};

    #[test]
    fn new_wallet() {
        let wallet = Wallet::new();
        wallet.print();
    }

    //private_key,public_key,address
    const KEYPAIR:(&str,&str,&str) = (
        "0x862fe916208e8f6820c773e290c30ed1f04f2e283644f2ca2668335a3e9f569f",
        "0x807114576f8004aaaebff87f2e631662f1080e4fc07ff70ca7c43d5150fa069f0d3c0e0c30156ef5ebebf14efdbcc915824acc494f90e725e5fad600fd1966df",
        "0x62f6f90ee8955ebec3573841c932c64cedc54a1f"
    );

    #[test]
    fn new_wallet_from_secret_key_string() {
        let wallet = Wallet::from_secret_key_string(KEYPAIR.0.to_string()).unwrap();
        assert_eq!(KEYPAIR.2.to_string(), wallet.address);
    }

    #[test]
    fn test_verify_sign() {
        let message = b"hello world";
        let wallet = Wallet::from_secret_key_string(KEYPAIR.0.to_string()).unwrap();
        let signature = wallet.sign(message.to_vec());
        assert!(wallet.verify(message.to_vec(), signature));
    }

    #[test]
    fn test_verify_sign_by_address() {
        let message = b"hello world";
        let wallet = Wallet::from_secret_key_string(KEYPAIR.0.to_string()).unwrap();
        let signature = wallet.sign(message.to_vec());
        assert!(Wallet::verify_by_address(
            message.to_vec(),
            signature,
            wallet.address
        ));
    }

    #[test]
    fn test_verify_bls_sign() {
        let message = b"hello world";
        let wallet = Wallet::from_secret_key_string(KEYPAIR.0.to_string()).unwrap();
        let signature = wallet.sign_by_bls(message.to_vec());
        assert!(wallet.verify_bls(message.to_vec(), signature));
    }

    #[test]
    fn test_verify_bls_aggregated_sign() {
        let message1 = "hello world1";
        let message2 = "hello world2";
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let mut signatures = Vec::new();
        let mut public_keys = Vec::new();
        let signature1 = wallet1.bls_private_key.sign(message1.as_bytes());
        signatures.push(signature1);
        let signature2 = wallet2.bls_private_key.sign(message2.as_bytes());
        signatures.push(signature2);
        public_keys.push(wallet1.bls_public_key);
        public_keys.push(wallet2.bls_public_key);
        let aggregated_signature = aggregate(&signatures).unwrap();
        let messages = vec![message1.as_bytes(), message2.as_bytes()];
        let result = verify_messages(
            &aggregated_signature,
            messages.as_slice(),
            public_keys.as_slice(),
        );
        assert!(result);
    }
}
