# 月台上巷坡肩绿植小样

依据[横带诊断](../../TASK-045/terrain-bands-r10/diagnosis.json)，下方细横线属于 roads 274/729 的真实铺装，不能通过修改草地法线或阴影消除；本轮只为道路 274 北侧增加两小组住户维护的低灌木，让横巷与坡面边界获得可见的生活尺度

落点依据[小店与坡地住宅规格](../../../../docs/dev/design/locations/district-architecture.md)的横街、短台阶与树坡关系，锚定 `v_a08_door_landing`，五株范围为地图 `x=93.2..104、north=280.3..283`。原 2 m 入户巷、高程 30.021515 m、道路入口和地图唯一源保持；组间冠幅之外留有超过 2 m 的可见空隙，两端与主梯保持净空

只修改 [scene.rs](../../../../game/src/world/scene.rs) 的视觉实例布置和窄测，复用[原创院落灌木](../../../../source-assets/environment-kit/vegetation/shrub-readme.md)。原高 0.8 m 的模型使用 1.0–1.2 倍尺度，五株共增加 29,120 个三角，没有新材质、纹理、模型或地图点位；未增加或修改 `GeometryPart`，玩家碰撞仍取原几何

新组在 `add_forest_understory` 完成后追加，既有全部植被优先占位；布置沿用既有道路、建筑、入口和实例包络避让；根圈按实例尺度检查真实 Ground，底部埋入最低支撑 0.025 m。新增窄测读取实际 GLB 根部顶点和完整水平包络，检查浮根/埋深、道路净空、两组空隙以及五个候选确实被接受；低灌木形成短组团，完整裸坡和挡土细节仍未制作

| 检查 | 本轮结果 |
| --- | --- |
| 新小样实际根部、净空与两组间隙 | PASS，[native-test-appended-final.log](native-test-appended-final.log) |
| 全量植被来源、包络与公共空间 | PASS，[all-vegetation-appended-final.log](all-vegetation-appended-final.log) |
| 原林群承托与既有布置回归 | PASS，[forest-regression-appended-final.log](forest-regression-appended-final.log) |
| 单文件 rustfmt 与 git diff --check | PASS |
| GPU 固定出生点 | PASS，60/60 帧、4 原生断言，Root 实际查看第 59 帧 |
| 作者美术判断 | NOT RUN |

复验沿用[出生点诊断脚本](../../TASK-045/terrain-bands-r10/script.json)，同分辨率、光照和人物相机查看第 59 帧，比较路肩组团、两端入口留空、冠幅和根部接触；机器通过不能证明这五株已在画面中形成合适的覆盖密度

具体坐标、输入 hash、检查边界见 [snapshot.json](snapshot.json)，实现差异见 [change.patch](change.patch)。本子任务未产生截图、视频或临时探测可执行文件，未提交或修改任务卡

Root 顺序审查后将小样由候选前移到全部既有植被之后，复跑仍接受相同五株。原林群回归曾因 `zip` 假定完整旧列表都是新列表前缀而失败，详见 [首次回归日志](forest-regression-appended-first.log)；修复为按稳定 source 找同实例并继续比较 model/Transform，未放宽净空或根部阈值。Ponytail-review 删除窄测中每株复制整组实例的冗余，实例间距仍由全量植被窄测覆盖

## 集成运行与输入隔离

Root 复用同一出生点脚本执行真实游戏，首次在 34 帧等待 world 时超时并回到 title；[失败状态与日志](input-isolation/failed-spawn/run.json)保留。源码确认离屏入口仍接收 Gilrs 物理手柄事件，现只在离屏 capture 禁用 Gilrs，正常窗口保留原后端；既有 ScriptGamepad 输入与断连验证仍运行。原失败没有设备或取消键记录，不追认具体外部按键归因

禁用后同一脚本[复跑](input-isolation/spawn-isolated/run.json)60/60 帧与 4 条检查 PASS；[画面自查](input-isolation/spawn-isolated/visual-review.json)记录 Root 查看两小组灌木、组间与主梯留空及无明显浮根。覆盖密度仍小，大片裸坡和真实横巷仍可见，改善只限这一处坡肩

临时画面在实际查看后清理，见 [清理记录](cleanup.json)；所有脚本、参数、状态与失败证据保留。没有改正式曝光、地形或地图，未做作者验收
