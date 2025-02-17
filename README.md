## How to use

### 1.Install Rust

For windows：

[安装 Rust - Rust 程序设计语言](https://www.rust-lang.org/zh-CN/tools/install)

For linux:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2.Run

- `n` for number of nodes
- `t` for transactions per second

```
cargo run --release -- -n 100 -t 10
```

