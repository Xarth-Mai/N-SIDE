# 游戏工程

Rust 2024 / Bevy 0.19.1，所有命令从仓库根执行

```sh
cargo run --manifest-path game/Cargo.toml --locked -- --project-root .
cargo run --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer -- --project-root .
```

默认游戏入口提供标题、加载、固定镜头街区预览与返回标题；当前是人物尺度原型的运行基础。正式入口与 Map Viewer 读取 `source-assets/district-map/district.json`、`source-assets/district-scene/appearance.json` 与 `source-assets/district-scene/daylight.json`，运行素材来自 `game/assets/`；模型和材质随仓库提供，启动无需下载。地图、外观及光照配置修改后重启，Rust 修改由 Cargo 增量编译

## GitHub Actions

`CI` 在推送 main 和 Pull Request 时执行 Rust 格式、类型、文档、任务、叙事与工具测试，地图测试包含在工具测试中；自动检查不编译 Bevy、不构建 Wiki、不运行 GPU 验收

Actions 页面选择 `Build Windows` → `Run workflow` → 分支，构建完成后从运行摘要链接或 Artifacts 下载 `n-side-windows-x86_64-msvc`，保存期为 14 天。手动入口需要 workflow 已存在于默认分支

完整解压后双击 `run-game.cmd` 启动标题与街区预览，或 `run-viewer.cmd` 启动街区 Viewer；保留 `game/`、`source-assets/` 与 `licenses/` 的目录结构，运行需要 Windows 与支持 Vulkan 的显卡驱动。启动脚本支持含空格的解压路径，无需安装 Rust 或克隆仓库

构建使用 `x86_64-pc-windows-msvc`、现有 release 配置和 Cargo.lock，上传前从打包目录执行 Viewer `--validate`；该检查验证地图、几何、素材绑定与光照数据，不证明 Windows 图形启动或视觉验收通过

两个 workflow 使用最新稳定系列的 Actions、`ubuntu-latest` / `windows-latest` runner 和最新稳定 Rust；CI 显式选择 Bun `latest`、Python `3.x` 并查询最新稳定版本。Action 跨主版本需要更新 workflow 引用，项目库依赖仍按锁文件安装，日志记录实际工具链版本

右键按住观察，M 切换鼠标捕获，WASD 移动，Q/E 下降或上升，Shift 加速，滚轮调整速度，Esc 释放鼠标；失焦时停止运动，重新按右键或 M 恢复控制。自由相机可以穿过实体

## 正式入口与场景恢复

标题提供「进入街区」和「退出」，进入后固定在月台杂货附近的观察位置；Esc 或返回按钮退出预览并清理当前场景。方向键选择、Enter 确认，手柄方向键／南键／东键走相同入口；窗口失焦不接收确认操作。人物移动、碰撞、跟随镜头、任务和存档仍由后续原型接入

场景准备在 Bevy 任务池中进行，标题输入和加载反馈保持响应；取消后丢弃旧准备结果。每次进入建立新的 `SceneLoading`，依次核对资产依赖和模型实例化，真实就绪才切换到街区。返回标题移除带 `MapSource` 的场景层级与加载资源，保留宿主镜头、灯光和 UI

`game/src/capture.rs` 为正式入口和 Viewer 的唯一录制实现，默认游戏无需启用 `viewer` 特性。正式入口脚本覆盖两次进入与清理，使用真实输入驱动并比较两次源对象数量

```fish
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/game-entry.json --output output/capture/game-entry
```

脚本的 `waits` 在指定帧等待真实 `game_page`，期间继续异步加载和渲染，暂停录制时间线；它不设置应用状态，墙钟超时仍有效。`game_page`、`world_ready`、`min_world_entities`、`max_world_entities` 和 `same_world_entities` 从实际 ECS 状态检查标题、加载、清理与重入；录制时长不代表真实加载耗时

## 世界运行时

`world::map` 负责 v7 数据和引用校验，唯一坐标转换为 `[x,y,h] → [x,h,-y]`，1 unit = 1 m。`world::geometry` 使用 f64 空间计算生成外部几何，`world::assets` 校验表现资产，`world::scene` 负责加载、材质和实例；`world::visual` 共享白天成像与照明，正式入口使用 Bevy States 管理标题、加载、街区和失败状态，Viewer 使用官方 [FreeCamera](https://docs.rs/bevy/0.19.1/bevy/camera_controller/free_camera/index.html)

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

`eye-slope-support` 从上街实际节点观察平台，`eye-transfer-support` 从镜厅低层公共连廊的3/4位置、上方1.7m看换层平台底面；两者朝向目标的仰角不作为水平街景视角。`inspect-cinema-bearing` 是镜厅屋顶连接旁的自由检查机位，用于近看短梁搭接，不代表行人位置

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

`map_viewer --capture` 使用上面的世界、材质、相机与控制器，通过输入资源驱动已有的 WASD、鼠标观察、M 捕获和 Esc 释放。当前首个场景是小店外的自由相机操作，不包含尚未实现的人物碰撞、任务或动画验收

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

每份 `game/capture/*.json` 是该路线输入与预期的唯一源：`scene` 支持 `district` 和 `ui-signal`，`view` 使用已有镜头名；`width` / `height`、`fps`、`frames`、`timeout_seconds`、`keyframes` 均可修改。`events` 的帧区间为左闭右开，`keys` 使用 W/A/S/D/Q/E/Shift/M/Escape/Tab/Enter/Up/Down/PageUp/PageDown，`look` 是每个模拟帧的鼠标增量。`assertions` 指定帧区间、最小位移、整个区间最大偏移、最小转角或结束时控制器启用状态

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
