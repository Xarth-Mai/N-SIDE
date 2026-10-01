# R5 环境反射最小 A/B 准备

2026-10-01，状态 `COMPLETED / candidate rejected`。候选 [candidate.patch](candidate.patch) 已在 R4 后实际做 A/B 运行与看图，未获得足够收益并撤回；以下保留准备约束、实际机器结果与负实验结论，正式源码沿用原环境光

## 单一实验变量

A 保持当前 `EnvironmentMapLight::hemispherical_gradient`：diffuse / specular 共用六个 1px 色面。B 将同一幅已有 `128×128×6` 程序天空交给 `GeneratedEnvironmentMapLight` 原生过滤，Skybox 与环境光复用图片 handle；移除旧静态环境组件，保留 `environment_intensity=1100`、`sky_brightness=1300`、曝光 `9.7`、太阳、阴影、材质和全部几何

这是环境图内容及原生过滤的实验，两种 diffuse / specular 分布均会变化；相同强度不保证截图中每面墙亮度相同。目标是检查不透明窗面是否获得随视角变化的天空明暗与更清楚的材质响应，不生成室内、街道或人物镜像，不把整体变亮或目标图相似度当作成功依据

## 当前版本与计算边界

- `game/Cargo.toml` 使用 Bevy 0.19.1 默认 feature，加 `jpeg` / `dds`；现有 `lib-bevy.json` 构建指纹确认 `3d_bevy_render`、`bevy_pbr`、`bevy_light`、`bevy_render` 已启用，无需改依赖
- 组件路径为 `bevy::light::GeneratedEnvironmentMapLight`，也在当前 prelude 中；`DefaultPlugins → PbrPlugin → LightProbePlugin → EnvironmentMapGenerationPlugin` 已安装该路径。正式入口和 Viewer 只禁用 Audio，以及 headless 的 Winit / Gilrs，没有禁用 PBR；现有 RenderPlugin 仍通过 Vulkan 建立 RenderApp
- 本机 `bevy_pbr-0.19.1/src/light_probe/generate.rs:1018` 仅在缺少 `EnvironmentMapLight` 时创建原生输出贴图，不能同时预插旧环境光充当占位，否则不会生成输出
- 同文件 `:842` / `:942` 的 downsampling 与 filtering 系统在每个 Render 调度运行，查询没有 Changed 过滤，也没有 dirty / update-mode 字段；当前静态天空仍逐帧复制、生成 mip、过滤 diffuse 与 specular。现有 API 没有一次完成事件或一键静态烘焙开关，本轮不添加自有 GPU 回读或完成状态框架
- 原生组件依赖 compute 与至少 6 个 storage texture 绑定，不支持的设备会关闭生成插件。B 必须核对实际启动日志与画面，确认环境图没有缺失或永久黑图；静态 CPU 窄测不证明 GPU 过滤完成
- 现有 Skybox 与环境采样 shader 均在采样时翻转 Z，可复用同一程序天空；不增加第二幅方向相反的图片

上述是已安装源码和当前构建指纹证据，具体路径与 hash 见 [preparation.json](preparation.json)。若每帧过滤的实际成本不适合本项目，应放弃本候选或另行评估预过滤资产，不在本次实验提前建设 IBL 框架

## 同机位与成本记录

直接复用 R3 V-55 西立面的 [v55-west.json](v55-west.json)：1280×720、30 FPS、120 帧、MSAA4，相同横移输入与停止断言。A 和 B 均使用完成 R4 后冻结的地图、GLB、纹理及 daylight；先用当前基线 Viewer 跑 A，再应用 patch、顺序构建 Viewer 并跑 B，两次 `run.json` 分别保存实际二进制和输入版本，避免用旧 R3 画面代替当前 A

```sh
python3 tools/capture.py --binary game/target/debug/map_viewer --script todo/evidence/TASK-045/env-reflection-r1/v55-west.json --aa msaa4 --no-video --output output/env-reflection-r1/A
```

B 只将输出目录改为 `output/env-reflection-r1/B`；目录均须为新目录。源码改变后先跑 `cargo test --manifest-path game/Cargo.toml --locked --lib world::visual::tests`，再顺序执行 `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer -j 1`

- 核对两组所有相机 position / rotation、脚本、材质与几何输入一致，原捕获断言均 PASS；允许记录原生环境图自身新增的图像资源，不将其误判为世界几何变化
- 实际查看关键帧 29 / 49 / 79 / 119 和连续帧 30–34、60–64，分别记录窗面、窗框投影、相邻墙面和字牌。检查反射随横移的变化及边缘稳定性，不只选一张更亮的静态截图
- 从两组 `state.json.performance` 比较 `world_capture_frame_interval_ms` 与 `world_update_interval_ms` 的 mean / p50 / p95、ready 资源计数及总体墙钟耗时；相同构建 profile、GPU、分辨率、输入和 capture 设置下顺序运行，启动加载与稳态分开记录
- 这些间隔包含读取等待、渲染及 PNG 写入，衡量本工具路径的捕获通量，不能称作纯 GPU 帧时或原生 FPS。GPU 各 pass 时间本轮没有额外启用仪表时记 NOT RUN；若单次差异不足以区分波动，如实记不确定

观察明确改善才保留候选，颜色整体变亮而玻璃仍无有效层次时可判负实验。任何改善均不代表概念图目标品质、作者审美或全城反射已验收

## 本次准备结果

PASS：候选通过 `rustfmt --edition 2024 --check` 的 stdin 检查和 `git apply --check output/env-reflection-r1.patch`；正式 `visual.rs` hash 未改变。Ponytail-review：Lean already. Ship.

NOT RUN：Cargo 编译、CPU 窄测、GPU A/B、性能与视觉验收。本次没有生成截图、视频或临时可执行文件；执行后须先实际查看、记录 hash 与结论，再按项目规则清理视觉产物，保留脚本、patch、参数、日志与状态

## 已执行结果：撤回候选

A／B实际运行各120帧、6项断言PASS；[comparison.json](comparison.json)核对每帧相机、固定时间、世界就绪和实体数一致，测试期间仅visual.rs变化。源与材质均被冻结，分别保留实际二进制SHA，详细归档见[captures.json](captures.json)。Root及独立自查实际看过同帧和连续横移，窗片及玻璃门仍大面积平蓝灰，主要变化为整体稍暗，未见足够的角度层次收益；样品柜本来为开放浅柜，没有作为玻璃评判。结论与图片hash见[R5画面自查](../../TASK-049/blender-integration-r5/visual-review.md)

捕获间隔A→B：mean34.29→36.88ms，p5035.95→38.51ms，p9538.59→42.92ms。单次测量包含PNG及回读，不能当作纯GPU成本或原生FPS；没有足够视觉收益，不继续为这一候选扩大优化范围。已反向应用candidate.patch，正式visual.rs恢复原字节并通过最终构建，当前不使用GeneratedEnvironmentMapLight

技术检查PASS：[3项CPU检查](cpu-B.log)、[候选构建](build-B.log)、两次原生真实渲染；实验结论为不采用，不计为新渲染能力完成。后续只在有明确反射内容与成本依据时评估预过滤资产。媒体完成观察后由主线清理，日志与状态保留
