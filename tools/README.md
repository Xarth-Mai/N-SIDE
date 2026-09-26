# 检查与发布工具

在仓库根目录执行：

```sh
bun run check:docs
bun run check:roadmap
bun run check:map
bun run check:narrative
bun run check:templates
bun run test:tools
```

文档检查覆盖文本格式、ID、依赖、Skills、本地链接和 Wiki 发布路径。叙事检查扫描 `docs/quests/`，核对结构、节点、信息顺序、选择、场景与台词

Python 检查器接受 `--json`。`EMPTY` 表示当前任务数据数量为零；`incomplete` 表示路径分析达到预算，`--max-states` 用于调整预算

Wiki 构建与发布见 [Wiki](../docs/production/wiki.md)

## Demo 进度

`bun run roadmap` 只读显示阶段计数与下一项，`bun run roadmap --stage M0` 查看阶段详情，`bun run roadmap --json` 输出汇总数据

编辑 `todo/demo-progress.json` 后运行 `bun run roadmap:sync` 生成看板，再运行 `bun run check:roadmap` 检查依赖、证据引用与生成一致性。工具只核对记录结构，验收依据见[工作流](../docs/production/solo-workflow.md)

进度工具使用 Python 3.10+ 标准库；测试纳入 `bun run test:tools`，窄测使用 `python3 -B -m unittest discover -s tools/tests -p test_roadmap.py`
