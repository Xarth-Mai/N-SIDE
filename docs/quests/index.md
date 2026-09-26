# 游戏任务

完整章节与生活故事见[故事百科](../story/index.md)，本页组织任务的制作规格与运行数据

| 任务 | 制作资料 |
| --- | --- |
| QST-002 最后的玩具 | [任务说明](QST-002/README.md)、[Demo 范围](QST-002/demo-scope.md)、[行动与恢复](QST-002/development.md)、[信息与节点](QST-002/narrative.json) |

[故事索引](quests.json)维护章节与角色、场所关联；具体任务按下列目录维护可执行结构，索引与引擎实现分别验证

任务使用 `QST-*` ID，包含日常片段与异常委托。从[日常模板](../templates/daily-scene.md)或[委托模板](../templates/quest.md)建立具体内容

```text
QST-001/
├── README.md              事件、玩家体验与制作需求
├── development.md         大纲、可玩序列与场景规格
├── narrative.json         节点、信息、选择与状态
├── dialogue.csv           正式台词
├── asset-manifest.json    资产关系
└── playtests/             试玩计划与结果
```

辅助文件随制作建立，短内容可用一个 `QST-001.md`。任务状态、奖励、重复触发与恢复规则写在该任务的流程中
