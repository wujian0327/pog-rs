# PoW/PoS/PoG 区块链共识对比框架

一个完整的、可扩展的区块链共识对比研究框架，支持 Proof-of-Work (PoW)、Proof-of-Stake (PoS) 和 Proof-of-Gossip (PoG) 三种共识机制。

## 🎯 项目特性

- ✅ **三种共识机制** - PoW、PoS、PoG 完整实现
- ✅ **结构化指标** - 20+ 个关键性能指标
- ✅ **自动化分析** - 4 个 Python 分析工具
- ✅ **完整文档** - 学术级别的详细文档
- ✅ **易于扩展** - 模块化架构支持新共识

## ⚡ 快速开始 (5 分钟)

### 前置要求

- Rust 1.70+ ([安装 Rust](https://www.rust-lang.org/tools/install))
- Python 3.8+ (可选，用于分析)

### 一键运行

```bash
# 编译项目
cargo build --release

# 自动运行所有对比实验
python quick_start.py
```

这会：
1. 编译项目
2. 运行 PoS、PoG、PoW 各 5 个 epoch
3. 生成对比数据和图表
4. 执行自动分析

### 手工运行

```bash
# 运行 Proof-of-Stake
cargo run --release -- -n 100 -t 10 -c pos

# 运行 Proof-of-Gossip  
cargo run --release -- -n 100 -t 10 -c pog

# 运行 Proof-of-Work (新增)
cargo run --release -- -n 100 -t 10 -c pow
```

**参数说明：**
- `-n` : 验证者数量 (默认: 10)
- `-t` : 总 epoch 数 (默认: 1)
- `-c` : 共识类型 (pos/pog/pow)

## 📊 自动生成的数据

每次运行产生两个 CSV 文件，包含结构化指标：

**metrics_slots.csv** - 每个 slot 的详细数据
- epoch, slot, miner, proposer_stake
- tx_count, stake_concentration
- consensus_type, consensus_state

**metrics_epochs.csv** - 每个 epoch 的聚合数据
- block_count, total_tx, throughput (tx/s)
- miner_distribution
- stake_concentration (公平性指标)

## 📈 分析对比

### 快速对比（推荐）
```bash
python python/quick_compare.py
```

### 详细对比
```bash
python python/consensus_comparison.py
```

### 单个分析
```bash
python python/analyze_metrics.py
```

## 📚 文档指南

| 文档 | 适合人群 | 内容长度 |
|-----|---------|---------|
| [CONSENSUS_QUICKSTART.md](CONSENSUS_QUICKSTART.md) | 初学者 | 5分钟 |
| [CONSENSUS_COMPARISON.md](CONSENSUS_COMPARISON.md) | 研究者 | 详细 |
| [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) | 开发者 | 技术 |
| [PROJECT_COMPLETION.md](PROJECT_COMPLETION.md) | 管理者 | 总结 |

## 🔬 共识机制对比

| 特性 | PoS | PoG | PoW |
|------|-----|-----|-----|
| **能耗** | 低 ✓ | 低 ✓ | 高 ✗ |
| **公平性** | 中 | 高 ✓ | 中 |
| **去中心化** | 中 | 高 ✓ | 高 ✓ |
| **吞吐量** | 稳定 ✓ | 动态 | 波动 |

### Proof-of-Stake (PoS)
- 基于权益比例的随机选择
- 选择完全公平，与网络无关
- 能源效率最高

### Proof-of-Gossip (PoG)
- 基于网络贡献度的自适应共识
- 动态调整网络参数 (ntd)
- 激励更好的网络参与

### Proof-of-Work (PoW)
- 基于计算难度的竞争
- 自动难度调整
- 完全去中心化

## 🧪 研究应用

### 公平性研究
```bash
./target/release/pog -n 100 -t 50 -c pos
python python/analyze_metrics.py
```

### 性能对比
```bash
# 运行三个实验并对比
python python/consensus_comparison.py
```

### 参数影响分析
```bash
# 测试不同验证者数量的影响
./target/release/pog -n 50 -t 20 -c pos
./target/release/pog -n 100 -t 20 -c pos
./target/release/pog -n 200 -t 20 -c pos
```

## 📁 项目结构

```
src/
├── consensus/
│   ├── mod.rs         # Consensus trait
│   ├── pos.rs         # PoS 实现
│   ├── pog.rs         # PoG 实现
│   └── pow.rs         # PoW 实现 ✨
├── metrics.rs         # 指标定义
└── network/
    └── world_state.rs # 指标收集

python/
├── quick_compare.py           # 快速对比
├── consensus_comparison.py     # 详细对比
└── analyze_metrics.py          # 单个分析

文档/
├── CONSENSUS_QUICKSTART.md     # 快速开始
├── CONSENSUS_COMPARISON.md     # 详细指南
├── IMPLEMENTATION_SUMMARY.md   # 实现细节
└── PROJECT_COMPLETION.md       # 完成总结
```

## 🏆 项目成果

- ✅ 3 种共识完整实现
- ✅ 20+ 关键指标
- ✅ 4 个分析工具
- ✅ 4 份详细文档
- ✅ 1500+ 行代码
- ✅ 零编译错误

## 🚀 下一步

```bash
# 1. 编译
cargo build --release

# 2. 运行实验
python quick_start.py

# 3. 查看结果
# CSV 文件已生成，可用 Excel/Python 打开分析
```

## 📞 常见问题

**Q: 需要什么依赖？**
A: 只需 Rust 1.70+。Python 分析是可选的。

**Q: 实验要多久？**
A: 取决于 epoch 数。默认 10 个 epoch 约 2-3 分钟。

**Q: 能添加新共识吗？**
A: 可以！实现 Consensus trait 即可。详见 IMPLEMENTATION_SUMMARY.md

**Q: 数据可以用来发论文吗？**
A: 可以！所有结果可重复，支持学术研究。

## 📄 许可

MIT License - 自由使用和修改

## 🔗 相关资源

- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- [Ethereum 2.0](https://ethereum.org/en/eth2/)
- [Substrate Framework](https://substrate.io/)

---

**准备好开始你的共识研究了吗？** 🚀

```bash
python quick_start.py
```
