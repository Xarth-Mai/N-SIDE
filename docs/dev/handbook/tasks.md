# 任务卡与当前看板

## 输入与权威来源

执行前读取指定任务、关联规格、依赖结果及最新证据，核对工作区与被验收输入。`todo/tasks/TASK-xxx-title.md` 是单项任务的唯一编辑源，路径在任务生命周期中保持稳定；`todo/README.md` 从卡片生成。`todo/roadmap.md` 只维护里程碑成果、门槛与风险

旧 M0—M7 与 61 项清单冻结为历史输入，通过迁移映射追溯。新的 G0—G4 按有效基线、可玩闭环、完整样板、内容集成、真实试玩与发布组织，不沿用原分母。重构工作使用 DOCS-PIPELINE 里程碑，不能计作游戏能力

## 最小格式

```yaml
---
id: TASK-012
type: experiment
status: ready
milestone: G1
depends_on: []
specs:
  - docs/dev/design/systems/input.md
---
```

正文保留一级标题及目标与范围、验收条件、当前工作与下一步、结果与证据。标题不在 frontmatter 重复保存。字段要求如下

| 字段 | 规则 |
| --- | --- |
| id | 唯一 TASK-数字 ID，从未使用编号分配，历史编号不再分配给别的工作 |
| type | design、experiment、feature、asset、fix、document |
| status | backlog、ready、active、review、blocked、done、cancelled |
| milestone | roadmap 中有定义的成果组 |
| depends_on | 真实前置结果的任务 ID 数组，不笼统等待整个前阶段 |
| specs | 仓库内长期规格路径数组，所需锚点必须存在 |
| migrated_from | 仅迁移需要时列出旧工作项或任务 ID |
| blocked_reason | blocked 时必填，说明当前无法继续的具体条件 |
| resolution | cancelled 时必填，说明取消原因、范围影响和依赖替代 |
| acceptance | done 时必填 role、revision、record，指向真实验收输入和记录 |

acceptance.role 表示 codex、author 或 playtester 记录类型，不认证人的身份；revision 是实际被检查输入的完整 commit 或内容 hash，record 是任务 evidence 中已有记录。字段存在只证明可追溯，不能代替实际验证

## 执行与状态

先处理用户指定工作，其次续接可继续的 active/review，最后选择依赖已满足的 ready。只问进度时读取卡片与生成看板，保持文件不变。近期展开可交付任务，远期只保留里程碑目标和重大风险

设计任务形成可判断的规则与范围；实验先写假设与判定规则，否定假设也可完成；功能与资产任务接入真实路径后检查；文档任务核对语义、来源、引用和发布。按类型选择 Skill 与检查，不固定要求每项走完六轮教学或游戏录像

开始实际工作设为 active；确实等待体验或设计评审才设 review，卡片写清需要谁判断什么。技术任务通过适用检查即可由 Codex 结束，无需作者批准每个实现细节。作者体验、陌生玩家理解和阶段放行只引用真实反馈，不由自动测试代签

done 必须有本任务验收依据，既不等于上线，也不增加游戏百分比。cancelled 不满足其他任务依赖，必须替换或移除受影响依赖并说明范围调整。旧验收被新变更破坏时重开原卡，旧记录保留在正文，移除当前 acceptance，复查实际受影响后续任务

## 生成与检查

在仓库根使用 `bun run tasks:list` 读取当前项，`bun run tasks:sync` 生成看板，`bun run tasks:check` 校验卡片与看板。check 只读，发现过期看板返回失败；sync 对同一输入重复生成没有漂移

检查 ID、字段、路径、依赖循环、被取消依赖和 done 证据。证据目录采用 `todo/evidence/TASK-xxx/run-id/`；运行大文件进入被忽略的 output，任务内保存摘要、版本、命令和必要小证据。任务计数描述工作组织，不表示工时或游戏完成比例

## 输出与失败处理

收尾留下实际交付、PASS/FAIL/NOT RUN、下一动作和需要作者的具体判断；已确定的长期结论回写 specs。失败时保留复现输入与日志，在授权范围内修复、复验。缺少设备、工具或真实反馈时明确缺口，继续独立工作，不把跳过写成通过
