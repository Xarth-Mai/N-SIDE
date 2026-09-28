# 失焦自动暂停与显式恢复验收

基线 `a49e857a6787d4a0ff1469416c8fdb1ed26dbc77`；本轮复用正式入口、人物物理、镜头和暂停菜单。输入内容hash见 `provenance.json`，原始PNG、视频和过程日志在 `output/focus/2026-09-28/r1/`

## 实现及失败复现

主窗口失焦在固定循环之前请求已有Paused状态，沿用待切换状态门禁阻止同帧物理、镜头和复位。读取WindowFocused消息，覆盖同一批先失焦再回焦；加载可完成，但未回焦时不运行人物。窗口恢复后先等待菜单输入释放，再接受一次新的继续操作；晚到的系统按键重复不能冒充新的Enter／Esc／Tab操作。暂停保留原会话及下落速度，恢复不重新生成代理

修前3项CPU复现均FAIL：下落脚点由27.512169降至27.492168；加载完成但失焦时未请求暂停；晚到Enter重复造成PendingWorld。日志在 `cpu-focus-before.log`，它们验证旧实现缺口，不作为最终失败项。修后入口9项PASS，完整测试覆盖0／1／3个固定步、长时下落、短暂失焦回焦、加载、真实InputPlugin重复事件、持有鼠标／模拟手柄、无窗口及次窗口兼容

Capture只为含focused事件的正式入口脚本创建逻辑PrimaryWindow，更新焦点字段并发送同名消息；Winit仍关闭，渲染目标仍为离屏Image。不直接写GamePhase、人物或预期结果；焦点延续到下一次变化，等待GPU回读期间不重复发消息。旧脚本仍无窗口，Viewer拒绝该字段。CPU另测类型、适用场景、消息次数及无结果篡改

## 实际检查

| 命令或路径 | 结果 | 证据 |
| --- | --- | --- |
| app焦点／暂停窄测 | PASS，9项 | `cpu-focus-after.log` |
| cargo test --locked --features viewer | PASS，53项库测试＋3项Viewer测试 | `tests-final.log` |
| script-contract窄测 | PASS，新增walk-focus脚本纳入检查 | `script-contract.log` |
| cargo fmt --check | PASS | `fmt.log` |
| cargo clippy --locked --features viewer --all-targets -- -D warnings | PASS | `clippy-final.log` |
| cargo build --locked --bin n-side | PASS | `build-final.log` |
| 文档／Skills检查 | PASS | `docs-final.log`、`delivery-checks.log` |
| 地图门禁及双站构建 | PASS，玩家95页／开发158页 | `wiki-final.log` |
| 任务生成、只读检查、重复生成稳定 | PASS，25张卡片 | `delivery-checks.log` |

最后一次binary构建后，仅在既有测试清单加入walk-focus.json并独立运行该测试；生产代码没有后续变更。三条最终录制使用同一binary，SHA256为 `3553a8f0a476b885931eec108f2962cad9238f143e7a823e59c649eb20d7e61f`

## 实际渲染与状态断言

AMD Radeon RX6650XT／RADV Mesa26.2.3-arch3.1／Vulkan，1280×720、30fps、固定输入与时间步。三次程序和包装器退出0，视频均生成

| 录制 | 范围 | 结果 |
| --- | --- | --- |
| focus-final | 600帧／20模拟秒；加载中失焦、回焦持有Enter、行走中失焦、持有手柄确认／暂停及摇杆、无输入回焦、明确继续、返回标题和重入 | 14项PASS |
| pause-final | 原暂停与原会话恢复录制，保持无窗口 | 600帧／14项PASS |
| entry-final | 原标题／加载／清理／重入录制，保持无窗口 | 540帧／11项PASS |

focus-final的失焦与等待区间内脚点和相机位置无变化，旋转峰值约0.00069弧度，在0.002弧度浮点容差内；复位计数0。重新确认后键盘与模拟手柄均能移动，分别超过5m和3m；返回标题清空世界，重入恢复同样的源对象数且能继续移动。下落期间的重力冻结由CPU真实调度夹具验证，不声称这条平地录像覆盖下落

实际逐张查看focus-final的34、99、179—181连续帧、234、269、309、359、459、559帧：加载失焦后及回焦持有输入时菜单仍打开；180帧人物已停止，181帧暂停面板出现；269和359帧明确继续后回到原位置；459帧标题清理、559帧重入后有新位置。中文及焦点清楚、场景与代理正常显示。另查看pause-final的121、272、419帧和entry-final的182、329帧，原两种输入提示、暂停和重入画面保持可用

这是Codex画面自查，视频未完整播放，已查看帧以WebP质量80归档。run.json保留工具当时的visual_review=NOT RUN，以上文字才是之后的实际看图记录；不把截图或机器断言称为作者手感、盲测或物理手柄验收

## 原生窗口限制

真实X11窗口探针确实启动游戏并尝试切焦，但niri没有活动窗口，X11内部焦点不证明桌面合成器激活。修后探针退出0仅代表操作脚本走完；实际截图存在暂停未退出及连续画面不更新，未确认完整行走／回焦／清理链路。诊断停止并清理本次启动的进程，证据及最后的探针源码见 [原生尝试范围](native-attempt/scope.md)

原生桌面失焦／回焦完整验收为INCOMPLETE；物理手柄、Windows、作者体验为NOT RUN。最终focus-final证明正式系统对模拟焦点输入的行为与真实渲染，不证明操作系统焦点交付。当前暂停只覆盖已有系统，没有NPC、战斗、动画和音频暂停的结论

## 复验与后续

在仓库根执行，输出选择新目录，以下兼容fish

```fish
cargo fmt --manifest-path game/Cargo.toml --check
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-focus.json --output output/capture/focus-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-pause.json --output output/capture/pause-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/game-entry.json --output output/capture/entry-review
bun run check:docs
bun run docs:build
bun run tasks:sync
bun run tasks:check
```

技术任务依据CPU与模拟焦点渲染验收，不放行G1整体。下一步在活动桌面用正常窗口补验：行走中切走窗口、释放输入后切回、确认仍暂停、重新继续；再测加载中切走及持续按住Enter切回。启动命令为 `cargo run --manifest-path game/Cargo.toml --locked --bin n-side -- --walk-preview`，必要时用 `env RUST_LOG=n_side::app=debug` 前缀记录焦点与菜单输入诊断

本轮交付后停止，不启动下一工作包

独立只读审查核对实现、脚本、23个输入文件hash、三份最终录制及验收边界，无阻断；简化审查结论为 `Lean already. Ship.`。评审者未运行Cargo／GPU，不以代码审查替代实际操作
