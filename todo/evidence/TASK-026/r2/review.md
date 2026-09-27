# 街区信号第二轮引擎验收

基线 `b64759e`，实际修改输入和二进制hash见[inputs.json](inputs.json)。作者已选择保留首轮比例并强化图形细节，反馈见[TASK-015](../../TASK-015/r1/review.md)。本轮沿原比例补面板与按钮硬阴影、双边标签、标题错位底片，不改阅读布局和输入规则

## 已交付

同一 `WorldScenePlugin` 场景上接入 `SignalUiPlugin`，包括工作菜单、两条明确标注的记录样例、空记录、街区说明、100%/125%缩放、减少动态效果和返回街景。未接入的委托入口保持不可用；文本来自排版样例，不创建游戏线索、任务或存档

AST-005保留固定上游字体、OFL与版权原文，`bun tools/export-ui.ts --check`只读验证运行字节。结构化tokens提供实际使用的色板、字重以外的主要字号、圆角及面板时长；局部布局仍在原生UI中，不声称所有规范参数已实现

## 命令与机器结果

| 命令 | 结果与证据 |
| --- | --- |
| `cargo build --manifest-path game/Cargo.toml --features viewer --bin map_viewer --locked` | PASS，[build.log](build.log) |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked` | PASS，22项库测试、4项Viewer测试，[rust-tests.log](rust-tests.log) |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --all-targets --locked -- -D warnings` | PASS，[clippy.log](clippy.log) |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `python3 tools/capture.py --script game/capture/ui-signal.json --output output/ui-signal/2026-09-27/r2/main --binary game/target/debug/map_viewer` | PASS，600帧/20秒、18项检查，[主流程](main/run.json) |
| 同一capture，脚本换为首轮 `narrow.json` / `wide.json`，输出分别为r2/narrow、r2/wide，加 `--no-video` | PASS，1280×1024与2560×1080各180帧、7项检查；脚本内含125%缩放、首次阅读为0与滚动断言，[窄屏](narrow/run.json)、[宽屏](wide/run.json) |
| 同一capture，`expected-failure.json`，r2/expected-failure、`--no-video` | 预期失败，native与wrapper退出1；仅 intentional_wrong_page 失败，3项基础检查通过，[失败状态](expected-failure/state.json) |

所有运行的实际命令、参数、源脚本、版本、GPU诊断、截图保存与状态分别保存在各目录的run.json、script.json、runtime.log和state.json。断言读取真实页面/焦点/缩放/设备/滚动位置以及自由相机轨迹，没有直接写验收结果。20秒MP4和全部原始PNG保存在忽略的 `output/ui-signal/2026-09-27/r2/main/`，任务内保留quality80 WebP

## 仓库检查

`bun run check:types`、`bun run test:tools`通过，实际103项Python测试与108项Bun测试，[工具日志](tools-checks.log)。`bun run check:docs`通过292份Markdown、91个ID和31个Skills；`bun run docs:build`通过33项地图测试，实际player包95页/547文件、dev包157页/788文件受众校验通过，[构建日志](docs-checks.log)。开发站仍有既有chunk大于500kB提示

`bun run tasks:sync`、`bun run tasks:check`、`bun tools/export-ui.ts --check`及`git diff --check`均通过。任务看板和导出check保持只读

## 实际看图

Codex查看[主菜单](main/frame00029.webp)、[调查工作台](main/frame00150.webp)、[窄屏125%](narrow/frame00149.webp)、[宽屏125%](wide/frame00149.webp)，并检查连续120–127帧[打开面板](open.webp)与540–547帧[减少动态效果重开](reduced-reopen.webp)。标签、标题错位和按钮厚度可见，中文未出现缺字，长标签按实际宽度换行，阅读内容超过视口时滚动；重开菜单没有首帧跳动。这里只检查关键帧和连续帧，未声称完整播放视频或隔离盲评

首轮实际发现并修正长标签溢出、窄屏列表挤占阅读区、菜单滚动位置污染首次记录、输入切换重排归零、关菜单当帧输入穿透和减少动态效果首帧跳动。对应历史画面与数据保留在r1，最终修复后由本轮同一路线复验

字形覆盖及8对不透明语义色对比见[字形](../r1/font-coverage.json)与[对比](../r1/contrast.json)，对比最低约6.51:1；这些结果对应已使用的稳定底板，不能替代后续透明层/新页面的最终合成检查。字体导出的损坏源、损坏运行副本与只读检查见[负例](../r1/font-export-negative.json)

## 限制与下一步

- 手柄测试为实际Bevy输入路径的脚本数字状态注入，物理手柄、热插拔、鼠标实操、Windows运行、陌生玩家试玩均NOT RUN
- 锁定Parley 0.9.0的非复杂脚本分词器在中文上记录ICU4X诊断，日志保留；字体字形与可见换行已检查，按词选择、分词语义未验证，不为本实验升级引擎
- 正式游戏入口、角色移动、碰撞、真实对话/线索、潜梦、设置持久化、重映射与UI声音不在本实验交付中
- 固定模拟步长和输入可重演状态，不保证跨GPU像素相同；录制开销不能当正式游戏帧预算，UI增量性能尚未隔离测量

TASK-026按上述实验范围由Codex技术验收，TASK-015保留review，实际手感与完整设计尚未放行。下一步先在已认可比例上做HUD/对话的真实流程接入，所用线索来自任务运行时；道路由TASK-025继续分批修复

## 复杂度复核

沿用原生Node/Text/Button/Grid/ScrollPosition与同一capture，未增加依赖、平行录制器或通用主题框架；BoxShadow与BorderColor仅增表现。复核结论：Lean already. Ship.

入库日志仅去除行尾空白，诊断内容完整保留；逐字节原始运行日志仍位于对应output目录
