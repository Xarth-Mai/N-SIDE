# 院落近景灌木 r1

基线 `e6b398b`，针对前轮真实院落画面中的大块尖叶制作可替换的近景灌木。只新增原创源文件和运行 GLB，不修改地图、实例坐标、碰撞、共享资产清单或绑定

## 实际制作与修订

先读取旧 `plant_bushDetailed.glb` 的真实节点变换与几何：高 0.8 m，最大水平半径 0.669339 m，104 三角。旧模型的同机位 CPU 图呈现大块折面尖叶，无法表达小叶灌木的密度与分枝

首轮制作 840 片小叶和真实分枝，CPU 正面观察发现枝叶集中上半部，裸茎像扫帚束。此轮未作为最终资产交付；第二轮把侧枝下移到主枝低处，增加一层侧枝、减细主茎，并增加顶梢叶，形成中下层较密、顶端较疏的轮廓

第二轮主文件 `shrub-courtyard.blend` 已实际重开并导出，运行 GLB 与重导出结果逐字节一致。叶片为真实不透明六边浅弯几何，单叶 4 三角；全部枝叶共享一个顶点色材质，没有烘焙光照、图片依赖、混合 alpha 或付费生成步骤

## 技术结果

| 检查 | 实际结果 |
| --- | --- |
| Blender 制作与四角度 CPU 渲染 | PASS，Blender 4.5.14 LTS、Cycles CPU、2 threads、24 samples、720×720 |
| 实际 GLB 几何检查 | PASS，5,824 三角、1 primitive、1 材质、1,022 小叶、高 0.8 m、水平半径 0.568525 m |
| 属性与材质 | PASS，有限 POSITION / NORMAL / UV0 / COLOR_0、归一法线、有效索引、无退化三角、OPAQUE、双面、非金属 |
| 真实失败路径 | PASS，将实际 GLB 的材质改为 BLEND 后检查器退出 1，明确报告 `shrub must be opaque`；临时坏文件已删除 |
| 主文件重导出与运行派生物 | PASS，SHA-256 `24c0b97da3d4b64297cb75d4b5920d15c61fea0d1bb033c9be197c20bb8384ee`，670,752 B，逐字节一致 |
| Bevy 绑定、近景及连续移动 | NOT RUN，本子任务按分工不运行 Cargo / GPU，由根 Agent 接入与验收 |

正式检查输出见 `geometry.json`、`runtime-file.json`、`negative.json`、`reexport.json`；接入字段与源文件 hash 见 `integration.json`。模型高度与根部枢轴保持原契约，冠幅略收窄，未超过既有实例包络；按当前 15 个灌木实例计算，替换将增加 85,800 个三角形，没有新增贴图资源

## CPU 视觉自查

实际查看旧模型正面与斜上方、首轮正面、第二轮正面／侧面／斜上方／高角度。新模型能分辨主枝、小叶和层叠间隙；叶片方向随枝向变化，不再是少量尖面绕中心放射。第二轮底部裸茎明显减少，较高的新梢保留开放轮廓；四种绿色以枝簇组织，真实受光形成可读层次

这是参与制作后的 self-audit，不是隔离评审或作者验收。浅弯六边叶仍有角，叶簇在实际几米视距的细节保留和连续移动闪烁未检查；CPU 渲染没有证明 Bevy 运行品质、全城成本或 LOD 完成

## 根 Agent 接入

1. 将源 `vegetation/shrub-courtyard.glb` 及 MPL-2.0 来源登记进既有环境清单，使用普通逐字节复制，不再套旧 Kenney 缩放
2. 将 `appearance.models.shrub.file` 指向 `environment/vegetation/shrub-courtyard.glb`，scene=0、scale=1 保持不变；原第三方源文件与许可保留
3. 运行环境统一导出与窄测，使用既有院落和小店花盆近景比较新旧轮廓、遮挡与接地；不改原点位来迁就模型

根 Agent 与独立 art_review 已查看第二轮正面／斜上方／高角度。确认无 Blender 进程使用后，清理本子任务 `output/assets/task049-shrub-r1/{before,r1,r2}/` 中 12 张 CPU PNG；清理数量与字节见 `cleanup.json`，源模型、正式 GLB、日志、参数、hash 与文字结论保留。根 Agent 后续 GPU 距离检查单独记录
