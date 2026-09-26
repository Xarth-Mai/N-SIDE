<p align="center">
  <img src="source-assets/branding/n-logo.svg" alt="N:SIDE Logo" width="180">
</p>

# N:SIDE

![月台杂货与采购街概念预览](docs/public/images/shop-street.webp)

在 Null City，兄妹与家庭 Agent 共同经营一家生活杂货店，在都市日常中调查异常、潜入梦境，帮助当事人重新回到生活

核心体验是“我真的在 Null City 生活过一段时间”。项目采用清爽、通透的日系现代都市表达

首发支持 PC，系统为 Linux 和 Windows，输入支持手柄和键鼠，图形接口采用 Vulkan

## 入口

[Wiki 与项目愿景](docs/index.md) · [Roadmap：至可玩 Demo](todo/roadmap.md) · [开发任务](todo/README.md) · [Agent 入口](AGENTS.md) · [游戏工程](game/README.md)

[故事百科《回声之后》](docs/player/story/index.md) · [Demo 定义](docs/dev/direction/demo-scope.md) · [当前任务](todo/README.md)

## 本地 Wiki

```fish
bun install
bun run docs:dev
```

[Wiki 构建与预览](docs/dev/handbook/wiki.md) · [检查工具](tools/README.md) · [目录约定](docs/dev/handbook/documentation.md)

## 游戏开发

当前工程使用 Rust 2024 与 Bevy 0.19.1，从 `game/` 启动：

```fish
cd game
cargo run --locked
```

格式、编译、测试与构建命令见[游戏工程](game/README.md)

[开发与运行验收](docs/dev/validation/runtime.md) · [资产生产](docs/dev/production/asset-pipeline.md) · [Skills 与验证工具](tools/README.md#skills-与来源检查)

## 许可证

[Mozilla Public License 2.0](LICENSE)

迁入的开发 Skills 和方法分别遵守[第三方来源与许可](THIRD_PARTY_NOTICES.md)中保留的条款
