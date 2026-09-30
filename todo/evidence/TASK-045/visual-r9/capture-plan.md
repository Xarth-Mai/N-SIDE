# 第 9 轮实机取景与对照计划

基线 `13397c5`，资源冻结于 `output/visual-r9/baseline`。保留既有真实场景、资产加载和控制器，仅新增树庭挡墙取景点及一条 150 帧脚本；本页记录覆盖计划与只读核对，不提前记录画面通过

## 复用与唯一缺口

| 目标 | 入口 | 覆盖与观察范围 |
| --- | --- | --- |
| 草簇、石块、灌木、树根 | `game/capture/understory.json`，5 秒 | 原 `inspect-understory-r1` `[160,823,358] → [145,835,356]`；r8 已真实覆盖这些对象，继续用 29/59/89 及 60–64 横移帧检查实体形体、材质、接地与遮挡，不换回原先正对开口的旧机位 |
| 林缘疏密及山路中景 | `game/capture/forest-overview.json`，3 秒 | 原 `inspect-forest-r1` `[225,650,380] → [225,850,380]`；用于整体林群和山面关系，29/44/74/89 配同条件转头，远景不能证明近景草石质量 |
| 人物颈肩、脸、发束 | `game/capture/character-orbit.json`，每人 8 秒 | 分别指定真实 `CHR-001` 与 `CHR-002`；59 正面、89 侧面、139 背面、204/219 前三分之四，补 80–82 与 190–194 连续帧。沿用真实 Idle 与锁鼠相机环绕，不写人物 Transform |
| 人物走跑中的穿插与材质稳定 | `game/capture/walk-character.json`，每人 18 秒 | 只为本轮实际改动的角色复验，检查 Walk/Run、两种疾跑、暂停恢复；骨骼状态通过与颈肩/发束观感分开记录 |
| 小店树庭北侧挖方挡墙 | 新 `game/capture/tree-court-retaining.json`，5 秒 | 新 `inspect-tree-court-retaining`，真实入口 `[62,202,23] + 1.7m` 看 `[60,223,24.7]`；A/D 横移后轻转头，观察墙冠、施工缝、两端收口及后方 V-A07 与地面的关系 |
| 住宅窗外立面 | 既有 `game/capture/exterior-details.json`，5 秒 | `V-A13` 近景能检查横移中的窗饰与玻璃，不能看到本次 `shop-tree-court` 挡墙；仅在共享材质或住宅代码受影响时补回归，不为复用旧脚本把不同地点当成同一结果 |

现有 `shop-retaining.json` 面向小店用地南缘 `y245` 的低矮墙体，本次新构件位于 `surfaces[2]` 北缘 `y223`，两者之间隔着已有建筑与场地，不能沿用旧图证明新墙冠和接缝

## 树庭机位的来源核对

`shop-tree-court` 为 +23m 的真实庭院，北边 `[69,223] → [45,223]`。相机基于 `nodes.tree_entry=[62,202,23]`，该点原 Ground 为 23m；按自由相机左右约 2m 的脚本运动保留人尺度观察高度，未为构图移动墙、树或建筑

唯一附近原树是 `trees[45]=[53,211]`，位于中心射线左约 8m。以原树 3.5m 水平包络做保守观察，主要检查带 `x58–66/y223` 在相机短横移中保持与树干/树冠中心分开；西端墙可能被树局部遮挡，不能声称一次取景看清整段 24m 墙面。北墙背后的 `V-A07` 从 `y229` 开始，不穿相机至 `y223` 墙面的中心射线，但其与墙冠的实际高差需要最终画面确认

facade 子任务已确认新机位与构件范围一致，不需向东北入口挪动。此项只是源位置与包络核对，枝叶实际形状、阴影、屏幕 UI 及新墙体遮挡仍须 GPU 看图

## 同条件对照

新 view 尚不存在于旧二进制，仅用 `--project-root` 换基线资源不能回滚已改变的 `scene.rs` 构件。新墙基线应由主 Agent 使用相同新相机代码、同参数和基线场景实现生成，再恢复本轮实现生成 after；分别记录实际二进制、场景实现与资源 hash，不把新二进制加载旧素材写成完整代码基线

普通资源改动可使用现有 `--project-root output/visual-r9/baseline`，但要核对该次改变是否仅在 GLB/贴图中。草石模型、材质或摆放同时变化时，把整体差异记录为组合结果，不从单张图归因某一参数

示例为仓库根的 fish 可直接执行命令，输出目录必须不存在；主 Agent 决定基线二进制的具体位置

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/tree-court-retaining.json --aa taa-ssao --output output/capture/task049-tree-court-after-r1
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/understory.json --aa taa-ssao --output output/capture/task049-understory-r2
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/forest-overview.json --aa taa-ssao --output output/capture/task049-forest-overview-r2
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-002 --script game/capture/character-orbit.json --output output/capture/task047-ling-orbit-r2
```

新挡墙脚本的明确断言只检查真实横移、转头与释放后的稳定状态，资源/Transform/帧输出沿用 capture 自动检查；接缝厚度、材质与好看程度由实际画面自查判断。至少查看 29/59/89/104 及 60–64 连续帧，再记录问题或修订并复验

## 本子任务检查

- `rustfmt --edition 2024 game/src/bin/map_viewer.rs`、`git diff --check` 通过
- Bun 实际解析上述五种脚本，核对 district 机位存在、事件/断言范围及关键帧均在帧数内；这不是原生 schema 解析通过
- Cargo、原生脚本解析、GPU capture 与本轮最终看图均由主 Agent 执行，本页写入时 NOT RUN
- 本子任务没有生成图像、视频或临时程序；主 Agent 实际查看并记录 hash/结论后统一清理视觉产物
