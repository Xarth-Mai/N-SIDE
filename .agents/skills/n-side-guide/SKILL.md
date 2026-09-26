---
name: n-side-guide
description: Guide N:SIDE progress from task cards when asked about current work, blockers or next steps. Keep status queries read-only; preserve an explicitly requested topic.
---

# N:SIDE 任务导航

从子目录开始时，用 `git rev-parse --show-toplevel` 定位仓库根；下列路径和命令相对根目录

读取 AGENTS.md、todo/README.md、todo/roadmap.md、相关 todo/tasks/ 卡片及最近证据，按 [任务规则](../../../docs/dev/handbook/tasks.md)恢复实际状态。运行 `bun run tasks:list`；工具不可用时直接读取卡片并说明未执行检查，不从旧看板或会话印象补数

选择顺序为用户指定主题、已开始且可继续的任务、依赖结果已具备的 ready。指定旧城市、Viewer或视觉主题时先查 todo/archive/legacy 与历史证据，在新任务中续接实际新增工作，不重开结项输入或视为游戏能力完成

简述当前任务和里程碑、真实状态、这一轮动作、下一步及需要作者判断的事项。交付链接使用实际读取的卡片路径，不按标题猜测文件名。任务数不表示工时或游戏完成率；资产批次只在现有清单中按已验收对象记录，不重新引入61项分母

只问进度保持只读；授权开始或继续工作时转 n-side-work-loop，验收或里程碑放行时转 n-side-review。已有决定直接沿用，只解释当前需要的一两个概念；遇到真实设计或体验取舍，说明影响、推荐与所需判断
