# 第1轮上街与山脚路口修复

基线 `b64759e`，源从 `49f46aed0e9d3a578304f0a99d7174864e7a9c2dcc9382c7ffcd1c2ca402acf0` 更新为 `77d5870f1d145d02ea353f2f6f351665261fe2735a2816916fb921a7e1501a8c`。本轮沿用既有路幅与台阶几何检查，阈值仍为交叠面积大于0.5m²且高差大于0.35m；全部候选继续输出，没有按共节点豁免

## 源修复

| 连接 | 原因与处理 | 移除的候选 |
| --- | --- | --- |
| old_home | 台阶直接从6m宽横街起步，先留8m水平接入，再重排四梯段；保留每处2m休息段 | 2对，原最大2.012m |
| home_north | 后街与第一梯段急角交叠，先留6m水平接入，两梯段重新分配长度 | 2对，原最大1.262m |
| upper_homes西侧横街 | 西路高程变化延伸进上下端平台，分别留6m与8m水平接入 | 3对，原最大0.434m |
| foothill_east | 向东北上升的梯道与向东下降的车道在急角处重叠，梯道先向北，再以水平段转入上坡；车道入口8m同高 | 2对，原最大9.050m |
| north | 西山口梯道直接从7m宽横路上升，第一踏步前留4m水平接入 | 1对，原0.613m |

增加8个节点、4段明确的landing道路，原节点ID全保留；18个既有梯段端点仅作平面位置调整，高程不变。实际变更与修前10对候选见[source-check.json](source-check.json)。建筑、所有场所及到达、平台、树木、地块、公共连接、架空对象和后勤数据保持语义不变；相关剖面与长登山路线加入新增节点，短登高路线完全不变

## 实际检查

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `bun tools/check-road-width.ts all` | FAIL，108降为98对，原10对消失且0新增；剩余最大3.327m | [全城剩余](road-width-remaining.json) |
| `bun tools/check-road-width.ts hill-short` | PASS，0对 | [短登高](road-width-short.json) |
| `bun test tools/tests/road-width.test.ts tools/tests/terrain-shape.test.ts` | PASS，6项；受影响梯段最多47级、最小水平踏面约0.320m | [测试日志](geometry-tests.log) |
| `bun tools/terrain-shape.ts --check` | PASS，8359采样，重烘焙无漂移 | [烘焙检查](terrain-check.log) |
| `game/target/debug/map_viewer --project-root . --validate` | PASS，1518节点、858道路、231楼体、82平台、72源树点、3220网格；此运行早于下述挡墙修复 | [CPU检查](viewer-validate.log) |

新增回归的首次范围选择将同一长路远端的其他路口也计入本批，保留[失败日志](geometry-tests-first.log)。修正为被修共同节点及新增接入节点与全部道路的交叠，未修改几何阈值；远端问题仍在98对全量报告中。该修正不把相连长路的全部端点宣告通过

## 挡墙修复后的客观检查

东山脚首轮实图暴露内部挡墙穿路，随后修复 `game/src/world/geometry.rs`。以下命令已实际执行，表格为执行结果摘要，未伪造或补写终端原始日志；本次证据整理没有重跑编译

| 命令 | 实际结果 | 证明范围 |
| --- | --- | --- |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib world::geometry::tests::real_map_generates_finite_geometry_without_raising_ground_to_roof` | PASS，1项、20项过滤，测试7.74s | 实际地图经修后生成器产生有效几何；不证明视觉正确 |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib world::geometry::tests::road_walls_follow_exposed_ground_and_real_neighbor_levels` | PASS，1项、21项过滤，测试0.01s | 同高路口无内部地形墙、异高接缝保留、外露切坡保留、真实生成路径的台阶立面高度正确 |
| `cargo clippy --manifest-path game/Cargo.toml --lib --features viewer --locked -- -D warnings` | PASS，1.41s | 库与Viewer功能组合无Clippy警告 |
| `cargo fmt --manifest-path game/Cargo.toml`、`git diff --check` | PASS | 格式及差异空白检查 |

第一项运行发生在新增回归写入前，后续新增测试没有改变被测运行逻辑。新增回归首次编译遇到测试断言对二重引用使用 `flatten()` 的类型错误，修正为 `flat_map(|tri| tri.iter())` 后通过；没有修改断言边界或运行算法来消除该编译错误

检查完成后的 `geometry.rs` SHA-256为 `fb86a856778e22edf048883958b982947e349d0103ae40010ac6806c91791ecd`；地图SHA-256仍为 `77d5870f1d145d02ea353f2f6f351665261fe2735a2816916fb921a7e1501a8c`。几何修复过程及独立只读复核见[source-review.md](source-review.md)

## 运行检查与范围

首轮8个机位的机器检查完成后，实际查看 `output/road-junctions/2026-09-27/r1/inspect-road-foothill-east/inspect-road-foothill-east.png` 发现明显三角竖墙，视觉FAIL。修复后GPU复拍由统一运行检查补充，本记录尚未签署视觉PASS。检查机位按源坐标x/y/z取五处接入点，见下表；这些是检查相机，不代表玩家位置

| 位置 | Eye | Target |
| --- | --- | --- |
| old_home | -8 / 318 / 52 | 0 / 340 / 47 |
| home_north | 91 / 324 / 51 | 109 / 345 / 50 |
| upper_homes | 143 / 363 / 68 | 123 / 373 / 63 |
| foothill_east | 592 / 695 / 191 | 578 / 720 / 177 |
| north | 56 / 560 / 160 | 70 / 577 / 154 |

道路宽度检查覆盖地表道路顶面及真实踏步，不包含建筑遮罩、挡墙、桥梁净空或人物碰撞。人物控制器尚未接入，人物通行与登高手感为NOT RUN；本任务仍为active，下一批按源数据与画面继续处理剩余路口
