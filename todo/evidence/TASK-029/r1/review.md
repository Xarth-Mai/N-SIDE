# 暂停与原会话恢复验收

基线 `fa61e20`；本轮沿用正式入口、实际城市、人物碰撞与街区信号tokens，不启用Viewer调查样例。被检查源码与脚本hash见同目录 `provenance.json`；原始连续PNG和视频保留在 `output/pause/2026-09-28/r1/`

## 实现与检查

`GamePhase::Paused` 保留世界、人物、碰撞与镜头，继续只恢复状态，返回标题沿用真实清理。菜单输入移到固定循环之前；已有待切换状态立即屏蔽移动与镜头，防止开菜单当帧多走一步。继续前等待原始玩法按键、鼠标和摇杆释放，包括W+S等相抵按键；OnEnter World检查代理是否存在，避免重复生成

CPU暂停测试使用真实入口、人物与调度系统，独立碰撞夹具覆盖0／1／3个固定步；检查同帧停止、持有移动／镜头／复位输入、相抵输入、Esc／Tab／Start／B、鼠标继续、代理Entity不变、标题清理及再次进入。正式城市路径由GPU及既有真实网格测试覆盖，夹具不作为全城画面证据

Capture增加 `paused` 页和 `max_rotation`，按区间内每一帧求相对起点的转角峰值。回归输入0→0.5→0弧度的头尾相同但中间转动轨迹，`finish()`实际报告FAIL；保持零转角报告PASS，并拒绝负上限及上下界相反的脚本。相机四元数自身比较可能有约0.00069弧度浮点误差，本轮稳定性容差为0.002弧度

| 实际命令或路径 | 结果 | 证据 |
| --- | --- | --- |
| 默认库暂停窄测 | PASS，5项 | `cpu-pause-first.log` |
| 默认库完整测试 | PASS，47项 | `cpu-lib.log` |
| viewer feature完整测试 | PASS，47项库测试＋3项Viewer测试 | `cpu-viewer.log` |
| cargo fmt --check／clippy --all-targets --features viewer -- -D warnings | PASS | `fmt.log`、`clippy.log` |
| 双binary构建 | PASS | `build.log` |
| Python capture工具测试 | PASS，3项；进程包装测试不冒充GPU | `capture-tools.log` |
| 文档及Skills | PASS，最终316篇Markdown、92个ID、31项Skills／25项导入 | `docs-check.log`、`docs-final.log` |
| 地图门禁及玩家／开发站构建 | PASS，95／158页 | `wiki-build.log` |
| 任务生成与只读检查 | PASS，24张卡片；重复生成无变化 | `final-checks.log` |

## 实际运行与画面

GPU为AMD Radeon RX6650XT／RADV Mesa26.2.3-arch3.1／Vulkan。`walk-pause.json` 在实际小店出生后行走，通过键盘和模拟手柄暂停，持续发送移动／鼠标／摇杆／复位，再继续、返回标题和重入。600帧、20模拟秒、14项检查PASS，原生程序与包装器退出0，视频生成

暂停前实际脚点 `(100.01514,28.35533,-261.38232)`。两次暂停及恢复等待区间的脚点、相机位置均保持不变，旋转峰值约0.00069弧度；恢复后新输入分别行走3.215m和6.400m。世界源对象数量保持4572，返回标题归零，重入恢复4572；CPU另外检查同一代理Entity，源对象计数不代替代理唯一性断言

本轮为Codex视觉自查：实际打开119—121连续帧，确认120帧输入已停止移动、121帧出现暂停菜单；查看179、272、329帧的两种输入提示与正文，中文完整、焦点清楚、背景仍是原街景；查看212、332、419帧的恢复与重入，代理和场景可见；362帧显示返回标题的亮色焦点。视频已生成但未完整播放，抽帧不称为真人手感或盲测

既有四条正式入口录制同步为「暂停→选择返回标题」，原移动阈值和路线保持：标题540帧／11项、普通步行650帧／16项、坡面镜头600帧／13项、登高首段1050帧／10项均PASS，原生及包装器退出0、视频生成。分别保留在 `entry-regression/`、`walk-regression/`、`ramp-regression/`、`ascent-regression/`。另实际查看182、612、572帧，确认固定街景及不同人物位置均能正常显示暂停与返回焦点；登高959帧仍在真实台阶上，1022帧正常打开菜单。完整上山GPU录像仍不在本次范围

## 复验命令

从仓库根执行，输出使用新目录；以下兼容fish

```fish
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo build --manifest-path game/Cargo.toml --locked --features viewer
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-pause.json --output output/capture/pause-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/game-entry.json --output output/capture/entry-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-preview.json --output output/capture/walk-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-ramp-camera.json --output output/capture/ramp-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-ascent-entry.json --output output/capture/ascent-review
python3 -B -m unittest discover -s tools/tests -p test_capture.py
bun run check:docs
bun run docs:build
bun run tasks:sync
bun run tasks:check
```

## 范围与续接

技术功能可验收，G1整体、TASK-014镜头及登高节奏不据此放行。物理手柄、Windows、人工点击、实际窗口切换及作者手感NOT RUN；CPU鼠标Interaction和模拟Gamepad事件有自己的证据。暂停目前屏蔽已有的人物与镜头系统，尚无NPC、战斗、动画、声音暂停结论

下一项是窗口失焦自动进入现有暂停并在回焦后显式继续：现状失焦仅清移动意图，固定物理仍可能推进；无窗口的capture不验证OS焦点。按用户要求，本里程碑提交后停止，该项本轮未实施

独立代码与简化审查已核对调度、资源保留、释放门槛和完整区间旋转断言，未发现阻断，结论 `Lean already. Ship.`；审查者未运行Cargo或代签实际画面
