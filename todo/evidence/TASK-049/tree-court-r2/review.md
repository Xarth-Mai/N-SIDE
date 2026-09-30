# 月台杂货两棵街树 · r2

状态：已合入本轮真实场景，Ground 窄测 PASS，包含于全库 139 PASS；实机范围见[集成运行](../blender-integration-r1/runtime-review.md)，不构成作者放行

## 空间与资产

- 近树以 `home` 为锚点，偏移 `[19,-17]`，地图坐标 `[119,238]`，位于两条 6m 绕行坡道之间的既有草地，缩放 0.85、高 5.1m
- 远树以 `v_a08_door_landing` 为锚点，偏移 `[8,9]`，地图坐标 `[99,286.5]`，位于 V-A09 东侧、上层门前支路以南的既有草地，缩放 0.95、高 5.7m
- 月台杂货是 `V-04`；`V-A08` 是坡上住宅，地图、稳定节点与建筑坐标均不修改
- 两棵均复用 `tree_a` 的 `environment/vegetation/street-tree.glb`，每棵 11,686 三角面、两材质，合计增加 23,372 三角面；不新增纹理、材质、模型或全城散布
- 实际 GLB 的 21 个根部顶点半径最大为 0.181760m；16 点 `0.2 × scale` 支撑环及中心调用现有 Rust `Ground::height`，不另写高度算法
- 每个候选先通过既有 `vegetation_space_clear`，覆盖建筑、平台、道路、入口及其他 prop；再限制根环高差 0.25m，以环与中心最低点向下 0.02m 埋根，并复用 `summit_views_clear`
- 放置在既有林下植物与 5 棵 urban-bank 低灌木之后，保留原放置优先级；本批不额外添加低植物，现有低灌木承担远树附近的地面层次
- 两处平面空隙较紧，只有实际几何与 Ground 窄测通过后才能确认位置；输入与资产哈希见 `inputs.json`

## 验证入口

准备时 `output/tree-court-r2/scene.patch` 通过 `git apply --check`，主任务随后合入 `game/src/world/scene.rs`；补丁不是长期编辑源，后续维护实际 scene 实现

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib world::scene::tests::shop_street_trees_keep_actual_roots_supported_and_routes_clear -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --lib world::scene::tests::vegetation_uses_real_assets_and_keeps_public_space_clear -- --nocapture
```

新增窄测要求两个位置都实际生成，读取运行资产的真实顶点，逐根部顶点检查支撑与过深埋入，检查真实树冠界限、道路/入口/物体净空与摘星台视线，并输出 Ground 根高、顶高及实际埋入范围；已有 vegetation 全体检查负责所有新增植被的确定性、数量和净空回归

实际运行后沿用小店 orbit `59 / 139 / 204`、门前 `59` 与走近 `330` 观察近远树是否形成前后层次、树冠有没有挡住门口/橱窗/招牌/主梯、根部和阴影是否贴地；视觉构图与标识可读性由画面判断，数值检查不替代这一结论

补丁在内存中应用后通过 `rustfmt --edition 2024 --emit stdout` 解析与排版检查；现有源文件未写入

独立复查 `/root/character_art_next/hair_review` 已核对补丁、既有 helpers 和真实 GLB，未发现类型、净空规则或复杂性问题，结论 `Lean already. Ship.`；复查未运行 Cargo 或 GPU，Ground 结果仍待主任务执行

实际 [geometry-check.log](geometry-check.log) 确认近树根高 26.552860m、顶高 31.652861m，真实根部埋入 0.00997–0.04321m；远树根高 32.723667m、顶高 38.423668m，埋入 0.01371–0.14887m。两个候选均通过现有净空规则，没有为了摆入树木放宽规则

GPU 画面与清理由主任务统一记录；本子任务自身没有生成额外图片或视频
