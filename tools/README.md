# 开发与检查工具

在仓库根目录执行：

```fish
bun run check:types
bun run check:docs
bun run tasks:check
bun run check:map
bun run check:narrative
bun run check:story-design
bun run check:templates
bun run test:tools
```

文档检查覆盖文本格式、ID、依赖、Skills、本地链接和 Wiki 发布路径。叙事检查扫描 `docs/dev/design/quests/`，核对结构、节点、信息顺序、选择、场景与台词

Python 检查器接受 `--json`。`EMPTY` 表示当前任务数据数量为零；`incomplete` 表示路径分析达到预算，`--max-states` 用于调整预算

Wiki 构建与发布见 [Wiki](../docs/dev/handbook/wiki.md)

## 人物、故事与场所索引

`bun run check:story-design` 对照地图主数据核对[故事](../docs/player/encyclopedia/story/index.md)、[人物](../docs/player/encyclopedia/characters/index.md)和[场所](../docs/player/encyclopedia/locations/place-network.md)的目录、引用、正文路径、潜梦分工及共楼关系；`--json` 输出机器可读结果

索引分别位于 `docs/dev/design/catalogs/characters.json`、`docs/dev/design/catalogs/quests.json`、`docs/dev/design/catalogs/place-catalog.json` 与 `docs/dev/design/catalogs/city-story-map.json`。人物 `source_file` 相对仓库根，故事 `source_file` 相对 `docs/`，角色正文与故事章节使用各自稳定 ID

检查对象是设计索引，运行任务仍由 `check:narrative` 检查。索引包含可选、替代与重访关系，不要求人物／地点／任务关联穷尽双向一致；工具不能证明正文因果、手感或实际路线可玩，正文连续性另行审阅

窄测：`python3 -B -m unittest discover -s tools/tests -p test_validate_story_design.py`

## 任务与生成看板

`bun run tasks:list` 只读显示里程碑计数与可执行任务；编辑 `todo/tasks/TASK-xxx-title.md` 后运行 `bun run tasks:sync`，再运行 `bun run tasks:check` 校验结构、引用、依赖、证据和看板一致性。check 不写文件，sync 对相同输入稳定

任务卡的状态含义和完成依据见[任务手册](../docs/dev/handbook/tasks.md)。Bun 原生 YAML 解析，无额外解析依赖；窄测 `bun test tools/tests/tasks.test.ts`，纳入 test:tools。旧进度 JSON 与 Python 生成器退出活动入口，历史记录只供追溯

## Skills 与来源检查

原版 Skill 保留完整目录与原名，项目差异集中在 [.agents/skills/nside/](../.agents/skills/nside/SKILL.md)。[第三方清单](../THIRD_PARTY_NOTICES.md)记录完整上游 commit、作者、采用方式及许可；`third_party/skills/manifest.json` 是迁入文件的来源与校验清单，不承载项目进度

```fish
bun run check:skills
python3 tools/probe_skills.py --output output/skills/discovery.json
```

第一条检查 YAML 元数据、名称、引用、许可与原文哈希，已纳入 `check:docs`；第二条通过已安装的 Codex `app-server` 在仓库根与 `game/` 实际查询 `skills/list`，不启动模型回合。发现成功不代表隐式路由和所有 Skill 行为已经验证；没有 Codex 或宿主访问失败时，报告非零结果并保留诊断

更新来源时先固定完整 commit，比较原文与适配层，再更新哈希；不要直接重写上游正文或删除许可来消除检查错误

## 真实运行与 Capture

从需求和当前实现出发，完成适用测试，再按[运行验收](../docs/dev/validation/runtime.md)运行真实场景、检查状态与画面、修复并复验。当前入口复用 Viewer 的城市和自由镜头；配置字段、输入与范围见[游戏工程](../game/README.md)

```fish
python3 tools/capture.py --script game/capture/viewer-tour.json --output output/capture/my-run
python3 tools/capture.py --script game/capture/assertion-failure.json --output output/capture/my-failure
```

输出目录必须尚不存在。第一条应退出 0，第二条故意要求一秒移动 1000 m，应退出 1，并在 `state.json` 留下失败条件。依次检查 `run.json`、`state.json`、`runtime.log`、`keyframes/` 和 `video.mp4` 或 `frames/`；另记实际看的帧与发现，机器报告不会替你填写视觉结论

Capture 使用 Python 3.10+ 与 Pillow，Rust 按 `game/` 现有环境构建，运行需要 Vulkan。FFmpeg 可用时生成 H.264 视频，否则明确记录视频 NOT RUN，保留 PNG 序列。命令均可直接用于 fish；不要把模拟 18 秒的录制耗时当作游戏性能测量

## 资产报告与实际尺寸预览

使用 `create-game-assets` 组织需求、参考、制作与导入，按[资产管理](../docs/dev/production/asset-pipeline.md)写入现有资产包和导出路径。当前环境需要 Pillow 时可按原版脚本依赖安装：

```fish
python3 -m pip install -r .agents/skills/create-game-assets/scripts/requirements.txt
python3 .agents/skills/create-game-assets/scripts/asset_report.py game/assets/environment/signs/shop.png --expect-size 1024x128 --json
python3 tools/asset_preview.py game/assets/environment/signs/shop.png --display-size 512x64 --out output/assets/shop-preview.png
python3 .agents/skills/create-game-assets/scripts/build_preview_sheet.py game/assets/environment/signs/*-display.png --out output/assets/displays.png --columns 5 --cell-size 192
bun tools/export-district-scene.ts --check
bun tools/export-environment.ts --check
```

预览保留原件，用三种对比底色检查边缘；`--require-cutout` 只用于需要透明区域的素材，招牌实底和 Logo 不必抠图。资产检查、预览、capture 均写入忽略的 `output/`，不混入运行资产。工具取舍与后续采用条件见[资产工具审查](../docs/dev/decisions/asset-tools.md)

## CI

[checks.yml](../.github/workflows/checks.yml)分别执行无 GPU 的工具／Wiki 检查和 Rust 格式／编译／测试；真实渲染只在手动启用后调度带 `nside-vulkan` 标签的自托管 Linux runner，要求预装 Rust、Python、Pillow、Vulkan 驱动与 FFmpeg。未配置 runner 或未调度的渲染任务为 NOT RUN，本地可用上面的同一入口复验

## Skills 来源与补丁复验

`bun run check:skills` 检查本地清单、必需入口、双向文件覆盖、引用、metadata 与命令依赖；来源更新时另运行 `bun run check:skills:upstream` 从固定上游 Git 对象核对原件、完整目录和补丁。后者需要网络；离线或上游不可达记录 NOT RUN，不能用本地 hash 自洽代替来源证明

已缓存独立 Git checkout 时可向 `tools/check_skill_upstream.py` 传入 `--checkouts` JSON（来源 ID 到 checkout 路径）；工具仍以清单中完整 commit 读取 Git 对象，不信工作区文件。Codex 发现命令 `python3 tools/probe_skills.py --output output/skills/discovery.json` 只证明发现；真实显式/隐式模型行为单独保存会话记录
