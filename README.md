## How to use

### 1.Install Rust

For windows：

https://www.rust-lang.org/tools/install

For linux:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2.Run

- `n` for number of nodes
- `t` for transactions per second
- `c` for consensus type [pos,pog]
- `g` for gossip propagation delay in milliseconds (optional, default: 0)

```
cargo run --release -- -n 100 -t 10 -c pos
```

#### Gossip Propagation Delay

To simulate sequential gossip propagation (one node per time interval) instead of instant broadcast:

```
cargo run --release -- -n 100 -t 10 -c pos -g 100
```

This will add a 100ms delay between propagating blocks and transactions to each neighbor node.

```
cargo  test test_stake_real_c_total_both_increase --release -- --nocapture
```