# 自然地面纹理与大尺度调色 r1

基线 `dbe5ad3`；范围为真实街区的自然坡面及公园地表，保留所有地图坐标、稳定 ID、三角面空间位置与碰撞，陡坡 UV 依据实机问题修订

## 诊断与输入

既有地面拥有平滑法线及按高程、坡度变化的顶点色，但 `appearance.materials.terrain` 没有颜色或法线图。大片坡面只有低频色块，走近也没有地表细节；继续增加纯色强度不能补足近景材质

实际阅读 `source-assets/area-previews/shop-street.png` 与 `source-assets/environment-kit/materials/Ground037_1K-JPG_Color.jpg`。概念图用于近景材质密度与植被层次方向，不能据此改动地图；Ground037 实际包含灰棕泥土、疏苔、细枝与枯叶，适用于自然裸露地表

素材来源、许可证、原件哈希、尺寸及可复现导出参数由 [AST-003 清单](../../../../source-assets/environment-kit/asset-manifest.json)维护，不在这里建立第二份资产登记

## 实现

- `game/src/world/geometry.rs::color_ground` 继续使用真实表面法线与高程，在已有网格顶点上增加固定世界坐标的 180 m 主色斑与 75 m 轻变化，主次权重为 0.75 / 0.25，色斑只调制 0.90–1.00 的亮度
- 平缓地面保留草绿色，坡度约 20°至 50°平滑转为较中性的暖土色；高处的绿色略深，仍依同一片地面的连续法线与世界坐标计算
- 调色为颜色图的乘色，顶点色先由 sRGB 转为线性值；颜色图仍按 sRGB、OpenGL 法线图仍按线性加载，沿用标准 PBR 与既有切线生成入口
- `source-assets/district-scene/appearance.json` 的 `terrain` 绑定 `ground-color.dds` / `ground-normal.dds`，纹理一周期为 2.1 × 2.1 m，粗糙度为 0.98
- 不新增 shader、材质类型、地形块或 draw call；新增两张 1K DDS。纹理保留完整 mip，既有 sampler 为 repeat、线性过滤与 8×各向异性

这仍是一套苔土地表与连续调色，不是独立的草层／岩层 PBR 混合

## 检查入口

