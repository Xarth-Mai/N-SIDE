---
id: "TASK-014"
type: "experiment"
status: "review"
milestone: "G1"
depends_on: ["TASK-028"]
migrated_from: ["M1-01", "M1-02", "M1-03"]
specs: ["docs/dev/design/systems/input.md", "docs/dev/engineering/player-preview.md", "docs/dev/validation/runtime.md"]
---

# 正式入口的人尺度移动与镜头实验

## 目标与范围

同一真实路线验证输入→移动/碰撞→镜头假设，保留可重复比较，不等待全部M0或重造地图

## 验收条件

用同一路线比较真实移动、坡阶、墙角和镜头；记录假设、状态、画面与操作反馈，否定某方案也可完成实验

## 当前工作与下一步

第2轮，步骤5/6 体验验收；坡面停滞与桥栏杆接入口已修复。64Hz／30Hz均从home连续通过hill-short全部130节点到450m摘星台，零自动恢复；35秒真实输入录制已通过并查看。等待作者实际操作判断镜头、台阶和约7分钟上山节奏；TASK-032已补齐完整登高GPU画面，室内与G1保持未验收，独立的暂停恢复功能继续推进

依赖由TASK-013调整为实际需要的TASK-028运行入口：本轮代理与参数仅供实验，正式主控及可达内容范围仍由TASK-013决定；不代签其设计验收，也不把室外短段通过扩大为全城可通行

## 结果与证据

第2轮见[坡面、桥口与完整登高探针](../evidence/TASK-014/r2/review.md)，保留首次失败、逐步修复、两种时间步和真实画面，原始录制位于 `output/player/2026-09-28/r2/`

历史输入：todo/archive/legacy/demo-progress.json；迁移来源见R1完整索引。本轮[技术检查与画面自查](../evidence/TASK-014/r1/review.md)进入 `todo/evidence/TASK-014/r1/`，实际GPU记录与临时探针放在 `output/player/2026-09-28/r1/`
