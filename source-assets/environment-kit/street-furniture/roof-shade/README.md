# 开敞屋顶条栅遮阴架

`AST-ROOF-SHADE` 是可重复放置的两后柱、向前悬挑木条遮阴组件，唯一可编辑主文件为 [roof-shade.blend](roof-shade.blend)，源导出为 [roof-shade.glb](roof-shade.glb)。本轮由本机 Blender 5.2.2 LTS 实际建模并完成保存后复读；已登记于 AST-003 并复制运行候选，状态保持 `needs_revision`，数值检查与四组真实Viewer运行通过，实际画面已自查；作者品质验收待反馈

## 构成与放置

两根后柱、三条悬挑钢臂、后横梁、两道后侧短斜撑、十根横向木条与两块薄底板组成单侧承托的轻架；前方与左右两侧开敞。原 Poly Haven 长椅保持独立复用，不复制其几何、贴图或维护第二份座椅。没有增加室内、玻璃围合、咖啡经营、座椅交互或植物恢复状态

| 项目 | 实际合同 |
| --- | --- |
| 米制尺寸 | 宽4.04m、深2.56m、高2.52m |
| 枢轴／朝向 | 地面0、复用座椅水平中心；Blender长轴X、上方+Z、正面−Y，glTF上方+Y、正面+Z |
| glTF包络 | `[-2.02,0,-1.46]` 至 `[2.02,2.52,1.10]`，Scene0、identity根、scale1 |
| 初始地图锚点 | `[312,237,37]`；游戏侧只做一次 `map_to_world`，不再次翻轴 |
| 初始地图包络 | x309.98..314.02、north235.90..238.46、h37..39.52 |
| 后柱中心 | x310.25与313.75、north238.30；柱截面0.095×0.12m |
| 条栅／悬挑 | 十根木条各4.04×0.15×0.055m，间距0.235m；钢臂下表面高2.345m，前方没有落柱 |
| 座椅 | 沿用 `street_bench` 位于 `[312,237,37]`、朝南，1.92×0.670394×0.866914m，坐面约0.45m |
| 实际开销 | 1,104三角、2,612顶点、1mesh／2primitives、2材质、2内嵌图、1,514,200bytes |
| 源工程 | 28个可编辑mesh，2张packed图；没有相机、灯光或动画进入GLB |

整件位于实际休息岛 x309..315、north235..240 内。实际 GLB 三角包围盒与现有长椅包络分离，另检查前方1.85m高接近体与左右接近体；上方遮阴与座位有意平面重叠，不用两件整体包络判互穿

检查沿实际地图六条 RF 相关路径 `/roads/168,169,170,238,240,770` 的段与宽度计算投影间隔，最近边界间隔约11.997m。本轮地图以 `cinema-roof-planting-reserve` 单独保留 x329..336、north228..236 的草地，检查直接读取该区划，组件真实投影包络与它至少相隔14.98m；这是物理布置约束，QST-025 的苗位、照料与空位状态仍沿用故事设定，没有新增种植或把空位画成恢复

## 材质与来源

木条使用 AST-003 已有 ambientCG [WoodSiding009](https://ambientcg.com/view?id=WoodSiding009) 原 Color JPG 和本项目已验证的 `WoodSiding009-NormalGL-scale035.png`；沿用 [CC0-1.0](../../licenses/CC0-1.0.txt)，原输入／派生关系见 [环境清单](../../asset-manifest.json)。颜色按sRGB、法线按Non-Color，源强度0.35已烘入PNG，Normal Map Strength／glTF scale均为1；两张图片都以原字节内嵌，没有新增贴图

已实际查看原木颜色图，使用其浅暖色木纹；条栅长向U按2m周期展开，板宽方向避开源图板缝，避免把一根细条画成多排木板。金属从 `appearance.materials.metal` 读取当时的灰绿色 `[0.59,0.61,0.57]`、roughness0.58、metallic0.3，颜色从sRGB转线性写入PBR；这是构建快照，不是运行时同名绑定。没有调整全局曝光或光照

几何、UV和制作脚本为 N:SIDE 原创，使用项目 MPL-2.0。长椅来源继续由[既有公共座椅](../README.md)管理；本资产只引用其实际大小和首个放置关系，来源、原图哈希和当前输出哈希统一见 [asset-manifest.json](asset-manifest.json)

## 制作与检查

默认命令导出当前主文件；只有显式 `--rebuild` 才按初始配方覆盖本主文件。Blender单进程、宿主无音频、2线程；源资产阶段未另做Blender渲染，接入后的四组真实Viewer画面见运行记录

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/environment-kit/street-furniture/roof-shade/build.py
python3 -B source-assets/environment-kit/street-furniture/roof-shade/check.py
bun tools/export-environment.ts
bun tools/export-environment.ts --check
```

[本轮源检查](../../../../todo/evidence/TASK-049/roof-rest-r1/asset/source-check.json)确认28件mesh、两张packed图、法线强度和地面枢轴；重新打开保存主文件导出的GLB完整字节相同。[几何检查](../../../../todo/evidence/TASK-049/roof-rest-r1/asset/geometry-check.json)直接读取GLB位置／法线／切线／UV／索引，检查有限值、单位方向、绕序、无退化三角、真实包络、原图许可与嵌图原字节、现有座椅和三处接近体、六条RF相关路径

既有环境导出器按 AST-003 唯一条目复制到 `game/assets/environment/street-furniture/roof-shade.glb`，`appearance.models.roof_shade` 使用该文件、Scene0、scale1；没有新增导出框架或第三方来源。源与运行 GLB 字节相同，旧37个环境运行文件字节不变，见[登记检查](../../../../todo/evidence/TASK-049/roof-rest-r1/asset/registration-check.json)。场景与真实网格碰撞已接入；四组实际Viewer共420帧、22项原生及22项包装断言通过，具体材质、轮廓与连续帧观察见[运行与看图记录](../../../../todo/evidence/TASK-049/roof-rest-r1/review.md)。机器胶囊通行与地面支承已检查，真人角色游走与作者品质验收未运行
