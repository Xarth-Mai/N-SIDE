# 街树冠层与叶序 r3

范围是 `AST-003` 的原创 `tree_a` 街树造型，保持现有树点、6m 米制、接地枢轴、初秋配色、双面不透明叶片与树皮 UV0；未改地形、碰撞、角色、玩法或共享清单。模型、树皮与源程序仍为 N:SIDE 原创内容，沿用仓库 MPL-2.0；没有使用外部图片或 3D 生成服务

## 观察与制作

实际查看 `source-assets/area-previews/cinema-music-street.png` 与 r2 同机位 CPU 树木渲染。参考的中景树冠由具有透空的体积叶簇组成；r2 中大部分叶面接近水平、叶簇集中在主枝外端，侧面呈现薄层叶架和较大冠内空洞。本轮借用参考的冠层关系，不将夜景参考与单株 CPU 灯光作颜色测量对比

修改前 vegetation 工作区无未提交改动；原 `.blend` 重导出与复制的原生成程序重建均得到相同 GLB hash，确认未覆盖未收入程序的手工编辑，见 [inputs.json](inputs.json)、[master-check.json](master-check.json)

候选一只改叶柄扭转、叶片俯仰和细枝位置，保留 1,089 叶与 11,686 三角，实际看到水平层架减轻，但叶片偏大。候选二将八边圆肩叶的中心扇形八三角改为从叶根展开的六三角，保留浅弯轮廓，以相同总面数容纳 1,452 片较小叶。最终候选再把部分细枝从外端移至主枝中段，叶簇沿冠内到冠外分布，保留有方向的枝条和几何透空

## 实际命令与机器结果

仓库根目录执行，以下可直接用于 fish；本轮 Blender 4.5.14 LTS、Cycles CPU、4 线程、24 samples、720×840，原脚本固定 seed `45019`，渲染使用三向全树和树干近景原机位

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/build_street_tree.py
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/export_street_tree.py -- --render --render-dir output/assets/task045-vegetation-r3/after
python3 source-assets/environment-kit/vegetation/check_street_tree.py
```

机器检查 PASS：有限 Transform/顶点、单位法线、无退化面、树皮有效 UV、颜色范围、OPAQUE、双面叶、Y-up、6m 高、接地根节点、叶冠离地与 3.5m 半径约束，详见 [geometry-check.json](geometry-check.json)。最终主文件直接重导出与候选 GLB 逐字节一致，见 [export-check.json](export-check.json)。原树皮色图 hash 保持 `ffc18a18a15fba174fb44b055054c2b2f96a6b19e33263386ced0b58e946a2e2`，来源、主文件及交付 hash 见 [outputs.json](outputs.json)

## 资源变化

| 项目 | r2 | r3 | 增量 |
| --- | --- | --- | --- |
| 三角／实例 | 11,686 | 11,686 | 0 |
| 实际 GLB 顶点／实例 | 13,099 | 14,914 | +1,815，约 +13.86% |
| 叶片 | 1,089 | 1,452 | +363，约 +33.33% |
| GLB bytes | 709,372 | 789,236 | +79,864，约 +11.26% |
| 材质／primitive／内嵌纹理 | 2／2／1 | 2／2／1 | 0 |
| 最大水平半径 | 2.992843m | 2.855632m | 保持现有布置范围 |

数据来自实际 GLB accessor，见 [geometry-cost.json](geometry-cost.json)。三角不增加不等于零成本；新增边界顶点带来存储和顶点处理变化。按已核实的 16 个 `tree_a` 实例，三角仍为 186,976，GLB 顶点引用量由 209,584 变为 238,624；这不是 draw call、GPU 显存或帧时间测量。树皮纹理继续共享，不按实例复制估算

## 画面自查与下一验证

作者 Agent 实际查看三向全树和树干近景；额外 art_review Agent 实际查看同机位 before 与最终三个视角。共同观察为冠层中心更连贯、叶面方向更自然、侧面能读到体积，原先的水平薄层减轻；主干分叉和枝间透空仍可辨，未观察到大块背面消失或明显悬空叶簇。第三视角为背光，较暗不通过修改灯光掩盖

这是 self-audit，不是隔离盲测或作者审美验收。当前仍是细节有限的修剪街树，叶片边缘偏平、颜色块较统一；没有叶风、LOD 或树皮法线，不能称完整概念品质或全城植被完成。真实 Bevy 连续移动、背面照明、阴影与资源成本由根 Agent 在严格源 hash 合并后沿既有 `street-tree-detail.json` 复验，本记录不把 CPU 图片当运行通过

共享清单与运行资产交接字段在 [integration-entry.json](integration-entry.json)；本分工没有修改 `game/assets` 或降低源 hash 检查。所有临时视图 hash 与参数记在 [views.json](views.json)。根 Agent 实际查看 before/after 后，本轮 16 张 CPU PNG 与 baseline 临时副本已清理，清理前 `ps -C blender -o pid=,comm=` 确认没有 Blender 进程；正式源文件、渲染与候选程序、日志、JSON 和本记录保留，详细路径、字节及 hash 见 [cleanup.json](cleanup.json)，未清理根 Agent 的 capture 目录
