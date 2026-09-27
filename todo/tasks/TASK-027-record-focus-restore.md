---
id: TASK-027
type: fix
status: done
milestone: G0
depends_on: [TASK-026]
acceptance:
  role: codex
  revision: 8be25a7c77387856d24a8af4740dbcca56bd44de7f87e15ef28b1006466fc3d3
  record: todo/evidence/TASK-027/r1/review.md
specs:
  - docs/dev/design/systems/ui.md
  - docs/dev/validation/runtime.md
---

# 调查工作台重入焦点恢复

## 目标与范围

修正现有街区信号样板中再次进入记录页时，内容保留第二条而焦点跳到第一条的问题。保留既有比例、记录样例和输入路径，复用当前选中状态

## 验收条件

选第二条、返回菜单、重新打开、再次确认后，焦点与当前内容保持一致；第一条长文、滚动恢复、输入切换和返回隔离原有检查仍通过。保存实际失败与修复测试、真实输入录制和实际查看的重入关键帧；样例验证不计作正式委托功能

## 当前工作与下一步

第1轮，步骤6/6 完成；窄测RED→GREEN，真实28秒／840帧录制的25项检查通过，重入后的第二条焦点与当前内容一致。保持既有比例，未代签物理手柄、Windows或作者手感

## 结果与证据

[修复与真实验收](../evidence/TASK-027/r1/review.md)保留复现、录制、状态、关键帧及输入hash；原始连续帧和视频位于 `output/ui-signal/2026-09-27/r3-focus/main/`
