# TASK-050 连续路线独立评审

评审日期：2026-09-30；范围为新增的 `walks_prologue_handoff_places_continuously` 与冻结后的 `game/capture/prologue-route.json`，沿用运行基线 `e92802ffd39daf5b7c990306ab23eeb1a78de821`

## 输入与结论

- `game/src/player.rs` SHA-256：`3f2e4d378ef070d38da2889ecb04b0db133a26487ba903ac48cb1556aadc19d5`
- r2 脚本 SHA-256：`87f396199b6eb2133ac691cfeeefa05fbecdc5649562fccce74f590f748aa68f`，2672 帧、30fps、12 项路线断言
- 代码与输入契约评审 PASS：未发现本次改动直接写人物位置、观察目标、已观察集合或任务结果；没有通过更改碰撞规则或放宽断言补齐路线
- 复杂度评审：Lean already. Ship.

## 核对依据

新增 CPU 测试从真实 `home` 出生，在每个时间步配置中仅创建一个 `PlayerState`，沿 27 个停点持续调用生产 `PlayerState::step`；目标高度只用于核对真实网格支撑，没有赋给人物脚点。每段要求水平距离小于 0.06m、脚点与真实支撑高度吻合并着地，保留停滞、总时长、零恢复和零跳跃检查

脚本只注入普通输入，经 `capture::input → player::read_input → move_player/PlayerState::step` 移动；观察目标由 `places::observation::eligible` 读取实际距离、朝向与视线判定，打开观察窗才调用 `ShopHandoff::observe`，完成记录后仍由显式选择调用 `choose`。capture 的 handoff 和 observation 字段读取实际资源并逐帧核对所声明区间

独立解析 r1 留存脚本与 r2 冻结脚本，确认 12 项断言除 `from`／`to` 外逐项深相等，所有输入负载保持不变；唯一时长变化为末段上坡 `[855,860)` 延长到 `[855,893)`，后续输入与对应检查统一后移 33 帧。到达台阶仍要求目标 `29`、着地和高度 39.4–39.9m，完整路线仍要求零恢复、零跳跃，正确路线仍只允许一次确认

## 实际证据与范围

- CPU PASS：已读取[连续路线日志](cpu-walk.log)，64Hz 与30Hz各完成27停点，记录为1项通过、0项失败；本评审未重复运行 Cargo
- r1 GPU FAIL：已核对 `output/capture/task050-full-route-r1/state.json` 与 `run.json`，2639帧保存完整，8项路线断言失败；第861帧脚点 `[118.95690,38.93309,-307.13098]`，距台阶锚点约3.05m且 `observation.target=null`，随后 Escape 打开暂停导致余下行走未执行，不能记为完整路线通过
- r2 沙箱运行 BLOCKED：`output/capture/task050-full-route-r2/run.json` 记录退出101，Vulkan在该执行环境未枚举到有效GPU，未产生 `state.json`；这不是宿主缺少硬件或路线失败的证据
- r2 宿主GPU PASS：独立读取 `output/capture/task050-full-route-r2-native/state.json`、`run.json` 与留存脚本，2672帧、15/15检查通过、运行退出0；留存脚本与冻结文件逐字节一致，二进制SHA-256为 `283501dfe5d7a4c40b0769e97e89f0b4091631227b3dda48b0b2e7b652e626fb`
- 连续性复核 PASS：第59–2671帧共2613个样本均为就绪的world、零恢复、零跳跃，世界实体数量恒定，室内房间均为空；累计水平移动243.441m，相邻帧水平位移峰值0.106667m，与30Hz下3.2m/s的移动上限一致
- 观察与选择复核 PASS：第159／895／2516帧分别开窗取得04／29／28，三点开窗时记录才增加；第894帧台阶脚点 `[119.99912,39.68233,-309.99500]`，第2515帧侧院脚点 `[25.35589,28.04551,-270.20731]`，均着地且目标正确。第2549帧错误选择计数变为1，第2582帧明确选择 `service_yard` 后确认计数变为1，关闭、手柄重开及再次确认至末帧仍保持1
- 画面与体验：本评审仅检查源码、脚本与记录状态，未独立查看画面；主代理另做实际画面／UI评审，作者／玩家体验、物理手柄、完整序章和任务存档不由这份代码评审放行

检查命令：`git diff --check -- game/src/player.rs`、只读Python脚本的JSON时序／状态断言、`bun run check:docs` 均通过；文档检查覆盖412个Markdown文件、102个ID与31个Skills

本评审未修改生产源码、规则文件或任务状态，未运行GPU，未生成截图、原始帧、视频或临时可执行文件
