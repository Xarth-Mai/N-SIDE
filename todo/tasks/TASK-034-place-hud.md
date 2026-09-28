---
id: TASK-034
type: feature
status: done
acceptance:
  role: codex
  revision: 3f65baaefaf439bab1d882594c59331f08d6f90b61d6bb82e884bba16dd894fe
  record: todo/evidence/TASK-034/r2/review.md
milestone: G1
depends_on: [TASK-028]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# 正式入口的真实地点提示

## 目标与范围

从地图公共到达点计算邻近地点，展示名称与当前设备提示

## 验收条件

位置与高度约束有效、未知区回退、暂停隐藏或冻结、两种输入与窄画幅检查；技术交付不代签作者体验与G1整体放行

## 当前工作与下一步

第2轮，步骤6/6 完成；实际地点、高度与设备提示接入正式入口，宽窄屏各720帧、11项检查通过；继续就近查看与字号设置

## 结果与证据

[技术与画面自查](../evidence/TASK-034/r2/review.md)；首轮记录保留在r1，最终代码复验在r2，窄屏字号放大由TASK-036接续
