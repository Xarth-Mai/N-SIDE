# 镜厅北侧后勤前场 R5 源接入

基于 `5cd3037436e2fdd569569daa1a853f660d11d312` 应用已核定的 [candidate.json](candidate.json)，本记录与前期候选检查分开；候选、原地图 hash 及原 `check.json` / `props-report.json` 保持历史状态

## 源数据与模型

- 正式地图仅追加一块 `cinema-service-court`，面积 198m²、标高 25m、用途和权限仍为 service；道路、节点、建筑、地块、已有 surfaces 和 terrain 均未修改，语义比较及前后 hash 见 [source-delta.json](source-delta.json)
- `game/src/world/scene.rs` 的 `props()` 追加北墙 `aircon_wall` 与西缘 `streetlight` 两个实例，高度读取同一 source surface；保留原模型、源文件、导出文件和其他实例，没有新增 GLB、纹理或夜间灯光逻辑
- 空调锚点保留候选中的北向 `250.203`，以真实后垫块及收水口最外点对齐北墙；灯杆保留既有 glTF 节点的米制变换，完整模型高 4.5m
- 两处用途、坐标、来源和许可沿用 [AST-003 清单](../../../../source-assets/environment-kit/asset-manifest.json)内的 `service_court_placement`，复用说明在[环境资产 README](../../../../source-assets/environment-kit/README.md#镜厅北侧后勤复用)

## 本次已执行检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| `bun test tools/tests/road-width.test.ts --test-name-pattern 'cinema service court'` | PASS，1 项通过 | [窄测日志](source-narrow-test.log) |
| `bun run check:map` | PASS，62 项通过、0 失败 | [完整地图日志](source-map-tests.log) |
| 应用源与 R4 比较 | PASS，仅追加已批准 surface；原 85 项 surface 保留，现 86 项 | [源差异](source-delta.json) |
| `rustfmt --edition 2024 game/src/world/scene.rs` | PASS | 实际命令退出码 0 |
| 修改文件 `git diff --check` | PASS | 实际命令退出码 0 |
| 本轮差异的 ponytail-review | Lean already. Ship. | 复用现有模型、放置路径及几何工具，无新增系统、库或资产管线 |

地图测试检查地块边界、与建筑及其他地面空间的分离、原服务路线权限、全宽路带同高、门前 3.6m 通道及 8×8.5m 卸货区。两个失败回归分别把铺地降下 10cm、向北墙内扩展 1m，均被实际几何约束拒绝；前者专门覆盖一般 0.35m 粗筛不会拒绝的路唇

## Rust 与运行交接

新增 `world::scene::tests::cinema_service_props_keep_supported_feet_and_clear_working_space` 读取实际运行 GLB，在源节点、appearance 与实例变换后检查有限顶点、脚底高度、脚底位于铺地、完整外廓避开道路及每侧 0.32m 余量、卸货净空、空调朝向与窗间空档，以及路灯 4.5m 高度；缺失 source court 时要求给出带 `[scene/service-court]` 的诊断

本子任务未运行 Cargo、Blender 或 GPU；新增 Rust 检查的编译执行、碰撞接入、取景及真实路径证据由 R5 集成继续完成，源检查通过不等于运行或作者美术验收通过。前期布局图已实际查看并清理，本次没有生成图片或视频；历史源布局结论保留在 [layout-review.json](layout-review.json)

## 整合结果

Root 已完成实际 GLB 碰撞与脚底支承、三列服务通道净空检查，完整 Rust 库142项通过、1项忽略，Viewer3项通过。120帧后场横移与180帧空调近景／下看脚本机器通过，Root已查看第49／104帧；完整范围与后续独立自查见 [R5整合](../blender-integration-r5/review.md)和[画面记录](../blender-integration-r5/visual-review.md)。这是真实场景渲染和CPU控制器几何检查，未代替真人行走或作者外观验收
