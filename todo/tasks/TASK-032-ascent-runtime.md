---
id: TASK-032
type: experiment
status: done
acceptance:
  role: codex
  revision: b67eece61b59bb8167a4ef40060534966cb91386929b92e8c96831adfeda31d4
  record: todo/evidence/TASK-032/r1/review.md
milestone: G1
depends_on: [TASK-031]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# 完整短登高路线的真实输入录制

## 目标与范围

从小店沿 hill-short 逐节点输入行走到摘星台，连续录制真实人物、碰撞和镜头，不改结果位置

## 验收条件

130节点到达、零恢复、资源就绪、连续帧落盘和终点稳定；错误路线及停滞明确失败；技术交付不代签作者体验与G1整体放行

## 当前工作与下一步

第1轮，步骤6/6 完成；13500帧／450秒真实录制通过8项检查，130节点全部到达，420.33秒抵达450m摘星台，零恢复；继续原路下山回店

## 结果与证据

[技术与画面自查](../evidence/TASK-032/r1/review.md)；原始连续帧与视频位于 `output/ascent/2026-09-28/r1/full/`
