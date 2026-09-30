# 月台杂货公共主梯扶手小样

本轮只增加室外可视金属扶手，复用真实地图、Ground、既有 metal 与盒体批处理。没有新增防坠或扶手碰撞能力；地图、道路宽度、高程、踏步生成及材质配置保持不变，既有橱窗改动完整保留

## 实际范围

| 稳定起点 | 稳定终点 | 水平长 / 净高差 / 坡度 |
| --- | --- | --- |
| level_home_to_shop_north_junction | level_shop_north_junction_from_home | 10.972m / 2.022m / 18.42% |
| level_shop_north_junction_to_shop_upper_junction | level_shop_upper_junction_from_shop_north_junction | 9.897m / 4.296m / 43.41% |
| level_shop_upper_junction_to_steps_mid | pause_steps_mid_level_shop_upper_junction_to_steps_mid | 16.155m / 5.319m / 32.93% |

当前对应 roads[58]、[60]、[62]，实现按稳定节点对寻找，并要求 steps 类型；地图重排不改变关联

每段两端退 0.4m，扶手中心距道路中心线 1.86m，6cm 截面的内缘离原 3m 路带 0.33m。两道横杆沿真实节点坡线布置，分别高于中心坡线 0.525m 与 1.025m；原生射线检查实际生成的踏面，避免把平均坡线当作踏步表面

48 根立柱分别以实际 Rust Ground 四角最低点向下埋入 0.06m，原生测得可见高度 0.692–1.177m；柱顶接到上横杆。横巷平台保持独立断口，V-A08、V-A09 入户路、东侧上巷以及 stairs-rest 停步台维持开放

共 3 个 metal 网格批次、48 根立柱、12 根横杆、720 三角形，没有新增模型或纹理。派生源采用 nodes[起点]/derived-handrail[终点]，按现有装饰归属保持结构碰撞不变

## 验证

- PASS：[原生几何窄测](geometry-check.log)检查实际横杆顶点坡度、生成踏面高度、Ground 柱脚、路带与额外 0.32m 行走净空、建筑及停步台无相交、横巷断口、有限坐标和预算；缺失指定梯段时明确失败
- PASS：原始 geometry::generate 结构碰撞及追加扶手后均为 106757 三角形，来源数量与 bounds 相同。这是该结构集合的比较，不把它冒充包含全部外立面的正式场景总数
- PASS：[出生点与台阶行走](walk-start-check.log)复验既有真实 PreparedScene 路径、墙体和上坡；[主梯往返](walk-roundtrip-check.log)在 1/64s、1/30s 两个固定步长下走完三段及原有返回路线
- PASS：[格式检查](fmt.log)
- PASS：[输入保留](preserved-inputs.json)核对地图、geometry、collision、appearance 的字节 hash；移除本次三个新增代码区块后，scene.rs 与接手的冻结橱窗版本逐字节相同
- 复查修订：初轮高度测试只从横杆中心构造理论坡线，独立审查指出它不能识别真实网格的坡度错误；补上逐顶点残差不超过 0.04m 的检查，随后重跑原生几何窄测通过，独立复查无剩余有效发现，Lean already. Ship.
- NOT RUN：本子任务未编译新的发布/Viewer 二进制、未运行 GPU；主流程仍需构建后查看 shop 同机位和连续主梯画面，不能以几何 PASS 代替实机视觉验收

随后主线程构建并完成 [uphill-runtime](uphill-runtime/run.json) 的 60 帧、4 项检查 PASS，实际查看第 59 帧可辨三段随坡扶手与横向平台断口；同批小店及正式输入复验见[集成运行](../blender-integration-r1/runtime-review.md)。这补齐已记录视点的运行证据，不把扶手当作已经具备碰撞的防坠设施

## 复现

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib shop_stair_handrails_follow_ground_and_keep_routes_and_collision -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --lib real_shop_walk_reaches_steps_and_wall_through_swept_geometry -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --lib walks_prologue_handoff_places_continuously -- --nocapture
cargo fmt --manifest-path game/Cargo.toml --all --check
```

建议运行同一 shop 固定机位及已有 game/capture/walk-preview.json；walk-ascent-entry.json 走另一条主路，不代替本次三段主梯覆盖

本子任务未生成截图、关键帧、视频或临时可执行文件，无新增视觉产物需清理。输入版本见 [inputs.json](inputs.json)
