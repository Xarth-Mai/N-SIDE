---
id: TASK-037
type: feature
status: done
acceptance:
  role: codex
  revision: bfe826e7bb936280f3690d0a0fe45ba1015fd99ca36148f73e75acce4c138264
  record: todo/evidence/TASK-037/r1/review.md
milestone: G1
depends_on: [TASK-036]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# 正式入口设置跨启动保存

## 目标与范围

保存现有字号与镜头偏好，独立用户目录与capture隔离，坏档和写失败保留原文件

## 验收条件

独立进程读取前次选择；失败不伪报保存、不破坏已有设置；真实UI与镜头路径复验

## 当前工作与下一步

第1轮，步骤6/6 完成；跨进程恢复、坏档与写失败GPU验证通过，原文件精确比较保留，设置窄测5项、全测77项库测试与3项Viewer测试通过；继续本轮其余交付，不据此放行G1

## 结果与证据

[设置跨启动保存验收](../evidence/TASK-037/r1/review.md)保留4条GPU的脚本、命令、断言摘要、画面自查、原文件比较与源码快照；Windows实机、物理手柄、断电恢复及作者体验另验
