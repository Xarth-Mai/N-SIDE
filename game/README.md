# 游戏工程

Rust 2024 / Bevy 0.19.1，所有命令从仓库根执行

```sh
(cd game && cargo run --locked)
cargo run --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer -- --project-root .
```

默认游戏入口展示现有 Logo。Map Viewer 启动时读取 `source-assets/district-map/district.json` 与 `source-assets/district-scene/appearance.json`，运行素材来自 `game/assets/`；模型和材质随仓库提供，启动无需下载。地图和外观数据修改后重启，Rust 修改由 Cargo 增量编译

右键按住观察，M 切换鼠标捕获，WASD 移动，Q/E 下降或上升，Shift 加速，滚轮调整速度，Esc 释放鼠标；失焦时停止运动，重新按右键或 M 恢复控制。自由相机可以穿过实体

## 世界运行时

`world::map` 负责 v7 数据和引用校验，唯一坐标转换为 `[x,y,h] → [x,h,-y]`，1 unit = 1 m。`world::geometry` 使用 f64 空间计算生成外部几何，`world::assets` 校验表现资产，`world::scene` 负责加载、材质和实例；Viewer 入口单独负责白天照明、窗口和官方 [FreeCamera](https://docs.rs/bevy/0.19.1/bevy/camera_controller/free_camera/index.html)

地图保持唯一空间主数据，建筑容量和室内规划不生成实体。建筑外墙、天井、屋面、入口、道路、台阶、水面、平台及其支撑保留源对象关联；材质、原创招牌和公共模型的来源见[街区外观](../source-assets/district-scene/README.md)与[环境素材](../source-assets/environment-kit/README.md)

地形采用地面道路节点与地形样点的 Delaunay 插值，桥面、屋顶、电梯高层和室内节点排除在自然地面之外。地形与道路、平台、建筑、水域通过多边形裁切接合；挡墙、桥厚、支柱、立面构件属于明确的派生表现，当前参数和验收记录见[Viewer 任务](../todo/map-viewer.md)

## 诊断与检查

加载错误输出到终端并返回非零退出码，地图错误包含文件、字段路径、对象 ID、原值和原因；资产错误包含文件、源对象、模型场景、材质槽及底层依赖错误。模型、颜色和法线贴图的递归依赖成功、模型完成实例化后才输出 `[world/ready]`。显式可选模型的省略会报告 Warning 和降级计数

```sh
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --all-targets --features viewer --locked -- -D warnings
cargo test --manifest-path game/Cargo.toml --features viewer --locked
cargo build --manifest-path game/Cargo.toml --features viewer --locked
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --validate
```

`--validate` 校验地图、几何和绑定，不启动 GPU，也不替代异步图像解码和视觉验收。渲染验收命令按全图、小店、街景、影院、桥下、山坡六个固定镜头运行，输出 PNG 和帧时间日志后退出；PNG 为测试产物，发布 Wiki 时另导出质量 80 的 WebP

```sh
cargo run --release --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --verify /tmp/n-side-viewer-check
```

每个镜头预热 3 秒并测量约 5 秒，默认窗口为 2560 × 1440、Vulkan、VSync。自动退出码检查截图分辨率、每镜头至少 120 个样本与 P95 ≤ 25 ms，60 FPS 目标另核对日志中的平均帧率。固定镜头采样与持续飞行、输入和完整视觉验收分别记录，实际观察结果见[任务记录](../todo/map-viewer.md)

没有活动桌面时，可使用同一 Vulkan 渲染管线向 2560 × 1440 纹理离屏渲染，检查资产、画面及渲染吞吐；该结果不包含桌面合成、VSync 和人工输入，单独记录

```sh
cargo run --release --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --verify-headless /tmp/n-side-viewer-offscreen
```
