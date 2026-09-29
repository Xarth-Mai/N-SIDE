# 地表、针叶灌木与住宅窗的第 6 轮对照

基线 `e6b398b`，复用真实世界和 Viewer 控制器。只增加一个取景点及一条 5 秒输入脚本，保留旧视点与旧脚本；构建与 GPU 由主 Agent 串行执行。本页不记录尚未发生的运行或视觉通过

## 三条检查

| 修改对象 | 脚本 | 构图与实际来源 | 必看帧 |
| --- | --- | --- | --- |
| 陡坡地表 | [descent-cut.json](../../../../game/capture/descent-cut.json)，3 秒 | 原有 `eye-descent-cut`；同源地图的射线诊断显示已采样区域坡度约 36.6°–55.4°，与本轮平坡共同检查分材质效果 | 29、44、74，以及 43–45 连续帧 |
| 平地、完整针叶与灌木 | [planting-detail.json](../../../../game/capture/planting-detail.json)，5 秒 | 新 `inspect-plateau-planting`；从共享花园北侧现有园地看 `surfaces[50]` 的原有 edge-group 0，模型、草地和摆放均来自真实场景 | 0、59、89、119，以及 60–62 连续帧 |
| 住宅窗与玻璃 | [exterior-details.json](../../../../game/capture/exterior-details.json)，5 秒 | 原有 `inspect-residential-detail`；`east_mid_junction` 上方 1.7m 看 `V-A13` 西立面，初始中心距约 18.54m，真实接近与横移可检查窗框深度和反射变化 | 29、59、89、119，以及 60–62 连续帧 |

三条共 390 帧、13 秒，均为 1280×720、30 fps、55°垂直视场。前后使用相同脚本、分辨率、帧号、光照与 AA 参数；所有材质/模型一起变化时说明为综合结果，不能将整个画面差异归给单项

## 为什么补一个花园取景点

旧 `inspect-plateau-garden` 继续保留，其 `[218,494,102.4]` 视角中原秋树与松树存在画面叠放，上一轮已注明。新视点为 `[212,508,102.4]`，目标 `[189.5,500.5,103.3]`，位于原 `park` 轮廓 `x175–219 / y488–509` 内，起始眼高为原地面 `100.678571m + 1.721429m`

目标松树中心来自原分组规则 `[189.08,502.07,100.663571]`，两丛灌木位于 `[192.68,502.67]` 和 `[189.48,497.57]`。按原 6.5m 树高、2.3m 树包络及 0.7m 灌木包络，对接近和横移后的五个检查姿态进行视锥采样，针叶树和两丛灌木均完整落在画幅内；两个秋树包络与目标松树的水平角间距最少仍余约 22.85°，没有沿用原先的前后叠放

按真实源码的 12m/s 速度及摩擦估算脚本相机位置，再采样相机到六个地表对象的视线，未穿源建筑体积。住宅取景在 `V-A13` 西侧，前方邻楼 `V-A12` 止于 `y270`，中心视线保持 `y282.5–288`，未穿邻楼。来源坐标、原 GLB hash、实际顶点包络与采样结果保存在 [source-sampling.json](source-sampling.json)

这些只读检查约束取景选择，不等于 GPU 全网格可见性或人物通行证明；针叶自身遮挡、灌木枝叶密度、道路遮罩、法线/色彩和所有实际新模型边界仍须看最终截图与连续帧。新资产如果超出既有包络，先复查契约与此机位，不为好看移动场景对象

## 运行入口

在仓库根执行以下 fish 可直接使用的命令；指定已统一构建的二进制，每次输出目录必须不存在

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/descent-cut.json --output output/capture/task045-visual-r6-cut
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/planting-detail.json --output output/capture/task045-visual-r6-planting
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/task045-visual-r6-windows
```

本轮源地形 hash 与上一轮射线诊断一致，因此沿用其像素对应关系；诊断来自未裁剪源三角形，Delaunator 与 Bevy Spade 的共面分割、道路掩膜和实际平滑法线可能不同。陡坡左上角仍含约 3.2° 掠视区域，不能把过滤模糊或亮暗变化自动解释为 UV 修复

## 当前证据状态

- 源地图、原包络、建筑视线及视锥只读采样：已完成，结果见 JSON；实际运行脚本解析与 GPU 构建由主 Agent 后续执行
- 新脚本使用现有合法输入、断言与视点，未改场景状态、人物位置或已有地图数据
- 新材质、模型与窗的 GPU 对照及 self-audit：NOT RUN；计划与源码采样不替代实际看图
- 图像清理由主 Agent 在实际查看、hash 与结论记录后统一完成，此处尚未生成图像
