# 小店主梯阶鼻外观样板

## 问题与选择

接续 [R2真实画面与逐级投影诊断](../stair-surface-r1/projection-review.md)：上坡观察时，高于眼睛的水平踏面背向相机；相同朝向、材质的踢面缺少可见级线，读起来像连续深灰带。样板增加真实阶鼻外表面，不改变道路高差或通过规则

本轮读取 nside、nside-blender-pipeline、bevy-testing 与 ponytail 技能，沿用现有世界生成器和材质。该构件依赖每级实时生成的真实边缘，直接接入 `geometry.rs`；没有新增 Blender独立模型或另一个道路数据源

## 实现范围

仅三个稳定起点对应的既有小店主梯：`level_home_to_shop_north_junction`、`level_shop_north_junction_to_shop_upper_junction`、`level_shop_upper_junction_to_steps_mid`，当前分别12／26／32级。其他城市和山梯没有同步添加样板

每级在下坡侧前沿增加25mm高、25mm出挑的45°小外帽：斜面2三角、下封2三角、两端封口各1三角，共6三角。70级合计420三角、3个mesh批次。最大高度等于原踏面，不增加新的走面高程；内侧沿用原踢面作为承托，不导出重叠背板

沿用既有 `paving` 材质、米制UV及实际法线，未新增材质、贴图、灯光或emissive。斜面朝上且朝下坡，因此在当前太阳方向下存在少量几何直射；最终可读性与反差必须看实际运行画面，不能只用法线点积宣布成功

新增来源为 `nodes[起点]/derived-stair-nosing[终点]`，沿用既有栏杆的节点派生装饰归属。`CollisionWorld`只接收既定结构来源，不接收该视觉附件；原 `/roads/...` 的踏面和structure仍按原路径负责支撑与阻挡。没有通过关闭碰撞或修改原道路来掩盖变化

## 窄测与交接

新增单个测试 `world::geometry::tests::shop_stair_nosing_preserves_treads_and_collision`，实际生成当前地图，核对：

- 仅3梯收到样板，原12／26／32级踏面数量和每级走面高程保持
- 外帽顶点有限、每三角非退化且winding与法线一致，斜面朝上且朝下坡，外帽顶面不超过原踏面
- 用相同3梯结构分别构造不含／包含新附件的 `CollisionWorld`，三角数、source数相同，70处真实支撑命中的位置与来源完全一致

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib world::geometry::tests::shop_stair_nosing_preserves_treads_and_collision
```

交接时 `rustfmt --edition 2024 game/src/world/geometry.rs` 与 `git diff --check -- game/src/world/geometry.rs` exit0；Cargo与GPU由Root统一执行，此处不代填结果。初始代码提交时的视觉状态为NOT RUN，须使用既有小店低眼位和抬高侧视、相同MSAA4／光照／FOV完成对照，另复验原玩家上阶路径

看图重点为原深灰带内是否能看清级距、近处是否出现过亮条纹、侧端是否漂浮、与原铺装是否协调。远到级距或阶鼻本身不足1像素时允许自然融合；根据实际结果决定保留、调整或撤回候选，没有全城铺开或作者外观批准
