# 游戏任务

完整章节与生活故事见[故事百科](../../../player/story/index.md)，本页组织任务的制作规格与运行数据

| 任务 | 完整制作脚本 | 中文台词 |
| --- | --- | --- |
| QST-001 门还开着 | [逐镜脚本](QST-001/script.md) | [六列表](QST-001/dialogue.csv) |
| QST-002 最后的玩具 | [逐镜脚本](QST-002/script.md) | [六列表](QST-002/dialogue.csv) |
| QST-003 多出来的掌声 | [逐镜脚本](QST-003/script.md) | [六列表](QST-003/dialogue.csv) |
| QST-004 没有放学的那一天 | [逐镜脚本](QST-004/script.md) | [六列表](QST-004/dialogue.csv) |
| QST-005 初秋以外 | [逐镜脚本](QST-005/script.md) | [六列表](QST-005/dialogue.csv) |
| QST-006 仍在一起 | [逐镜脚本](QST-006/script.md) | [六列表](QST-006/dialogue.csv) |
| QST-007 河没有倒流 | [逐镜脚本](QST-007/script.md) | [六列表](QST-007/dialogue.csv) |
| QST-008 把今天带回来 | [逐镜脚本](QST-008/script.md) | [六列表](QST-008/dialogue.csv) |
| QST-009 回声之后 | [逐镜脚本](QST-009/script.md) | [六列表](QST-009/dialogue.csv) |
| QST-010 明天照常营业 | [逐镜脚本](QST-010/script.md) | [六列表](QST-010/dialogue.csv) |
| QST-101 旧外套的新口袋 | [逐镜脚本](QST-101/script.md) | [六列表](QST-101/dialogue.csv) |
| QST-102 一桌不同的晚饭 | [逐镜脚本](QST-102/script.md) | [六列表](QST-102/dialogue.csv) |
| QST-103 橱窗还差一点 | [逐镜脚本](QST-103/script.md) | [六列表](QST-103/dialogue.csv) |
| QST-104 雨停之后取衣服 | [逐镜脚本](QST-104/script.md) | [六列表](QST-104/dialogue.csv) |
| QST-105 球场边的一张椅子 | [逐镜脚本](QST-105/script.md) | [六列表](QST-105/dialogue.csv) |
| QST-106 一本书的几种读法 | [逐镜脚本](QST-106/script.md) | [六列表](QST-106/dialogue.csv) |
| QST-107 晚场以后 | [逐镜脚本](QST-107/script.md) | [六列表](QST-107/dialogue.csv) |
| QST-108 把今天拍进去 | [逐镜脚本](QST-108/script.md) | [六列表](QST-108/dialogue.csv) |
| QST-019 把我的名字写大一点 | [逐镜脚本](QST-019/script.md) | [六列表](QST-019/dialogue.csv) |
| QST-020 还没写完的第二首 | [逐镜脚本](QST-020/script.md) | [六列表](QST-020/dialogue.csv) |
| QST-021 借你一把伞 | [逐镜脚本](QST-021/script.md) | [六列表](QST-021/dialogue.csv) |
| QST-022 巧克力今天不值班 | [逐镜脚本](QST-022/script.md) | [六列表](QST-022/dialogue.csv) |
| QST-023 雨棚下的第二条路 | [逐镜脚本](QST-023/script.md) | [六列表](QST-023/dialogue.csv) |
| QST-024 名字写回去以后 | [逐镜脚本](QST-024/script.md) | [六列表](QST-024/dialogue.csv) |
| QST-025 空着的一格 | [逐镜脚本](QST-025/script.md) | [六列表](QST-025/dialogue.csv) |
| QST-026 不是备用的那一天 | [逐镜脚本](QST-026/script.md) | [六列表](QST-026/dialogue.csv) |
| QST-027 寄往新地址 | [逐镜脚本](QST-027/script.md) | [六列表](QST-027/dialogue.csv) |

[跨章因果与状态](../story-continuity.md)维护跨章铺垫、世界任务回收及各案保留边界，[关键场景定位](scenes.md)保留原十一段样稿的稳定入口并指向正式脚本；[五条世界任务规格](world-stories.md)补充选择、交付与重访

[全主线分场](campaign-sequences.md)逐段记录玩家目标、阻力、信息变化、后果与下一段承接；[首发任务体量与长度](pacing.md)给出二十七条任务的分项预算、参考资料和试玩调整口径

[故事索引](../catalogs/quests.json)维护章节与角色、场所关联；各项任务均交付脚本和台词；QST-002 另保留现有节点数据，其他任务按需要逐步建立执行结构，索引与引擎实现分别验证

任务使用 `QST-*` ID，包含日常片段与异常委托。从[日常模板](../../handbook/templates/daily-scene.md)或[委托模板](../../handbook/templates/quest.md)建立具体内容

```text
QST-001/
├── README.md              事件、玩家体验与制作需求
├── development.md         大纲、可玩序列与场景规格
├── script.md              连续可读的逐镜表演、玩家操作与分支
├── narrative.json         节点、信息、选择与状态（按接入需要建立）
├── dialogue.csv           正式台词
├── asset-manifest.json    资产关系
└── playtests/             试玩计划与结果
```

[共用脚本格式](script-format.md)规定镜号、控制权、分支与 CSV 台词投影。脚本承接 `development.md` 的详细场景职责，开发纲只维护结构与机制，避免重复修改整段演出。QST-002 的[制作入口](QST-002/README.md)、[Demo 范围](QST-002/demo-scope.md)及 QST-009 的[行动与恢复](QST-009/development.md)继续有效。其他辅助文件按实际制作需要建立。任务状态、奖励、重复触发与恢复规则写在该任务的流程中

## 梦魇事件约束

委托人、受影响者和梦主分别记录，兄妹的潜梦与接应角色随剧情轮换。事件复用已有场景、工具与敌人，按需要补充内容

分别确定形成前提与维持机制、睡眠依赖和多梦连接关系。现实意外的伤害尺度为皮外伤至骨折，上限为骨折；精神影响、受困、救援和后续关系变化由具体事件记录
