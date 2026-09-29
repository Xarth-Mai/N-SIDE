# CHR-001 跑步姿态修订

基线为 `f93af27`，本轮在上一轮正确的支撑轨迹上改善 Run 掌心朝上、直躯干和平脚摆动。最终动作源只修改 Run；Idle、Walk 与控制器的 3.2 / 5.6 m/s 世界位移归属保持，头发形体另由 `model-r6` 负责

## 制作与取舍

前臂内旋承担主要手掌朝向，腕部补少量旋转，肘屈随前后摆变化；手指改为松握候选。现有每指单骨只能表达简化弯曲，本轮没有增加指节或修改骨骼身份

Spine 加入小幅前倾与随步伐变化的俯仰，Chest 反向扭转，Head 作少量补偿，让上身各部分有相对运动。Hips、大腿、小腿原有动作保持，避免上身修正破坏已经验证的接地

Run 摆动脚加入趾下垂、回收与抬趾，接地期仍保持原平地支撑。这是空中脚俯仰，不是已经完成足跟到足尖的接地滚动；后者需要以鞋底真实接触点验证，不能用踝点速度代替

第一候选手掌已向内，但前摆高点仍明显朝上，模型定义的掌面法线 Z 分量最高 0.654；把内旋更多分配到前臂后，候选 b 降到 0.302，向内 X 分量最低 0.851。该数值只描述此模型掌面方向，不是通用人体规范或视觉评分

## 实际验证

候选 b 在同一 r5 master、灰阶与固定相机上，以 CPU Cycles 2 线程、12 samples、384×512 生成 20 格 Run 侧视完整周期及源帧 1 / 7 / 13 正面。制作 Agent 与 art_review 已实际查看全部；知情自查结论为托盘手势明显减弱、躯干前倾和空中脚俯仰可见，未见袖口撕裂、肘网格折断或腰部穿裤阻断

剩余观感包括简化手指、偏张的肩肘及温和的躯干动力；本批可冻结技术候选，不表示自然跑步或人物最终质量验收。旧 r5 头发仍出现在这些动作隔离小样中，不能把它们当作 r6 头发结果

|检查|候选结果|
|---|---|
|真实导出 GLB 120 FPS 足轨|PASS，复用 r1 读取真实 glTF STEP / LINEAR / quaternion slerp 的工具|
|Walk 支撑均速 / 位移漂移|约 -3.2 m/s，峰值 0.283 mm|
|Run 支撑均速 / 位移漂移|约 -5.6 m/s，峰值 0.642 mm|
|120 FPS 源鞋底最低值|Walk -1.509 mm，Run -2.537 mm；小于候选 5 mm 检查界限|
|循环首尾骨矩阵|差为 0|
|检查器反例|未改动作的正式基线实际退出1，明确拒绝 `Run_changed=false`，记录 `baseline-rejected.json` / `.log`|
|与 f93af27 的 GLB 语义对比|节点/bind/材料/atlas、完整 Idle / Walk、Run Root/Hips/Thigh/Shin 动作相同，Run 其余姿态发生预期变化|

`candidate-a-dcc.json` 对应仍有朝上倾向的第一候选；`candidate-final-dcc.json`、`candidate-final-foot-tracks.json` 与 `candidate-contract.json` 对应已看过 Run 造型的最终动作候选。正式主文件已与 r6 头发源码统一重建，最终证据为 `final-dcc.json`、`final-foot-tracks.json`、`preserved-contract.json` 与 `delivery.json`；正式 DCC/GLB 检查及 master 重导出逐字节一致由 model-r6 负责者实际执行通过

## 复现

在仓库根目录运行，命令可直接用于 fish；原型只在内存中重算现有 master 的动作，并写入专属 output 目录

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/animation-r2/prototype.py -- --render
python3 todo/evidence/TASK-047/animation-r1/measure.py --asset output/characters/CHR-001/animation-r2/candidate-b/yao-grey-study.glb --fps 120 --gait --check --output todo/evidence/TASK-047/animation-r2/candidate-final-foot-tracks.json
python3 todo/evidence/TASK-047/animation-r2/check-contract.py --asset output/characters/CHR-001/animation-r2/candidate-b/yao-grey-study.glb --output todo/evidence/TASK-047/animation-r2/candidate-contract.json
```

`check-contract.py` 每次从实际 Git 基线读取旧 GLB，比对解码值而非仅比较文件名或本地声明；用于读取的临时旧 GLB 自动清理。对正式交付 GLB 运行时省略 `--asset` 即可，仍只检查本轮明确保留的契约，不要求头发网格与旧版相同

正式重建由动作负责者在双方源码冻结后唯一执行，root 负责最终 manifest 和串行 Bevy 验证。自有图片、接触表和探测 GLB 在 root 看完后清理，保留脚本、日志与 JSON；root 的 capture 产物另行处理


## 正式交付与清理

最终 master SHA256：`298da4644c90adf0551afb7178fdee0087a5b1d7fce2339693c520a43cbdac54`；最终 GLB SHA256：`5063920196cb1476e080f7d8acd6f8d16f0e55ab3cbfc94c30da9c1ad368a108`

第一次统一导出后，形体负责者在冠顶发现新增封口 UV 采到错误图集区域，补正 hair UV 并验证旧版会被新断言拒绝；`build-first.log` / `first-foot-tracks.json` / `first-contract.json` 对应被替代的版本，不作为最终视觉交付。第二次 `build-final.log` 与正式轨迹检查绑定上列最终 SHA

root 已实际看候选 b 全部 20 格周期，art_review 另看三张正面；两方同意本次改动可保留，仍记录手指简化、肩肘外张等需要继续修订。root 串行执行本轮真实 Bevy capture，其结果见 TASK-045 / visual-r7 集成记录；本页 CPU 小样不替代该运行证据

自有小样图、接触表、探测 GLB 及本次 build 备份已在查看和记录后清理，详见 `cleanup.json`；保留复现脚本、日志、状态 JSON、正式源文件和资产，未触碰 model-r6 最终图或 root GPU 产物
