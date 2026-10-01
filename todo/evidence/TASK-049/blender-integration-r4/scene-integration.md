# V-35 北立面运行时接管

基线为 `d7222a2`，本项修改 `game/src/world/scene.rs`，不改变地图、楼层、入口权限或可进入范围

## 接管边界

- `V-35` 且边两端 north 坐标均为 153 的外立面由 `v35_byte_beat_facade` 接管，泛型窗、楼层腰线、压顶与公共门构件退出生成
- 保留入口可用高度和地形校验，公共门保持闭合；南侧 `game_service_entry` 继续生成原门框、门叶及对应断言
- 北侧 1.5 m 泛型整段雨棚由候选内同深度雨棚替换；其他三面、原楼体与屋顶保持不变
- `props` 使用单个 `buildings[V-35]/blender-attachment`，源锚点 `[-386,153,12]` 经一次 `map_to_world` 转换；旋转与缩放保持默认
- `business_signs` 没有改动，BYTE BEAT 的原招牌纹理、源关联和独立生成逻辑继续保留

## 门前站位的实际依据

读取 `candidate.glb` 的真实索引三角形，取门中线左右 0.30 m、离地 0.021–1.721 m 的保守 AABB 相交范围，共 44 个三角形；最大北向外凸为 `0.18000000715255737 m`，来自门把手所在范围

门前中心距 0.55 m 减去该外凸、0.30 m capsule 半径与 0.02 m skin，保守余量为 `0.04999999284744268 m`。因此沿用 0.55 m 不是直接复制 V-55 的阈值；新增 Rust 窄测会从实际运行 GLB 重新计算外凸量，并检查完整接近路线和继续前进时的闭门接触

这项静态量测不代替 Ground 支撑、真实 capsule_cast 或角色连续路线

## 检查范围与复现入口

`blender_building_attachments_fit_existing_shells_and_routes` 增加 V-35：3,612 三角面、单一实例、世界包围盒 `[-402,11.82,-154.5]..[-370,23,-153.02]`、源门位置、实际门体投影，以及已存在的 `game_front_court → game_entry` 六米宽道路。接近检查使用基底几何的 Ground 支撑和导入 GLB 三角形构建的 collision query，站位后再向门内移动 0.3 m 必须命中 V-35 模型

同一测试通过仅改内存建筑 ID 恢复泛型立面，逐块核对只有北面构件被移除、旧雨棚恰好移除一组，其余三面构件保持原字节。`all_authored_facades_cover_entries_courts_and_keep_roofs_clear` 仅将北侧公共门交给模型碰撞检查，南后勤门仍在泛型门叶检查中

```fish
cargo test --manifest-path game/Cargo.toml --features viewer --locked -j 1 blender_building_attachments_fit_existing_shells_and_routes -- --nocapture
cargo test --manifest-path game/Cargo.toml --features viewer --locked -j 1 all_authored_facades_cover_entries_courts_and_keep_roofs_clear
cargo test --manifest-path game/Cargo.toml --features viewer --locked -j 1 business_signs_keep_source_aspect_arrival_direction_and_openings
```

本项交接时 `rustfmt --edition 2024 game/src/world/scene.rs` 与 `git diff --check -- game/src/world/scene.rs` 通过；按集中构建安排未执行 Cargo，CPU 窄测、完整 collision 接入、GPU 与连续路线结果由本轮整合验收另行记录

本项未生成截图、视频、接触表或临时可执行文件，无待清理视觉产物
