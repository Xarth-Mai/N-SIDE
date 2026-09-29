# 连续坡度地表过渡 r4

基线 `f93af27`，目标是消除 r3 按三角形硬切草／岩的材质边界。TASK-045 保持 active；本轮复用已登记的 Ground037 和 Rock043L，不增加资产供应商、采样噪声或通用渲染框架

## 实现

`geometry.rs` 恢复完整自然地面 `/terrain` 批次，移除 45° 逐面分区和专属岩面顶点染色。原顶点、三角顺序、连续法线、米制主轴 UV 与碰撞几何保留，地图稳定 ID 不变；r3 的 `/terrain/rock` 是渲染派生批次，现在合回原来源。人工平台、道路与建筑继续使用既有标准材质

`terrain_material.rs` 使用一个 `ExtendedMaterial<StandardMaterial, TerrainBlend>`。Ground037 保留在 StandardMaterial 的基础颜色／法线槽，扩展仅增加 Rock043L 颜色／法线与两组参数。纹理及尺度继续从 `appearance.json` 已有 `terrain`、`terrain_rock` 读取；缺少任一必需图对时明确失败

`terrain-slope.wgsl` 根据未被法线贴图扰动的、插值后的世界空间地表法线计算权重，35° 以下以地被为主、55° 以上以岩面为主，中间用 smoothstep 过渡。这是围绕旧 45° 值的本轮试验区间，不由绝对高程、镜头或随机数驱动

颜色、粗糙度与真实法线使用同一权重；颜色按 sRGB 解码到线性后混合，两个 NormalGL 图按线性读取并经 Bevy 的 MikkTSpace 变换。基础颜色仍保留现有地被顶点色，岩面使用其已登记原色。材质复用 Bevy PBR 光照与后处理，不改真实阴影和顶点位置

法线 prepass 与普通前向使用同一岩面法线混合函数；启用 `LOAD_PREPASS_NORMALS` 时前向保留已经混合的预通道法线，避免再次混合。运动向量仍由当前与上一帧真实世界位置生成。当前地表为不透明正面材质，扩展不承担透明、双面、meshlet 或 deferred 材质通用支持

## 本地 API 核对

- `bevy_pbr-0.19.1/src/extended_material.rs`：`MaterialExtension` 的前向及预通道入口；扩展不声明 bindless，避免混用绑定布局
- `render/pbr_fragment.wgsl`：`pbr_input_from_standard_material`、`LOAD_PREPASS_NORMALS` 与标准法线图路径
- `render/pbr_functions.wgsl`：`calculate_tbn_mikktspace`、`apply_normal_mapping` 与光照／后处理
- `render/pbr_prepass.wgsl`、`prepass/prepass_io.wgsl`：NORMAL、MOTION_VECTOR、UNCLIPPED_DEPTH 各预通道输出条件
- 当前 DDS 导出为完整 mip 的未压缩 BGRA8，两个 NormalGL 不是双通道法线编码；沿用原采样、缩放与生成流程

## 检查与运行入口

本子任务已运行 `rustfmt --edition 2024 game/src/world/geometry.rs game/src/world/terrain_material.rs` 及 `git diff --check`，均通过。Cargo 与 GPU 后由主 Agent 串行完成，结果见文末集成复验

按 `ponytail-review` 自查本子任务差异：一个实际使用的材质扩展替代逐面批次，复用既有纹理、标准材质和真实地形；没有额外供应商、独立演示或通用采样框架，结论 `Lean already. Ship.`。这仅评价复杂度，不替代编译与画面验收

```fish
cargo test --manifest-path game/Cargo.toml --lib terrain_
cargo test --manifest-path game/Cargo.toml --lib clipped_terrain_keeps_continuous_normals_without_moving_the_surface
cargo test --manifest-path game/Cargo.toml --lib real_map_generates_finite_geometry_without_raising_ground_to_roof
cargo test --manifest-path game/Cargo.toml --lib lifecycle_missing_terrain_rock_fails_immediately_and_stays_failed
```

窄测分别覆盖图对依赖失败、原生阴影／预通道入口、原地形坐标／法线与实际碰撞、道路裁切后共享点连续法线／颜色、全图高程和 UV 面积下界。它们不能证明 WGSL 实际编译、材质观感或预通道最终一致

额外生命周期负例从实际 appearance 删除 `terrain_rock`，沿 `SceneLoading` 与真实加载系统运行；首帧必须报告明确 binding 错误，之后仍失败、无已生成地图实体、宿主继续运行。该测试不模拟错误返回或等待超时

运行需在同一 `eye-descent-cut` 脚本做前后对照，并覆盖 TAA+SSAO 的法线预通道。重点实际查看草岩硬边是否变弱、两种材质是否仍可辨认、转头时边界是否稳定以及阴影是否一致；大型楔形坡体属于另外的地图形状问题，不把贴图过渡记作山体整体完成

`integration.patch` 是交给主 Agent 的 scene/mod 最小接入差异，不是第二份实现源；正式代码为唯一运行路径。此补丁已应用，真实结果、画面自查与清理见批次记录

## root 集成复验

集成、编译及 Clippy 通过；实际普通 MSAA 与 TAA／SSAO 切坡录制各90帧／6项通过，并实际查看画面。林群总览可辨连续草岩交界，远景仍有平铺重复；缺失 shader 的隔离副本以状态失败和退出码1结束。具体命令、失败试案与范围见[批次记录](../visual-r7/review.md)，原始脚本、日志、状态摘要及画面 hash 一并保留
