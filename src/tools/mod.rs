use sha3::{Digest, Sha3_256};

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
