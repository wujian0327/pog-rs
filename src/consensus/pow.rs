use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::{Consensus, Validator, ValidatorError};
use log::{info, trace, warn};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Proof-of-Work 共识
/// 基于计算难度的共识机制，proposer 需要完成特定的计算工作来赢得出块权
#[derive(Debug, Clone)]
pub struct PowConsensus {
    /// 当前难度目标（leading zeros 的数量）
    difficulty: usize,
    /// 当前 epoch 的块数（用于判断是否需要调整难度）
    blocks_in_epoch: usize,
    max_threads: usize,
    slot_duration: Duration,
}

impl PowConsensus {
    /// 创建新的 PoW 共识实例
    pub fn new(initial_difficulty: usize, max_threads: usize, slot_duration: Duration) -> Self {
        PowConsensus {
            difficulty: initial_difficulty,
            blocks_in_epoch: 0,
            max_threads,
            slot_duration,
        }
    }

    /// 验证工作量证明
    /// 检查 hash 是否满足难度要求（leading zeros）
    fn verify_pow(hash: &[u8], difficulty: usize) -> bool {
        // 检查前 difficulty 位是否为 0
        for i in 0..difficulty {
            let byte_index = i / 8;
            let bit_index = 7 - (i % 8);

            if byte_index >= hash.len() {
                return false;
            }

            let bit = (hash[byte_index] >> bit_index) & 1;
            if bit != 0 {
                return false;
            }
        }
        true
    }

    /// 计算所需的工作量（Hashes attempted）
    /// 难度为 d 时，平均需要 2^d 次哈希尝试
    fn compute_work_amount(difficulty: usize) -> f64 {
        2_f64.powi(difficulty as i32)
    }

    /// 进行 PoW 计算，返回满足难度要求的 nonce 和对应的 hash
    fn mine_pow(data: &[u8], difficulty: usize, max_attempts: u64) -> Option<(u64, Vec<u8>)> {
        for nonce in 0..max_attempts {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.update(nonce.to_le_bytes());
            let hash = hasher.finalize();
            let hash_bytes = hash.to_vec();

            if Self::verify_pow(&hash_bytes, difficulty) {
                return Some((nonce, hash_bytes));
            }
        }
        None
    }

    /// 动态调整难度（每个 epoch 调整一次）
    /// 基于 epoch 内的块生成时间
    fn adjust_difficulty(&mut self, blocks: &[Block]) {
        if blocks.is_empty() {
            return;
        }

        // 计算整个 epoch 的平均块时间
        let first_time = blocks.first().unwrap().header.timestamp;
        let last_time = blocks.last().unwrap().header.timestamp;
        let time_diff = if last_time > first_time {
            last_time - first_time
        } else {
            1
        };

        let avg_block_time = time_diff / (blocks.len() as u64);
        let target_block_time = self.slot_duration.as_secs();

        // 根据实际块时间调整难度
        if avg_block_time < target_block_time {
            // 块生成太快，增加难度
            self.difficulty = self.difficulty.saturating_add(1);
            info!(
                "PoW: Difficulty increased to {} (avg block time: {}s)",
                self.difficulty, avg_block_time
            );
        } else {
            // 块生成太慢，降低难度
            self.difficulty = self.difficulty.saturating_sub(1);
            info!(
                "PoW: Difficulty decreased to {} (avg block time: {}s)",
                self.difficulty, avg_block_time
            );
        }

        self.blocks_in_epoch = 0;
    }
}

