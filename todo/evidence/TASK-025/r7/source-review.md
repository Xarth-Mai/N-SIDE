# 山脚住宅与西组平台源修复

## 输入与实际范围

基线 `389c309`，修前源 SHA256 `93d682bce484cb57f182577f504e98d5ff7c22b8f54acdfa252e967da864f315`；本源阶段 SHA256 `b725070d95322c1b0612dabb006066a4dfd6b160ed8b5dbcadf7f61a4dec1d22`。本页只对应该源阶段；站前平台处理与实际画面由本轮总记录补充，后续源变化需对应自己的 hash 与复验

修复 `foothill_house_gate` 的山路／住宅支路合流，以及西组住宅的 `platform_west`、`res_n`、`fw_f_plateau_w2/w3/w5`、`fw_f_junction83`、后排梯道和门前支路。采用已有道路、梯段与水平落地类型，不改检查阈值、入口、建筑、地块或平台

- 山脚合流沿原直线先维持 170 m，待 4 m 住宅支路分离后再上坡，两条同路位的 4 m 步道与 7 m 车行支路共用新节点；S1 剖面与 hill 环线同步展开
- 西组顶部路口 `fw_f_plateau_w5` 从 106.425052 m 调至相邻 `res_n` 的 105.831746 m，形成同高合流；两段上行楼梯先接水平落地，再接沿等高的横街
- w2、w3 梯脚先平接街宽，w2 上端水平段同步延长，避免缩短梯段后斜踏面压住 V-F50 的既有门路
- 后院支路与后排 V-F53 门路分别在各自标高的水平段分离；V-F54 门路旁梯段的上端落地加长，原门点和门口标高保留
- `platform_west` 向东下坡先通过 8 m 平段，再接既有缓路，保持平台转角同高

共增加 8 个节点、5 段 landing，修改 4 个既有道路节点；无节点或道路删除。4 个旧节点的实际变化、全部道路与地形差异见 [source-check.json](source-check.json)

## 数值回归

新增一项西平台／山脚连通回归，在旧源上得到 7 PASS、1 FAIL，记录于 [road-width-before.log](road-width-before.log)。首次临时方案把 w2 梯脚平段加长后，使其上端与 V-F50 门路出现约 0.365 m 的新候选；上端水平段由 0.5 m 延长至 1.75 m 后消除，没有放宽判定

| 检查 | 结果 | 范围 |
| --- | --- | --- |
| 3 组 Bun 回归 | PASS 45、FAIL 0 | [tests.log](tests.log)，地图契约、路幅、地形 |
| 全城路幅 | FAIL 45 个既有候选 | [road-width-all.json](road-width-all.json)，原 61 对移除 16 对，0 新增 |
| 长环线 hill | PASS 0 候选 | [road-width-hill.json](road-width-hill.json)，同时对其他道路检查 |
| 短登高 hill-short | PASS 0 候选 | [road-width-short.json](road-width-short.json)，路径与节点值未改变 |
| terrain bake | PASS 8389 样点 | [terrain-check.log](terrain-check.log)，使用现有生成器 |
| TypeScript 与 Vue 类型 | PASS exit 0 | [typescript-check.log](typescript-check.log) |
| 改动道路与建筑轮廓 | PASS 0 处面积大于 0.1 m² 的交叠 | [source-check.json](source-check.json)，19 段受改动或新增道路；二维数值检查不是人物碰撞验收 |

受改动阶梯的最小踏面为 0.296875 m，现有 0.28 m 下限与每个连续梯段最多 48 级仍通过。长路线三维长度 5228.757754→5228.799287 m，短路线维持 1311.187345 m

地形手工 42 个控制点及 bake 参数未变，重新生成后 64 个既有采样坐标改变高程、270 个新增采样位置、271 个旧采样位置退出；最大既有采样高差为 `[560,630]` 的 -1.003 m。总样点 8390→8389；这些是道路及边缘采样重排，仍需用真实画面核对局部接缝

## 重跑命令

在仓库根执行，以下命令可直接用于 fish

```fish
bun test tools/tests/district-map.test.ts tools/tests/road-width.test.ts tools/tests/terrain-shape.test.ts
bun tools/terrain-shape.ts --check
bun tools/check-road-width.ts hill
bun tools/check-road-width.ts hill-short
bun tools/check-road-width.ts all
bun run check:types
```

全城 `all` 仍预期 exit 1，并明确列出 45 个未修候选；不把剩余失败转成通过。源数据检查不能代替真实画面、人物移动／碰撞、车辆转弯或最终美术验收
