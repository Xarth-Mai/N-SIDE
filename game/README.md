# 游戏工程

Rust 2024 / Bevy 0.19.1，所有命令从仓库根执行

```sh
cargo run --manifest-path game/Cargo.toml --locked
cargo run --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer -- --project-root .
```

默认游戏入口展示现有 Logo。Map Viewer 启动时读取 `source-assets/district-map/district.json`、`source-assets/district-scene/appearance.json` 与 `source-assets/district-scene/daylight.json`，运行素材来自 `game/assets/`；模型和材质随仓库提供，启动无需下载。地图、外观及光照配置修改后重启，Rust 修改由 Cargo 增量编译

右键按住观察，M 切换鼠标捕获，WASD 移动，Q/E 下降或上升，Shift 加速，滚轮调整速度，Esc 释放鼠标；失焦时停止运动，重新按右键或 M 恢复控制。自由相机可以穿过实体

## 世界运行时

`world::map` 负责 v7 数据和引用校验，唯一坐标转换为 `[x,y,h] → [x,h,-y]`，1 unit = 1 m。`world::geometry` 使用 f64 空间计算生成外部几何，`world::assets` 校验表现资产，`world::scene` 负责加载、材质和实例；`world::visual` 共享白天成像与照明，Viewer 负责窗口和官方 [FreeCamera](https://docs.rs/bevy/0.19.1/bevy/camera_controller/free_camera/index.html)

地图保持唯一空间主数据，建筑容量和室内规划不生成实体。建筑外墙、天井、屋面、入口、道路、台阶、水面、平台及其支撑保留源对象关联；材质、原创招牌和公共模型的来源见[街区外观](../source-assets/district-scene/README.md)与[环境素材](../source-assets/environment-kit/README.md)

地形采用地面道路节点与地形样点的 Delaunay 插值，桥面、屋顶、电梯高层和室内节点排除在自然地面之外。地形与道路、平台、建筑、水域通过多边形裁切接合；挡墙、桥厚、支柱、立面构件属于明确的派生表现，当前参数和验收记录见[Viewer 任务](../todo/archive/legacy/map-viewer.md)

## 诊断与检查

加载错误输出到终端并返回非零退出码，地图错误包含文件、字段路径、对象 ID、原值和原因；资产错误包含文件、源对象、模型场景、材质槽及底层依赖错误。模型、颜色和法线贴图的递归依赖成功、模型完成实例化后才输出 `[world/ready]`。显式可选模型的省略会报告 Warning 和降级计数

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

```sh
game/target/release/map_viewer --view eye-shop
game/target/release/map_viewer --aa taa-ssao --view eye-shop --verify-headless /tmp/n-side-ao-check
game/target/release/map_viewer --uncapped --view eye-corner --verify /tmp/n-side-window-check
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

`game/capture/viewer-tour.json` 是输入与预期的唯一源：`scene` 当前支持 `district`，`view` 使用已有镜头名；`width` / `height`、`fps`、`frames`、`timeout_seconds`、`keyframes` 均可修改。`events` 的帧区间为左闭右开，`keys` 使用 W/A/S/D/Q/E/Shift/M/Escape，`look` 是每个模拟帧的鼠标增量。`assertions` 指定帧区间、最小位移、整个区间最大偏移、最小转角或结束时控制器启用状态

资源依赖和场景实例就绪后预热 30 个渲染帧，再按 `1 / fps` 推进模拟；每张截图收到异步回调并成功落盘后才推进下一帧，等待期间模拟时间为零。固定输入与时间步改善同环境复现，不能保证跨平台、跨 GPU 的像素一致性；当前场景没有随机行为，`seed` 只作为证据记录，尚无随机系统消费它

输出包含 `frames/` 连续 PNG、`keyframes/` 选定帧、`script.json`、`state.json` 断言与每帧状态、`runtime.log`，以及 `run.json` 中的提交、工作树状态、二进制与 Cargo.lock 哈希、命令和耗时。检测到 ffmpeg 时生成 `video.mp4`，缺少时明确记录 `NOT RUN` 并保留完整图片序列；墙钟时间不作为游戏帧率指标

机器检查覆盖资产就绪、每帧有限 Transform、完整截图与脚本断言。读取、写入、超时和断言错误均返回非零退出码，原始诊断留在日志。交付前实际打开关键帧和连续帧，或播放视频，分别记录构图、遮挡、材质和动作观察；生成成功不自动等于视觉通过。`output/` 属于忽略的验收产物，不能进入 `game/assets/`

```fish
python3 -B -m unittest discover -s tools/tests -p test_capture.py
cargo test --manifest-path game/Cargo.toml --features viewer --locked
```

这些测试不依赖 GPU，检查输入脚本、证据序列和进程超时；实际渲染仍需 Vulkan 设备。无设备时保留失败诊断并在具备 GPU 的环境运行录制命令，不将跳过计为通过
