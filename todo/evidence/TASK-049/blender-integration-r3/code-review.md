# R3 接入代码审查

2026-10-01，只读审查基线 `8effe5ddd7f7a3afc18a24482a05670d0fe5f1a6` 的未提交改动，审查者仅新增本记录，未改主代码、启动 Cargo／GPU／Blender，也未观察本批运行画面

## 结论

本次范围内未发现阻断正确性的代码问题；代码审查不能代替尚待主任务执行的 Rust 检查、真实运行、画面自查或作者观感

- `CollisionWorld::from_scene` 沿用同一 GLB 静态三角导入路径，新增 V-55 和空调白名单；新前场长椅复用已纳入碰撞的 `street_bench`，导入的模型矩阵、统一 scale 与摆放 Transform 次序和渲染路径一致
- V-55 接管西／南两面泛型窗、腰线、压顶和门框，仍执行门位／地形入口校验，北／东面保留；新增测试核对两关闭门真实三角、实际道路接近包络及未替换两面的几何，不把关闭门描述为室内入口已开放
- 三处小店梯段踏鼻合计 420 三角，source 使用 `nodes[…]/derived-stair-nosing[…]`，不匹配结构碰撞白名单；新增检查比较踏面高程、绕序、碰撞数量及支承点，语义与视觉-only范围一致
- 前场初稿仅增加三块 surface；后续镜厅门前站位修订见下节，原节点坐标、建筑和稳定 ID 保留。窄测覆盖凹形前场、地栽边界、真实道路宽度，保留一个原矩形会盖住斜坡的负例，没有靠一般 0.35m 粗筛阈值放过台阶
- Viewer 只增四个检查机位，其中 V-55 人眼由真实道路插值得到；这些机位不构成人物可通行或构图验收

已复读主任务补充的前场长椅和空调低位收水口检查：读取纳入正式碰撞后的真实 GLB 接地顶点，向不含导入模型的原场景查询支承，验证座前胶囊首先碰到新长椅，以及原服务通道保留净空

服务通道探针由 `25.04` 改为 `25.075` 未隐藏物件穿透：原 `/roads/236` 的该段标高25m，渲染／碰撞路面另加0.025m；旧胶囊脚部仅离路面0.015m，小于0.02m skin，测试在起点就包含路面接触。新探针离路面0.05m，保留原半径、高度、横向路线和碰撞集合；它是离地5cm的通道净空检查，不是角色接地行走测试。首跑日志只有失败断言而未打印命中 source，故路面接触诊断依据为代码与几何契约，不冒充首跑已打印的接触事实

## 实际检查

`bun test tools/tests/road-width.test.ts`：PASS，16 项、0 FAIL，10.49 秒；其中两项为本批前场测试

`git diff --check -- game/src/world/scene.rs game/src/world/collision.rs game/src/world/geometry.rs game/src/bin/map_viewer.rs tools/tests/road-width.test.ts`：PASS

复读主任务日志：[首轮库测试](lib-tests.log) 为140 PASS、1 FAIL、1 ignored；[修订后碰撞窄测](collision-test.log) 为1 PASS、0 FAIL。未由审查者重跑 Cargo，也不将窄测拼成整库重新通过的记录

本轮没有生成截图、视频或临时探测可执行文件，无此类产物待清理

## 源清单与归属复核

独立只读核对 `game/assets/environment/manifest.json` 的37个文件，实际尺寸与SHA-256全部匹配；V-55源／运行两项和CHR-001当前带hash的三项路径均匹配。V-55按本建筑清单直接导出，不属于AST-003环境批量导出器的37项，未因不在该运行清单内而漏掉校验

实际执行 `python3 -B source-assets/buildings/V-55/check.py`、`python3 -B source-assets/environment-kit/aircon/check.py`、`python3 -B source-assets/environment-kit/aircon/mount-check.py` 均PASS；没有启动Blender。前者核对7362三角、四张内嵌图与既有AST-003源hash和CC0字段，后两者核对9个原件、候选及完整装配，装配10117三角、6张图保留、原单机9493三角加624个原创安装三角

空调保留官方API原始文件、Monsta3D署名、取得URL／时间、MD5／SHA-256和CC0正文；成品MPL字段同时细分原单机／贴图CC0与原创安装件MPL，没有把原素材写成原创。V-55区分原创几何、ambientCG贴图与Noto字体，字体版权及OFL文本均存在，相关证据路径可解析。此处核对仓库已保存的来源记录，不声称重新查询上游网页或进行法律审定

模型实际安装hash分别为V-55 `8683357c862f708904de63bbe8a88a196f6e327b48ea5c638a1c318de8c6349b`、空调 `186759c4eef4728bc09cca57e3c2fd2685f3d98c3767064c6417cd615d143be0`；未发现需阻断本批集成的清单或归属遗漏

## 镜厅门前站位增量

按JSON语义复核：新增 `cinema_entry_landing=[306,219.35,25]`，原门锚仍在南墙；`roads[649]` 只做同高共线分段，同步场所15到达链与两条生活路线。唯一重烘差异为 `[320,270]` 从24.936到24.937，原9752样本数、楼体、门位、路线权限和控制器均保留，与 [源修订记录](cinema-landing.md) 一致

通用目的地窄测只接纳到达链末段的 `${door}_landing`，同时要求直接公共道路相连、距离0.32–1m、同高且位于楼体外；本例另有0.65m退界、中梃／胶囊包络、原路幅、前场支承及往返检查，不是单靠名称豁免。Rust附件接近测试改以具名站位为终点且不再二次后退，未改变GLB碰撞或输入驱动

复读 [61项Bun通过日志](check-map-final.log)、[附件接近复验](attachment-final.log) 与 [碰撞复验](scene-models-final.log)；原 `map-final.log` 的不存在命令错误继续保留，未算通过。实际读取 `output/blender-integration-r3/cinema-gentle/run.json`，9900帧状态PASS、原生退出0、41/41节点到达且重置均0；这只核对状态记录，审查者未看本批画面。运行输入记录已区分前8段录制与后续地图站位修订，Rust改动仅在 `cfg(test)`，运行二进制复用范围一致

## 快照 SHA-256

| 文件 | SHA-256 |
| --- | --- |
| `game/src/world/scene.rs` | `37629d8b065341012fb8b5f46c31935890e5f836af2dee3736d04de2a2f82428` |
| `game/src/world/collision.rs` | `89cafec2794c2a99ac76d3fb7cae6b505b097674e58654686be3f5be4c4c5fbe` |
| `game/src/world/geometry.rs` | `ff0fff6be92bec1d7015a4307e1d516715588c86c0b06465829332b3a55ba842` |
| `game/src/bin/map_viewer.rs` | `e6b783dcf6e1b6e05de0c76e699664067985381f06ce60edec26c553ddf88d29` |
| `tools/tests/road-width.test.ts` | `67ad24e2901522baf58ba7a852185d8493506ba481c597cdccf156e5fb60f9c0` |
| `tools/tests/district-map.test.ts` | `8cf3324c55988d741b9939232f19f2de84707d52d60ca98d5e643f0de815131a` |
| `source-assets/district-map/district.json` | `a2f15c59246a57c1250ee76db54e0a12fa5ce766f17f15dc1535e5895128038f` |

## 简化审查

上述六个代码文件复用现有场景、碰撞、网格和测试入口，没有新增依赖、抽象层或并行导入路径；当前可执行回归服务于实际几何边界

`appearance.json` 已恢复原数组格式，只保留两个 model 绑定；前场 source 改用本地枚举，不再受此前其他 prop 数量影响

Lean already. Ship.
