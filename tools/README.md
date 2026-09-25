# 检查与发布工具

在仓库根目录执行：

```sh
bun run check:docs
bun run check:map
bun run check:narrative
bun run check:templates
bun run test:tools
```

文档检查覆盖文本格式、ID、依赖、Skills、本地链接和 Wiki 发布路径。叙事检查扫描 `docs/quests/`，核对结构、节点、信息顺序、选择、场景与台词

Python 检查器接受 `--json`。`EMPTY` 表示当前任务数据数量为零；`incomplete` 表示路径分析达到预算，`--max-states` 用于调整预算

Wiki 构建与发布见 [Wiki](../docs/production/wiki.md)
