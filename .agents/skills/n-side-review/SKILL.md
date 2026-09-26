---
name: n-side-review
description: Review N:SIDE task evidence or milestone readiness. Distinguish technical checks from real author or player feedback, and reopen results invalidated by changes; keep a read-only review read-only.
---

# N:SIDE 交付与里程碑验收

从子目录开始时，用 `git rev-parse --show-toplevel` 定位仓库根；读取目标任务卡的验收条件、specs、依赖和实际产物，核对证据输入的完整提交或内容 hash，以及之后变更是否使结论失效

按 [任务规则](../../../docs/dev/handbook/tasks.md)分别核对决定来源、交付接入、适用检查、需要的作者或真实玩家反馈。机器通过、画面自查、隔离评审和实际试玩分别标注；代码评审不能证明手感，截图不能证明连续操作，构建不能证明双平台，设计接受不能证明实现完成

依据 [运行验证](../../../docs/dev/validation/runtime.md)给出通过、需要修订或缺少证据，附最小补充动作。技术任务可由 Codex 依据实际结果验收；主观体验和里程碑放行必须有对应真实反馈。只请求评审时先交结论，保持状态只读

获授权更新且验收条件满足时写 done 和 acceptance.role/revision/record；record须指向本任务 evidence 中的真实记录，字段存在只证明可追溯。未满足时保持 active 或需要反馈的 review；确实无法继续时 blocked 并说明原因

里程碑按 todo/roadmap.md 的成果门槛检查完整体验、覆盖与范围，不把卡片数量视为完成百分比。取消依[任务规则](../../../docs/dev/handbook/tasks.md)记录原因和依赖替代，不满足其他任务的前置结果

新变更推翻旧结果时重开原卡，移除当前 acceptance，将旧证据保留在正文，复查受影响依赖。更新后运行 `bun run tasks:sync`、`bun run tasks:check` 和适用回归；报告通过依据、未测范围与下一动作
