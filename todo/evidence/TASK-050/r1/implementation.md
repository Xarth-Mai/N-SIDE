# 门前交接片段：实现范围与验证入口

## 输入

基线提交 `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3`；已读取当前序章、完整主线制作分场的序章第 3 项、QST-002 的 Demo 因果与恢复边界，以及现有地点观察、正式入口、任务状态责任和 UI 输入实现

调用项目 nside、n-side-work-loop、bevy-ecs-systems、dialogue-systems 与 ponytail 技能；没有另装对话引擎或新建任务解释器

## 实际接入

`game/src/story.rs` 维护这一段交接的运行内状态；观察信息与路线提交分别存储。只有既有距离、朝向与无遮挡检查产生观察对象，再经真实 F / 手柄 South 开窗，才登记对应场所；经过场所、打开暂停或等待不会增加记录

三处场所沿用地图的 `04` 月台杂货、`29` 店侧台阶、`28` 小店货运侧院。乱序观察允许提前取得记录；记录可在任一相关观察窗回看，HUD 按尚缺的事实给出下一目标。三处观察齐备以后仍未提交，玩家必须在货运侧院明确选择配送路线

选择包含「配送走货运侧院」与「配送走店侧台阶」。台阶选项说明推车不能通过，保留当前记录与待办，允许重新选择；服务院选项提交一次，重复打开与确认不会重复获得成果。该判断仅记录路线，不假装角色已实际搬完货或完成序章

键盘 ↑ / ↓ 或手柄方向键切换，F / South 确认，鼠标可点选；Esc / East 关闭，Tab / Start 暂停。开窗输入必须释放后才接受选择；保护性暂停与输入恢复沿用正式入口，鼠标请求也须通过相同输入门禁

## 生命周期

暂停、关闭与重访保留当前进度。返回标题以及重新加载明确重新开始本片段，暂停界面写明重开语义；本轮没有任务磁盘存档，没有将设置保存误称为剧情保存，也未标记 QST-001 或 QST-002 完成

## 适用检查

主 Agent 统一运行 Cargo，子 Agent 不竞争构建锁。窄测入口如下，实际结果另存运行日志

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib story::
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib places::observation::
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib app::tests::observation_
```

状态机窄测覆盖乱序、重复观察、无关场所、缺项提交、错误地点提交、错误路线修订与幂等。观察输入窄测覆盖开窗连按、释放、错误选择、关闭重开与重复确认；这些测试不是实际 GPU 操作证据

当前真实完整路线、鼠标命中、手柄硬件、作者体验和任务存档均为 NOT RUN，后续依据实际日志分别更新；capture 必须注入按键而非写入 observed_places 或 confirmed_route

## 已执行检查与修复

- 初次统一编译 FAIL：`entry_input` 增加叙事状态后超过 Bevy 函数顶层 SystemParam 数量；将同一交互职责的 Observation / ShopHandoff 组合为现有 tuple 形式，未新增调度框架
- 统一 CPU 复验 PASS：[叙事状态及生产入口暂停／重访／标题重开 2 项](../../TASK-049/integration-r1/cpu-r2-2.log)、[地点观察与开窗选择输入 5 项](../../TASK-049/integration-r1/cpu-r2-3.log)、[实际 capture 叙事断言契约 1 项](../../TASK-049/integration-r1/cpu-r2-4.log)
- `git diff --check` PASS；18 秒 `prologue-handoff.json` JSON 及仅输入字段核对 PASS，包含 19 条断言；这些静态结果不替代原生执行
- 独立只读审查确认 125% 大字／窄画幅的累计记录需要滚动；已改为 Bevy ScrollPosition 正文区，标题、选择和返回固定，复用现有 UI 的未布局尺寸保护。滚轮、PageUp / PageDown 与手柄右摇杆滚动；暂停恢复保留滚动位置，新开观察回顶部，实际布局待 GPU 复验
- 独立审查未发现确定的指针冒泡或状态错误；已核对 Bevy 0.19.1 Pointer 传播会更新事件目标，点击按钮子文字仍进入同一 RouteButton 请求门禁

## 本轮证据边界

`game/capture/prologue-handoff.json` 是 540 帧、30 Hz 的门前首片段：实际行走接近月台杂货、观察获取 `04`、设备暂停与恢复、重复查看、标题重开及重入。全程不写 `observed_places` 或 `confirmed_route`；它不会证明已走到 `29` / `28` 或完成配送选择

完整三点路线仍需实际行走录制。道路图中店门到店侧台阶最短 70.5m、台阶到货运侧院最短 339.4m，后者包含同 XY 的 +44m 到 +28m 节点，不能把道路图连通当成玩家碰撞可通行；此结果只用于避开不可靠的自动路线推断，不代表行走验收

本子任务未生成截图、视频或临时探测程序，无新增视觉产物需清理；主 Agent 生成的 GPU 证据依实际查看与日志另行记录
