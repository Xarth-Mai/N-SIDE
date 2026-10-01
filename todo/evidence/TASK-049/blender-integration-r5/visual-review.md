# Blender integration R5 实际画面自查

独立 self-audit，基线 `5cd3037436e2fdd569569daa1a853f660d11d312` 加本轮工作区；实际读取原始 PNG，未重新生成画面，未把程序字段中的 `visual_review=NOT RUN` 改为自动通过；作者造型与目标美术品质验收仍为 NOT RUN

## 环境反射 A/B：本轮不保留 B

实际查看 `output/env-reflection-r1/{A,B}/keyframes/frame00049.png` 以及两组 `frames/frame00030.png`、`frame00031.png`、`frame00032.png`，后者处于脚本真实 A 横移输入内；原始脚本为 `eye-v55-west`、1280×720、30 FPS、120 帧、MSAA4

A 使用既有半球渐变环境，B 使用相同程序天空的原生环境过滤；本次同时改变漫反射与镜面环境分布，不能视为仅改镜面项的实验，来源与参数见 [实验记录](../../TASK-045/env-reflection-r1/README.md)

- 两组运行各 120 帧、6 项检查 PASS，横移实际位移均为 2.0865 m；这是资产、截图及脚本行为的机器结果
- 同帧 49 的楼上窗格和右侧玻璃门仍是整片均匀蓝灰，B 没有增加可辨的环境轮廓、玻璃表面分层或局部亮暗结构；主要差异为墙面、窗面整体稍暗，不能将其计为玻璃质感提升
- 连续 30–32 帧中窗框、窗台的几何深度随横移正常变化，A/B 均可见；未观察到 B 新增足够清楚的随视角变化的玻璃响应，也未见这三帧中产生新的跳闪
- 展示样品柜使用 Graphite／Indigo 与实物样本，本来没有 Glass，不能据柜内无倒影判断反射失败；上层窗与右侧门才是这次的玻璃观察对象
- 单次 capture 间隔 mean 从 34.29 ms 到 36.88 ms，p95 从 38.59 ms 到 42.92 ms；包含读回及 PNG 工作，仅为本次采集通量，不能宣称游戏 FPS 或纯 GPU 时间变化；本次未做重复性能统计

结论：本机位未获得足以支持保留每帧过滤的视觉收益，建议撤回 B，保留可复现的失败实验；不新增第二套反射框架，也不将整体亮度变化当作窗口质量改善

## 曜 r12 袖筒：实机回归未见新增穿洞

实际查看 `output/blender-integration-r5/yao-motion-retry/frames/` 的 Run 190–192、Jump 68–70 与落地 79；均为本轮新 GLB 的真实游戏输出，不复用已清理的 r11 画面

- `yao-motion-retry` 保存 540 帧，wrapper 28 项检查 PASS；实际状态 68–70 为 `CHR-001 / Jump`、未着地，190–192 为 `CHR-001 / Run`、疾跑，79 已为着地 Idle；因此没有将 80 帧误写为跳跃姿态
- 背面跑步的袖筒、袖口与下摆保持连贯，摆臂时未在本次抽样中看到新的皮肤白块、布面开洞或袖口突然脱离；正面跳跃的腋下、开襟、T 恤与手腕也未见新穿插
- 肘外侧有方向转折，袖筒仍偏平滑；正常全身画面中的改动较轻，灰阶大衣身、简化发面和人物识别度仍与目标参考有明显差距，本轮只支持局部修型接入与动作回归
- 第一次 `yao-motion` 因错误传入 Viewer 专用 `--aa` 在启动前退出；保留原失败记录，修正命令后的 retry 才是上述运行依据，这不属于玩法或模型失败

形体改动与冻结检查见 [sleeve r12](../../TASK-047/sleeve-r12/review.md)；本次只检查实际运行的可见回归，不将有限帧自查扩展为所有姿态无穿插或作者已批准服装

## 镜厅后场：铺地与设备关系成立，场所仍偏空

实际查看 `output/blender-integration-r5/inspect-cinema-service-court/` 的关键 49、横移连续 31–32，以及 `eye-cinema-service-aircon/` 的关键 49／104、横移连续 31–33；总览采用抬高的检查机位，空调采用人眼机位，两个 Viewer 都不替代角色通行或碰撞验证

