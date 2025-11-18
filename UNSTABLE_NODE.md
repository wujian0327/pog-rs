# Unstable节点功能说明

## 概述

Unstable节点是一种新的节点类型，用于模拟网络中不稳定的节点行为。这些节点会随机离线一个epoch，然后自动恢复在线。

## 特性

- **随机离线**：在每个epoch切换时，Unstable节点按照配置的概率离线（默认10%）
- **可配置概率**：通过`--offline-probability`参数调整离线概率（0.0-1.0）
- **自动恢复**：离线后会在下一个epoch开始时自动恢复在线
- **离线期间**：节点不处理任何消息，模拟真实的网络故障

## 使用方法

### 命令行参数

使用 `-u` 或 `--unstable-node-num` 参数指定不稳定节点的数量：

```bash
# 创建50个诚实节点 + 5个不稳定节点（默认10%离线概率）
cargo run -- -n 50 -u 5

# 创建不稳定节点并指定20%的离线概率
cargo run -- -n 50 -u 5 --offline-probability 0.2

# 完整示例：50个诚实节点 + 5个恶意节点 + 10个不稳定节点（30%离线概率）
cargo run -- -n 50 -m 5 -f 0 -u 10 --offline-probability 0.3 -t 10 -c POW
```

### 参数说明

- `-n, --node-num <NUM>`：诚实节点数量（默认50）
- `-m, --malicious-node-num <NUM>`：恶意节点数量（默认0）
- `-f, --fake-node-num <NUM>`：恶意节点伪造身份数量（默认0）
- `-u, --unstable-node-num <NUM>`：不稳定节点数量（默认0）
- `--offline-probability <PROB>`：不稳定节点离线概率，范围0.0-1.0（默认0.1即10%）
- `-t, --trans-num <NUM>`：每秒交易数量（默认10）
- `-c, --consensus <TYPE>`：共识类型（POW/POS/POG）

## 实现细节

### Node结构新增字段

```rust
pub struct Node {
    // ...现有字段
    pub is_online: bool,                  // 当前是否在线
    pub offline_until_epoch: Option<u64>, // 离线到哪个epoch
    pub offline_probability: f64,         // 离线概率（0.0-1.0）
}
```

### NodeType枚举

```rust
pub enum NodeType {
    Honest,    // 诚实节点
    Selfish,   // 自私节点
    Malicious, // 恶意节点
    Unstable,  // 不稳定节点（新增）
}
```

### 离线逻辑

1. **消息处理拦截**：
   ```rust
   if !self.is_online {
       debug!("Node[{}] is offline, skipping message", self.index);
       continue;
   }
   ```

2. **随机离线触发**：
   - 在`UpdateSlot`消息处理中检测epoch变化
   - 根据`offline_probability`字段触发离线
   - 离线时长：1个epoch

3. **自动恢复**：
   - 当 `current_epoch >= offline_until_epoch` 时自动恢复在线

## 日志输出

运行时会输出不稳定节点的状态变化：

```
[WARN] Node[52] goes offline at epoch 5 until epoch 6
[INFO] Node[52] is back online at epoch 6
```

## 测试示例

```bash
# 测试1: 基本功能测试（默认10%离线概率）
cargo run -- -n 10 -u 2 -t 5

# 测试2: 高离线概率测试（50%）
cargo run -- -n 10 -u 5 --offline-probability 0.5 -t 5

# 测试3: 混合节点类型
cargo run -- -n 20 -m 3 -f 2 -u 5 --offline-probability 0.15 -t 10 -c POG

# 测试4: PoW共识下的不稳定节点（20%离线概率）
cargo run -- -n 30 -u 10 --offline-probability 0.2 -t 20 -c POW --topology BA
```

## 性能影响

- 不稳定节点离线时不消耗计算资源
- 可用于测试网络容错能力
- 可用于研究部分节点不可用时的共识性能

## 注意事项

1. 不稳定节点离线时不参与共识
2. 离线期间接收的消息会被丢弃
3. 离线概率通过`--offline-probability`参数配置，范围0.0-1.0
4. 概率值会自动限制在[0.0, 1.0]范围内（使用`clamp`）
5. 每次离线时长固定为1个epoch

## 未来改进方向

- [x] 可配置的离线概率 ✅
- [ ] 可配置的离线时长
- [ ] 多种离线模式（短暂闪断、长期离线等）
- [ ] 离线恢复后的状态同步机制
- [ ] 每个节点独立的离线概率配置
