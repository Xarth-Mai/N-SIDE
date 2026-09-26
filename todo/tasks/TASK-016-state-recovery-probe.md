---
id: "TASK-016"
type: "experiment"
status: "backlog"
milestone: "G1"
depends_on: ["TASK-012"]
migrated_from: ["M2-01", "M2-06", "M3-06"]
specs: ["docs/dev/design/systems/state-and-recovery.md", "docs/dev/engineering/content-contracts.md"]
---

# 提前找到玩具的事实、认知与退出恢复探针

## 目标与范围

最小真实状态路径验证设计数据与运行语义、幂等和保存，不把完整M2/M3作为笼统前置

## 验收条件

真实状态路径覆盖提前发现、重复事件、取消、退出和恢复；结构检查、运行断言与玩家认知分开

## 当前工作与下一步

本轮仅重编任务，取得对应前置结果后再按用户授权开始

## 结果与证据

历史输入：todo/archive/legacy/demo-progress.json；迁移来源见 R1 完整索引，未新增运行验收
