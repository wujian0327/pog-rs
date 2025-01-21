use chrono::Local;
use sha3::{Digest, Sha3_256};
use std::time::SystemTime;

pub struct Hasher {}

impl Hasher {
    pub fn hash(data: Vec<u8>) -> [u8; 32] {
        //sha3
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let hash_result = hasher.finalize();
        <[u8; 32]>::from(hash_result)
    }
}

pub fn get_timestamp() -> u64 {
    let now = SystemTime::now();

    now.duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn get_time_string() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}
