# 内容与验收方法适配

在制作叙事、关卡、表现或准备试玩时按需使用，成果归入对应设计和任务记录

## 可验证承诺与玩家信息

把本次设计承诺写成能检查的状态规则：触发条件、玩家输入、状态变化、可见反馈、失败/恢复。已存在的系统用真实操作驱动；可计算结果进入断言，构图、材质、遮挡与动作通过连续画面检查

操作目标、交互可用性和危险反馈需要可理解；世界真相、异常机制和剧情谜底按信息顺序揭示，不因上游街机规则要求持续读数而提前公开。保留可重访线索与事实/推测的区别

表现调整按事件服务行动：确认交互、提示危险、表达工具生效。表现层的挤压、震动与位移不能改变碰撞权威状态，效果结束后恢复，不为“手感”改变已定战斗规则。视觉参考和当前美术文件是依据，不另建 `VISUAL_DESIGN.md` 或 `FEEL_TUNING.md`

## 试玩与自查

记录构建、路线、输入、实际行为、原话、主持人提供的帮助和中断。区分目标/线索/空间问题、操作与难度问题以及玩家建议背后的需求；后续方案应针对观察到的原因。人工反馈与日志互相核对，不能把玩家解释一概视作无效

只有评审者未读设计和源码、输入证据满足所用隔离协议时，才报告盲评结果。作者、实现者或读过上下文的 Codex 看图属于 `self-audit`。真实玩家试玩也要单独报告人数、经验、协助与环境，不将代理评审计作玩家反馈

重复操作、等待、提前发现、退出、重试、读档等按现有玩法选择测试。没有计分循环的调查不计算街机分数比；简单策略偶尔成功不等于支配所有选择。规则漏洞审查用 `stress-testing-game-concepts`，完整阶段验收继续由 `n-side-review` 处理

## 叙事与关卡

使用既有 `create-case`、`create-content`、`review-narrative` 和 [叙事流程](../../../../docs/dev/production/narrative.md)，把台词、环境和选择连接到玩家能观察到的状态或关系。人物语言与知情范围按正式档案核对，生活场景允许安静停留，选择后果的显现时机服从委托因果，不套用“两场内全部可见”或未经校准的通过率

关卡主入口为 `level-design`；poorvith 的布局方法作为补充，不同时启用两个同名 Skill。技术美术和音频方法归现有[美术](../../../../docs/dev/production/art-direction.md)、[声音](../../../../docs/dev/production/sound.md)与[资产管理](../../../../docs/dev/production/asset-pipeline.md)，LOD 数量、移动端变体、音频中间件以实际范围和性能证据决定

## 事件、空间与恢复

空间的源对象 ID 连接外观、碰撞、交互与导航，层次之间共享锚点，不从装饰推断可通行性。保留既有坡地、台阶与桥梁；上游 Three.js 单平面限制不适用本项目

遭遇按目标、空间、敌人作用、预警、退出与恢复窗口组织，每次增加一种压力后观察玩家怎样应对。特效与声音记录触发事件、所属对象、时长、优先级、并发预算和清理时机；暂停、重试、卸载与重复触发不得留下孤立效果或重复奖励。只有实际制作相关内容时才落实对应检查，不预建战斗或音频框架

## Provenance

基于以下项目摘编，原文在 `third_party/skills/` 保留用于比较更新；本文件由 N:SIDE 改编，将方法限定到调查、生活与潜梦行动，并归入已有数据与验收入口

- [abagames/agentic-gamedev-skills](https://github.com/abagames/agentic-gamedev-skills/tree/24a4cdce3b629f123162c0bdcf61647eeb85f8db)：`.agents/skills/implementing-gameplay-invariants/`、`directing-game-visuals/`、`maximizing-game-feel/`、`evaluating-gameplay-balance/`、`gating-intent-legibility/`，Copyright (c) 2026 abagames；[完整 MIT 许可](../../../../third_party/skills/agentic-gamedev-skills/LICENSE)
- [poorvith-mp/skills-gamedev](https://github.com/poorvith-mp/skills-gamedev/tree/e8b87e9086fbf2322b1c216c2d2de85954bf4015)：`skills/narrative-design/`、`playtesting/`、`level-design/`、`tech-art/`、`game-audio/`，Copyright (c) 2026 Poorvith M P；[完整 MIT 许可](../../../../third_party/skills/skills-gamedev/LICENSE)
- [MengTo/Skills](https://github.com/MengTo/Skills/tree/a965851e27dc179e693fde1bee94457a64e1a7a5)：`agent-skills/game-development/author-game-levels/`、`design-game-encounters/`、`create-game-vfx/`、`build-game-audio-feedback/`，Copyright (c) 2026 Meng To；[完整 MIT 许可](../../../../third_party/skills/mengto-skills/LICENSE)。提炼空间分层、事件表达与恢复检查，未采用单平面、浏览器和移动端强制规则
