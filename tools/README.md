# 检查与发布工具

在仓库根目录执行：

```sh
bun run check:docs
bun run check:narrative
bun run check:templates
bun run test:tools
```

文档检查覆盖文本格式、ID、依赖、Skills、本地链接和 Wiki 发布路径。叙事检查扫描 `docs/quests/`，核对结构、节点、信息顺序、选择、场景与台词。

检查器使用 Python 标准库；站点导航与数据导出工具由 Bun 运行，使用 Bun 兼容的 `node:` 标准库 API，相关测试由 `bun test` 执行。

`wiki-data.mjs` 在开发时提供源 JSON / CSV，构建完成后按相对路径导出到站点目录。侧栏从 `docs/` 下的主题页面生成。

Python 检查器接受 `--json`。`EMPTY` 表示当前任务数据数量为零；`incomplete` 表示路径分析达到预算，`--max-states` 用于调整预算。

Wiki 构建与发布见 [Wiki](../docs/production/wiki.md)。
