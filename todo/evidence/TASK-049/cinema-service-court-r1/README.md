# 镜厅北侧后勤前场候选

状态：`CANDIDATE_NOT_APPLIED`，本轮只写候选与检查证据，正式地图、appearance、Rust 和运行资产均未修改，未启动 Blender 或 GPU。源地图 SHA-256 为 `a2f15c59246a57c1250ee76db54e0a12fa5ce766f17f15dc1535e5895128038f`

## 落位与用途

[候选数据](candidate.json)在 `P09-A` 内增加一块 `cinema-service-court`，范围 x314–326、north250–266.5，标高25m、面积198m²，`kind: service` / `access: service`。沿用镜厅已有「工作人员、设备与垃圾经独立后台」用途，不增加停车系统、垃圾处理玩法、室内或公众到达路线

原 `cinema_loading=[320,265,25]` 至 `cinema_service_entry=[320,250,25]` 的3m服务路贯穿前场，中央保留3.6m宽连续净空，卸货节点周围保留8×8.5m活动区。西面原服务路、南西侧公共前场、公众电梯、后门和货梯位置保持；周边草坡没有扩大为整块公共广场

只放两件现有配套，不另堆箱子或创建新模型

| 已有资产 | 候选位置与方向 | 使用与来源 |
| --- | --- | --- |
| `aircon_wall` | 地面锚 `[324,250.203,25]`，world Y 旋转π，单位缩放 | 北墙两组窗之间的设备装配，保留背垫、回墙管和接地收水口；Monsta3D / Poly Haven 原单机及纹理CC0，N:SIDE安装件MPL，沿用 AST-003 |
| `streetlight` | `[314.65,261.6,25]`，无额外旋转，单位实例缩放 | 前场西侧设施识别，原GLB节点自带6.666667倍米制变换、最终高4.5m；Kenney City Kit Roads、CC0，沿用 AST-003；这里只复用可见设施，不宣称已实现夜间照明 |

原始文件与许可证继续由 [环境包清单](../../../../source-assets/environment-kit/asset-manifest.json)维护。本轮只增加候选实例，未复制素材或建立另一份资产登记，新增实例共10,209三角面、没有新增运行模型文件

## 实际检查

[GLB检查](props-report.json)读取实际运行GLB、对应环境导出hash及源许可，再应用节点变换和候选实例变换。空调装配包络为 x323.600014–324.399986、north249.968–250.390080、高25–28.431230m；路灯为 x314.483333–314.816667、north261.433333–262.933333、高25–29.5m

[地图检查](check.json)使用现有 `buildGround`、`roadOffsets`、`intersectionArea` 和 `roadSurfaceConflicts`，在内存副本加入候选后验证

- PASS：全场位于P09-A，没有与建筑或既有地面surface形成面积重叠；只有 `roads[236]` 的北侧横段与到门段相交，都是25m同高服务路
- PASS：0.25m网格共3,283点原地形24.9811–25.1m，局部最大挖0.10m／填0.0189m；没有重新烘焙或写入地形
- PASS：两件道具的整个投影包络再外扩0.32m，仍不接触真实道路路幅；两块活动净空完全不被占用
- PASS：实际GLB全部最低顶点落在新铺地且同高；空调位于北墙x322／326窗间，包含窗框余量后最近0.795m，最高点低于29m楼层分带
- PASS：故意把铺地降0.5m会被道路高差检查拒绝，把物件放入中央通道会被净空检查拒绝；探针前后冻结地图hash一致

初放置采用标称墙面距离0.198m，检查发现收水口实际最背点在墙内5mm，且该点原裸地约高0.052–0.055m，不能直接当作新铺地支承。查看既有 `mount.py` 和真实GLB后改用背垫／收水口的实际后缘0.203m，将整套实例向外移5mm；不修改模型、不删除检查顶点、不降低支承阈值。最终背垫接墙、套管仍入墙约32mm，全部底部顶点通过新铺地支承检查

以上是源几何和保守包络检查，尚未执行 Rust Ground/capsule、模型实际碰撞、夜间、真人推车或GPU。原门仍关闭；走到门锚不是人物应穿门成功的条件

## 复现与整合交接

仓库根目录执行，fish可直接使用

```fish
python3 -B todo/evidence/TASK-049/cinema-service-court-r1/inspect-props.py > todo/evidence/TASK-049/cinema-service-court-r1/props-report.json
bun todo/evidence/TASK-049/cinema-service-court-r1/check.ts > todo/evidence/TASK-049/cinema-service-court-r1/check.json
python3 -B todo/evidence/TASK-049/cinema-service-court-r1/layout.py
```

整合时由同一 `candidate.json` 取surface与两个实例，复用既有surface管线、`PropPlacement` 和模型碰撞白名单；此稿没有建立新placement系统。图上的净空矩形仅为检查约束，不写成额外surface或道路

实际验收应覆盖西侧服务来路到卸货节点、原后门外接近并折返、前场横移、空调排水口与路灯脚支承；检查同高道路顶面是否仍有重复色带、北墙设备是否贴合、空调与窗／分层腰线及人物是否相撞。如地形freshness变动，先核对范围再沿既有工具重烘，不能用新平面遮盖原斜路

`layout.py` 生成的是基于源坐标与实际GLB包络的平面示意，不是游戏截图；图中新增铺地、北侧服务路、中央净空和两件物体的关系已由执行者实际查看。临时PNG已在执行者查看后清理，hash与观察范围见 `layout-review.json`，保留本脚本可重新生成；未声称主线程或作者看过该示意图
