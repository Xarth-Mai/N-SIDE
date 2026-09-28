---
id: TASK-030
type: fix
status: done
milestone: G1
depends_on: [TASK-029]
acceptance:
  role: codex
  revision: 480428be4066843eed0f8e06f556318e1560f5b24533f216e72d640b1acda323
  record: todo/evidence/TASK-030/r1/review.md
specs:
  - docs/dev/design/systems/input.md
  - docs/dev/engineering/player-preview.md
  - docs/dev/production/ui.md
  - docs/dev/validation/runtime.md
---

# 失焦自动暂停与显式恢复

## 目标与范围

正式游戏失去主窗口焦点时进入已有暂停，保留当前世界、人物与镜头；回到窗口后由玩家明确继续。沿用现有暂停状态和释放门槛，覆盖短暂失焦回焦、加载完成时仍失焦与持续输入，不新增菜单或改变Viewer自由相机

## 验收条件

失焦当帧及之后不执行移动、重力、镜头或复位；回焦不自动进入世界，也不把仍持有的确认、返回、鼠标或手柄输入当作继续。加载可完成，但失焦时不能提前运行人物；无窗口capture仍可运行。CPU使用真实调度验证失败及修复，实际窗口能力与离屏渲染分别记录

## 当前工作与下一步

第1轮，步骤6/6 完成；失焦同帧冻结物理与镜头，回焦需释放菜单输入后明确继续。CPU53项＋Viewer3项、模拟焦点20秒／14项及既有两条录制回归通过，已实际查看画面。真实桌面完整切焦仍受合成器激活限制，下一步在活动桌面补验；本轮提交后停止

## 结果与证据

基线a49e857，工作区起始干净；[验收记录](../evidence/TASK-030/r1/review.md)保留修前FAIL、修后CPU／GPU检查、画面自查、内容hash及原生窗口尝试的范围。原始运行产物归 `output/focus/2026-09-28/r1/`。技术实现已验收，原生桌面完整验收INCOMPLETE，物理手柄、Windows及作者手感NOT RUN，G1整体不据此放行
