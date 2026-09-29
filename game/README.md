# 游戏工程

Rust 2024 / Bevy 0.19.1，所有命令从仓库根执行

```sh
cargo run --manifest-path game/Cargo.toml --locked -- --project-root .
cargo run --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer -- --project-root .
```

默认游戏入口提供标题、加载、固定镜头街区预览与返回标题；当前是人物尺度原型的运行基础。正式入口与 Map Viewer 读取 `source-assets/district-map/district.json`、`source-assets/district-scene/appearance.json` 与 `source-assets/district-scene/daylight.json`，运行素材来自 `game/assets/`；模型和材质随仓库提供，启动无需下载。地图、外观及光照配置修改后重启，Rust 修改由 Cargo 增量编译

## Linux 本地预览包

从已编译的正式入口和 Viewer 生成独立目录，运行资源、来源许可、项目源码与启动脚本共用当前资源布局。输出必须是新目录，`--strip` 只移除包内二进制副本的调试信息；清单记录源提交、相关未提交文件、输入二进制及包内文件hash，`--check`只读核对文件、权限与内容

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
bun tools/package-preview.ts --binary game/target/debug/n-side --viewer game/target/debug/map_viewer --output 'output/packages/N-SIDE Linux Preview' --strip
bun tools/package-preview.ts --check 'output/packages/N-SIDE Linux Preview'
tar -czf output/packages/n-side-linux-preview.tar.gz -C output/packages 'N-SIDE Linux Preview'
```

将包解压到新位置，从解压目录运行 `./run-walk-preview.sh`；可从其他工作目录使用脚本绝对路径，含空格路径需要引号。无需安装 Rust 或 Bun，宿主仍需兼容的Linux动态库与Vulkan驱动。玩家操作见[游玩指南](../docs/player/guide/index.md)

验收使用实际解压根目录，capture与正常启动仍复用同一二进制、系统与资产；`--project-root`同时指定运行资源和子进程工作目录，外层记录工具仍来自仓库

```fish
python3 tools/capture.py --binary '/tmp/N-SIDE Linux Preview/game/n-side' --project-root '/tmp/N-SIDE Linux Preview' --script game/capture/walk-observation-pointer.json --output output/capture/package-observation
```

本入口交付本地预览，不构成完整Demo、跨Linux发行版兼容或公开发布放行。素材与Parry声明随包保留；公开分发前另核对所有链接依赖的许可、目标系统、发布渠道和完整流程

## GitHub Actions

`CI` 在推送 main 和 Pull Request 时执行 Rust 格式、类型、文档、任务、叙事与工具测试，地图测试包含在工具测试中；自动检查不编译 Bevy、不构建 Wiki、不运行 GPU 验收

Actions 页面选择 `Build Windows` → `Run workflow` → 分支，构建完成后从运行摘要链接或 Artifacts 下载 `n-side-windows-x86_64-msvc`，保存期为 14 天。手动入口需要 workflow 已存在于默认分支

完整解压后双击 `run-game.cmd` 启动标题与街区预览，`run-walk-preview.cmd` 启动人物步行实验，或 `run-viewer.cmd` 启动街区 Viewer；保留 `game/`、`source-assets/` 与 `licenses/` 的目录结构，运行需要 Windows 与支持 Vulkan 的显卡驱动。启动脚本支持含空格的解压路径，无需安装 Rust 或克隆仓库

构建使用 `x86_64-pc-windows-msvc`、现有 release 配置和 Cargo.lock，以 `--features viewer --bins` 构建所有当前启用的可执行目标，并从本次 Cargo 产物消息收集程序，避免带入缓存里的旧 exe。完整清单在包内 `executables.txt` 和运行摘要中；当前为 `game/n-side.exe`、`game/map_viewer.exe`，`run-walk-preview.cmd` 只是正式入口的 `--walk-preview` 模式

三个启动脚本均透传附加参数并保留退出码，例如 `run-viewer.cmd --validate`。上传前从包内执行 Viewer 数据校验、三个启动入口的 `--help` 和未知参数失败检查；这些检查验证资源和命令入口，不证明 Windows 图形启动或视觉验收通过

两个 workflow 使用最新稳定系列的 Actions、`ubuntu-latest` / `windows-latest` runner 和最新稳定 Rust；CI 显式选择 Bun `latest`、Python `3.x` 并查询最新稳定版本。Action 跨主版本需要更新 workflow 引用，项目库依赖仍按锁文件安装，日志记录实际工具链版本

Viewer 自由相机使用右键按住观察，M 切换鼠标捕获，WASD 移动，Q/E 下降或上升，Shift 加速，滚轮调整速度，Esc 释放鼠标；失焦时停止运动，重新按右键或 M 恢复控制。自由相机可以穿过实体，步行入口的操作见下文

## 正式入口与场景恢复

标题提供「进入街区」「退出」和「设置」，进入后固定在月台杂货附近的观察位置；Esc／Tab或手柄Start／东键打开暂停菜单；继续保留当前会话，返回标题清理当前场景。方向键选择、Enter 确认，手柄方向键／南键／东键走相同入口；窗口失焦不接收确认操作。追加 `--walk-preview` 可进入同地图的人物尺度实验，标题改为「开始门前交接」或「继续门前交接」，已观察的信息与确认路线支持自动保存；完整序章尚未实现

场景准备在 Bevy 任务池中进行，标题输入和加载反馈保持响应；取消后丢弃旧准备结果。每次进入建立新的 `SceneLoading`，依次核对资产依赖和模型实例化，真实就绪才切换到街区。返回标题移除带 `MapSource` 的场景层级与加载资源，保留宿主镜头、灯光和 UI

`game/src/capture.rs` 为正式入口和 Viewer 的唯一录制实现，默认游戏无需启用 `viewer` 特性。正式入口脚本覆盖两次进入与清理，使用真实输入驱动并比较两次源对象数量

```fish
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/game-entry.json --output output/capture/game-entry
```

脚本的 `waits` 在指定帧等待真实 `game_page`，期间继续异步加载和渲染，暂停录制时间线；它不设置应用状态，墙钟超时仍有效。`game_page`、`world_ready`、`min_world_entities`、`max_world_entities` 和 `same_world_entities` 从实际 ECS 状态检查标题、加载、清理与重入；录制时长不代表真实加载耗时

暂停复用街区信号的颜色、字体与焦点组件，显示继续、返回标题和设置；步行模式已有交接观察时另有「重新开始交接」，需单独确认。菜单在固定更新前处理，打开当帧与暂停期间隔离人物移动、镜头观察和复位；恢复沿用原人物、镜头与世界，持续按住的玩法输入需要先释放。主窗口失焦自动进入暂停，回焦后需重新确认继续；同一帧先失焦后回焦也会暂停，系统重复按键不会作为新的菜单动作。加载可继续完成，但失焦时不启动人物操作；无窗口capture保持原流程。当前暂停覆盖已实现的人物、镜头和显式角色预览的Idle／Walk／Run；NPC、战斗及声音尚未接入，不据此宣称其暂停行为已验证

```fish
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-pause.json --output output/capture/walk-pause
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-focus.json --output output/capture/walk-focus
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-gamepad.json --output output/capture/walk-gamepad
```

该20秒脚本在真实街区走动后，通过键盘和模拟手柄打开菜单、在暂停期间持续发送移动／观察／复位输入、恢复、返回标题并重入。`max_rotation` 检查整个断言区间的相机转角峰值，能检测转动后又回到原朝向的泄漏；检查采用0.002弧度的浮点容差，另与画面观察分别记录

`walk-focus.json` 使用模拟窗口焦点验证加载期间失焦、回焦持键、显式继续与再次进入。`events.focused` 仅用于正式游戏场景，省略时保持上一次焦点值；仅含该字段的脚本创建没有Winit宿主的逻辑主窗口，仍向原离屏纹理渲染。输入按原生路径先更新 `Window.focused` 再发送 `WindowFocused`，不写游戏阶段或人物位置；这是焦点事件的渲染回归，桌面合成器切窗和物理设备另验

当前手柄输入为聚合控制，任一手柄断开都会暂停并切回键鼠提示；暂停说明连接中断或已经重连，玩家释放旧输入后可用键鼠或手柄明确继续。加载期间断连仍完成资产准备，进入街区时暂停；返回标题、失败或重新加载清理本次连接恢复提示。初次接入设备不会主动暂停行走，连接变化当帧不接受菜单动作

`walk-gamepad.json` 录制18秒的断连、重连持键、键鼠接管和重入。`events.gamepad_connected` 仅用于正式游戏场景，省略时保持上一连接状态；脚本在PreUpdate发送 `GamepadConnectionEvent`，由Bevy InputPlugin实际移除／恢复Gamepad组件，再由正式入口决定是否暂停。`gamepad_connected`状态断言核对结束帧实际组件存在情况，运动及镜头冻结仍检查整个区间；模拟连接不证明USB、蓝牙或驱动行为

标题或暂停中的「设置」提供通用与画质两页签，退出设置返回来源菜单的原焦点；可在公共观察暂停期间修改，再继续原观察。正式入口自动保存文字100%／125%、镜头100%／65%及已确认的画质设置，下次启动恢复；交接进度使用独立文件。用户目录、版本、备份、失败处理、三种镜头输入与布局见[会话设置契约](../docs/dev/engineering/player-preview.md#会话设置)，19项画质及显示确认见[画质契约](../docs/dev/engineering/graphics-settings.md)，不与下方Viewer的SignalUi样板状态混用

改变窗口模式或分辨率会试用15秒，默认选中恢复，只有主动选择保留才接受新值；Esc、失焦、手柄断开或倒计时结束恢复原显示设置，未确认值不写入设置文件。其他画质即时应用，显示恢复不会回退字号、镜头或其他画质项

画质页末尾「恢复默认画质」重置19项，保留字号与镜头；若改变了窗口模式或分辨率，同样走显示确认，取消仅恢复原显示两项

已保存的画质导致启动画面不可用时，可追加 `--reset-graphics`，只恢复默认画质并保留字号与镜头设置；正常文件继续备份保存，无法读取的原文件仍受保护，不被默认值覆盖。Linux和Windows的正式入口启动脚本均透传该参数，以下命令分别用于仓库根与已解压Linux包

```fish
cargo run --manifest-path game/Cargo.toml --locked -- --project-root . --walk-preview --reset-graphics
./run-walk-preview.sh --reset-graphics
```

```fish
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-settings.json --output output/capture/walk-settings
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-observation-settings.json --output output/capture/walk-observation-settings
```

两份脚本分别声明20秒设置往返和26秒观察／设置整合路径，复验使用新的输出目录。实际执行结果与画面判断留在对应任务证据

capture默认隔离真实用户设置。以下使用一组全新的输出目录与独立设置目录，先保存再以第二个进程验证恢复；正常启动也可用 `--settings-dir DIRECTORY` 覆盖用户目录，目录不可读写或文件损坏时页面保留失败提示，本次偏好仍可使用

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib settings::tests
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-settings.json --output output/capture/settings-write --settings-dir output/capture/settings-user
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-settings-restore.json --output output/capture/settings-restore --settings-dir output/capture/settings-user
```