新增 `terrain_tint_is_continuous_and_preserves_surface_data` 检查调色函数的重复点颜色一致、跨负坐标格线连续、平地有低频变化、陡坡暖土调色及 positions / normals / UV / indices 不变；既有 `clipped_terrain_keeps_continuous_normals_without_moving_the_surface` 和实地图测试继续覆盖裁切共享点及原始高程

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib terrain_
cargo test --manifest-path game/Cargo.toml --locked --lib real_map_generates_finite_geometry_without_raising_ground_to_roof -- --nocapture
bun tools/export-environment.ts --check
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/mountain-city.json --output output/capture/task045-terrain-surface-r1
```

本子任务执行 `rustfmt --edition 2024 game/src/world/geometry.rs` 与范围 `git diff --check` 均通过；Cargo、导出、GPU 由主 Agent 串行执行，实际结果见下文；不以静态机器检查冒充美术认可

## 首次实机自查与投影修复

实际查看 `output/capture/task045-terrain-before/keyframes/frame00149.png` 与 `output/capture/task045-terrain-after/keyframes/frame00149.png`：缓坡新增地表细节，但近处台地侧面与山腰陡面出现明显竖向拉伸，平坡仍能看见规律重复；本次外观自查判为需要修订，不能据机器断言通过宣布成功

本地 Bevy 0.19.1 `StandardMaterial` 只提供 UV0 / UV1 选择与单个 `Affine2` 变换，没有内置 triplanar。修复限于地形材质的现有网格：按每个三角面的真实几何法线最大分量选择 XZ、YZ 或 XY 世界坐标投影，三个方向都保持米制坐标，不混入高度偏移或随机旋转；相邻同轴 chart 使用相同映射，正反法线的映射一致

主轴投影的 UV 面积与三角形真实面积之比至少为 `1 / sqrt(3)`，避免顶视投影在接近竖直时压到接近零。`terrain_projection_keeps_shared_charts_on_both_face_directions` 检查同轴共点、共边与正反面一致，实地图测试逐三角检查该面积界并打印实际最小值。投影在原始三角面阶段生成，保留空间顶点和法线；UV chart 边界可以保留不同 UV 的重复顶点，不改变三角面或碰撞

这是有明确 chart 边界的主轴投影，不是平滑 triplanar；不同主轴交界处的纹理方向变化，以及大坡面重复，需要在修复后的同路线实际复验。没有用降低截图分辨率、改地形轮廓或遮住问题的布置来代替修复

## 投影修复后的实际 CPU 结果

主 Agent 运行的 `output/assets/terrain-r1/world-tests-final.log` 显示 42 项 world 测试通过，包含同轴正反面／共享边检查、色调连续性、真实地面与碰撞支持回归。真实地形 16,130 个三角面的最小 UV 面积比为 `0.5868630942989362`，大于 `1 / sqrt(3) = 0.577350269`；8,584 个地形控制点全部保留，渲染地形高程范围为 `1.8369863–449.91873 m`，顶峰平台另由既有铺装覆盖测试验证

该结果证明真实三角面不再因水平投影压扁纹理坐标，不证明 chart 交界没有可见纹理转向；修复后的 GPU 自查和隔离诊断见下文

## 修复后的 GPU 自查与法线隔离诊断

实际查看 `output/capture/task045-terrain-final/keyframes/frame00149.png`、`output/capture/task045-cut-final-r1/keyframes/frame00029.png` 与 `frame00044.png`。小店机位左下近处台地侧面的严重竖向拉伸明显减轻，说明主轴投影改善了原来近乎退化的表面映射；山腰远处暗面及下山机位左侧大坡面仍有清楚的规律细条，缓坡也能辨认重复图案。不能把面积下界测试通过说成全部条纹消失

主 Agent 保持同一新二进制、镜头、图像与光照，通过独立 `project-root` 只将 `terrain.normal_texture` 改为 `null`。实际查看 `output/capture/task045-cut-no-normal-r1/keyframes/frame00029.png`，斜条纹仍在。该诊断没有支持“关闭法线即可修好”的判断，正式配置继续保留真实法线图

检查本地 Bevy 0.19.1 的 `bevy_image/src/dds.rs`、`ImageSamplerDescriptor::linear` 和 `bevy_pbr/src/render/pbr_fragment.wgsl`：DDS 层数进入运行图像，默认三项过滤均为 linear，当前采样器使用 8×各向异性与最大 LOD 32；标准 PBR 先应用 UV 缩放，再采样颜色／法线图。实际 DDS 的第 0 级 RGB 与原始 JPG 逐字节一致，两个文件各有完整的 11 级 mip，法线细节标准差随 mip 降低；[DDS 数值记录](dds-mips.json)没有显示缺 mip、通道错位或导出高频放大的证据

本轮结论为“严重 UV 压缩已修复，剩余平铺／掠视纹理表现仍需改进”，不是完整地表品质通过。保留原件、颜色／法线图和有界主轴投影，任务继续 active

## 下一轮平铺抑制

先复用 `eye-shop-mountain` 与 `eye-descent-cut`，在独立测试配置中比较原周期与两倍周期，核对细条是否随纹理重复周期变化，同时记录近景细枝／泥土尺度；放大周期是诊断，不自动成为正式参数。随后从同一许可源制作降低大尺度色斑对比、保留局部地表细节的运行候选，继续使用既有 StandardMaterial 与导出流程，比较同机位与连续转向

本轮不增加通用 shader、地形框架或修改山体来掩盖问题。若上述小范围素材处理仍不能压低规则重复，下一轮再以真实对照决定是否需要局部的双尺度材质实现；保留原图与派生关系，不把新处理后的文件声称为上游原件
