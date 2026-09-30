# Jump 与离屏输入隔离独立代码评审

2026-10-01，范围为当前未提交的 `game/src/character.rs`、`game/src/app.rs`、`game/src/bin/map_viewer.rs`、`game/src/capture.rs`、`game/capture/character-jump-pause.json` 和 `game/capture/walk-character.json`

最终源码复查 PASS，首轮发现的两项阻碍已修复，未发现剩余正确性或复杂度问题；运行与视觉范围见下文

## 已解决发现

- PASS · 原 P1 · `game/src/capture.rs:200` 的 `CharacterCheck::valid` 最初只接受 Idle／Walk／Run，导致 Jump 脚本被拒绝；现共用 `pub(crate) character::CLIPS`，覆盖同一四动作表，旧负例已改为 `MissingClip`，避免运行与验证列表再次漂移
- PASS · 原 P2 · 起初 `walk-character.json` 在 `from=60,to=95` 的完整起跳区间要求每帧 Idle；现保留该区间原有的运动、计数与高度检查，把 Idle 独立放到 `85..95` 落地区间，并检查动画持续推进

两项均由主执行者修复，本评审对最终 diff 作窄复查，不改受审代码或脚本

## 源码核对

- PASS · `character.rs` 的四动作加载仍使用具名 clip、真实蒙皮与目标检查，缺 Jump 会显式失败；空中优先选择 Jump，接地后按实际水平速度返回 Idle／Walk／Run，保留控制器对世界位移的唯一职责
- PASS · `stop_all().play(...)` 清空旧活动动作后创建新实例，Jump 省略 `repeat()` 对应 Bevy 默认 `RepeatAnimation::Never`；完成后仍参与曲线采样，曲线使用 `sample_clamped` 固定末姿态，下一次落地再起跳会从新实例的零时刻开始
- PASS · 暂停门禁在 `Update` 调用 `pause_all`，早于 `PostUpdate` 的 `advance_animations`；恢复只修改 paused 标志，保留同次起跳的 elapsed 与 seek time；状态采样排在 `AnimationSystems` 之后，能记录当帧实际结果
- PASS · 两个入口只在 `headless` 时禁用 `GilrsPlugin`，正常窗口入口保留物理手柄；`InputPlugin` 仍注册连接及原始手柄消息处理，现有 `ScriptGamepad` 和模拟断连路径不依赖 Gilrs 后端

以上以本地锁定的 Bevy 0.19.1 源码核对：`bevy_animation/src/lib.rs` 的 `RepeatAnimation`、`ActiveAnimation::update`、`AnimationPlayer::play/stop_all/pause_all/resume_all`、`advance_animations`、`AnimationPlugin::build`，`bevy_animation/src/animation_curves.rs` 的 `AnimatableCurve::apply`，以及 `bevy_gilrs/src/lib.rs`、`gilrs_system.rs` 和 `bevy_input/src/lib.rs`；依赖目录为 `/home/lzzz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`

## 输入隔离失败证据的边界

已核对 `todo/evidence/TASK-049/urban-bank-r1/input-isolation/failed-spawn/` 的 script、run、state 与 runtime log：脚本只有 30 帧 Enter，31–33 帧记录 loading，等待 34 帧 world 时未满足，180 秒超时报告 title、ready=false、saved=34

原逻辑由 Gilrs 持续接收真实设备输入，入口聚合所有 Gamepad，capture 只覆盖 ScriptGamepad，源码支持“离屏录制可能混入物理手柄”的风险判断；现有失败日志没有具体取消键或设备来源，不能据此认定该次返回标题由真实手柄触发

已读取主执行者复跑的 `output/urban-bank-r1/spawn-isolated/run.json` 与 `state.json`：同一 script SHA-256 `9b4c68c958e703639685273bc723402a09dd3462b17bd9ac5436b0f9c6c98db8` 返回 0，60/60 帧及全部断言 PASS，第 59 帧为 world、ready=true、grounded=true、jumps=0、resets=0；该结果支持隔离后的路径可完成，不补写原失败的按键归因，实际画面由主执行者查看

## 验证范围与简化评审

已读取主执行者的 `lib-tests.log`，结果 134 passed、0 failed、1 ignored；`build.log` 显示构建完成，主执行者确认覆盖两个可执行入口；本评审执行的 `git diff --check` PASS

本子评审没有执行 cargo 或 GPU 命令；两位角色的跳跃／暂停实际录制、连续画面观察、物理手柄操作及作者验收在本评审中均为 NOT RUN，分别由对应运行证据与反馈支撑，源码 PASS 不代替这些结果

本轮未生成截图、帧、视频或探测可执行文件，无需清理视觉临时产物

ponytail-review：Lean already. Ship.
