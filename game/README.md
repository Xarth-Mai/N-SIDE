# 游戏工程

当前工程使用 Rust 2024 与 Bevy 0.19.1，入口为 `src/main.rs`，创建默认插件与 2D 摄像机

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

开发配置为主工程 `opt-level = 1`、依赖 `opt-level = 3`；发布配置使用 `opt-level = 3`、单 codegen unit 与 Thin LTO

当前没有运行资产或自定义资源路径，后续运行资产放入 `game/assets/`，从本目录启动以使用 Bevy 默认的 `assets/` 路径

[工程设计](../docs/production/engineering.md)记录实现组织与验证方式
