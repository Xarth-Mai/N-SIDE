---
name: n-side-review
description: Review N:SIDE work-package completion, playtest evidence or milestone readiness. Use when asked to accept work or enter the next stage; distinguish automated checks from actual author or player acceptance and reopen invalidated results.
---

# N:SIDE 交付与阶段验收

下列路径和命令以仓库根目录为基准；从子目录开始时，先用 `git rev-parse --show-toplevel` 定位根目录再执行

读取 `docs/production/solo-workflow.md`、`todo/demo-progress.json` 中目标项的 `verify`、`reviewer` 与依赖，以及真实交付和证据。核对证据针对的提交／配置／构建，检查之后的变更是否使结论失效

分别检查：决策是否有来源；交付是否存在且接入；适用的客观测试是否实际运行；需要的作者操作、品质判断或陌生玩家测试是否确有记录。PASS、FAIL、NOT RUN 分开。代码评审不是手感验收，固定截图不是连续移动验收，构建成功不是双平台实测，批准规划不是完成游戏阶段

给出“通过”“需要修订”或“缺少证据”的结论，写明最小补充动作。作者验收角色由作者实际反馈提供；试玩角色由真实操作者记录提供。技术项可以基于实际执行结果由 Codex 记录通过。字段和文件存在只能用于结构检查，不是验证真实性的替代品

全部条件成立才写 `status: done`、证据路径与 `accepted` 的 `role`、`by`、`revision`、`record`、`result: pass`。保留最后 work 时将步骤设为 `record`。未通过保持 `in_progress` 或带原因的 `blocked`；历史结果留在日志

阶段验收项检查本阶段全部未取消工作包、获批取消的范围影响及整体体验，并取得作者明确放行。已批准范围取消单列 `cancelled` 与作者 `scope_change`，不计入 done，不取消阶段出口。范围增减或拆分增加计划修订并保留 ID 与批准来源

变更推翻旧结论时重开受影响项，移除有效 `accepted`，核查依赖项和阶段出口是否需要重开；保留旧证据，不悄悄沿用。执行 `bun run roadmap:sync`、`bun run check:roadmap` 和适用回归

报告已验收 x/y 的真实变化、通过依据、未测范围与一个下一动作。用户只要求评审而未要求更新状态时，先输出评审结果，保持数据只读
