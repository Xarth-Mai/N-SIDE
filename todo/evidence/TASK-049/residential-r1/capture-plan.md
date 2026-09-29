# 外皮与街树近景证据路线

只在 Viewer 的 camera_views 增加两个 `inspect-` 机位，继续加载完整 district 场景；没有移动建筑、树木、地面、灯光、角色或布置额外样品。源坐标与实际计算结果见 [capture-views.json](capture-views.json)

- `inspect-street-tree-detail`：地图 `trees[68]=[136,321]` 位于既有 `shop-steps-low` 公共绿荫台地，实际支撑高程 +45.127273 m；原派生规则中索引 68 非 11 的倍数、为偶数且低于 hillgate，因此确实加载 `tree_a` 新街树。相机在真实 `shop-steps-low-junction → shop-steps-low-entry` 1.8 m 公共小路的 30% 处加 1.7 m 眼高，距树干约 9.17 m，瞄准树高 3 m 处
- `inspect-residential-detail`：从真实 `east_mid_junction=[144,288,+31.9]` 加 1.7 m 眼高，观看 V-A13 原西侧立面／v_a13_door 上方住宅层，水平约 17.87 m。该栋属于本轮 8 栋已生成窗檐、窗台、护栏与窗下设备的公寓，不以另一栋旧楼代替新外皮证据

各脚本 150 帧／30 Hz，先开启既有自由相机，然后向前 4 帧、左右各横移 4 帧、释放控制。位移断言来自真实自由相机系统，停止后验证稳定；这是 Viewer 控制与渲染证据，不声称完成角色碰撞或真人步行验收

在仓库根目录执行，命令可直接用于 fish；root 统一编译和串行 GPU 运行：

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/street-tree-detail.json --output output/capture/task049-street-tree-detail-r1
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/task049-exterior-details-r1
```

实际观看首帧、接近后的关键帧及 30—34、60—64 帧连续变化：街树检查枝叶空隙、干枝连接、叶片双面和投影；住宅检查窗檐厚度、护栏与窗面间距、设备悬挂和材质重复。看完保留日志、状态 JSON 与文字结论，清理临时截图、帧序列、关键帧和视频

## 本次准备检查

- PASS：JSON 可解析、事件／关键帧／断言时序在 150 帧范围内，两个脚本场景均为实际 district
- PASS：源树对应 tree_a 条件、原公共道路节点及 V-A13 原立面核对；实际相机参数写入源码并在运行时打印
- PASS：`rustfmt --edition 2024 game/src/bin/map_viewer.rs`、`git diff --check -- game/src/bin/map_viewer.rs game/capture/exterior-details.json game/capture/street-tree-detail.json`
- NOT RUN：本子任务不执行 Cargo 或 GPU，最终编译、原生脚本解析、状态断言、画面与成本由 root 接续实测
