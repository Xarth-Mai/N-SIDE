---
document_id: DOC-NARRATIVE-DATA
---

# 叙事数据

`narrative.json` 保存一个任务的结构、信息、选择与场景关联。实际格式见[模板](../handbook/templates/narrative.json)

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

## 从内容到运行实现

输入为接受的百科事实、[委托规格](../design/quests/index.md)、同目录叙事数据与对白。先确认对象 ID、信息来源、玩家可见内容和本次范围；当前结构检查器并非引擎任务执行器，尚未接入的行为保持待验证

1. 修改信息与节点前，列明进入条件、一次性事件、世界事实和玩家认知的归属，使用[状态与恢复契约](../design/systems/state-and-recovery.md)确定保存与提交时机
2. 在真实加载入口解析数据；稳定 ID 解析失败或必需资源缺失时给出对象、路径及错误，不用默认成功状态继续
3. 运行系统在明确的场景生命周期内创建和清理实体、消息订阅、声音与 UI 焦点；退出、取消加载和重入分别检查，不把仅启动一次的实现当成重复进出已通过
4. 将节点结果通过现有系统提交，状态断言读取真实结果。存档按版本恢复稳定 ID 与一次性事件；引擎实体 ID 只用于当前进程，不作为持久对象身份
5. 输出改动数据、接入代码和短路线；先运行下列结构检查，再按实现范围执行 [运行验收](../validation/runtime.md)，将本次日志、结果和未测项写入任务证据

```fish
bun run check:narrative
bun run check:story-design
bun run check:templates
```

结构失败先修复最短失败路径及 ID 引用；实际恢复失败保留重现输入、提交点和日志，修复后重测重复执行与中断路径。观察事实与推测的呈现属于玩家可理解性检查，世界真相不因可计算而提前显示
