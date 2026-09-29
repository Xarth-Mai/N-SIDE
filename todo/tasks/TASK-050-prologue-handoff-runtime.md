---
id: TASK-050
type: feature
status: done
milestone: G1
depends_on: [TASK-035]
specs:
  - docs/player/story/main/prologue.md
  - docs/dev/design/quests/campaign-sequences.md
  - docs/dev/design/systems/state-and-recovery.md
  - docs/dev/engineering/player-preview.md
acceptance:
  role: codex
  revision: e92802ffd39daf5b7c990306ab23eeb1a78de821
  record: todo/evidence/TASK-050/route-r1/runtime-review.md
---

# 序章门前交接路线的真实操作片段

## 目标与范围

接入序章第 3 分场已有的步行与配送路线区别，在当前街区行走入口中通过实际接近、观察与明确选择完成一次路线确认。沿用月台杂货、货运侧院、店侧台阶的稳定场所 ID 与地图位置，复用现有输入、观察窗、HUD 与暂停流程

设计输入固定为已提交的 `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3` 序章与分场。作者已明确允许接手实现；TASK-046 后续小调重开不撤销这份已交付基线，因此本项不笼统依赖整项再次完成，后续相关提交逐项核对

本轮实现单次运行内的观察记录、下一目标、错误选择提示、修订与一次性确认，不把这段路线核对当成搬货、NPC 表演、整个序章或《最后的玩具》已完成；未制作新室内、战斗或磁盘任务存档

## 验收条件

- 只有实际距离、朝向及视线满足既有观察条件，并由玩家打开观察窗后才获得对应记录，空跑经过或菜单停留不自动完成
- 三处信息可乱序取得并在再次观察时回看，集齐只进入待确认，仍需玩家作出配送路线选择
- 台阶选项说明推车限制并允许修订，正确选择幂等提交，重复打开、重复确认不产生第二次成果
- 观察与选择使用键鼠和手柄，开窗输入不连带确认；暂停、关闭及重访保留当前运行的记录，返回标题明确重开语义
- 适用窄测与真实运行证据分别记录，capture 只注入输入并读取实际状态，不写任务结果

## 当前工作与下一步

第 1 轮，步骤 6/6：会话内三点观察、显式选择、修订、HUD与正文滚动技术验收完成。连续生产碰撞测试在30／64Hz通过；约89秒真实入口录制2672帧、15项检查通过并实际看图，整条路线无传送、重置或跳跃。外部叙事新提交 `1bfc1cf` 未改变第三分场职责

下一步接入这个片段的独立进度保存与明确继续／重开；当前任务只完成既定会话内操作，不把路线确认当作实物交付或完整序章。作者体验、完整叙事演出及里程碑放行仍未取得

## 结果与证据

[实现与窄测](../evidence/TASK-050/r1/implementation.md)、[店前与大字运行](../evidence/TASK-050/r1/runtime-review.md)、[完整三点路线验收](../evidence/TASK-050/route-r1/runtime-review.md)及[独立评审](../evidence/TASK-050/route-r1/independent-review.md)。保留首次输入时长不足的真实失败与沙箱GPU不可用记录，最终复验未放宽观察条件；技术验收不替作者、陌生玩家或完整游戏签署通过
