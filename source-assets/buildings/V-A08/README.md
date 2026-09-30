# V-A08 深檐与雨槽

`AST-V-A08-ROOF` 是月台杂货北邻楼 `V-A08` 的独立屋檐样板，主文件为 [roof-eaves.blend](roof-eaves.blend)，运行派生物为 [v-a08-roof-eaves.glb](../../../game/assets/environment/buildings/v-a08-roof-eaves.glb)，登记见 [asset-manifest.json](asset-manifest.json)

## 用途与尺度

保留 [district.json](../../district-map/district.json) 的 18×13m、三层、三个入口与屋顶高程 40.421515m，只将细压顶替换为连续折面深檐；初始预算不超过 128 三角，当前为 56 三角、28 个可编辑四边面和三个材质 primitive

| 项目 | 契约 |
| --- | --- |
| Blender 米制轴 | X 向东、Y 向北、Z 向上，单位缩放为 1 |
| 主文件局部原点 | 屋顶中心，地图 `[79,277.5,40.421515]` |
| GLB 运行轴 | 导出器转换为 Y-up；局部 X±9.45、Y−0.35…0、Z±6.95m |
| 运行放置 | `Scene0`，`map_to_world([79,277.5,40.421515])`，缩放 1，不再次转轴 |
| 新外包络 | 地图 x69.55…88.45、y270.55…284.45、高程40.071515…40.421515m |
| 檐口 | 外挑 0.45m，向下 0.35m，不抬高屋顶；内侧底缘嵌墙 0.03m 形成隐藏接合 |
| 雨槽 | 截面外偏移0.33…0.42m为低槽，外沿回升0.03m；四角连续斜接 |
| UV | 每个平面硬边分岛，1 UV 单位对应 1m，保留折角硬法线 |
| 材质 | `roof`、`metal`、`trim`，沿用构建时 appearance 的颜色、roughness 和 metallic |

材质来自 [appearance.json](../../district-scene/appearance.json)，sRGB 色值转换为线性后写入 Blender Principled BSDF 与 glTF 的 baseColorFactor，同名材质不会在运行时自动重绑到 appearance。源工程保留构建时 appearance 哈希；一般修改在 `.blend` 中完成，再导出，不随 appearance 的后续变化自动覆盖。当前无外部贴图和扩展，保留可用 UV，未新增资产服务或生成费用

## 编辑与导出

以下命令从仓库根目录执行，fish 可直接使用。`build.py` 默认从主文件导出；只有明确重建初始网格时添加 `--rebuild`，此选项会覆盖 `.blend` 中手工修改。主文件只包含目标屋檐；预览中的标准尺寸方盒、灯光和相机只存在于本次进程，不写入主文件或 GLB

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --factory-startup --python-exit-code 1 --python source-assets/buildings/V-A08/build.py
output/tools/blender-4.5.14-linux-x64/blender -b --factory-startup --python-exit-code 1 --python source-assets/buildings/V-A08/check.py -- --output output/assets/v-a08-roof-check.json
output/tools/blender-4.5.14-linux-x64/blender -b --factory-startup --python-exit-code 1 --python source-assets/buildings/V-A08/build.py -- --render-output output/assets/v-a08-roof
```

本机 Blender 路径属于当前可用工具位置；换机器时使用该机器的 Blender 4.5 LTS 可执行文件，不把本机二进制放入源资产包

## 接入边界

`game/src/world/scene.rs` 的 `facades` 外轮廓循环中，只禁用 `building.id == "V-A08" && !court` 下的 `Roof coping` 四个 `add_box`：其中心为 `top−0.15`，截面0.3m高、0.16m深，中心外偏移0.04m。楼体、屋面、楼层腰线、窗饰、雨水管与其他建筑保持现有生成路径；不应删除整个 `trim` 类别

现有东侧横雨槽位于外偏移0.05…0.21m、高度 `top−0.42…−0.28`，与新檐的下部接合有约7cm竖向重叠。此处是有意隐藏接合，当前样板没有模拟汇水、坡降和整套排水连接，接入后仍需检查檐下横槽与落水管的可见关系

GLB 是可视附件，不产生碰撞，根对象应关联稳定 `V-A08` 源。旧四条压顶当前进入 `derived-facade` 碰撞，禁用后减少48个结构三角；楼体仍负责阻挡。应复验既有街行路线，不将新檐作为可行走屋顶或防坠设施，也不记录“碰撞数量不变”

北邻 `V-A09` 实墙与新包络的最小间隔为0.55m；上层窗檐最高39.861515m，新檐下缘高40.071515m，相距0.21m。这些是源码与主数据的数值核对，最终接缝仍由实际游戏镜头检查

## 来源与验收

原创几何、UV和建模脚本由本项目使用本机 Blender 4.5.14 LTS 制作，按仓库 [MPL-2.0](../../../LICENSE) 管理，无外部模型、图片或服务授权新增；设计输入为 V-A08 主数据与现有屋面／金属／边框材质

首轮 [机器检查与看图记录](../../../todo/evidence/TASK-049/roof-r1/README.md) 分开保存。Blender 整体、雨槽近景和檐下视角已实际查看，主文件／GLB 的有限数值、封闭性、朝向、UV、预算和材质一致性通过；Blender 临时方盒预览不等于游戏截图，整栋建筑精修和作者外观验收仍未完成
