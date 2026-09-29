---
id: TASK-046
type: document
status: done
milestone: DOCS-PIPELINE
depends_on: []
specs:
  - docs/player/story/index.md
  - docs/dev/design/story-continuity.md
  - docs/dev/design/story-graph.md
  - docs/dev/design/quests/scenes.md
  - docs/dev/design/quests/campaign-sequences.md
  - docs/dev/design/quests/pacing.md
  - docs/dev/design/quests/world-stories.md
acceptance:
  role: codex
  revision: 2aacc98740f5c4dd2f7f0c8634fc6a887e040c85d19d3d8152d21d69b51ac86c
  record: todo/evidence/TASK-046/final/result.md
---

# 剧本、角色关系与世界任务闭环

## 目标与范围

根据本轮作者要求，只扩充文档与文档使用的故事目录数据，在既有十章主线和十二条街坊故事上补具体场景、跨地点伏笔、角色接力和五条世界任务。第 2 轮根据作者最新要求，以原神 1.0 首发任务分层安排逐项长度，替代此前 40–60 小时硬目标。保留世界规则、人物身份、旧案处置和 Demo 范围，不改游戏代码、地图或资产；完成后提交本任务范围

## 验收条件

二十七条故事都有请求或目标、主动行动、转折、结束及重访后果；三十条伏笔链能追到正文中的铺垫、补证与回收，标明实物、机制和主题呼应的区别；主线每章有制作分场与关键场景对白；逐项时长有参考出处和未实测说明；可选内容不成为主线唯一钥匙，未做支线时尾声不冒称经历；人物、场所、时间轴及故事目录一致，适用静态检查与双站构建通过

## 当前工作与下一步

第 2 轮文档交付完成：十章主线、十二条街坊故事、五条世界任务、81 个主线分场单元、十一段关键场景及三十条伏笔连接已通过静态语义、引用、故事关系和双站构建检查。逐项预算为主线 11–15.5 小时、全部二十七条主动任务 17.1–25.3 小时，均非实测；下一阶段按规格制作代表性章节并计时，实际玩法、表演、真实玩家理解和作者最终叙事锁定另行验收

## 结果与证据

本轮以当前分支与工作区为输入，保留并行的美术、城市、人物管线、运行验证、工程和其他任务变更；结果、内容哈希与检查日志记录到 `todo/evidence/TASK-046/final/`。新增内容属于文档交付，不增加当前游戏可玩能力或代签作者体验
