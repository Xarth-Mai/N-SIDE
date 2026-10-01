# 镜厅门锚与门前站位

2026-10-01，TASK-049，真实路线录制前的源数据修正。最终地图 SHA-256 为 `a2f15c59246a57c1250ee76db54e0a12fa5ce766f17f15dc1535e5895128038f`；本记录覆盖源数据、Bun 窄测与几何推算，后续原生碰撞和GPU结果见 [R3汇总](review.md)

## 原问题与依据

`cinema_entry=[306,220,25]` 是 `V-15` 南立面的建筑门锚，原 `cinema-gentle` 要求人物中心抵达此点再折返。`geometry.rs` 只为 `V-04` 的已实现小店开门洞，镜厅主体与门仍关闭；`scene.rs::facades` 保留3.6×2.75m玻璃、门框及中梃，源资产文档也明确其未提供室内

中梃中心在门外0.16m，厚0.10m，南侧前沿为north219.79；`player.rs` 的半径0.3m、skin0.02m和分离0.001m意味着沿中线理论脚点最多到north219.469，仍距门锚约0.531m。`RouteDriver` 要求距离小于0.06m、真实支承且无重置，两者含义冲突。该数值为源码几何推算，尚未假称一次实际堵门录像

## 源迁移

| 数据 | 本次改动 |
| --- | --- |
| `nodes.cinema_entry` | 原 `[306,220,25]` 与建筑条目保持 |
| `nodes.cinema_entry_landing` | 新增 `[306,219.35,25]`，距闭门0.65m，沿用已有附件接近窄测的站立距离 |
| `roads[649]` | 原 `front → entry` 改为 `front → landing → entry`；仍为lane、宽3m，三点共线同高 |
| 场所15的public arrival | 在原门前插入landing，最后的建筑门锚与使用权限保持 |
| `cinema` 生活路线 | 从门锚经过landing再到front，维持原完整路线含义与邻接 |
| `cinema-gentle` | 在landing折返，仍关联场所15、从home出发并返回；41节点不变 |
| 地形派生样本 | 既有生成器重烘后只将 `[320,270]` 的24.936m改为24.937m；9752样本不变 |

闭门、建筑基底与入口标高、控制器、到达容差、移动速度、实际玩家出生点均不改。新增站位在真实铺地内，并非capture专用偏移；0.65m退界后比中梃的胶囊接触线多约0.119m净距。原路幅的两端与插入点横向offset一致，不产生新平台、支路或并行重叠道路

现有生活路线目的地检查相应允许门锚或其arrival末段的具名 `*_landing`；后缀本身不足以通过，还要求公共道路直接邻接、水平距离0.32–1m、同高且站位在建筑外。本例额外窄测精确检查0.65m、当前中梃+人物包络、前场支承范围、3m道路原轮廓、原门条目与完整往返

主任务同步调整 `scene.rs` 现有资产接近测试的终点为landing，避免对站位再次减去0.65m；这只改变测试目标语义，不改变碰撞或游戏输入

## 已执行

```fish
bun test tools/tests/district-map.test.ts --test-name-pattern 'cinema arrival|lifestyle routes|metre scale|detailed places'
bun tools/terrain-shape.ts
bun tools/terrain-shape.ts --check
```

4项窄测PASS，覆盖全图缓行路径、详细场所到达链、按序目的地、镜厅新站位与原路幅；详见 [测试日志](cinema-landing-test.log)。初次地形freshness检查FAIL，经比对定位上述唯一1mm派生差异后使用原生成器更新，最终freshness PASS，未为消除检查错误改变其他地形设计

[源检查结果](cinema-landing-source-check.json)记录地图hash、全部修改位置、路线估算与1m网格采样：镜厅地块包围盒内4988点的Bun地面差异最多约0.001m，仅与那一个重烘样本相关。比较前态由撤销上述节点/链变化并恢复24.936m样本重建，保留本批原有前场与建筑变更；该采样不替代原生碰撞查询

更新后的往返水平长959.826124m，按3.2m/s约299.945664s；9900帧脚本保留约30.1s余量，理想到门前第4560帧、回家第9059帧。实际到达时间仍以原生 `route_all_nodes_reached` 的41条访问记录为准，镜厅室内、电梯与绿岛横穿不在此路线证明范围内
