---
name: n-side-guide
description: Guide N:SIDE project progress when the author asks where we are, what comes next, what to learn now, or to continue the project. Read the actual roadmap and evidence; do not use for unrelated questions or replace a specifically requested task.
---

# N:SIDE 阶段导航

下列路径和命令以仓库根目录为基准；从子目录开始时，先用 `git rev-parse --show-toplevel` 定位根目录再执行

读取根目录 `AGENTS.md`、`todo/README.md`、`todo/demo-progress.json`、相关阶段的 `todo/demo-roadmap.md`、`docs/production/solo-workflow.md`，以及当前工作包或原任务的最近记录。核对当前分支、提交与工作区，计划与实现不一致时先指出差异

在仓库根运行 `bun run roadmap`，需要细项时运行 `bun run roadmap --stage M0` 等对应命令。工具不可用时直接读取 JSON，按同一计数规则计算；说明未执行的检查。不得用旧看板或会话印象推断状态

用户明确指定任务时优先该任务；否则先续接已开始且不受阻的 work，再取依赖满足的关注项。关注项是建议位置，不代表已开工。城市、Viewer、白天视觉和收口成果已归档，先读 `todo/archive/` 与总路线图承接表，再在新工作记录中续接指定主题；不重开已结项记录或冒充游戏阶段完成

开头用简短状态块说明：阶段、已验收 x/y、当前工作包、清单位置、当前轮次／步骤、这次要解决的问题。只讲当前需要的一个或两个概念，连到 N:SIDE 实例

先查已定设计。真正需要作者决定时，说明体验与成本影响，给出推荐及最多两三个选项，只问当前阻塞问题。实现细节在已授权边界内自主推进，不要求作者学习完课程或批准每个函数

仅问进度时保持只读。获授权开始／继续开发时读取 `n-side-work-loop`；要求验收或阶段放行时读取 `n-side-review`。调用专业叙事流程时复用已有 skills

结束时报告本次实际变化、证据、未测项目与一个下一动作。没有验收证据就不增加 done，也不因为生成了路线文档就完成 M0

制作型工作包同时读取所链接的批次清单，显示已验收对象 x/y；轮次与批次分开。一个实际作者反馈可以覆盖多个工作包，以同一证据分别记录，避免重复询问
