use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 每个槽的指标
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotMetrics {
    pub epoch: u64,
    pub slot: u64,
    pub miner: String,
    pub proposer_stake: f64,
    pub timestamp: u64,
    pub block_hash: String,
    pub tx_count: usize,
    pub path_stats: PathStats,
    pub stake_concentration: f64, // Herfindahl index
    pub gini_coefficient: f64,    // Gini系数，衡量权益分布不平等程度
    pub consensus_type: String,
    pub consensus_state: String, // e.g., "pog(ntd=3)", "pos"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PathStats {
    pub avg_length: f64,
    pub min_length: usize,
    pub max_length: usize,
    pub median_length: usize,
}

/// 每个epoch的指标
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpochMetrics {
    pub epoch: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub block_count: usize,
    pub total_tx_count: usize,
    pub total_tx_throughput: f64, // tx/s
    pub miner_distribution: HashMap<String, usize>,
    pub path_stats: PathStats,
    pub stake_concentration: f64,
    pub gini_coefficient: f64, // Gini系数，衡量权益分布不平等程度
    pub consensus_type: String,
    pub consensus_state: String,
    pub pog_state: Option<PogEpochMetrics>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PogEpochMetrics {
    pub ntd_final: usize,
    pub c_n_stats: ContributionStats,
    pub s_virtual_stats: VirtualStakeStats,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContributionStats {
    pub avg_contribution: f64,
    pub min_contribution: f64,
    pub max_contribution: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VirtualStakeStats {
    pub avg_virtual_stake: f64,
    pub min_virtual_stake: f64,
    pub max_virtual_stake: f64,
}

impl SlotMetrics {
    pub fn to_csv_header() -> String {
        "epoch,slot,miner,proposer_stake,block_hash,tx_count,avg_path_length,\
         min_path_length,max_path_length,median_path_length,stake_concentration,\
         gini_coefficient,consensus_type,consensus_state"
            .to_string()
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{:.6},{},{},{:.2},{},{},{},{:.6},{:.6},{},{}",
            self.epoch,
            self.slot,
            self.miner,
            self.proposer_stake,
            self.block_hash,
            self.tx_count,
            self.path_stats.avg_length,
            self.path_stats.min_length,
            self.path_stats.max_length,
            self.path_stats.median_length,
            self.stake_concentration,
            self.gini_coefficient,
            self.consensus_type,
            self.consensus_state
        )
    }
}

impl EpochMetrics {
    pub fn to_csv_header() -> String {
        "epoch,duration_sec,block_count,total_tx,throughput_tx_per_sec,\
         avg_path_length,min_path_length,max_path_length,median_path_length,\
         stake_concentration,consensus_type,consensus_state,pog_ntd,pog_avg_c_n,\
         pog_min_c_n,pog_max_c_n,pog_avg_s_virtual,pog_min_s_virtual,pog_max_s_virtual"
            .to_string()
    }

    pub fn to_csv_row(&self) -> String {
        let duration = self.end_timestamp.saturating_sub(self.start_timestamp) as f64 / 1000.0;
        let (ntd, avg_c_n, min_c_n, max_c_n, avg_sv, min_sv, max_sv) =
            if let Some(pog) = &self.pog_state {
                (
                    pog.ntd_final.to_string(),
                    format!("{:.6}", pog.c_n_stats.avg_contribution),
                    format!("{:.6}", pog.c_n_stats.min_contribution),
                    format!("{:.6}", pog.c_n_stats.max_contribution),
                    format!("{:.6}", pog.s_virtual_stats.avg_virtual_stake),
                    format!("{:.6}", pog.s_virtual_stats.min_virtual_stake),
                    format!("{:.6}", pog.s_virtual_stats.max_virtual_stake),
                )
            } else {
                (
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A".to_string(),
                )
            };

        format!(
            "{},{:.2},{},{},{:.2},{:.2},{},{},{},{:.6},{},{},{},{},{},{},{},{},{}",
            self.epoch,
            duration,
            self.block_count,
            self.total_tx_count,
            self.total_tx_throughput,
            self.path_stats.avg_length,
            self.path_stats.min_length,
            self.path_stats.max_length,
            self.path_stats.median_length,
            self.stake_concentration,
            self.consensus_type,
            self.consensus_state,
            ntd,
            avg_c_n,
            min_c_n,
            max_c_n,
            avg_sv,
            min_sv,
            max_sv
        )
    }
}

/// 计算Herfindahl index（权益集中度）
pub fn calculate_stake_concentration(stakes: &[f64]) -> f64 {
    if stakes.is_empty() {
        return 0.0;
    }
    let total: f64 = stakes.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let shares: Vec<f64> = stakes.iter().map(|s| s / total).collect();
    shares.iter().map(|s| s * s).sum()
}

/// 计算路径长度统计
pub fn calculate_path_stats(paths: Vec<Vec<String>>) -> PathStats {
    if paths.is_empty() {
        return PathStats {
            avg_length: 0.0,
            min_length: 0,
            max_length: 0,
            median_length: 0,
        };
    }

    let lengths: Vec<usize> = paths.iter().map(|p| p.len()).collect();
    let min_length = *lengths.iter().min().unwrap_or(&0);
    let max_length = *lengths.iter().max().unwrap_or(&0);
    let avg_length = lengths.iter().sum::<usize>() as f64 / lengths.len() as f64;

    let mut sorted_lengths = lengths.clone();
    sorted_lengths.sort_unstable();
    let median_length = sorted_lengths[sorted_lengths.len() / 2];

    PathStats {
        avg_length,
        min_length,
        max_length,
        median_length,
    }
}

/// 计算Gini系数 (Gini coefficient)
/// 用于衡量财富/权益分布的不平等程度
/// 0 = 完全平等, 1 = 完全不平等
pub fn calculate_gini(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let n = values.len() as f64;
    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let sum: f64 = sorted_values.iter().sum();
    if sum == 0.0 {
        return 0.0;
    }

    let cumsum: f64 = sorted_values
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64 + 1.0) * v)
        .sum();

    let gini = (2.0 * cumsum) / (n * sum) - (n + 1.0) / n;
    gini.max(0.0).min(1.0)
}

/// 根据目标Gini系数生成权益分配
/// 返回长度为node_num的权益数组
pub fn generate_stake_by_gini(node_num: u32, target_gini: f64) -> Vec<f64> {
    let n = node_num as usize;
    if n == 0 {
        return vec![];
    }

    // 简单方法：使用指数分布来近似目标Gini
    // target_gini为0时，所有节点权益相等
    // target_gini接近1时，权益高度集中

    let lambda = if target_gini < 0.01 {
        0.0
    } else if target_gini > 0.99 {
        5.0
    } else {
        // 通过二分查找找到合适的lambda
        let mut low = 0.0;
        let mut high = 5.0;
        for _ in 0..20 {
            let mid = (low + high) / 2.0;
            let test_stakes: Vec<f64> = (0..n)
                .map(|i| (-(mid * (i as f64 / n as f64))).exp())
                .collect();
            let gini = calculate_gini(&test_stakes);
            if gini < target_gini {
                low = mid;
            } else {
                high = mid;
            }
        }
        (low + high) / 2.0
    };

    let mut stakes: Vec<f64> = (0..n)
        .map(|i| (-(lambda * (i as f64 / n as f64))).exp())
        .collect();

    // 标准化使总权益为node_num（平均每个节点1单位）
    let sum: f64 = stakes.iter().sum();
    let scale_factor = n as f64 / sum;
    stakes.iter_mut().for_each(|s| *s *= scale_factor);

    stakes
}
