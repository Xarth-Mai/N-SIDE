---
id: TASK-029
type: feature
status: done
milestone: G1
depends_on: [TASK-028]
acceptance:
  role: codex
  revision: bbefc7a74ecabbc2f82c0cbe5cad9122ad64667835a5048535674804bdc8412e
  record: todo/evidence/TASK-029/r1/review.md
specs:
  - docs/dev/design/systems/input.md
  - docs/dev/design/systems/ui.md
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# 正式入口暂停与原会话恢复

## 目标与范围

复用正式入口的街区信号外壳，连接「探索→暂停→继续原会话→返回标题」。保留真实城市、代理、碰撞与镜头，暂停提供继续和返回标题；不启用Viewer的调查样例或新增尚无数据的菜单

## 验收条件

键鼠和模拟手柄均能暂停与继续；开菜单同帧和暂停期间，移动、转向、复位输入不执行。恢复保留位置、视角、世界实体和唯一代理；继续按钮的输入不触发其他动作。暂停返回标题正常清理，再次进入可操作。保留CPU断言、真实输入录制及实际画面自查，物理手柄与作者手感分别记录

## 当前工作与下一步

第1轮，步骤6/6 完成；正式游戏600帧／20秒／14项检查通过，暂停同帧及持有输入均不移动／转向／复位，继续保留人物与世界，返回标题后能重新进入。CPU47项＋Viewer3项、既有4条录制和双站构建通过，关键帧及连续帧已实际查看。按用户要求，本里程碑提交后停止；下一项为失焦自动暂停，尚未实施

## 结果与证据

基线fa61e20；[验收记录](../evidence/TASK-029/r1/review.md)保留实际命令、检查、输入hash与画面自查，原始连续帧和视频归 `output/pause/2026-09-28/r1/`。本任务依赖TASK-028的实际入口结果，复用TASK-014技术实验，不代签其待作者判断的镜头与登高节奏
