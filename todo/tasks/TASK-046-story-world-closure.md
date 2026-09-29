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
  revision: dab934f781f4901cbd08da81a5069ae441c65fd8b1f14d57aed83cbafa41e381
  record: todo/evidence/TASK-046/reader-review/result.md
---

# 剧本、角色关系与世界任务闭环

## 目标与范围

只扩充与修订故事文档及文档使用的故事目录数据，覆盖十章主线、十二条街坊故事和五条世界任务。第 2 轮按原神 1.0 首发任务分层安排逐项长度，替代此前 40–60 小时硬目标；第 3 轮按作者要求分工严格审读，允许必要重构，修复与优秀影视叙事相比的结构差距。保留世界规则、人物身份、旧案处置、首发任务体量和 Demo 范围，不改游戏代码、地图或资产；完成后提交本任务范围

## 验收条件

二十七条故事都有请求或目标、主动行动、转折、结束及重访后果；三十条伏笔链能追到正文中的铺垫、补证与回收，标明实物、机制和主题呼应的区别；主线每章有制作分场与关键场景对白；逐项时长有参考出处和未实测说明；可选内容不成为主线唯一钥匙，未做支线时尾声不冒称经历；人物、场所、时间轴及故事目录一致，适用静态检查与双站构建通过。第 3 轮另要求全 27 篇经过分组与交叉复读，人物相互阻碍并承担后果、高潮产生未预演的新取舍、支线保留不同立场，修复已发现的分支矛盾并同步制作分场

## 当前工作与下一步

第 3 轮文稿修订与验收完成：27 篇全量分组审读和交叉复读，修复五项结构差距、具体分支矛盾及分场时序；重构兄妹跨章预约冲突、同一转运时段中的群像取舍与终章拆檐代价。保留 30 条伏笔链、81 个分场、11 段关键场景与原任务预算；当前没有待确认的文档取舍。后续场景接入、表演、真实读者与玩家时长验证属于制作与体验工作，本任务不代签通过

## 结果与证据

本轮以当前分支与工作区为输入，保留并行的美术、城市、人物管线、运行验证、工程和其他任务变更；第 1、2 轮结果、内容哈希与检查日志保存在 `todo/evidence/TASK-046/final/`，第 3 轮记录在 `todo/evidence/TASK-046/reader-review/`。新增内容属于文档交付，不增加当前游戏可玩能力或代签作者体验

第 2 轮结果保留在上述目录，验收输入为 `2aacc98740f5c4dd2f7f0c8634fc6a887e040c85d19d3d8152d21d69b51ac86c`，已提交 `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3`。第 3 轮证据包含逐篇发现、修复、交叉审阅、74 文件的输入哈希及新检查日志；本轮发现的高影响结构与分支问题均已复核关闭，旧结果不替代新版验收
