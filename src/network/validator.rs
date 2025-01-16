use crate::wallet;
use crate::wallet::Wallet;
use num_bigint::{BigUint, ToBigUint};
use rand::rngs::{OsRng, StdRng};
use rand::{Rng, RngCore, SeedableRng};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
struct Validator {
    wallet: Wallet,
    stake: u64,
}

impl Validator {
    fn new(wallet: Wallet, stake: u64) -> Self {
        Validator { wallet, stake }
    }
}

#[derive(Debug, Clone)]
struct VdfSeed {
    address: String,
    seed: [u8; 32],
    signature: String,
}

impl VdfSeed {
    fn new(wallet: Wallet) -> Self {
        let seed = VdfSeed::generate_seed();
        let signature = wallet.sign(Vec::from(seed));
        VdfSeed {
            address: wallet.address,
            seed,
            signature,
        }
    }

    fn generate_seed() -> [u8; 32] {
        let mut rng = OsRng::default();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        seed
    }
}

struct Randao {
    vdf_seeds: Vec<VdfSeed>,
}

impl Randao {
    fn new(vdf_seeds: Vec<VdfSeed>) -> Self {
        Randao { vdf_seeds }
    }

    fn combine_seed(&self) -> [u8; 32] {
        let mut result = [0u8; 32];
        for v in self.vdf_seeds.clone() {
            let valid = Wallet::verify_by_address(Vec::from(v.seed), v.signature, v.address);
            if valid {
                for i in 0..32 {
                    result[i] ^= v.seed[i];
                }
            } else {
                println!("Randao combine seed warning:invalid seed");
            }
        }
        result
    }
    fn weighted_random_selection(&self, validators: Vec<Validator>) -> Validator {
        // 加权选择一个 出块者
        // 计算总的权重
        let total_stake: u64 = validators.iter().map(|v| v.stake).sum();

        // 使用combine seed
        let seed = self.combine_seed();
        let mut rng = StdRng::from_seed(seed);

        let random_value = rng.gen_range(0..total_stake);

        // 选择对应的验证者
        let mut accumulated_weight = 0;
        for validator in validators.clone() {
            accumulated_weight += validator.stake;
            if accumulated_weight > random_value {
                return validator;
            }
        }

        // 如果没有找到验证者，返回第一个（理论上不应该发生）
        validators[0].clone()
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

    #[test]
    fn randao() {
        let mut vdf_seeds: Vec<VdfSeed> = Vec::new();
        for _ in 0..5 {
            vdf_seeds.push(VdfSeed::new(Wallet::new()));
        }
        let randao = Randao::new(vdf_seeds);
        let result = randao.combine_seed();
        println!("Randao: {:?}", result);
    }

    #[test]
    fn vdf() {
        let seed: [u8; 32] = [1; 32];
        println!("seed: {:?}", seed);

        // 控制计算延迟，实际应更高
        let difficulty = 10_000;

        // 使用 VDF 计算延迟函数
        let (vdf_result, modulus) = simple_vdf(&seed, difficulty);

        // 显示 VDF 结果和模数
        println!("VDF Output: {}", vdf_result);
        println!("Modulus Used: {}", modulus);
    }

    #[test]
    fn select() {
        let mut validator_list: Vec<Validator> = Vec::new();
        for _ in 0..5 {
            let wallet = Wallet::new();
            validator_list.push(Validator::new(wallet, 32));
        }
        let mut vdf_seeds: Vec<VdfSeed> = Vec::new();
        for v in validator_list.clone() {
            vdf_seeds.push(VdfSeed::new(v.wallet));
        }
        let randao = Randao::new(vdf_seeds);
        let validator = randao.weighted_random_selection(validator_list);
        println!("winner: {:#?}", validator);
    }
}
