# 小店门前实体陈列 r1

2026-09-30，本子任务只修改 [scene.rs](../../../../game/src/world/scene.rs) 的室外陈列生成及同文件窄测，地图、公共入口、建筑外壳、材质配置与现有招牌保持原输入；构建与正式入口 capture 由主 Agent 统一执行

## 输入与观察

已实际查看 [shop-street.png](../../../../source-assets/area-previews/shop-street.png)：店门两侧的矮架、层板和商品构成独立于墙面橱窗的近景层次，室外陈列没有封住入口。概念图的布局不是米制地图依据，本轮使用 V-04 共用店门和既有 shop-site 平台定位

已阅读 [TASK-045 第 2 轮运行记录](../runtime-r2/review.md)及 [TASK-049 外立面记录](../../TASK-049/facades-r1/review.md)。当前雨棚已有坡面、下缘和支架，橱窗已有 0.28 m 厚窗框与窗台，因此本轮保留这些构件，补充缺少的店外实体陈列

实际查看主 Agent 新鲜基线 `output/capture/task045-shop-r3-before/frames/frame00125.png` 与 `frame00199.png`，门两侧存在可补充经营内容的空白墙面；铺装细缝由独立几何修复接续，角色占位体与整条采购街品质不在本子任务完成范围

| 输入 | SHA-256 |
| --- | --- |
| shop-street.png | `3fbba13beda3d9d91d37eb56536e79de2998bd9259e3769d14b75500c0953cf8` |
| district.json | `a4ab861f756037206520258725a32232de40a96dc1a8da380d26a1de33c9e45a` |
| appearance.json | `b4f10d5c8669005b5e65a444484e43f0880c860cebf9c1ef493aca2fc358d268` |
| scene.rs 本次实现 | `357c49a03d05ba98a1e1d092f1185dfdb9984b0f622c39a6d9586b29c0bcdfbf` |

## 制作与边界

两组固定文具／盒装小件陈列架锚定 `shop_front_door [88,255,28]`，沿 `design.front` 前后各偏移 3.4 m、向外偏移 0.9 m，底部中心分别为 `[88.9,251.6,28]` 和 `[88.9,258.4,28]`。每组宽 1.48 m、深 0.64 m、最高 1.3175 m，四根细立柱接地、三层木板、两层有标签的盒装小件，标签只提供颜色层次，没有新增文字、玩法或搬货交互

几何使用现有 `add_box`、米制 UV、材质合批与碰撞路径，材质复用 `wood_siding`、`trim`、`awning`、`terracotta` 和 `shop_ceiling`。没有新纹理、模型、下载或许可输入；五个网格批次均保留 `buildings[V-04]/derived-facade/street-display` 源关联，按生成循环为 52 个盒构件、1,248 个顶点、624 个碰撞三角形，最终实测以主 Agent 窄测日志为准

陈列向外最远 1.22 m，与现有窗框、窗台及两端花盆分离；入口两侧各留出至少 1 m 的无新增物件区域，原门洞仍按源数据净宽 1.2 m，不将门洞误报为 2 m。沿街通行带的 2 m 要求单独检查，测试范围为平台内中心线 `[91.6,252..264,28]`

## 检查与接续

- PASS：`rustfmt --edition 2024 game/src/world/scene.rs`、`git diff --check -- game/src/world/scene.rs`
- 已新增可执行窄测 `world::scene::tests::shop_street_displays_are_supported_and_keep_public_clearance`：真实地图与材质绑定、有限正体积、米制 UV、完整平台覆盖、八只脚接地、建筑外侧、既有立面构件包络不相交、源关联碰撞三角数量、0.32 m 胶囊的共用入口接近、2 m 沿街通道扫掠，以及对陈列架本体的真实碰撞命中
- NOT RUN：本子任务没有运行 Cargo、GPU 或新增画面的视觉判断，主 Agent 统一运行上述窄测、现有场景检查及 `game/capture/walk-shop-exterior.json`，结果由集成证据承接

正式人物路线可直接观察南侧架与门前关系；需要细部镜头时目标取地图 `[88.9,251.6,28.7]`，重点观察四脚接地、薄层板与小件的接触、转动中的边缘和公共门辨识，不新增只供录像的场景

本子任务没有生成截图、PNG 帧、接触表、视频或临时可执行文件，没有删除主 Agent 的基线画面；基线与后续 capture 由主 Agent 在实际查看并记录后统一清理

## Root 实机收口

同一 `walk-shop-exterior.json` 最终450帧、9项机器检查PASS，root实际查看199、330关键帧和181—182连续横移帧，确认两架陈列、上下层包装／标签及四腿接地，门口未被物件封堵；无异常恢复，路线始终在室外。画面仍为待精修样板，包装语义与木材质仍简化，未取得作者品质验收。原始参数、hash与结果见[集成证据](../integration-r3/review.md)