## 人物尺度实验

`--walk-preview` 复用正式入口和地图，在 `/nodes/home` 放置1.7m高、半径0.3m的中性胶囊代理。WASD／左摇杆移动，左右Shift／鼠标右键／左摇杆按钮按住疾跑，空格／西键跳跃，锁定鼠标观察（M释放，解锁后左键拖动）／Q E／右摇杆转动镜头，F／南键A查看近处公共地点信息，R／Select返回起点；未打开观察面板时Esc／Tab／Start／东键B暂停。当前为城市与小店首层公共区的步行技术实验，不确定正式主控、角色造型或最终手感

```fish
cargo run --manifest-path game/Cargo.toml --locked -- --project-root . --walk-preview
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-preview.json --output output/capture/walk-preview
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-ramp-camera.json --output output/capture/walk-ramp-camera
```

显式 `--character-preview` 隐含步行模式，将曜的灰阶骨骼候选挂到原人物视觉root，复用真实移动、碰撞和相机；模型、具名Idle／Walk／Run与所有动画目标就绪后才隐藏胶囊。动作依据实际碰撞后速度切换，暂停停止实际AnimationPlayer，返回标题递归清理；加载失败保留代理、输出错误，capture返回非零。该造型尚未获作者或美术验收，默认入口不替换代理；混合过渡、足滑校准和Jump动画仍未完成

