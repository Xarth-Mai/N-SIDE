---
id: TASK-031
type: fix
status: done
milestone: G1
depends_on: [TASK-030]
acceptance:
  role: codex
  revision: 38b6d2a426d56234962728225c3178eec43da0cf1c763b3da6b26e003a6f6c57
  record: todo/evidence/TASK-031/r1/review.md
specs:
  - docs/dev/design/systems/input.md
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# 手柄断连暂停与键鼠接管

## 目标与范围

正式入口在手柄断开时保护当前行程，复用暂停、原会话恢复及输入释放门槛。当前手柄按聚合输入处理，任一设备断开即暂停，不引入设备所有权、重映射或多人系统

## 验收条件

断连当帧及暂停期间停止移动、重力、镜头和复位，保留原会话；键鼠可以明确继续，重连及持有输入不自动继续。同帧断连重连、加载期间断连、标题及重新进入分别验证。原生Bevy连接消息经InputPlugin处理，CPU与实际渲染分别记录，物理设备和Windows不能用模拟连接代签

## 当前工作与下一步

第1轮，步骤6/6 完成；57项库测试＋3项Viewer测试通过，18秒模拟断连录制13项及失焦／暂停回归28项通过，已实际查看关键帧与连续帧。断连暂停、重连持键保护、键鼠接管及重入成立；物理手柄、Windows与作者手感未测。本批提交后停止，下一项为完整短登高路线连续画面

## 结果与证据

基线1926c31，工作区起始干净；[验收记录](../evidence/TASK-031/r1/review.md)保存修前失败、实际命令、源码hash、真实渲染及画面自查，连续帧及视频归 `output/gamepad/2026-09-28/r1/`。TASK-030的真实桌面焦点限制保留，未以模拟消息代签硬件及G1验收
