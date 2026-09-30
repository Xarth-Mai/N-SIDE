# 林缘浅沟与道路支撑复验

## 输入与制作

基线为 `13397c5626c8952511ceda52d3c4a8877b901acc` 的地图，保留在 `output/assets/terrain-shape-r4/before-district.json`；SHA-256 为 `58423556559f791d046544d4a5b6c2e756e8cba64abebcf5798ab8ec055ff2f1`

在 `hill_short_rest2` 东侧林缘加入中心 `[144,850]`、半宽 12m、半长 30m、沿坡方向旋转的浅沟，最大解析下切量为 2.4m；道路与场地 8m 内不下切，8–16m 平滑过渡，局部自然样点间距 2m。近邻道路保留原 6m 控制并追加 1m 中线和双侧控制，生成样点从 9300 增至 9752，其中新增自然点 260、道路支撑点 192

新增 1m 道路点只约束最终地面网格，不加入既有坡面包络或外围残差权重。原 42 个手工点、450m 山顶、所有旧采样坐标、terrain 元数据和全部非 terrain 数据保持

最终地图 SHA-256 为 `aecbdb41adc3c4fac4cf8fb36884d80708b9b776086c01a8f00e4ef6210f8779`，生成器 SHA-256 为 `07fe85df8542dd3f5f5e880a04de701b58c0c2739228e52c5fd07a4ee8561dac`

## 失败与修复

- [首轮原生探针](native-probe.json)：9560 点版本通过 Bun 路面检查，但实际 Spade Ground 在道路 827 的 `t=.25, side=1` 位置高出路面 0.160289m，说明两种 Delaunay 实现的同圆点选边差异需要原生复验
- [道路加密试验](native-probe-final.json)：9752 点版本消除上述道路回归，但[旧样点比对](source-probe-final.json)显示新增支撑进入包络后改变了浅沟外 93 个旧样点，最大漂移 0.266m，该结果未作为最终通过
- 最终将新增支撑排除旧包络与残差，保留最终网格的路幅约束；[完整源数据断言](source-probe.ts)和四个曾漂移旧点的工具回归固定此失败路径，未放宽原道路 0.01m 门槛

## 实际检查

| 检查 | 结果与证据 |
| --- | --- |
| `bun tools/terrain-shape.ts --check` | PASS，[烘焙日志](bake-check-envelope-fixed.log)，9752 点与源生成规则一致 |
| `bun test tools/tests/terrain-shape.test.ts` | PASS，[地形窄测](terrain-tests-envelope-fixed.log)，覆盖旧点恢复、浅沟凹度、道路和平台保护、峰顶、重烘焙幂等与无效输入 |
| `bun run check:map` | PASS，[地图检查](map-tests-envelope-fixed.log)，58 pass、0 fail |
| `bun todo/evidence/TASK-045/terrain-shape-r4/source-probe.ts` | PASS，[最终源数据结果](source-probe-envelope-fixed.json)，全部非 terrain 数据与 terrain 元数据相等、42 手工点相等、所有旧坐标保留、450m 峰顶保持；仅沟内 15 个旧点下切，范围外旧点变化为 0，最大下切 2.372m |
| 原生 Spade Ground 道路比较 | PASS，[最终原生结果](native-probe-envelope-fixed.json)，25770 个道路中心及双侧采样，最大新增绝对高程误差 `7.361882126133423e-8 m`，低于 0.01m |
| 原生坡度窗口 | PASS，同一原生结果内 405 点的 P50 为 24.3446°、P95 为 33.7380°、最大 50.2543°，超过 55° 的样点为 0 |
| 源网格截面 | PASS，`y=848` 两肩均高减沟底为 1.915667m；中心 `[144,850]` 新高程 360.015m，相对旧三角插值下降 2.114250m，2.4m 是解析下切参数 |

原生道路范围外有两条记录变化超过 1cm，均是相邻路段共用的 `[180.3614114,935.5287181]` 同一点：最终重剖分使原有绝对道路高程误差从 0.312269m 降至 0.301886m，未产生误差回归。旧样点不变不等于最终连续三角插值在所有位置逐点相等；本轮明确比较了实际道路误差，未宣称全地图已有道路误差归零

原生复验复用主代理编译的临时示例，当前检查没有另行运行 Cargo。源码为 [probe.rs](probe.rs)，源码 SHA-256 为 `9c5c710dfd6c87f4a340cdb35586e36d252c11d1a2cec1e8c228821295390792`，所用二进制 SHA-256 为 `1b5d6029597d832a73ac6916a997e2da16684d67b091b69a6c2fb708e9f4935f`

```sh
game/target/debug/examples/terrain_r9_probe output/assets/terrain-shape-r4/before-district.json source-assets/district-map/district.json > todo/evidence/TASK-045/terrain-shape-r4/native-probe-envelope-fixed.json
bun -e 'import assert from "node:assert/strict"; const p=await Bun.file("todo/evidence/TASK-045/terrain-shape-r4/native-probe-envelope-fixed.json").json(); assert.equal(p.non_terrain_unchanged,true); assert.equal(p.road_partitions.reduce((n,r)=>n+r.samples,0),25770); for(const r of p.road_partitions)assert.ok(r.maximum_absolute_grade_error_increase_sample.absolute_error_increase<.01); assert.ok(p.cut_window_slope_before_after[1].maximum<55); console.log("PASS native Spade: 25770 road samples, grade-error regression < 0.01m, hollow slope < 55deg");' > todo/evidence/TASK-045/terrain-shape-r4/native-check-envelope-fixed.log
```

[断言日志](native-check-envelope-fixed.log)为 PASS。原生结果比较连续 Ground 与道路高程包络，不代替最终踏步、人物碰撞净空或实际画面；渲染与真实路线由本轮 [visual-r9](../visual-r9/) 集成复验记录

## 收尾

本地形修复阶段没有生成截图、视频、原始帧或新探测可执行文件，已有视觉产物与主代理提供的临时探针二进制由集成阶段统一清理；保留基线 JSON、复现源码、历史失败结果与本轮日志
