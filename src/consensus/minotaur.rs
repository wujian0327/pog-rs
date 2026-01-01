use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::{Consensus, Validator, ValidatorError};
use log::{debug, info};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// PoW块数据结构：存储节点在某个slot的PoW计算结果
#[derive(Debug, Clone)]
pub struct PowBlock {
    /// 节点地址
    pub address: String,
    /// 在该slot中计算得到的哈希值数量（代表算力）
    pub hash_count: u64,
    /// slot编号
    pub slot: u64,
    /// nonce值（用于验证工作量）
    pub nonce: u64,
}

/// Minotaur共识：Multi-Resource Blockchain Consensus
/// 结合了PoW算力证明和PoS权益证明的混合共识机制
/// 核心原则：所有计算都是有效的，直接累积计算工作量，不需要满足难度目标
/// 每个slot中，节点进行PoW运算并存储计算结果到pow_blocks中
#[derive(Debug, Clone)]
pub struct MinotaurConsensus {
    /// 存储每个slot的PoW块数据：key为slot编号
    pow_blocks: HashMap<u64, Vec<PowBlock>>,
    /// base reward for block production
    base_reward: f64,
    /// PoW与PoS的权重参数（0-1之间），0表示纯PoS，1表示纯PoW
    pow_weight: f64,
}

impl MinotaurConsensus {
    /// 创建新的Minotaur共识实例
    pub fn new(base_reward: f64) -> Self {
        MinotaurConsensus {
            pow_blocks: HashMap::new(),
            base_reward,
            pow_weight: 0.5, // 默认50%权重
        }
    }

    /// 进行PoW计算（Minotaur风格）：累积所有计算尝试
    /// 论文中不需要验证难度，而是直接计算总的哈希尝试次数
    /// 返回进行的总尝试次数（即工作量）
    pub fn perform_pow_computation(&self, address: &str, slot: u64, max_attempts: u64) -> u64 {
        // Minotaur: 所有计算都计数，不需要验证难度
        // 节点可以根据自己的算力进行任意数量的计算
        // 返回实际进行的计算次数（工作量）

        let mut hash_count = 0u64;
        for nonce in 0..max_attempts {
            hash_count += 1;
            let mut hasher = Sha256::new();
            hasher.update(address.as_bytes());
            hasher.update(slot.to_le_bytes());
            hasher.update(nonce.to_le_bytes());
            let _hash = hasher.finalize(); // 计算但不校验难度
        }
        hash_count
    }

    /// 添加PoW块到指定slot
    pub fn add_pow_block(&mut self, block: PowBlock) {
        self.pow_blocks
            .entry(block.slot)
            .or_insert_with(Vec::new)
            .push(block);
    }

    /// 获取某个slot的PoW块列表
    pub fn get_pow_blocks(&self, slot: u64) -> Vec<PowBlock> {
        self.pow_blocks.get(&slot).cloned().unwrap_or_default()
    }

    /// 清理旧的PoW块（保留最近N个slot的数据）
    pub fn cleanup_old_pow_blocks(&mut self, keep_slots: u64) {
        if self.pow_blocks.is_empty() {
            return;
        }

        let max_slot = *self.pow_blocks.keys().max().unwrap();
        let min_slot_to_keep = max_slot.saturating_sub(keep_slots);

        self.pow_blocks.retain(|slot, _| *slot >= min_slot_to_keep);
    }

