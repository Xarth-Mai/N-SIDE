---
id: DOC-NARRATIVE-DATA
---

# 叙事数据

`narrative.json` 保存一个任务的结构、信息、选择与场景关联。实际格式见[模板](../templates/narrative.json)

## 顶层

| 字段 | 内容 |
| --- | --- |
| `schema_version` | 数据格式标识，使用模板中的取值 |
| `quest_id` | 所属任务 ID |
| `entry_beat` | 起点 |
| `terminal_beats` | 结束节点数组 |
| `information` | 信息命题数组 |
| `beats` | 流程节点数组 |
| `choices` | 选择数组 |
| `scenes` | 场景数组 |

局部 ID 使用任务前缀，例如 `QST-001-B001`。选择选项使用所属选择内的稳定键，例如 `inspect`

## 信息

每条信息包含 `id`、`proposition`、`world_truth`、`importance`、`initial_state`

`proposition` 是玩家判断的命题，`world_truth` 是实际情况及相关设定，`importance` 使用 `critical / supporting / optional`

认知状态为 `hidden`（尚未接触）、`hinted`（已有暗示）、`suspected`（形成推测）、`confirmed`（当前认定）、`disproved`（当前否定）。后续证据可以改变玩家认知

关键命题在使用节点的 `requires` 中登记。跨任务前置信息体现在初始认知与任务依赖中，角色自身认知写入场景规格

## Beat

| 字段 | 内容 |
| --- | --- |
| `id` | 节点 ID |
| `dramatic_change` | 目标、冲突、关系或解释的变化 |
| `player_goal` | 玩家此刻理解的目标 |
| `player_action` | 实际动作与交互 |
| `requires` | 进入时的信息要求：信息 ID → 可接受状态数组 |
| `delivers` | 完成时的信息效果：信息 ID → 新状态 |
| `state_changes` | 目标、关系或世界效果：键 → JSON 值 |
| `next` | 全部可能后继节点数组 |
| `target_seconds` | 预计主动游玩秒数，待测时为 `null` |

进入节点时检查 `requires`，完成时执行 `delivers`，再进入 `next`。可选线索、跳过、返回与补充呈现通过实际路径表达。结束节点的 `next` 为 `[]`

检查工具逐个遍历“节点 + 认知状态”，核对信息使用顺序与结束路径。引擎变量、条件和存档行为由实现测试核对

## 选择

选择包含 `id`、`beat_id`、`options`。每个选项记录 `id`、目的节点 `to`、即时响应 `acknowledgement` 和后果 `consequence`

选项可通往同一结构节点；不同认知结果分别用后继节点表达。反馈可感知性通过试玩观察

## 场景与对白

场景包含 `id`、`beat_id`、`intent`、`entry_state`、`exit_state`、`cast`、`line_ids`。详细空间、表演与人物意图保存在开发稿中

`line_ids` 引用同目录 `dialogue.csv`：

```text
line_id,speaker_id,text_zh_cn,context,recording_status,audio_path
```

说话者使用角色 ID、`NARRATOR` 或 `SYSTEM`；录制状态使用 `temp`、`ready`、`recorded`、`needs_pickup`、`not_required`。音频路径相对仓库根，待录行留空。台词修订沿用原 ID，文本内容与场景逻辑分别维护

## 试玩事件

事件使用 `narrative.beat_enter`、`narrative.info_changed`、`narrative.choice_selected`、`narrative.beat_exit`，关联任务、节点、信息、选项和发生时间

记录包含事件 ID、构建、场次、路线及主动游玩时长。参与者使用化名；采集内容与保存安排在试玩时说明。事件与玩家原话按同一节点对照
