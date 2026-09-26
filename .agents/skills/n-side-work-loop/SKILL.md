---
name: n-side-work-loop
description: Execute or resume an authorized N:SIDE task through design, production, checks and evidence. Use for implementation, experiments, assets or document work; not for read-only status or automatic milestone approval.
---

# N:SIDE 任务执行

从子目录开始时，用 `git rev-parse --show-toplevel` 定位仓库根；读取指定任务卡、关联规格、前置结果及最近证据，按 [Codex 工作流](../../../docs/dev/handbook/codex.md)继续，不重新访谈已有决定

依据任务类型明确输入与验收，然后设 active 并制作。原型比较少量假设，生产按既定清单分批，修复保留复现和回归；简单文档小修压缩步骤，不强制六轮或作者逐项审批。实际步骤、批次、轮次和下一动作写在卡片正文

专业工作读取 nside 适配入口后选对应 Skill。通用方法归 Skill，项目长期事实与契约归 docs，本轮实际输入、命令、结果和反馈归 todo/evidence/TASK-xxx/run-id/，沿用源资产与数据唯一源

检查深度按 [运行验证](../../../docs/dev/validation/runtime.md)选择：影响真实行为或表现时获取状态与画面，观察问题并修复复验；纯文档使用语义、引用与构建检查。PASS、FAIL、NOT RUN分别记录；实验否定假设也可以形成合格交付

需要作者试玩时给启动方式、短路线与具体观察问题，将卡片设 review 并写明待判断内容。技术任务依据实际检查可以完成；主观品质、陌生玩家理解和里程碑放行只引用真实反馈。范围内普通实现自主推进，真实阻碍写明所缺输入

依据 n-side-review 核对交付后更新任务，长期决定回写权威规格，再运行 `bun run tasks:sync`、`bun run tasks:check` 和适用检查。收尾报告交付、证据、限制与一个下一动作；一次反馈可供多个任务引用，不重复询问