    /// 计算节点的PoW贡献分数（基于平均hash数量）
    fn calculate_pow_score(&self, address: &str, _slots_to_check: u64) -> f64 {
        let mut total_hashes = 0u64;
        let mut count = 0u64;

        for pow_block in self.pow_blocks.values().flat_map(|v| v.iter()) {
            if pow_block.address == address {
                total_hashes += pow_block.hash_count;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        (total_hashes as f64) / (count as f64)
    }

    /// 混合选择proposer：基于PoW贡献和PoS权益
    fn select_with_hybrid_approach(
        &self,
        validators: Vec<Validator>,
        combines_seed: [u8; 32],
    ) -> Result<Validator, ValidatorError> {
        if validators.is_empty() {
            return Err(ValidatorError::NOValidatorError);
        }

        // 计算每个验证者的混合权重
        let total_stake: f64 = validators.iter().map(|v| v.stake).sum();
        let mut hybrid_weights: HashMap<String, f64> = HashMap::new();

        // 计算PoS权重（基于stake）
        for validator in &validators {
            let pos_weight = if total_stake > 0.0 {
                validator.stake / total_stake
            } else {
                1.0 / validators.len() as f64
            };

            // 计算PoW权重（基于计算贡献）
            let pow_score = self.calculate_pow_score(&validator.address, 10);
            let total_pow_score: f64 = validators
                .iter()
                .map(|v| self.calculate_pow_score(&v.address, 10))
                .sum();
            let pow_weight = if total_pow_score > 0.0001 {
                pow_score / total_pow_score
            } else {
                1.0 / validators.len() as f64
            };

            // 混合权重：加权平均
            let hybrid_weight = (1.0 - self.pow_weight) * pos_weight + self.pow_weight * pow_weight;
            hybrid_weights.insert(validator.address.clone(), hybrid_weight);

            debug!(
                "Validator {} - PoS: {:.6}, PoW: {:.6}, Hybrid: {:.6}",
                validator.address, pos_weight, pow_weight, hybrid_weight
            );
        }

        // 基于混合权重进行概率选择
        let total_hybrid_weight: f64 = hybrid_weights.values().sum();
        let mut rng = StdRng::from_seed(combines_seed);
        let random_value = rng.gen_range(0.0..total_hybrid_weight.max(0.0001));

        let mut accumulated_weight = 0.0;
        for validator in validators {
            let weight = hybrid_weights.get(&validator.address).unwrap_or(&0.0);
            accumulated_weight += weight;
            if accumulated_weight >= random_value {
                info!(
                    "Minotaur: Proposer {} elected with hybrid weight {:.6}",
                    validator.address, weight
                );
                return Ok(validator);
            }
        }

        Err(ValidatorError::NOValidatorError)
    }
}

impl Consensus for MinotaurConsensus {
    fn name(&self) -> &'static str {
        "Minotaur"
    }

    fn select_proposer(
        &mut self,
        validators: &[Validator],
        combines_seed: [u8; 32],
        _blockchain: &Blockchain,
    ) -> Result<Validator, ValidatorError> {
        self.select_with_hybrid_approach(validators.to_vec(), combines_seed)
    }

    fn on_epoch_end(&mut self, _blocks: &[Block]) {
        // 清理旧的PoW块数据，只保留最近10个slot
        self.cleanup_old_pow_blocks(10);
    }

    fn state_summary(&self) -> String {
        format!("minotaur(pow_w:{:.2})", self.pow_weight)
    }

    fn distribute_rewards(
        &self,
        block: &Block,
        validators: &mut [Validator],
        _nodes_index: HashMap<String, u32>,
    ) {
        // Minotaur: 基础奖励 + 交易费用
        if let Some(validator) = validators
            .iter_mut()
            .find(|v| v.address == block.header.miner)
        {
            let base_reward = self.base_reward;
            let tx_fees: f64 = block.body.transactions.iter().map(|tx| tx.fee).sum();
            let total_reward = base_reward + tx_fees;
            validator.stake += total_reward;
            info!(
                "Minotaur: Miner {} received reward: base={:.6} + fees={:.6} = {:.6}, new stake: {:.6}",
                validator.address, base_reward, tx_fees, total_reward, validator.stake
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_computation_counts_all_attempts() {
        // 测试PoW计算计数所有尝试（不过滤难度）
        let consensus = MinotaurConsensus::new(1.0);
        let hash_count = consensus.perform_pow_computation("node1", 0, 100);
        // 所有100次尝试都应该被计数
        assert_eq!(hash_count, 100);
    }

    #[test]
    fn test_pow_computation_zero_attempts() {
        let consensus = MinotaurConsensus::new(1.0);
        let hash_count = consensus.perform_pow_computation("node1", 0, 0);
        assert_eq!(hash_count, 0);
    }

    #[test]
    fn test_pow_block_management() {
        let mut consensus = MinotaurConsensus::new(1.0);
        let block = PowBlock {
            address: "node1".to_string(),
            hash_count: 1000,
            slot: 0,
            nonce: 42,
        };
        consensus.add_pow_block(block.clone());

        let blocks = consensus.get_pow_blocks(0);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].address, "node1");
    }
}
