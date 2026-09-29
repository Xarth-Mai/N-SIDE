# 门前交接完整路线运行验收

结论：本项限定的会话内三点观察与配送路线确认技术验收 PASS，完整序章、物件搬运、磁盘任务存档、真人手柄及作者体验为 NOT RUN。本轮不放行 G1 或人物、城市的最终美术品质

## 输入与复现

生产入口基线为 `e92802ffd39daf5b7c990306ab23eeb1a78de821`，故事对齐到 `1bfc1cf`；工作区新增的是连续步行测试、录制脚本及 Viewer 检查机位，没有为通过录制改写任务、碰撞或观察距离。实际正式入口二进制 SHA-256 为 `283501dfe5d7a4c40b0769e97e89f0b4091631227b3dda48b0b2e7b652e626fb`，最终脚本 SHA-256 为 `87f396199b6eb2133ac691cfeeefa05fbecdc5649562fccce74f590f748aa68f`

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib walks_prologue_handoff_places_continuously -- --nocapture
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/prologue-route.json --output output/capture/task050-full-route-r2-native
```

GPU 运行于 Linux／RX 6650 XT／Vulkan，1280×720、30 Hz、2672 帧，模拟时长约 89.07 秒；复用原白天场景、灰阶角色预览和真实输入。源版本、输入、二进制、日志、完整机器检查、选取状态与已查看帧的 hash 见 [runtime-summary.json](runtime-summary.json)，其中状态明确是摘录；完整原始 JSON、日志和脚本继续保留在各 output 目录

## 失败及校准

初次 `task050-full-route-r1` 正常录完2639帧但退出1，15项检查中7项通过、8项失败。台阶处861帧实际脚点距目标约3.053m，超过既有3m观察距离，按F没有开窗，随后Escape转入暂停，后续失败依次发生；不是三点已经完成后只因截图失败

按实测位移延长最后上坡输入33帧，后续输入和检查窗口整体平移，12项断言的条件逐项不变，依据见[校准记录](calibration.md)。未传送人物、注入线索、放宽距离或更改场所标高

沙箱尝试 `task050-full-route-r2` 未获得GPU，程序退出101，没有生成状态与视觉帧；正常授权的本机显卡复验使用全新 `task050-full-route-r2-native` 目录并退出0。沙箱失败不能推断主机没有显卡

## 机器结果

连续碰撞测试在30 Hz及64 Hz从同一出生点逐段调用生产移动，共27个停点；台阶、支撑、高差、停滞和零恢复检查通过，原始日志见[cpu-walk.log](cpu-walk.log)

实机15项检查全部PASS：必需资源就绪、2672帧完整落盘、Transform有限，以及12项真实输入与状态断言

| 帧 | 行为与实际结果 |
| --- | --- |
| 59 | 正式出生，无已观察地点与确认 |
| 188 | 实际接近并面向04，开窗后仅获得04 |
| 894／924 | 连续爬到29，脚高39.6823m、着地，开窗后获得04及29 |
| 1498 | 原台阶返回home附近，脚点地图坐标约 `[99.998,254.993,28.046]` |
| 2515／2545 | 沿店外公共铺面绕到28，脚点约 `[25.356,270.207,28.046]`；开窗集齐三点，确认仍为0 |
| 2578 | 明确选择公共台阶被拒绝，rejected_choices=1，仍待修订 |
| 2611 | 选择service_yard后confirmation_count=1 |
| 2626／2671 | 关闭、模拟手柄重开与重复确认后，结果仍只提交一次 |

整个59–2671帧维持World及同一场景实体集，人物resets=0、jumps=0，脚高在既定27.8–39.9m内；没有用跳跃跨过接缝。模拟摇杆与Confirm经原输入系统执行，未接物理手柄

## 实际看图 self-audit

根Agent实际查看188、894、924、2545、2578、2611、2671关键帧，台阶连续893–894帧、店外侧路1874–1877及服务巷2151–2154完整连续帧；视频已编码但没有播放，不称盲测或作者反馈

- 04说明、29已观察条目及28选择正文可读，来源只含当前实际取得的信息；1280×720下文字没有越出正文区，标题、选择和返回控件分开
- 错误选择后可见“台阶可供步行，推车无法通过”的修订提示，正确选择后变为“配送走服务院，步行走公共台阶”，重开保留结果并切换为手柄提示
- 台阶和店外侧路画面中的灰阶人物随真实输入前进、停步，侧路镜头因近墙缩短；所看片段没有整体穿墙或跌落，仍需真人检验镜头与路线理解
- 画质不足明确保留：台阶周围及服务巷的大面积平面、重复窗格和铺面、人物贴片感五官与动作硬切仍明显；这次功能PASS不代表日漫画风、角色动作或README目标品质通过

已有乱序、重复状态、选择输入、暂停与标题清理检查承接[r1实现](../r1/implementation.md)及[r1运行](../r1/runtime-review.md)，未把本次89秒路线当作所有存读档和设备情形的覆盖

## 评审与下一步

[独立代码审查](independent-review.md)核对输入到生产移动、真实观察资格及显式提交，没有直接写结果；[叙事对齐](narrative-alignment.md)核对最新第三分场。技术范围可由Codex验收，作者玩法感受、完整序章和阶段放行继续单列

下一增量优先接入这个小片段的独立进度保存与明确继续／重开，沿用[状态恢复规格](../../../../docs/dev/design/systems/state-and-recovery.md)，不混入用户设置；人物与城市分别沿原资产任务继续精修

收尾检查：`cargo fmt --manifest-path game/Cargo.toml --check`、`bun run tasks:sync`及`bun run tasks:check`通过；[文档检查](docs-check.log)通过413个Markdown／102个对象ID与31个Skills，[双站构建](docs-build.log)通过地图检查与player／dev实际产物校验。新增Viewer机位的构建及3项检查另见[住宅证据](../../TASK-049/residential-r1/runtime-review.md)，本次检查未执行Windows实机

视觉产物完成实际查看与hash登记后已清理，本任务三个目录删除5382个图片／视频，释放2,821,376,996字节；逐目录结果见[cleanup.json](cleanup.json)。源码、正式资产、原始状态JSON、日志、复现脚本与本记录保留
