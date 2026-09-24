# 游戏任务

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
