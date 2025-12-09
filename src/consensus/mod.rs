use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::network::node::Node;
use crate::tools;
use crate::wallet::Wallet;
use clap::ValueEnum;
use log::error;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod pog;
pub mod pos;
pub mod pow;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ConsensusType {
    POS,
    POG,
    POW,
}

impl Display for ConsensusType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ConsensusType::POS => {
                write!(f, "pos")
            }
            ConsensusType::POG => {
                write!(f, "pog")
            }
            ConsensusType::POW => {
                write!(f, "pow")
            }
        }
    }
}

pub trait Consensus: Send + Sync {
    fn name(&self) -> &'static str;
    fn select_proposer(
        &mut self,
        validators: &[Validator],
        combines_seed: [u8; 32],
        blockchain: &Blockchain,
    ) -> Result<Validator, ValidatorError>;
    fn on_epoch_end(&mut self, blocks: &[Block]);
    fn apply_block_feedback(&mut self, _block: &Block) {}
    fn state_summary(&self) -> String {
        String::new()
    }

    /// 分配区块奖励给验证者
    ///
    /// # 参数
    /// * `block` - 单个区块
    /// * `validators` - 所有验证者的可变引用，奖励会直接加到 stake 中
    ///
    /// # 说明
    /// - 默认实现不做任何操作，具体共识算法可覆盖此方法
    /// - 奖励应该直接加到相应验证者的 stake 字段中
    fn distribute_rewards(
        &self,
        _block: &Block,
        _validators: &mut [Validator],
        _nodes_index: HashMap<String, u32>,
    ) {
    }
}

pub fn combine_seed(validators: Vec<Validator>, vdf_seeds: Vec<RandaoSeed>) -> [u8; 32] {
    let mut result = [0u8; 32];
    for v in vdf_seeds.clone() {
        if !validators
            .iter()
            .any(|validator| validator.address.eq(&v.address))
        {
            error!("Randao combine seed warning: this seed is not from validators");
            continue;
        }
        let valid = Wallet::verify_by_address(Vec::from(v.seed), v.signature, v.address);
        if valid {
            for i in 0..32 {
                result[i] ^= v.seed[i];
            }
        } else {
            error!("Randao combine seed warning: invalid seed");
        }
    }
    tools::Hasher::hash(Vec::from(result))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: String,
    pub stake: f64,
}

impl Validator {
    pub fn new(address: String, stake: f64) -> Self {
        Validator { address, stake }
    }

    pub fn from_node(node: Node, stake: f64) -> Self {
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

            ValidatorError::NOValidatorError => {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RandaoSeed {
    pub address: String,
    pub seed: [u8; 32],
    pub signature: String,
}

impl RandaoSeed {
    pub fn new(wallet: Wallet) -> Self {
        let seed = RandaoSeed::generate_seed();
        let signature = wallet.sign(Vec::from(seed));
        RandaoSeed {
            address: wallet.address,
            seed,
            signature,
        }
    }

    pub fn generate_seed() -> [u8; 32] {
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

/// ============================================================================
/// 手续费机制（Fee Mechanism）
/// 根据论文设计：矿工和网络节点按照路径长度惩罚因子分享交易手续费
/// ============================================================================
use std::collections::HashMap;

/// 根据平均路径长度计算惩罚因子P(B)
/// P(B) = 1 如果 L ≤ NTD，否则 P(B) = (NTD/L)^2
///
/// # 参数
/// * `avg_path_length` - 块中交易的平均路径长度
/// * `ntd` - 当前的网络传输难度（Network Transmission Difficulty）
///
/// # 返回
/// 惩罚因子，范围 (0, 1]
///
/// # 设计原理
/// - 当路径长度在目标范围内时，没有惩罚（P=1）
/// - 当路径长度超过目标时，使用二次方惩罚以强烈激励优化
pub fn calculate_penalty_factor(avg_path_length: f64, ntd: usize) -> f64 {
    let ntd_f = ntd as f64;
    if avg_path_length <= ntd_f {
        1.0
    } else {
        let ratio = ntd_f / avg_path_length;
        ratio * ratio
    }
}

/// 计算矿工获得的手续费份额
///
/// F_miner(B) = 0.5 * F(B) * P(B)
///
/// # 参数
/// * `total_fees` - 块中所有交易的总手续费
/// * `penalty_factor` - 根据平均路径长度计算的惩罚因子
///
/// # 返回
/// 矿工可获得的手续费金额
///
/// # 设计原理
/// - 矿工基础获得50%的手续费
/// - 但如果路径太长（L > NTD），这一部分会被二次方惩罚
/// - 被惩罚的部分会重定向到网络节点
pub fn calculate_miner_fee_share(total_fees: f64, penalty_factor: f64) -> f64 {
    0.5 * total_fees * penalty_factor
}

/// 计算网络节点池获得的手续费
///
/// F_net(B) = F(B) * (1 - 0.5 * P(B))
///
/// # 参数
/// * `total_fees` - 块中所有交易的总手续费
/// * `penalty_factor` - 根据平均路径长度计算的惩罚因子
///
/// # 返回
/// 分配给网络传播节点的手续费池总额
///
/// # 设计原理
/// - 网络基础获得50%的手续费
/// - 当矿工因路径长度被惩罚时，惩罚部分会转给网络
/// - 这形成了零和的费用重分配，激励更优的传播
pub fn calculate_network_fee_pool(total_fees: f64, penalty_factor: f64) -> f64 {
    total_fees * (1.0 - 0.5 * penalty_factor)
}

/// 在网络参与节点中按虚拟权益分配费用池
///
/// # 参数
/// * `network_fee_pool` - 待分配的网络手续费池
/// * `participating_validators` - 参与交易传播的验证者列表，包含(地址, 虚拟权益)
///
/// # 返回
/// 一个HashMap，键为验证者地址，值为该验证者获得的手续费份额
///
/// # 设计原理
/// - 只有实际参与交易传播的节点才能获得网络手续费
/// - 分配比例与虚拟权益成正比
/// - 虚拟权益包含了实际权益和贡献度，形成了统一的激励
pub fn distribute_network_fees(
    network_fee_pool: f64,
    participating_validators: &[(String, f64)], // (address, virtual_stake)
) -> HashMap<String, f64> {
    let mut rewards = HashMap::new();

    if participating_validators.is_empty() {
        return rewards;
    }

    let total_virtual_stake: f64 = participating_validators.iter().map(|(_, vs)| vs).sum();

    if total_virtual_stake <= 0.0 {
        // 边界情况：虚拟权益为0，平均分配
        let reward_per_node = network_fee_pool / participating_validators.len() as f64;
        for (address, _) in participating_validators {
            rewards.insert(address.clone(), reward_per_node);
        }
    } else {
        // 正常情况：按虚拟权益比例分配
        for (address, virtual_stake) in participating_validators {
            let reward = network_fee_pool * (virtual_stake / total_virtual_stake);
            rewards.insert(address.clone(), reward);
        }
    }

    rewards
}

/// 计算矿工的总出块奖励
///
/// # 参数
/// * `block_subsidy` - 区块补贴（固定的区块奖励）
/// * `miner_fee_share` - 矿工获得的交易手续费份额
///
/// # 返回
/// 矿工的总收益（区块补贴 + 手续费份额）
pub fn calculate_miner_total_reward(block_subsidy: f64, miner_fee_share: f64) -> f64 {
    block_subsidy + miner_fee_share
}
