# 角色运行时与动画断言独立复核

2026-09-30，独立子 Agent 只读比较工作区与 HEAD，范围为角色、玩家、应用生命周期、地点观察、门前交接状态、capture 实现及相关脚本；只新增本记录，未评价其他 Agent 的未提交叙事正文

## 发现与修复

- 原动画断言只要求每帧 `clip_elapsed >= 0.1`，区间始终停在 0.2s 仍可通过；已改为检查实际区间推进量、同一 clip 与 transition 计数，暂停按全部帧检查最大漂移
- Bevy 0.19.1 `ActiveAnimation::update` 的 elapsed 按 delta 累加，seek_time 才乘播放速度，因此 `speed=0` 时 elapsed 仍会推进；已补正推进区间内实际 clip_time 必须变化，暂停同时约束 elapsed 与 clip_time
- 已读负测覆盖冻结、仅 elapsed 增长而 seek_time 不变、中段 elapsed／seek 漂移与片段重启；新增检查复用原 Sample fixture，无新增依赖或验收专用玩法状态
- 加载失败保留胶囊、标题清理、暂停输入门禁、观察信息与显式选择分离、幂等确认未发现确定的运行时错误；门前三点完整路线仍需独立运行证据

## 已核对证据

本评审读取主 Agent 产生的日志与原始状态，未自行运行 Cargo 或 GPU

- [动画断言窄测](animation-assertions.log)：1 passed；[完整库测试](all-tests.log)：109 passed、1 ignored，后者是最终断言补充之前的版本，不能替代最终版本全量测试
- [r3 正向运行](../character-r3/run.json) 与 [原始状态](../character-r3/state.json)：540 帧、26 项检查全部 PASS，原生退出码 0；复制脚本与当前 `game/capture/walk-character.json` 语义一致
- 独立从 r3 样本重算：Idle 推进约 0.300s、Walk 约 0.500s、两段 Run 各约 0.500s、恢复后 Idle 约 1.633s，全部具有实际 seek_time 变化且各区间 clip／transition 保持一致；暂停段 elapsed 与 seek_time 最大漂移均为 0
- [负向运行](../character-negative/run.json) 与 [原始状态](../character-negative/state.json)：90 帧，3 项基础检查 PASS，要求 10s 推进的断言按预期 FAIL，实际仅约 0.633s，原生退出码 1；复制脚本与当前 `game/capture/character-failure.json` 语义一致
- 正反运行二进制 SHA-256 相同：`283501dfe5d7a4c40b0769e97e89f0b4091631227b3dda48b0b2e7b652e626fb`
- 本评审执行受审代码与脚本的 `git diff --check`：PASS

## 结论与边界

原断言假通过缺口已处理，本次复核没有剩余确定问题；Ponytail-review：Lean already. Ship.

本评审未观看画面、未测原生窗口／实体手柄、未做作者或陌生玩家验收，动画时间线通过不代表造型、足滑、动作衔接或体验已验收；角色资产缺失／30s 超时故障未由本评审运行注入，完整三点交接与最终选择也不由上述角色路线证明

本子任务未生成截图、原始帧、视频或临时探测可执行文件，无新增视觉产物需清理；其他执行者的视觉产物由其在实际查看后按项目规则清理
