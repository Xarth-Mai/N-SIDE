---
id: TASK-046
type: document
status: done
milestone: DOCS-PIPELINE
depends_on: []
acceptance: {"role": "codex", "revision": "9f1f0e657a4f80114a49282fea6edf5f448f7f87a8bf9f0afe0a845073541841", "record": "todo/evidence/TASK-046/production-script/result.md"}
specs:
  - docs/player/story/index.md
  - docs/dev/design/story-continuity.md
  - docs/dev/design/story-graph.md
  - docs/dev/design/quests/index.md
  - docs/dev/design/quests/script-format.md
  - docs/dev/design/quests/scenes.md
  - docs/dev/design/quests/campaign-sequences.md
  - docs/dev/design/quests/pacing.md
  - docs/dev/design/quests/world-stories.md
---

# 剧本、角色关系与世界任务闭环

## 目标与范围

只扩充与修订故事文档及文档使用的故事目录数据，覆盖十章主线、十二条街坊故事和五条世界任务。第 2 轮按原神 1.0 首发任务分层安排逐项长度，替代此前 40–60 小时硬目标；第 3 轮按作者要求分工严格审读，允许必要重构，修复与优秀影视叙事相比的结构差距。保留世界规则、人物身份、旧案处置、首发任务体量和 Demo 范围，不改游戏代码、地图或资产；第 4 轮展开完整中文制作脚本与台词交付稿，完成后提交本任务范围，不推送远端

## 验收条件

二十七条故事都有请求或目标、主动行动、转折、结束及重访后果；三十条伏笔链能追到正文中的铺垫、补证与回收，标明实物、机制和主题呼应的区别；主线每章有制作分场与关键场景对白；逐项时长有参考出处和未实测说明；可选内容不成为主线唯一钥匙，未做支线时尾声不冒称经历；人物、场所、时间轴及故事目录一致，适用静态检查与双站构建通过。第 3 轮另要求全 27 篇经过分组与交叉复读，人物相互阻碍并承担后果、高潮产生未预演的新取舍、支线保留不同立场，修复已发现的分支矛盾并同步制作分场

第 4 轮另要求 27 篇从入场连续写到结项与必要重访，所有既有分支有实际动作和对白，CSV 与脚本文字完全一致；81 组主线分场、11 个旧入口及 30 条伏笔链对应具体制作位置，逐篇由非作者交叉审读并修复，保持逐项时长预算

## 当前工作与下一步

第 4 轮文档交付完成：27 篇完整中文制作脚本、195 场、673 镜、2,184 条台词与屏幕文字已完成分组写作、独立交叉审阅、修后复读和适用检查；81 组分场、11 个旧入口与 30 条伏笔链同步。保持既有时长预算，后续由实际镜头、演员、配音和玩家测试分别检验演出、操作与节奏，本轮不增加可玩能力

## 结果与证据

本轮以当前分支与工作区为输入，保留并行的美术、城市、人物管线、运行验证、工程和其他任务变更；第 1、2 轮结果、内容哈希与检查日志保存在 `todo/evidence/TASK-046/final/`，第 3 轮记录在 `todo/evidence/TASK-046/reader-review/`。新增内容属于文档交付，不增加当前游戏可玩能力或代签作者体验

第 2 轮结果保留在上述目录，验收输入为 `2aacc98740f5c4dd2f7f0c8634fc6a887e040c85d19d3d8152d21d69b51ac86c`，已提交 `4801b81bc3c91b5b4eebe04ebd86aa1f245b52c3`。第 3 轮证据包含逐篇发现、修复、交叉审阅、74 文件的输入哈希及新检查日志；本轮发现的高影响结构与分支问题均已复核关闭，旧结果不替代新版验收

第 3 轮完成记录及验收输入 `dab934f781f4901cbd08da81a5069ae441c65fd8b1f14d57aed83cbafa41e381` 保留，已提交 `1bfc1cff68311b9faef2fbeb279867b1129e7f9c`；第 4 轮证据独立保存在 `todo/evidence/TASK-046/production-script/`，完成文稿不等于实机镜头、录音或玩家体验已验收

第 4 轮 [完整交付与验收](../evidence/TASK-046/production-script/result.md)、[逐篇审阅](../evidence/TASK-046/production-script/review.md)和 [96 个源文件输入哈希](../evidence/TASK-046/production-script/inputs.json)保存本轮结果，验收内容 hash 为 `9f1f0e657a4f80114a49282fea6edf5f448f7f87a8bf9f0afe0a845073541841`；同一维修顾客跨章身份一致，26 位既有无名说话者已登记，全部音频路径保持为空。实际表演、镜头适配、录音、游戏时长与真实玩家体验均为 NOT RUN
