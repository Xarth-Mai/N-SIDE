# 住宅临街窗遮阳与遮挡层次

基线 `b948cab`，接续[上轮窗构造](../windows-r1/README.md)。已重新读取正式美术方向并查看镜厅、小店概念参考，重点是窗面使用差异与实物层次。本轮保持上轮十栋样板范围，不修改建筑轮廓、楼层、入口、店面、内院与道路源数据

## 制作

`scene.rs::add_residential_screen` 只从既有临街住宅面、上层且满足原净空检查的窗口调用，复用 `awning`、`wood_siding`、`metal`，并入原 `derived-facade/residential` 材质批次

- 第一居住层两端窗格：一扇安装部分放下的室外卷帘，另一扇及下方保留视窗；每组一块帘面、上卷轴与下压条，共 `3` 个盒体
- 其余朝西或朝南的临街住宅窗：上部四道倾斜木百叶与两侧支撑，共 `6` 个盒体；朝向由地图立面法线判断，主要视窗保持开口
- 卷帘和百叶各自沿实际窗框布置，位于不透明玻璃前方，没有透视房间、室内家具或新增每窗光源；不通过随机颜色替代构件，不宣称热工或日照模拟

每栋住宅按原材质继续合批，最多增加已有材质在该栋住宅批次中的使用；没有新材质定义或每窗实体。当前玻璃颜色、窗框、檐口、护窗栏、空调与雨水系统保持，窗饰在既有墙面条带内并保留地面通行净空

## 验证入口与当前结果

本子任务执行 `rustfmt --edition 2024 game/src/world/scene.rs` 与 `git diff --check -- game/src/world/scene.rs`，均 PASS；Cargo 与 GPU 由主 Agent 统一执行

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer residential_screens_leave_partial_views_and_share_existing_batches -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer residential_details_keep_headroom_entries_shells_and_collision -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer residential_glazing_is_recessed_batched_and_keeps_walk_envelope -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer shop_shell_keeps_foundations_and_facade_trim_out_of_public_rooms -- --nocapture
```

新窄测核对旋转窗面上的局部边界、玻璃间隔、下部开口和两材质合批，再从正式地图生成结果统计两类构件及净增成本。原住宅测试继续核对所有构件的真实地形净空、外壳/邻楼/入口/玻璃不相交、屋顶边界和碰撞一致性

主 Agent 的[原生库集成日志](../../TASK-045/visual-r8/lib-tests-retry.log)为 `132 passed; 0 failed; 1 ignored`。本轮新增窗饰、已有住宅净空与玻璃、坡地/邻楼遮挡、小店壳体检查均 PASS；已直接核对日志，真实地图产生 `20` 组部分卷帘、`60` 组固定百叶，净增 `420` 个盒体、`5,040` 个三角面。十栋住宅构件与排水批次合计 `1,541` 个盒体、`18,492` 个碰撞三角；上轮窗壳仍为 `381` 窗、`32,004` 个三角，这些总数和本轮净增分别记录，避免混算

GPU 尚未运行。运行对照复用 `game/capture/exterior-details.json` 的 `inspect-residential-detail` 机位，观察 `V-A13` 西面：第一居住层两端卷帘与楼上百叶应有可辨差异。应实际查看前后 `59`、横移 `60–64` 和稳定 `89`，分别记录画面自查与机器结果，不把 `70–74` 静止帧当成连续移动证据

本子任务尚未生成图片或录屏；统一运行后的临时视觉产物由主 Agent 在实际查看与记录后清理，日志、状态和文字证据保留
