# 检查与发布工具

在仓库根目录执行：

```sh
bun run check:docs
bun run check:roadmap
bun run check:map
bun run check:narrative
bun run check:story-design
bun run check:templates
bun run test:tools
```

文档检查覆盖文本格式、ID、依赖、Skills、本地链接和 Wiki 发布路径。叙事检查扫描 `docs/quests/`，核对结构、节点、信息顺序、选择、场景与台词

Python 检查器接受 `--json`。`EMPTY` 表示当前任务数据数量为零；`incomplete` 表示路径分析达到预算，`--max-states` 用于调整预算

Wiki 构建与发布见 [Wiki](../docs/production/wiki.md)

## 人物、故事与场所索引

`bun run check:story-design` 对照地图主数据核对[故事](../docs/story/index.md)、[人物](../docs/characters/index.md)和[场所](../docs/locations/place-network.md)的目录、引用、正文路径、潜梦分工及共楼关系；`--json` 输出机器可读结果

索引分别位于 `docs/characters/characters.json`、`docs/quests/quests.json`、`docs/production/place-catalog.json` 与 `docs/production/city-story-map.json`。人物 `source_file` 相对仓库根，故事 `source_file` 相对 `docs/`，角色正文与故事章节使用各自稳定 ID

检查对象是设计索引，运行任务仍由 `check:narrative` 检查。索引包含可选、替代与重访关系，不要求人物／地点／任务关联穷尽双向一致；工具不能证明正文因果、手感或实际路线可玩，正文连续性另行审阅

窄测：`python3 -B -m unittest discover -s tools/tests -p test_validate_story_design.py`

## Demo 进度

`bun run roadmap` 只读显示阶段计数与下一项，`bun run roadmap --stage M0` 查看阶段详情，`bun run roadmap --json` 输出汇总数据

编辑 `todo/demo-progress.json` 后运行 `bun run roadmap:sync` 生成看板，再运行 `bun run check:roadmap` 检查依赖、证据引用与生成一致性。工具只核对记录结构，验收依据见[工作流](../docs/production/solo-workflow.md)

进度工具使用 Python 3.10+ 标准库；测试纳入 `bun run test:tools`，窄测使用 `python3 -B -m unittest discover -s tools/tests -p test_roadmap.py`
