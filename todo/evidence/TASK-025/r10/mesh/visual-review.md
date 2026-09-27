# r10 河岸、站前与峰顶实际画面检查

审查方式为参与修复者的 self-audit，实际打开 5 张 after、3 张可用 before 以及绕序修复后的 2 张 final；不作为隔离盲测、作者验收或人物碰撞验收

## 画面与版本

地图 SHA-256 `b5c85885cca75c521f6d1289e0b35c0d7632b85c99c93b0265cb64cc85ed7d7d`，原 after 二进制 SHA-256 `a66f49f7b4716f10ed3df276834a88652c5e81d47c7db60e1c77561bbfb22720`；对应实际执行、机位与 exit_code 见 `../after/render-checks.json`

5 个本组机位 exit_code 均为 0，关键连接未被近景建筑或树挡住，均可用于此处几何连续性观察；截图生成成功与下表视觉结论分别记录

| 机位 | 对比与实际观察 | 原 after 结论 |
| --- | --- | --- |
| platform-river-square | before 道路从 +6m 广场中斜穿，形成明显楔形；after 坡在广场外结束，广场完整且平整，南北接入连续；薄铺装接缝仍可见 | 范围内 PASS |
| platform-station-west | before 斜路切入庭院，出现高起的三角边；after 在庭院标高平出后再爬升，原庭院面积保持，穿面消失 | 范围内 PASS |
| platform-summit-east | before 入路抵住平台厚侧墙；after 有清楚的 10 级短梯和平入段，但上半梯出现斜向绿灰三角斑 | 原 after FAIL；修复后的 final 已复验，见下文 |
| platform-summit-west | 无本批同机位 before；after 坡先完成高差，宽路平入峰顶，未见侧墙挡口或明显折断 | 范围内 PASS |
| platform-dock-tip | 无本批同机位 before；after +2m 低平台保持平整，南缘外三阶清楚连接原 +1.5m 末端，下降段不再切进平台内部 | 路面连接范围内 PASS |

实际查看路径均为 `../after/inspect-road-<机位>/inspect-road-<机位>.png`；前三项 before 使用相同机位的 `../before/inspect-road-<机位>/inspect-road-<机位>.png`

码头末端仍是灰盒：低平台与台阶的支撑、船舶接驳、水位和安全细节没有在这张图中得到生产验收；河岸与站前的大面积裸坡、铺装边缝和材质单调属于当前美术残留，不因此把所需连接证据标成不可用

## 峰顶绿斑的真实生成几何定位

`mesh-probe.rs` 直接链接当前游戏 crate 并调用 `world::geometry::generate`，读取生成后的真实 mesh；通过 mesh indices 还原 indexed terrain 三角形，不重新实现地形生成。`mesh-ray.ts` 使用本机位相机 eye/target、55° FOV 和 2048×1152 像素坐标查询几何

- `mesh-ray.json` 的像素 `[930,627]`、`[949,628]`、`[937,639]`、`[975,637]` 最近几何均为 `/roads/901/structure`，三角形对下坡观察者为背面；相邻踏面样本为正面
- 修正索引读取后的 `mesh-overlap.json` 中，其他生成面的 XY 面积 >0.0001m² 的踏面重叠为 0；没有证据支持“地形升到踏面上”的判断
- `riser-probe.rs` 另外调用真实 generate，构造同一座两级梯的升序与降序道路输入；首、中、末立板的 12 个三角形全部朝上坡，见 `riser-probe.jsonl`
- 初版临时 mesh 探针曾遗漏 terrain indices，所得“地形位于踏面下方 1.10–3.71m”的数字已撤回；修正后的文件覆盖初版输出，本记录明确保留这项证据纠正

根因位于 `game/src/world/geometry.rs` 的楼梯首/中立板 `wall(p0,p3)` 与末端立板 `wall(p2,p1)`，两者绕序使立板朝向上坡。实际材料使用正常背面剔除，阶梯正面由此透出后方地形，形成绿灰三角斑

经 root 授权仅修改该共享文件：交换两处 wall 的端点，不改变踏面高度、宽度、道路数据或地形。既有生成几何测试增加两种节点顺序、首/中/末立板的闭合数量与真实三角绕序检查

| 检查 | 实际结果 | 证据 |
| --- | --- | --- |
| 原实现运行新增回归 | FAIL，`ascending: riser at x=0 must face the downhill observer` | `riser-red.log` |
| 修复后同一回归 | 1 PASS / 0 FAIL | `riser-green.log` |
| `cargo fmt --check` | PASS | `riser-format.log` |
| 修复后 GPU 补拍与实际看图 | 峰顶东梯与码头末端均 PASS，见下文 | 原 after 保留为失败证据 |

重现命令（仓库根目录，fish 可执行）

```fish
cargo test --manifest-path game/Cargo.toml --offline --features viewer --lib world::geometry::tests::road_walls_follow_exposed_ground_and_real_neighbor_levels -- --exact
cargo fmt --manifest-path game/Cargo.toml --check
```

## 修复后实际复验

地图仍为相同的 `b5c85885…ed7d7d`，新二进制 SHA-256 为 `7df3c94744a3da14bc21293ec5395625c2b56954b3df456e3586e563e64cce18`，两机位均 exit_code 0，来源见 `../final/render-checks.json`

- 实际打开 `../final/inspect-road-platform-summit-east/inspect-road-platform-summit-east.png`：此前上半梯的绿灰三角斑消失，每级前立板成为完整、连续的灰色条面；前景原有梯段的透空条纹也消失。平台平入段、楼梯尺度和周边地形保持，没有通过改地形或遮挡机位掩盖缺陷
- 实际打开 `../final/inspect-road-platform-dock-tip/inspect-road-platform-dock-tip.png`：末端三阶仍完整清楚，平台与岸侧梯路连接保持，没有因立板绕序改动破坏反向下降段的可见面。台阶支撑和船舶接驳仍遵守前述灰盒范围，不由本次修复补齐

原 after 的视觉 FAIL 已由真实生成几何定位、原实现失败回归、修复后通过回归及同机位最终看图闭环；这两处最终画面的几何可读性在本次范围内 PASS

## 数值与未覆盖范围

初始隔离筛查按未加路面 +0.025m 的高度得到 20 项；正式检查器计入真实路面偏移后，旧源为 22 项，包含两条此前低于阈值的邻里 service 交叠，最终地图仍为 0，采用 `../formal-before-final-checker.json` 与最终正式检查结果。这里不把旧 20 计数继续表述为当前正式总数

道路路幅与平台高差零候选只证明已定义的静态几何规则；本轮画面指出绕序缺陷，说明仍须真实看图。人物碰撞、NPC 导航、手柄路线与完整玩法均为 NOT RUN，不从 Viewer 镜头画面推导这些能力已经通过
