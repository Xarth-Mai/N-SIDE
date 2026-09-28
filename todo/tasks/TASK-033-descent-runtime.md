---
id: TASK-033
type: experiment
status: done
milestone: G1
depends_on: [TASK-032]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
acceptance:
  role: codex
  revision: 7b892b0445dac49218f169bf65f71c9e1f83ab18e2e6d3e5146ebabf0ab51787
  record: todo/evidence/TASK-033/r1/review.md
---

# 摘星台原路下山回店

## 目标与范围

同一会话完成上山和原路下山，检查下阶、转角及回到小店

## 验收条件

连续往返真实输入、终点回店、无恢复、状态和画面一致，保留失败与修复证据；技术交付不代签作者体验与G1整体放行

## 当前工作与下一步

第1轮，步骤6/6 记录与下一步；本轮连续五项交付的第2项完成，接续TASK-034地点HUD；大块地形折面保留到TASK-023精修，作者手感与G1整体未放行

## 结果与证据

完整GPU 27000帧、259节点、零恢复，第12602帧到峰顶、第24035帧回店；实际下山暴露的挡墙蓝洞已修补，最终63项库测试、Viewer全部目标Clippy与90帧同位置GPU均通过，最后挡墙修补后完整往返依据CPU测试，没有再次完整GPU往返录制

见[运行验收与范围](../evidence/TASK-033/r1/review.md)、[逐节点摘要](../evidence/TASK-033/r1/state-summary.json)、[原始输入来源](../evidence/TASK-033/r1/provenance.json)及[最终修订来源](../evidence/TASK-033/r1/final-source.json)；保留原CPU接地失败、修补后窄测与挡墙碰撞失败／最终通过证据