```fish
cargo run --manifest-path game/Cargo.toml --locked --bin n-side -- --project-root . --character-preview
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-character.json --output output/capture/walk-character
```

角色脚本核对实际Idle、Walk、Shift／右键Run、暂停及恢复的时间推进，须同时检查连续画面；`CharacterStatus` 提供ready、错误、动作与时间、切换数和暂停读数。完整接口与已知边界见[灰阶角色动作预览](../docs/dev/engineering/player-preview.md#灰阶角色动作预览)，结果记录于TASK-047

碰撞从同一份渲染网格建立静态三角 BVH，覆盖地形、道路、建筑、平台及结构设施；道路台阶的踏面与立面均参与扫掠。Parry只负责空间查询，人物和镜头系统由 `player.rs` 维护；相机沿目标到期望位置作球体扫掠。参数、当前覆盖与失败处理见[人物尺度实验契约](../docs/dev/engineering/player-preview.md)

小店V-04首层的接待与陈列、主题陈列和预约洽谈开放直接行走。房间和门洞来自地图 `design.floors` 的1F `rooms`／`openings`，地板、墙、天花和门头进入相同渲染及碰撞路径；后场、私人内梯和楼上继续封闭。通过WASD／左摇杆从街道进入、穿过内门并原路返回，不使用F传送或场景切换，F仍负责公共信息观察

```fish
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-shop-interior.json --output output/capture/walk-shop-interior
```

该路线检查从真实出生点经过三间公共房再回到街道，室名来自脚点所在的源房间。门净宽、头顶、封闭墙与跟随镜头分别检查，运行记录和画面判断归任务证据，不据此放行人物造型或主观体验

短登高检查分为整段CPU移动探针和真实入口录制：前者从小店沿源路线逐点行走到摘星台，遇到停滞、异常恢复或超时返回失败；后者包含小店出发片段和完整上山脚本。二者均不写入目标高度或传送角色，人工体验另行记录

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib walks_complete_short_ascent_and_return -- --nocapture
python3 tools/capture.py --script game/capture/walk-ascent-entry.json --output output/capture/walk-ascent-entry --binary game/target/debug/n-side
python3 tools/capture.py --script game/capture/walk-ascent-full.json --output output/capture/walk-ascent-full --binary game/target/debug/n-side
```

完整路线脚本的 `route: {"id":"hill-short","start":60}` 从当前项目根读取地图路线，按真实脚点与相机朝向发摇杆及鼠标输入。节点必须有相连道路和真实可行走网格；到达检查水平距离、支撑高度、着地和零恢复。两秒无有效进展明确失败，终点释放输入；路线不得与同一时段的手写输入争用。`state.json` 的 `route_all_nodes_reached` 检查内保存 `route`每个实际到达节点与帧号，路线全部到达才通过。`round_trip: true` 在峰顶后按源节点逆序原路返回，峰顶只计一次；`walk-round-trip.json` 用同一会话验证259次节点到达和回店后的稳定状态

人物入口左上在街道显示地图公共入口附近的真实地名，小店首层公共房间内显示所在室名，左下显示当前键鼠／手柄操作提示。位置或高度超过判定范围时回退Null Site；进入暂停隐藏，继续后恢复

```fish
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-places.json --output output/capture/walk-places
```

靠近月台杂货公共入口（04）、摘星台（23）、店侧台阶（29）或货运侧院（28），转动镜头使目标在视野内，出现F／A提示后可打开现有用途、开放说明与到达方式。右摇杆只是转动镜头，查看信息需另按F／A；远处的「附近」标签不表示已经可以查看；实际记录范围见下方门前交接契约

门前交接记录04／29／28的实际观察，顺序不限、重复查看不重复登记；三点齐备后，在28面板显式选择配送走货运侧院或店侧台阶。上下方向键／手柄方向键切换，F／A或点击确认；台阶选项给出修订反馈，侧院选项只提交一次。返回标题、重新加载或场景失败保留交接事实，正常步行启动还会自动保存到用户目录；暂停保留当前人物位置，重新进入则从家门口出发

面板正文支持滚轮、PageUp／PageDown和右摇杆滚动，标题、路线选项与返回按钮保持固定；同一次打开输入不会提交选择。完整三点路线使用下方 `prologue-route.json`，本轮结果及体验边界见[TASK-050](../todo/tasks/TASK-050-prologue-handoff-runtime.md)

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib walks_prologue_handoff_places_continuously -- --nocapture
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/prologue-route.json --output output/capture/prologue-route
```

路线从真实出生点连续到店门、台阶并返回店外绕行至货运侧院，检查三项观察、错误选项修订和幂等确认；没有传送或任务状态注入。脚本的固定输入时序须随移动或几何修改复验，不能为通过检查扩大观察距离

保存范围仅为这段交接实际取得的信息与路线结果。标题和暂停里的「重新开始交接」默认选择「保留当前进度」，取消不变；另选「确认重新开始」才清除交接事实、写入新检查点并从家门口重建场景。保存或读取失败会显示状态，保留原文件与当前可玩的会话；修复文件或目录后重新启动，路径、字段及备份边界见[交接进度保存](../docs/dev/engineering/player-preview.md#交接进度保存)

capture默认不读取或修改真实用户进度，`--settings-dir` 也不启用进度写入。以下使用一组新的目录，两个进程先实际观察保存，再验证继续、取消重开与确认重开；`--progress-dir DIRECTORY` 同样可用于正常步行启动，固定镜头入口不保存交接

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib progress::tests
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/prologue-save-write.json --progress-dir output/progress/handoff-check --output output/capture/handoff-write
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/prologue-save-restore.json --progress-dir output/progress/handoff-check --output output/capture/handoff-restore
```

上述恢复脚本最后确认重新开始，只能用于独立测试目录；完整确认恢复、损坏文件与写入失败分别使用 `prologue-save-confirmed.json`、`prologue-save-corrupt.json` 和 `prologue-save-write-failure.json`，准备条件与实际执行结果见[TASK-051](../todo/tasks/TASK-051-prologue-progress-save.md)

打开观察时人物与镜头停留原位；Esc／B或鼠标左键点击「返回街区」关闭面板而不打开暂停，空白点击不关闭，Tab／Start仍可暂停。关闭当帧及旧输入持有期间继续隔离玩法，释放后再操作。断连或失焦保护暂时隐藏观察，明确继续后恢复原内容；回到标题清空瞬时面板与选定地点，已取得的交接事实保留。近距、高差、朝向、实际墙体遮挡和输入隔离的完整契约见[公共地点观察](../docs/dev/engineering/player-preview.md#公共地点观察)

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib observation -- --nocapture
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-observation.json --output output/capture/walk-observation
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-observation-pointer.json --output output/capture/walk-observation-pointer
```

该18秒脚本实际行走到小店入口，检查开关面板、持键隔离、模拟手柄断连恢复与标题重入；`observation_target`、`observation_selected`、`observation_open`、`observation_visible`记录真实目标、选定地点及面板状态，结合人物／镜头断言与实际看图复验，结果归本轮任务记录

鼠标脚本录制20秒，通过 `pointer: [x, y]` 与 `left_mouse` 驱动原生PointerInput及图像目标上的UiPicking实际命中，覆盖空白点击、按钮关闭、持键、重新移动和暂停恢复；没有直接写Interaction或观察结果。坐标随画幅与布局变化调整，窄屏与放大字体需使用对应脚本检查命中和全文可读性；这项证据不代替标题、暂停等其他Shell按钮的鼠标验收

## 世界运行时

`world::map` 负责 v7 数据和引用校验，唯一坐标转换为 `[x,y,h] → [x,h,-y]`，1 unit = 1 m。`world::geometry` 使用 f64 空间计算生成外部几何，`world::assets` 校验表现资产，`world::scene` 负责加载、材质和实例；`world::visual` 共享白天成像与照明，正式入口使用 Bevy States 管理标题、加载、街区、暂停和失败状态，Viewer 使用官方 [FreeCamera](https://docs.rs/bevy/0.19.1/bevy/camera_controller/free_camera/index.html)

地图保持唯一空间主数据，建筑容量和室内规划不生成实体。建筑外墙、天井、屋面、入口、道路、台阶、水面、平台及其支撑保留源对象关联；材质、原创招牌和公共模型的来源见[街区外观](../source-assets/district-scene/README.md)与[环境素材](../source-assets/environment-kit/README.md)

地形采用地面道路节点与地形样点的 Delaunay 插值，桥面、屋顶、电梯高层和室内节点排除在自然地面之外。地形与道路、平台、建筑、水域通过多边形裁切接合；挡墙、桥厚、支柱、立面构件属于明确的派生表现，当前参数和验收记录见[Viewer 任务](../todo/archive/legacy/map-viewer.md)

## 诊断与检查

正式入口的场景加载错误显示失败页并允许重试或返回；启动配置错误、Viewer 与 capture 的失败输出诊断并返回非零退出码。地图错误包含文件、字段路径、对象 ID、原值和原因；资产错误包含文件、源对象、模型场景、材质槽及底层依赖错误。模型、颜色和法线贴图的递归依赖成功、模型完成实例化后才输出 `[world/ready]`。显式可选模型的省略会报告 Warning 和降级计数

```sh
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --all-targets --features viewer --locked -- -D warnings
cargo test --manifest-path game/Cargo.toml --features viewer --locked
cargo build --manifest-path game/Cargo.toml --features viewer --locked
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --validate
```

`--validate` 校验地图、几何、绑定和光照配置，不启动 GPU，也不替代异步图像解码和视觉验收。渲染命令保留全图、小店、街景、影院、桥下、山坡、校园高差七个镜头，并增加 `eye-shop`、`eye-corner`、`eye-shade` 三个人眼高度视点；眼高为道路节点上方 1.7 m，视场统一 55°。输出 PNG 和帧时间日志后退出；PNG 为测试产物，发布时另导出质量 80 的 WebP

```sh
cargo run --release --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --verify /tmp/n-side-viewer-check
```

每个镜头预热 3 秒并测量约 5 秒，默认窗口为 2560 × 1440、Vulkan、VSync。自动退出码检查截图分辨率、每镜头至少 120 个样本与 P95 ≤ 16.67 ms。日志的 `cpu_frame_interval` 来自 `Time<Real>`，是应用帧间隔，包含 CPU、GPU 等待和呈现节奏，不能当作独立 GPU 渲染耗时。固定镜头采样与持续飞行、输入和完整视觉验收分别记录，实际观察结果见[白天样板](../todo/archive/legacy/daylight-visual.md)

没有活动桌面时，可使用同一 Vulkan 渲染管线向 2560 × 1440 纹理离屏渲染，检查资产、画面及渲染吞吐；该结果不包含桌面合成、VSync 和人工输入，单独记录

```sh
cargo run --release --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --verify-headless /tmp/n-side-viewer-offscreen
```

## 白天调校

默认采用固定曝光、HDR、TonyMcMapface、MSAA 4×及太阳阴影；接触阴影与 Bloom 默认关闭。三色天空与环境照明共用 Bevy 原生半球颜色贴图，提供基础环境反射；周边建筑的局部反射留给后续素材与探针

`--aa msaa4|taa|taa-ssao` 比较抗锯齿与遮蔽；后两者自动关闭 MSAA，切换固定镜头时重置 TAA 历史。`--visual PATH` 指定替代光照配置，可用关闭接触阴影或改变环境强度的配置做同机位对照；`--view NAME` 选择启动位置或单镜头检查。窗口性能对照使用 `--uncapped` 关闭 VSync，正常操作保留默认 VSync，离屏模式无显式帧率限制

山城灰盒使用 `eye-station`、`eye-cinema`、`eye-shop-mountain` 核对站前、镜厅前街公共路口与小店门前的山体视线，均取源道路节点上方 1.7m，以55°视场水平望向山顶方向；近山峰顶可以超出水平视场，保留该真实结果；`eye-shop-uphill` 在同一眼高明确仰视20°核对山顶，`hillside` 从小店起坡段侧看山腹，`mountain-profile` 从侧向检查河岸至后山的比例。摘星台位于最高点，山体与城区沿用源数据的米制比例，`overview` 随完整地形范围取景

`eye-upper-bridge` 从登高接入路回看上街桥口，检查护栏与道路交会；`eye-slope-support` 从上街实际节点观察平台，`eye-transfer-support` 从镜厅低层公共连廊的3/4位置、上方1.7m看换层平台底面；两者朝向目标的仰角不作为水平街景视角。`inspect-cinema-bearing` 是镜厅屋顶连接旁的自由检查机位，用于近看短梁搭接，不代表行人位置

`inspect-street-tree-detail` 从店侧低台阶平台来路观察既有树实例，`inspect-residential-detail` 从东侧公共节点观察 V-A13 西立面；二者随源节点及建筑标高取景。以下脚本检查接近、横移和释放后的真实相机与画面，不验证人物碰撞或建筑可进入性

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/street-tree-detail.json --output output/capture/street-tree-detail
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/exterior-details
```

```fish
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --view eye-station
python3 tools/capture.py --script game/capture/mountain-city.json --output output/capture/mountain-city
```

```sh
game/target/release/map_viewer --view eye-shop
game/target/release/map_viewer --aa taa-ssao --view eye-shop --verify-headless /tmp/n-side-ao-check
game/target/release/map_viewer --uncapped --view eye-corner --verify /tmp/n-side-window-check
```

## 全城灰盒检查

`city-overview` 按当前本岸楼体取景，`block-B01` 至 `block-B12` 按源地块归属覆盖各街坊的全部已落图建筑；`overview` 继续包含远山。`ascent-overview` 查看短登高全段，`eye-ascent-1` 至 `eye-ascent-3` 从三个停步台的临城侧、眼高1.7m回望城市。全城、逐坊机位按16:9画幅完整取景，使用相同55°视场，不改变地图尺度。构件覆盖包括建筑体量、屋面、按用途派生的门窗、天井和已有街道设施；入口与窗的几何检查不能代替人物碰撞与室内验收

```fish
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --view city-overview
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --view block-B06
python3 tools/capture.py --script game/capture/city-graybox.json --output output/capture/city-graybox
```

## 连续操作 capture

`map_viewer --capture` 使用上面的世界、材质、相机与控制器，通过输入资源驱动已有的 WASD、鼠标观察、M 捕获和 Esc 释放。当前首个场景是小店外的自由相机操作，只覆盖Viewer已有的自由相机控制与渲染；人物碰撞、交接观察和角色动画分别使用正式入口的对应脚本

脚本使用 Python 3.10+ 与资产流程共享的 Pillow 依赖，安装声明见 [create-game-assets requirements](../.agents/skills/create-game-assets/scripts/requirements.txt)。以下命令可直接在 fish 中执行；输出目录必须是新的，默认先构建 debug Viewer，已有构建可加 `--binary game/target/debug/map_viewer`

```fish
python3 tools/capture.py --output output/capture/viewer-tour
python3 tools/capture.py --script game/capture/assertion-failure.json --output output/capture/assertion-failure --binary game/target/debug/map_viewer
test $status -ne 0
```

第一条录制 18 秒、640 × 360、30 fps 的真实输入过程，检查横移、转向、释放后停稳和恢复控制。第二条故意要求相机在短时间内移动 1000 m，必须产生 `FAIL` 及非零退出码；它用于验证断言没有失效

`ascent-lookout.json` 从短登高第三停步台的既有1.7m人眼机位录制18秒环看，保持位置、通过同一自由相机转向并释放输入。它用于检查城市视线与近坡遮挡，不表示人物已经走完登山路线

```fish
python3 tools/capture.py --script game/capture/ascent-lookout.json --output output/capture/ascent-lookout --binary game/target/debug/map_viewer
```

每份 `game/capture/*.json` 是该路线输入与预期的唯一源：`scene` 支持 Viewer 的 `district` / `ui-signal` 以及正式入口的 `game-entry` / `walk-preview`；正式入口 `view` 为 `shop`，Viewer 使用已有镜头名；`width` / `height`、`fps`、`frames`、`timeout_seconds`、`keyframes` 均可修改。`events` 的帧区间为左闭右开，`keys` 使用 W/A/S/D/Q/E/F/R/Space/Shift/ShiftRight/M/Escape/Tab/Enter/Up/Down/Left/Right/PageUp/PageDown，`look` 是每个模拟帧的鼠标增量。`assertions` 指定帧区间、最小位移、整个区间最大偏移、最小转角、整个区间最大转角或结束时控制器启用状态

资源依赖和场景实例就绪后预热 30 个渲染帧，再按 `1 / fps` 推进模拟；每张截图收到异步回调并成功落盘后才推进下一帧，等待期间模拟时间为零。固定输入与时间步改善同环境复现，不能保证跨平台、跨 GPU 的像素一致性；当前场景没有随机行为，`seed` 只作为证据记录，尚无随机系统消费它

输出包含 `frames/` 连续 PNG、`keyframes/` 选定帧、`script.json`、`state.json` 断言与每帧状态、`runtime.log`，以及 `run.json` 中的提交、工作树状态、二进制与 Cargo.lock 哈希、命令和耗时。检测到 ffmpeg 时生成 `video.mp4`，缺少时明确记录 `NOT RUN` 并保留完整图片序列；墙钟时间不作为游戏帧率指标

## 街区信号 UI 样板

`SignalUiPlugin` 在同一真实城市场景中叠加工作菜单、调查记录排版样例、设置和街区说明。`--ui-preview` 是技术与视觉实验入口，正式入口的标题外壳复用相同 tokens 与字体；样例记录不写任务或存档，不可用入口显示原因

```fish
bun tools/export-ui.ts --check
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --view shop --ui-preview
python3 tools/capture.py --script game/capture/ui-signal.json --output output/capture/ui-signal
```

Tab 打开或返回，方向键移动焦点，Enter 确认，Esc 返回；鼠标可选可用按钮，PageUp/PageDown 或滚轮滚动正文。手柄使用方向键、南键确认、东键返回、Start 菜单、肩键翻页；具体提示随当前输入切换。界面缩放 100%/125% 和减少动态效果在本次会话中生效，未实现设置持久化或重映射

Capture 的 `gamepad` 字段发送 `Down`、`Up`、`Confirm`、`Back`、`Menu`、`ScrollDown`、`ScrollUp` 等允许的数字输入，直接经过同一 Bevy `Gamepad` 状态路径；这是模拟输入，不是实物连接、热插拔或手感验收。`ui_page`、`ui_focus`、`ui_record`、`ui_scale`、`ui_device`、`ui_reduced_motion`、`min_ui_scroll` / `max_ui_scroll` 断言读取实际 UI 状态，配合菜单开启期间镜头停移与返回后的真实相机操作检查

字体异步就绪后才开始 UI capture，失败诊断返回非零。UI 样板使用 `scene: ui-signal` 捕获，不与固定镜头 `--verify` 模式混用；固定镜头仍负责世界渲染检查。布局、字形和遮挡必须实际看图，参数与实现状态分别见[UI 规格](../docs/dev/design/systems/ui.md)、[源资产](../source-assets/ui-kit/README.md)与 TASK-026 的运行记录

当前锁定的 Parley 0.9.0 使用 `WordSegmenter::new_for_non_complex_scripts`，含中文时日志会报告 `No segmentation model for complex script: Chinese/Japanese`。本轮字形覆盖、实际显示和换行已分别核对；按词选择尚未实现，不能据此宣称中文分词正确。保留原诊断，暂未为此更换 Bevy 或加入本地 Parley 分支；后续接文本编辑或升级依赖时重新核查。背景、文字与焦点只在值变化时更新，减少无效重排

机器检查覆盖资产就绪、每帧有限 Transform、完整截图与脚本断言。读取、写入、超时和断言错误均返回非零退出码，原始诊断留在日志。交付前实际打开关键帧和连续帧，或播放视频，分别记录构图、遮挡、材质和动作观察；生成成功不自动等于视觉通过。`output/` 属于忽略的验收产物，不能进入 `game/assets/`

```fish
python3 -B -m unittest discover -s tools/tests -p test_capture.py
cargo test --manifest-path game/Cargo.toml --features viewer --locked
```

这些测试不依赖 GPU，检查输入脚本、证据序列和进程超时；实际渲染仍需 Vulkan 设备。无设备时保留失败诊断并在具备 GPU 的环境运行录制命令，不将跳过计为通过

## 跳跃、疾跑与画质回归

基础操作见[游玩指南](../docs/player/guide/index.md)，19项画质范围与兼容关系见[画质契约](../docs/dev/engineering/graphics-settings.md)。以下均经过正式入口和同一城市系统；画质录制通过真实菜单切换相机效果，后一次启动读取独立设置目录

```fish
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-controls.json --output output/capture/walk-controls
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-graphics.json --output output/capture/walk-graphics --settings-dir output/capture/graphics-user
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-graphics-restore.json --output output/capture/walk-graphics-restore --settings-dir output/capture/graphics-user
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-graphics-options.json --output output/capture/walk-graphics-options
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-graphics-large.json --output output/capture/walk-graphics-large
```

脚本中的鼠标锁定采样表示游戏意图，原生窗口锁定／释放、窗口分辨率与VSync需要桌面证据；离屏固定画布不随窗口选项改尺寸。不同GPU不支持某个MSAA采样数时，菜单沿切换方向跳过；首次读取不支持的保存值时回退FXAA，脚本预期需按该设备真实能力记录，不将不支持记为成功启用
