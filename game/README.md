# 游戏工程

Rust 2024 / Bevy 0.19.1

从仓库根目录进入本目录后运行：

```sh
cd game
cargo run --locked
```

验证与构建：

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo build --locked
cargo build --release --locked
```

从 `game/` 启动，运行资产放在 `game/assets/`

[工程设计](../docs/production/engineering.md)记录实现组织与验证方式
