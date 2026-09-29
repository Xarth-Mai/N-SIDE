---
id: TASK-044
type: experiment
status: done
milestone: G1
acceptance: {"role": "codex", "revision": "2f091939fbec0fcbb728a3902dc1cb710b53273915e76199031e08ebd346d32e", "record": "todo/evidence/TASK-044/r1/review.md"}
depends_on: [TASK-024, TASK-028, TASK-034]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/design/locations/district-architecture.md
---

# 月台杂货首层公共区真实进入与返回

## 目标与范围

从现有室外步行入口连续进入小店首层的接待与陈列、预约洽谈、主题陈列，再走回街道。复用V-04源房间、开口与标高生成可见且可碰撞的门洞、墙、地板和天花；后场与楼上保持关闭，不新增剧情、NPC或修改首委托范围

## 验收条件

从真实home出生点连续通过公共店门和三处公共房，再返回街道，无穿墙、坠落或自动复位；后场实体阻挡，跳跃和相机沿既有碰撞查询工作。房间HUD读取源多边形，不把附近提示当室内状态；按源数据检查门洞及支撑，CPU与GPU路径分别留证，实际观察室内画面

## 当前工作与下一步

第1轮，步骤6/6 已记录；三公共房真实往返、私人边界、暂停恢复、房间提示与画面穿插修复通过技术验收。独立包28秒录制11项检查通过，并已实际查看关键帧与连续帧；README预览作为长期实机目标，下一项TASK-045保持ready。作者镜头与空间体验仍需实际反馈，本项不放行G1；按作者要求提交后停下

## 结果与证据

本轮[技术验收、画面自查与限制](../evidence/TASK-044/r1/review.md)，输入SHA-256、真实检查和清理记录位于同目录；最初入口阻挡与首轮画面失败均保留，不用最终通过覆盖失败过程
