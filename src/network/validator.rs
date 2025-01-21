use crate::network::node::Node;
use crate::network::validator::ValidatorError::NOValidatorError;
use crate::wallet::Wallet;
use log::info;
use num_bigint::{BigUint, ToBigUint};
use rand::rngs::{OsRng, StdRng};
use rand::{Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: String,
    pub stake: u64,
}

impl Validator {
    pub fn new(address: String, stake: u64) -> Self {
        Validator { address, stake }
    }

    pub fn from_node(node: Node, stake: u64) -> Self {
        Validator::new(node.wallet.address.clone(), stake)
    }

    pub fn from_json(json: Vec<u8>) -> Result<Validator, ValidatorError> {
        let randao_seed: Validator = serde_json::from_slice(json.as_slice())?;
        Ok(randao_seed)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RandaoSeed {
    pub address: String,
    pub seed: [u8; 32],
    pub signature: String,
}

impl RandaoSeed {
    fn new(wallet: Wallet) -> Self {
        let seed = RandaoSeed::generate_seed();
        let signature = wallet.sign(Vec::from(seed));
        RandaoSeed {
            address: wallet.address,
            seed,
            signature,
        }
    }

    pub(crate) fn generate_seed() -> [u8; 32] {
        let mut rng = OsRng;
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        seed
    }

    pub fn from_json(json: Vec<u8>) -> Result<RandaoSeed, ValidatorError> {
        let randao_seed: RandaoSeed = serde_json::from_slice(json.as_slice())?;
        Ok(randao_seed)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug)]
pub enum ValidatorError {
    JSONError,
    NOValidatorError,
}
impl fmt::Display for ValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValidatorError::JSONError => {
                write!(f, "Invalid Json Error")
            }

            NOValidatorError => {
                write!(f, "NoValidatorError")
            }
        }
    }
}
impl From<serde_json::error::Error> for ValidatorError {
    fn from(_: serde_json::error::Error) -> Self {
        ValidatorError::JSONError
    }
}

pub struct Randao {
    vdf_seeds: Vec<RandaoSeed>,
    validators: Vec<Validator>,
}

impl Randao {
    pub fn new(vdf_seeds: Vec<RandaoSeed>, validators: Vec<Validator>) -> Self {
        Randao {
            vdf_seeds,
            validators,
        }
    }

    pub fn combine_seed(&self) -> [u8; 32] {
        let mut result = [0u8; 32];
        for v in self.vdf_seeds.clone() {
            if !self
                .validators
                .iter()
                .any(|validator| validator.address.eq(&v.address))
            {
                info!("Randao combine seed warning: this seed is not from validators");
                continue;
            }
            let valid = Wallet::verify_by_address(Vec::from(v.seed), v.signature, v.address);
            if valid {
                for i in 0..32 {
                    result[i] ^= v.seed[i];
                }
            } else {
                info!("Randao combine seed warning: invalid seed");
            }
        }
        result
    }
    pub fn weighted_random_selection(&self) -> Result<Validator, ValidatorError> {
        if self.validators.is_empty() {
            return Err(NOValidatorError);
        }
        // 计算总的权重
        let total_stake: u64 = self.validators.iter().map(|v| v.stake).sum();

        // 使用combine seed
        let seed = self.combine_seed();
        let mut rng = StdRng::from_seed(seed);

        let random_value = rng.gen_range(0..total_stake);

        // 选择符合条件的第一个验证者
        let mut accumulated_weight = 0;
        for validator in self.validators.clone() {
            accumulated_weight += validator.stake;
            if accumulated_weight > random_value {
                return Ok(validator);
            }
        }

        Err(NOValidatorError)
    }
}
fn simple_vdf(seed: &[u8; 32], difficulty: u64) -> (BigUint, BigUint) {
    // 1. 将种子转换为大整数
    let seed_int = BigUint::from_bytes_be(seed);

    // 2. 使用素数作为模数（RSA VDF 通常使用大的安全素数）
    let modulus = BigUint::from_bytes_be(&[
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // 示例 256 位模数
    ]);

    // 3. 设定基础值为 2，反复计算 mod
    let base = 2.to_biguint().unwrap();
    let mut output = seed_int.clone();
    for _ in 0..difficulty {
        output = output.modpow(&base, &modulus);
    }

    // 输出延迟结果和证明（模数本例中不变）
    (output.clone(), modulus)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::block::Block;
    use crate::blockchain::blockchain::Blockchain;

    #[test]
    fn randao() {
        let mut vdf_seeds: Vec<RandaoSeed> = Vec::new();
        for _ in 0..5 {
            vdf_seeds.push(RandaoSeed::new(Wallet::new()));
        }
        let randao = Randao::new(vdf_seeds, vec![]);
        let result = randao.combine_seed();
        info!("Randao: {:?}", result);
    }

    #[test]
    fn vdf() {
        let seed: [u8; 32] = [1; 32];
        info!("seed: {:?}", seed);

        // 控制计算延迟，实际应更高
        let difficulty = 10_000;

        // 使用 VDF 计算延迟函数
        let (vdf_result, modulus) = simple_vdf(&seed, difficulty);

        // 显示 VDF 结果和模数
        info!("VDF Output: {}", vdf_result);
        info!("Modulus Used: {}", modulus);
    }

    #[test]
    fn select() {
        let blockchain = Blockchain::new(Block::gen_genesis_block());
        let (world_sender, _) = tokio::sync::mpsc::channel(8);
        let node0 = Node::new(0, 0, 1, blockchain.clone(), world_sender.clone());
        let node1 = Node::new(0, 0, 1, blockchain.clone(), world_sender.clone());
        let node2 = Node::new(0, 0, 1, blockchain.clone(), world_sender.clone());
        let node3 = Node::new(0, 0, 1, blockchain.clone(), world_sender.clone());
        let node4 = Node::new(0, 0, 1, blockchain.clone(), world_sender.clone());

        let mut validator_list: Vec<Validator> = Vec::new();
        validator_list.push(Validator::new(node0.wallet.clone().address, 32));
        validator_list.push(Validator::new(node1.wallet.clone().address, 32));
        validator_list.push(Validator::new(node2.wallet.clone().address, 32));
        validator_list.push(Validator::new(node3.wallet.clone().address, 32));
        validator_list.push(Validator::new(node4.wallet.clone().address, 32));

        let mut vdf_seeds: Vec<RandaoSeed> = Vec::new();
        vdf_seeds.push(RandaoSeed::new(node0.wallet));
        vdf_seeds.push(RandaoSeed::new(node1.wallet));
        vdf_seeds.push(RandaoSeed::new(node2.wallet));
        vdf_seeds.push(RandaoSeed::new(node3.wallet));
        vdf_seeds.push(RandaoSeed::new(node4.wallet));

        let randao = Randao::new(vdf_seeds, validator_list);
        let validator = randao.weighted_random_selection();
        info!("winner: {:#?}", validator);
    }
}
