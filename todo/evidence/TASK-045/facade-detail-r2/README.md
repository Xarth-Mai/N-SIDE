# 临街住宅构件第 2 轮

基线 `dbe5ad3`，复用 `scene.rs` 的米制盒体、按建筑/材质合批与 `MapSource`，不新增材质、外部资产或地图对象

## 实施

- 样板由原有 8 栋扩至 10 栋，新增月台杂货 `V-04` 与南邻 `V-A07`；原有 `V-A08`、`V-A09`、`V-A13`、`V-A14`、`V-W08`、`V-13`、`V-A15`、`V-A16` 保留
- 上层窗增加有厚度的中梃与横梃，窗护栏和带格栅、支架的空调机组沿不同窗格交错；商住窗下保留木裙板，已有窗台和遮雨板继续使用
- 临街檐口收水槽、墙角矩形雨水立管和各层固定卡箍共用一个 `metal` 批次，独立源后缀 `derived-facade/rainwater`；立管按真实外侧地面与屋面高度落位，避开源门节点
- 室外构件依附原有上层窗位；护栏是窗护栏，未将无阳台门的窗格改成可进入阳台。窗构件保留 2.5 m 下方净空、距墙小于 0.61 m，雨水构件限制在 0.22 m 墙面带

已实际查看 [小店白天参考](../../../../docs/public/images/shop-street.webp)和[镜厅夜间参考](../../../../source-assets/area-previews/cinema-music-street.png)：提取安静住宅窗带、贴墙设备与细竖向管线，保留源建筑与公共台阶关系；参考中的具体阳台门与匿名室内布置没有加入地图

## 检查入口

以下命令由主 Agent 串行执行，初始状态 NOT RUN，实际运行结果和画面自查由运行日志与后续补录裁决

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer residential_details_keep_headroom_entries_shells_and_collision -- --nocapture
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/task045-visual-r4-residential
```

现有窄测直接读取正式地图，检查 10 栋覆盖、每栋不超过 6 个材质批次、有限顶点和 UV、窗/门/邻栋不相交、屋顶高度、管线不埋地、窗构件下方净空及碰撞三角面完整；它不证明全城视觉品质或实时帧预算。`exterior-details.json` 的接近与横移覆盖 `V-A13` 西立面，新加入的 `V-04` 仍需在小店路线画面核对

本子任务已运行 `rustfmt --edition 2024 game/src/world/scene.rs` 与 `git diff --check -- game/src/world/scene.rs`，均 PASS；没有单独启动 Cargo、GPU 或生成临时视觉产物
