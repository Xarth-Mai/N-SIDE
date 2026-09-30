# V-A08 与 V-15 Blender 模型接入

2026-10-01，本轮将两套本机 Blender 可编辑资产沿现有 `Appearance.models`、`props()`、WorldAsset 加载路径接入，保持地图、建筑主体、地形、正式光照与曝光。两套模型的 DCC 制作、材质来源、离线几何检查与看图结论分别由 [V-A08 源包](../../../../source-assets/buildings/V-A08/README.md) 和 [V-15 源包](../../../../source-assets/buildings/V-15/README.md) 维护；本页记录运行接入和 CPU 窄测，不代替 GPU 观感验收

## 接入和替换范围

| 项目 | V-A08 | V-15 |
| --- | --- | --- |
| model ID | `v_a08_roof_eaves` | `v15_mirror_hall_facade` |
| 运行文件 | `environment/buildings/v-a08-roof-eaves.glb` | `environment/buildings/v15-mirror-hall-facade.glb` |
| 放置 | `map_to_world([79,277.5,40.421515])` | `map_to_world([300,220,25])` |
| 实例 | 单个 Scene0、identity rotation、scale 1 | 单个 Scene0、identity rotation、scale 1 |
| 实际 GLB | 56 三角、3 primitives | 7154 三角、8 primitives、6 张内嵌贴图 |
| 稳定来源 | `buildings[V-A08]/blender-attachment` | `buildings[V-15]/blender-attachment` |

V-A08 仅取消外轮廓四条通用压顶，其余窗饰、腰线和雨水构件保留。V-15 只对南边 `north=220` 应用替换：含窗台的完整窗范围 `(width+0.28)/2` 与新门面 `x300.58..318.02` 相交时跳过整窗；29m、33m 腰线裁为 `x318..340`；只取消 `cinema_entry` 的旧雨棚，原门、门框、中梃及东面餐饮／北面后勤入口保持

新 GLB 是可视附件，不进入 CollisionWorld；旧泛型构件此前属于 `derived-facade` 碰撞，因此本次不是碰撞数量不变：V-A08 四压顶减少 48 三角，V-15 泛型立面从 2664 减为 2340 三角，减少 324 三角，合计净减少 372。建筑主体与原道路继续提供阻挡和支撑；V-15 原入口仍是关闭面板，本次未实现影院内部进入

材质继续使用每个 GLB 的内嵌标准 PBR 材质，不将同名材质自动重绑到 appearance。没有新增模型 schema、通用资产服务或碰撞路径。appearance 只新增两个 models 条目，所有旧字段保持，包括 `sign_shop`；根据主线程已拒绝的 reflectance 实验，移除 `scene.rs` 对该临时字段的赋值，失败实验由主线程保留证据

## 真实几何窄测

```bash
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib blender_building_attachments_fit_existing_shells_and_routes -- --nocapture
```

测试读取真实源地图、appearance、props 与两份 GLB，应用导出根矩阵、实例平移和 scale 后再检查有限值、索引与三角预算；实际世界包络为 V-A08 `[69.55,40.071518,-284.45]..[88.45,40.421516,-270.55]`、V-15 `[300.58,25,-250.08]..[343.15,36.829998,-216.965]`，没有二次转轴或错误高程

窄测从真实地图道路读取五段路径：`shop_north_junction → v_a08_door_landing → v_a08_door`、`fw_w_cinema_front → cinema_entry`、`cinema_roof → cinema_deck_turn → cinema_upper`。每段取 20 次原世界支撑采样，并仅对导入可视网格做高 1.7m、半径 0.3m、skin 0.02m 的胶囊扫掠，确认新外饰不占用人物空间；这不是控制器实走、相机运行或进入影院的验证

V-15 旧泛型对照通过测试内临时给地图副本中的该建筑更名，关闭本次 ID 特判后重新调用实际 `facades()`；其不属于其他样板或 appearance 店面绑定。新旧盒体按实际顶点逐一比较，确认差异只在覆盖窗、两条南腰线与入口旧雨棚，新增的泛型几何只有两条缩短后的腰线。V-A08 按原地图四边构建旧压顶，确认新输出不再含这些盒体，原压顶 CollisionWorld 确为 48 三角

首轮 `native-test-before-lean.log` 为 PASS；独立审查后删除测试中不必要的浮点位编码、解码与排序集合，直接比较实际顶点，最终 `native-test.log` 为 PASS（1 passed、0 failed、137 filtered out），独立复核结论为 `Lean already. Ship.`；`git diff --check` 与 `bun run check:docs` 同为 PASS（551 Markdown、129 IDs、31 Skills）。源码保持项、冻结 GLB hash 和输出成本见 [source-contract.json](source-contract.json)

## 运行交接

本子项交接时 GPU 为 NOT RUN；之后主线程已统一构建并完成真实模型加载、镜厅正面移动、入口、平台和小店总景，结果见 [实际运行自查](runtime-review.md)。主线程另已实际查看两套资产各三个 Blender CPU 视角；离线图与游戏证据分开记录，V-15 抽象海报仍待项目正式图形完善

V-A08 可复用 `walk-ascent-entry.json` 检查街道接近和支撑，另需能实际看到该楼屋檐的新固定机位；旧 `exterior-details` 观察 V-A13，不能作为 V-A08 覆盖证据。V-15 需南入口正面固定机位和门前接近，`inspect-cinema-bearing` 只用于东平台及斜撑；旧 `eye-cinema` 朝向山顶，不能替代南门验证。地图现有 `cinema-gentle` 本身已往返，后续复用 RouteDriver 时不重复打开 `round_trip`

本接入子项未运行 GPU、未新增图片、录屏或探测二进制；两份本项 `/tmp` 源码对照快照在同步读取与独立审查完成后记录 hash 并删除，未作为编译或 GPU 输入。源码、正式 GLB、日志和 JSON 保留，未提交。模型作者及主线程的视觉产物继续由各自完成实际查看和清理