- 总览 120 帧／6 项、空调 180 帧／8 项机器检查 PASS；本段视觉判断独立于上述结果
- 总览能辨认连续砖铺地、后门、临墙空调与立灯，横移中没有看到新增铺地面交替闪烁、灯基悬浮或与门重叠；照明设备出现不表示夜间照明已经实现
- 近景能读出空调两侧支架、下降管线与卡扣，管线没有穿过窗框；低头 104 能看见末端接到落地构件及其与砖面的接触，未看到管端悬空或穿进地面的明显缺口
- 当前后墙仍是大量同形蓝窗，砖坪与草地／道路交接较直白，细节丰富的旧空调与纯色低面数立灯的表现层级不一致；空调格栅在横移中仍有细密锯齿纹，保留为后续材质过滤与资产统一的实际观察，不据此假定本轮新建了 GPU 错误

结论仅支持后场设施与铺地接入，不将这块空场认定为完成的服务院落；上述视图没有提供室内、卸货行为、夜间或玩家路线的新证据

## 三栋建筑法线：源强度一致性复验

本次目标是把源工程已有 normal 强度烘入图像，使当前 Bevy 导入结果与源意图一致；不是盲评美术品质，也不是新增纹理方向或改变配色，源与 GLB 保持边界见 [共享法线修复](../normal-scale-shared-r1/README.md)

| 实际运行 | 实际查看 | 机器结果 | 本次可见结果与剩余问题 |
| --- | --- | --- | --- |
| `cinema-normal` | 关键 59、横移连续 61–63 | 150 帧、6 项 PASS | 镜厅大墙仍有细颗粒，未见明显黑色椒盐块或移动时整片明暗跳变，接缝和招牌保持清楚；整排玻璃仍平板，前景空坡与重复建筑继续限制场所感 |
| `va08-normal` | 关键 49、横移连续 31–33 | 120 帧、6 项 PASS | 白墙低对比纹理、窗台和遮檐投影可区分，未见本段新增黑点爆发或接缝闪烁；墙面大面积偏白，窗玻璃与相邻资产仍简化，不能把清洁墙面直接视为风格定稿 |
| `v55-normal` | 关键 49、横移连续 31–33 | 120 帧、6 项 PASS | 墙面细粒度与窗框体积共存，所看段没有明显跳闪；样品柜与门仍可读，但窗面反射不足、平面色块与周边空地问题仍在 |

未发现需要撤回本次法线源修的可见回归；结论只覆盖已看分辨率、光照和距离，单级纹理仍不能证明任意远距采样稳定，三栋建筑没有因此获得作者品质批准

## 已看代表帧与清理交接

所有下列路径相对 `output/`，hash 在看图后读取；媒体由主线完成全批核对后统一清理，日志、脚本、状态与本文件保留

| 原始帧 | SHA-256 |
| --- | --- |
| `env-reflection-r1/A/keyframes/frame00049.png` | `a3b719c281dbe8ce0e181f26bc55bf6b59436410a150793eee79ca37a56cb51e` |
| `env-reflection-r1/B/keyframes/frame00049.png` | `c6663d35a82fa63ba93be6a4293aeccf8e506a738d6be1ddc34753596ede867f` |
| `env-reflection-r1/A/frames/frame00031.png` | `380f139e691db5fdec50f99fed0e5a70832c840083a90e16f6a5663d4e4bfb56` |
| `env-reflection-r1/B/frames/frame00031.png` | `6736fbfa7c45cf03c1515eecd9ed306e931a88df5a0365ad21b348e833f984fc` |
| `blender-integration-r5/yao-motion-retry/frames/frame00069.png` | `4a0be290932eb35cc3c6fee1309070be6c70e24a4c120b0f027f2e31839e452d` |
| `blender-integration-r5/yao-motion-retry/frames/frame00191.png` | `ae1375b4a8361f365e22ee7668a2829210c0fa7f4de86e20c30350da2e5b7763` |
| `blender-integration-r5/inspect-cinema-service-court/keyframes/frame00049.png` | `a100a7bcbd4c122de2b5de7eef0801517d6d9ba5c86d3e7037a8b31928398621` |
| `blender-integration-r5/eye-cinema-service-aircon/keyframes/frame00104.png` | `8a3856002b4de9307178d9b34086f5ce3cbf7002eb69cd5b090130421973d3f3` |
| `blender-integration-r5/cinema-normal/keyframes/frame00059.png` | `1e744908732c97b9cd5ad22d96d5c1a5ca254aab6fd98fdad027e79d670c69ae` |
| `blender-integration-r5/va08-normal/keyframes/frame00049.png` | `06c38e97e4384d933bb9a01c2781f1c9889d58e8d44385efd959725785154ba9` |
| `blender-integration-r5/v55-normal/keyframes/frame00049.png` | `8026c217d4cffbcf8475e5c509722146a5580da92f2b038b1f46178565198f8b` |
