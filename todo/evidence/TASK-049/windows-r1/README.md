# 住宅上层外窗构造首轮

基线 `e6b398b`，本轮处理小店起坡至坡地住宅已有十栋样板的黑平窗表现：`V-04`、`V-A07`、`V-A08`、`V-A09`、`V-A13`、`V-A14`、`V-W08`、`V-13`、`V-A15`、`V-A16`

已查看仓库镜厅与小店预览参考，采用窗框、退进玻璃、窗台和既有滴水檐构成外墙层次；既有建筑、入口、道路、公共台阶和稳定 ID 保持原数据。本轮只处理符合原住宅构件净空条件的上层外窗；底层店面、门、内院及邻楼/坡地遮挡处继续沿用已有构造，不建立室内

## 实现范围

`scene.rs::add_residential_glazing` 复用 `add_frame` 与 `add_box` 的按材质合批路径：每窗左右两片玻璃、两侧框、上框、窗台和中梃共七个盒体。玻璃前表面较窗框最外表面退进约 `0.21m`，玻璃保持在建筑壳外侧；窗台较玻璃外挑约 `0.27m`，新增体量限于满足净空筛选的上层墙面

新增全样板共享的 `window_glass` 材质与每栋单个玻璃批次；商铺、门与其他建筑的 `glass` 不变。窗格仅采用三种低对比均匀明暗，不绘制环境倒影，不宣称提供真实环境反射。颜色通过顶点属性保存，合批时移动并补齐颜色数组，避免不带颜色的 Cuboid 合并后造成顶点属性长度不一致

既有住宅正面构件删除重复的两条旧窗棂与混凝土窗台，保留已有檐口、窗护栏、窗下空调、木饰面与排水管。未增加每窗实体、材质或光源；没有扩大至全部住宅家族，实际几何成本由读取正式地图的窄测输出

## 验证

本子任务已执行 `rustfmt --edition 2024 game/src/world/scene.rs` 和 `git diff --check -- game/src/world/scene.rs`，结果 PASS

新增 `residential_glazing_is_recessed_batched_and_keeps_walk_envelope` 检查两种朝向下玻璃退进、中梃间隙、连续窗口合批颜色完整、十栋源对象覆盖、单栋单玻璃批、真实地形净空、壳体及屋顶边界和有限几何；输出实际窗数与外窗三角成本。既有住宅构件避窗检查与坡地/邻楼遮挡回归同时纳入 `window_glass`，避免新材质绕开原有检查

初稿时 Cargo 与 GPU 尚未运行；root 集成阶段已完成下述窄测及完整库检查，实际结果追加于后文

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer residential_glazing_is_recessed_batched_and_keeps_walk_envelope -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer residential_details_keep_headroom_entries_shells_and_collision -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer slope_and_neighbour_occlusion_remove_windows_without_hiding_entries -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer shop_shell_keeps_foundations_and_facade_trim_out_of_public_rooms -- --nocapture
```

机器状态、实际画面自查与作者验收分别记录；root 后续已获取真实运行截图并查看，窗口制作 Agent 本身未执行 GPU；几何检查不替代画面品质通过


## Root 集成复验

完整库 126 PASS、1 worker ignored，Viewer 3 PASS、build／fmt／clippy PASS；窄测实测十个共享玻璃批次、381 窗、32,004 窗壳三角，此数未扣除移除的旧配件，不作净增量。实际日志见 [window-counts.log](../../TASK-045/visual-r6/checks/window-counts.log)

`exterior-details.json` 前后各 150 帧、6 项检查 PASS。root 实际看前后59、新60／62／64横移与70／74停下帧；厚框、侧面暗部和较亮的双片玻璃可辨，首层和入口保持，所看范围未见明显穿插或跳位。整楼仍同型重复，玻璃没有真实室内与局部反射，艺术审查仅将其记为层次改善；作者审美、其余九栋近景及原生性能预算尚未验收

帧 hash、命令、二进制与状态摘要保留在[整合证据](../../TASK-045/visual-r6/review.md)，本轮图片和视频实际查看后由 root 统一清理
