---
name: n-side-work-loop
description: Execute or resume a N:SIDE work package through explain, decide, build, verify, playtest and record. Use for authorized implementation, prototypes or revisions after feedback; not for read-only progress questions or automatic stage approval.
---

# N:SIDE 工作循环

下列路径和命令以仓库根目录为基准；从子目录开始时，先用 `git rev-parse --show-toplevel` 定位根目录再执行

先读取 `docs/production/solo-workflow.md`、`todo/demo-progress.json`、指定工作包的现有设计与记录。确认前置通过、范围和授权；用户明确指定旧主题时读取归档与总路线图承接表，在新工作日志中续接，不重开已结项记录或强行映射为主线完成

采用六步：`orient` 定位与必要讲解 → `design` 决策与设计 → `build` 制作 → `verify` 客观检查 → `review` 实际体验／作者验收 → `record` 记录与下一步。从上次记录恢复，不每次从头访谈。详细拆分当前与下一项，远期保持工作包粒度

开始时解释本轮问题与通过标准，读取已有决定。方案需要作者选择时给推荐和代价，等待真实选择；普通实现自主完成。每轮只验证少量假设，用同条件对照避免无法归因的同时大改

将当前工作包设为 `in_progress`，维护 `work.round`、`step`、`record`、`next_action`。轮次使用明确记录锚点，例如 `todo/demo-worklog.md#m1-03-r02`；持续记录输入提交／校验和、实际结果、未测项和反馈。简单客观交付可省略空 work，保留真实证据即可

专业方法先读 `nside` 项目适配入口，再按需复用原版 Skill 与既有 `create-case`、`create-content`、`review-narrative`。代码、数据和源资产按既有单一来源维护。检查与真实运行采用 `docs/production/runtime-validation.md`，获取画面及状态证据、实际观察、修复后复验；无法执行的图形、Windows、手柄、Codex 或人工测试明确记 NOT RUN

交给作者试玩时提供确切启动方法、短路线和最多三个观察问题。需要作者或玩家体验判断的项，在缺少真实反馈时停在 `review`，不能用截图、自动测试或模拟“作者满意”代替操作和品质判断；纯技术交付按 `reviewer: codex` 的实际检查结果验收。客观检查失败时在授权范围内依据新证据继续修复；重复失败且无法取得新证据时，报告具体阻碍和所需输入

根据反馈保留已成立部分，修改相关假设，必要时开启下一轮；不按轮数增加完成数量。验收读取 `n-side-review`。收尾更新长期设计、证据和 JSON，再运行 `bun run roadmap:sync`、`bun run check:roadmap` 与相关项目检查。报告真实进度变化及下次能直接续接的动作

制作型工作包同时读取所链接的批次清单，显示已验收对象 x/y；轮次与批次分开。一个实际作者反馈可以覆盖多个工作包，以同一证据分别记录，避免重复询问