impl Consensus for PowConsensus {
    fn name(&self) -> &'static str {
        "pow"
    }

    fn select_proposer(
        &mut self,
        validators: &[Validator],
        combines_seed: [u8; 32],
        _blockchain: &Blockchain,
    ) -> Result<Validator, ValidatorError> {
        if validators.is_empty() {
            return Err(ValidatorError::NOValidatorError);
        }

        // 如果只有一个验证者，直接返回
        if validators.len() == 1 {
            return Ok(validators[0].clone());
        }

        // 多线程 PoW 竞争：所有验证者并行计算，第一个找到结果的胜利
        let winner = Arc::new(Mutex::new(None::<Validator>));
        let mut handles = vec![];

        let max_attempts = 1_000_000u64; // 每个线程最多尝试 100 万次

        // 限制最大线程数
        let num_threads = std::cmp::min(validators.len(), self.max_threads);
        let thread_step = (validators.len() + num_threads - 1) / num_threads; // 向上取整

        for chunk in validators.chunks(thread_step) {
            for validator in chunk {
                let validator_clone = validator.clone();
                let winner_clone = Arc::clone(&winner);
                let difficulty = self.difficulty;
                let seed = combines_seed;

                let handle = thread::spawn(move || {
                    // 这里只是模拟pow运算，并没有使用节点的交易数据
                    // this is just a simulation of PoW mining without using the node's transaction data
                    let mut mining_data = Vec::new();
                    mining_data.extend_from_slice(&seed);
                    mining_data.extend_from_slice(&validator_clone.address.as_bytes());

                    // 开始 PoW 计算
                    for nonce in 0..max_attempts {
                        // 检查是否已有获胜者，若有则提前退出
                        if winner_clone.lock().unwrap().is_some() {
                            return;
                        }

                        let mut hasher = Sha256::new();
                        hasher.update(&mining_data);
                        hasher.update(nonce.to_le_bytes());
                        let hash = hasher.finalize();
                        let hash_bytes = hash.to_vec();

                        // 验证是否满足难度要求
                        if Self::verify_pow(&hash_bytes, difficulty) {
                            // 当前验证者找到了结果，尝试设置为获胜者
                            let mut winner_guard = winner_clone.lock().unwrap();
                            if winner_guard.is_none() {
                                *winner_guard = Some(validator_clone.clone());
                                info!(
                                    "PoW: Validator {} won with nonce {}",
                                    validator_clone.address, nonce
                                );
                            }
                            return;
                        }
                    }
                });

                handles.push(handle);
            }
        }

        // 等待所有线程完成或找到获胜者
        for handle in handles {
            let _ = handle.join();
        }

        // 获取获胜者
        {
            let winner_guard = winner.lock().unwrap();
            match winner_guard.clone() {
                Some(validator) => {
                    info!("PoW proposer selected: {}", validator.address);
                    Ok(validator)
                }
                None => {
                    // 如果没有找到获胜者（超过最大尝试次数），返回第一个验证者
                    warn!("PoW: No winner found, selecting first validator as fallback");
                    Ok(validators[0].clone())
                }
            }
        }
    }

    fn on_epoch_end(&mut self, blocks: &[Block]) {
        // 在 epoch 结束时调整难度
        self.adjust_difficulty(blocks);
    }

    fn state_summary(&self) -> String {
        format!(
            "pow(difficulty={}, work_amount={:.0})",
            self.difficulty,
            Self::compute_work_amount(self.difficulty)
        )
    }

    fn distribute_rewards(
        &self,
        block: &Block,
        validators: &mut [Validator],
        nodes_index: HashMap<String, u32>,
    ) {
        // PoW: 固定奖励 + 交易费用
        if let Some(validator) = validators
            .iter_mut()
            .find(|v| v.address == block.header.miner)
        {
            let base_reward = 1.0; // 固定奖励
            let tx_fees: f64 = block.body.transactions.iter().map(|tx| tx.fee).sum();
            let total_reward = base_reward + tx_fees;
            validator.stake += total_reward;
            info!(
                "PoW: Miner {} received reward: base={:.6} + fees={:.6} = {:.6}, new stake: {:.6}",
                validator.address, base_reward, tx_fees, total_reward, validator.stake
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_verification() {
        // 创建测试 hash：0x00 0x00 0xFF 0xFF
        let hash = vec![0x00u8, 0x00, 0xFF, 0xFF];

        // 16 位前导零应该通过
        assert!(PowConsensus::verify_pow(&hash, 16));

        // 17 位前导零应该失败
        assert!(!PowConsensus::verify_pow(&hash, 17));

        // 0 位应该总是通过
        assert!(PowConsensus::verify_pow(&hash, 0));
    }

    #[test]
    fn test_work_amount() {
        let work_1 = PowConsensus::compute_work_amount(1);
        let work_10 = PowConsensus::compute_work_amount(10);

        // 难度增加 9，工作量应该增加 2^9
        assert!(work_10 >= work_1 * 512.0);
    }

    #[test]
    fn test_mine_pow() {
        let data = b"test data for PoW mining";
        let result = PowConsensus::mine_pow(data, 2, 100_000);
        assert!(result.is_some());

        let (_nonce, hash) = result.unwrap();
        // 验证找到的 nonce 确实满足难度要求
        assert!(PowConsensus::verify_pow(&hash, 2));
    }
}
