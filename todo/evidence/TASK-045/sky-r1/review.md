# 白天天空层次第 1 轮

## 输入与根因

实际查看 `source-assets/area-previews/shop-street.png` 与 `cinema-music-street.png`，读取美术方向、当前 `daylight.json`、正式入口和 Viewer 的共用相机初始化、画质设置中的雾与曝光路径

当前依赖为 Bevy 0.19.1。本机 `bevy_light-0.19.1/src/probe.rs` 的 `EnvironmentMapLight::hemispherical_gradient` 实际生成六个 1×1 像素面，四个侧面都使用地平色；将其直接作为可见 Skybox，使平视天空缺少高度层次

## 实现与边界

- `game/src/world/visual.rs` 沿用原生 Skybox，创建 128×128×6 的方向立方图；现有天顶、地平与地面颜色在线性空间插值，保存为 sRGB，并显式使用线性采样
- 太阳周围的克制辉光沿用已有 `sun_position` 和 `sun_color`；立方图方位遵循 Bevy `bevy_core_pipeline-0.19.1/src/skybox/skybox.wgsl` 的 Z 翻转，避免光照与天空相反
- 地平线保持 `sky_horizon`，与现有 DistanceFog 相接；环境补光、太阳强度、曝光、阴影、抗锯齿和画质菜单行为保持原值
- 六面原始数据新增 393,216 bytes，不加载外部天空贴图、不新增光源、shader 或依赖；重用正式游戏与 Viewer 的同一初始化路径

本轮是按美术参数生成的方向天空，不是物理大气、动态云层、天气模拟或局部反射。粗粒度环境补光继续沿用原有近似；本轮不据此声称完成准确天空 IBL。原生 Atmosphere 会同时加入空气透视及 GPU LUT，本轮未引入，以保持现有距离雾开关职责和低成本路径

## 验证交接

`rustfmt --edition 2024 game/src/world/visual.rs` 已执行。新增一个可执行窄测，覆盖六面中心方向、每条边的相邻连续方向、地平色、天顶差异、同一太阳方向的辉光和不透明像素数量；原相机测试继续覆盖 MSAA/TAA/SSAO 兼容预通道，并确认天空为六面 128px 图像

编译、Rust 窄测与 GPU capture 由主 Agent 串行执行，写入后续日志；在收到这些实际结果前均为 NOT RUN。画面需在与修改前相同的正式游戏机位和路线中检查天空渐变、方位接缝、建筑轮廓与雾，不将 CPU 检查视作视觉验收

在仓库根执行，以下命令可直接用于 fish

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib world::visual::tests
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-shop-exterior.json --output output/capture/task045-sky-next
```
