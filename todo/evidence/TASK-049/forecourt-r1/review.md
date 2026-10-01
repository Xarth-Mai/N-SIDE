# 镜厅公共入口前场：地图实施与窄测

2026-10-01，TASK-049，实施基线 `8effe5d`；本记录覆盖地图和 Bun 检查，场景实例、真实碰撞与画面由 Root 集成后另记

## 实际改动

按[已审查的前场说明](../forecourt-brief-r1.md)在地图末尾追加 `cinema-west-garden-south`、`cinema-west-garden-north` 和 `cinema-arrival-court`，铺地西边直接凹进两块贴边绿岛，三块 surface 相邻而不重叠

整体前场 `391.503760 m²`，两块绿岛合计 `19.2 m²`，铺地净面积约 `372.303760 m²`，均在既有 `P09-A` 内且为 `25 m`。原地块、建筑、入口、道路和此前所有 surface 保持原值；[源差异](source-delta.json)记录追加前后 SHA-256 与字段对比

初稿使用前置 park 被外层 court 包围的方案，Viewer 会从同高铺地裁掉 park，但 Wiki 的 `buildScene` 绘制完整面，前景 court 可能覆盖绿岛。因此实施改成开放边缘凹口，复用两端已有可渲染 surface，没有增加多边形 holes、绘制特殊规则或新框架；窄测同时约束无重叠与共享边界

南西侧沿真实道路转角外缘裁角，不覆盖 `roads[55]` 的斜来路。新增窄测直接检查所有实际路带：与前场有面积交叠的路段必须两端同为 `25 m`，所有绿岛必须退出整条路幅；同时保留故意矩形扩大后失败的回归，避免后续只根据 `0.35 m` 粗筛阈值接受台阶

未增加空的 architecture，也未添加未实现的 planter fixture 类型。现有运行长椅、4 株原创建模灌木与 2 簇公共黄花的建议实例见[布局候选](layout-candidate.json)，不代表已完成实例接入。实际运行 GLB 的节点变换与顶点包络已独立读取，[包络记录](model-bounds.json)中黄花包含原节点 `1.81818` 缩放

## 实际结果

| 检查 | 结果与范围 |
| --- | --- |
| 实际源数据的前场窄测 | PASS，2 项，含错误矩形的拒绝回归，见[narrow.log](narrow.log) |
| 全地图 Bun 检查 | PASS，4 文件共 60 项，见[map-check.log](map-check.log) |
| 地面道路与 surface 交叠 | PASS，0 冲突，见[road-surfaces.json](road-surfaces.json) |
| 地形烘焙 | PASS，9,752 点；内存重烘结果与原 terrain 完全一致，实际源 `--check` 通过，见[impact](terrain-impact.json)与[日志](terrain.log) |
| 工具 TypeScript | PASS，`bun node_modules/typescript/bin/tsc -p tsconfig.json`；最初新测试使用未声明 string 索引的 JSON 导入类型失败，现已改用项目 `District` 类型并复验，见[初始诊断](types-before.log)与[通过日志](tsc.log) |
| 完整 types 入口 | FAIL，本机 `node` 缺 `libsimdjson.so.33`，`tools/check-vue.ts` 启动 Node 时退出 127，见[环境日志](types-host-failure.log)；未更换运行时或安装系统依赖，Vue 类型检查尚未运行 |
| 文档与 Skills | PASS，575 Markdown / 129 ID / 31 Skills，见[docs.log](docs.log) |
| 范围 | PASS，除 3 个新增 surface 外，全部旧对象与字段完全一致 |
| 运行场景、真实人物路径、GPU 自查 | NOT RUN，本子任务没有启动游戏、Blender 或 GPU |
| 作者审美 | NOT RUN，未代签 |

以下命令已在仓库根目录实际执行，可直接用于 fish

```fish
bun test tools/tests/road-width.test.ts --test-name-pattern 'cinema forecourt'
bun tools/check-road-width.ts all --surfaces
bun tools/terrain-shape.ts --check
bun run check:map
bun node_modules/typescript/bin/tsc -p tsconfig.json
bun run check:docs
git diff --check -- source-assets/district-map/district.json tools/tests/road-width.test.ts
```

## 接入与剩余检查

Root 接入 `scene.rs` 时按稳定 surface ID 找到基底与绿岛，复用既有 `street_bench`、`shrub`、`flowers` 槽；窄绿岛无法依赖普通 park 候选布点与间距规则自动出现恰当灌木，此处使用局部明确实例并检查实际包络即可

每个实例仍需检查落脚与绿岛／前场一致、植株在绿岛内、长椅和座前空间退出原公共及服务路带；保留屋顶长椅，不用新实例覆盖原例。门前、绿岛边、服务路和南西斜路交界需通过真实路线检查，不能用本轮 Bun 通过代替

Wiki 构建、工程构建及 GPU 由 Root 批次统一执行；这里没有改动游戏玩法、输入、地图相机或运行资产。准备阶段的完整地图副本、临时候选测试和测试拼接片段在实际源复验后已清理；最终任务证据保留参数、日志与事实差异，没有新建图片或视频中间产物

## R3 场景接入续记

2026-10-01，以下为 Root 完成集成后的续记，上方 NOT RUN 保留为地图制作阶段的实际范围

`props()` 已按3个稳定 surface ID 读取25m高程，增加1件 `street_bench`、4株 `shrub`、2簇 `flowers`；原屋顶长椅保留。前场长椅地图锚点 `[295.8,233.5,25]`、世界 Y 轴旋转 `+90°`、缩放1，坐者面向地图东；灌木与花沿两块绿岛放置，运行源路径为 `/surfaces/{surface_id}/{model}/{index}`。复用 GLB 的几何、贴图及许可保持不变

主任务报告本批8组短片采集通过并实际看图：`cinema-court` 第49帧可见长椅、灌木、花与铺地，镜厅整体第149帧显示建筑与前场衔接。局部画面结论见[视觉记录](../blender-integration-r3/visual-review.md)与[集成记录](../blender-integration-r3/review.md)，属于执行者自查，不代替作者审美放行

[最终碰撞窄测](../blender-integration-r3/scene-models-final.log)1项通过：从已变换的真实 GLB 三角取前场长椅足底，核对实际地面支承；从座前向椅体投射胶囊，核对命中本椅来源；服务路使用1.7m高、0.3m半径的胶囊沿22m路段检查净空。服务路脚点为 `[292,25.075,-220.5]`，相对25m地图高程抬高0.075m，距实际25.025m路面0.05m，这不是人物 walk 回放，也没有证明所有门前、绿岛边缘和斜路交界的人物路径均通过

本次续记只同步已完成接入和实际证据范围，没有改动地图、资产、脚本或游戏代码，也没有生成新临时媒体
