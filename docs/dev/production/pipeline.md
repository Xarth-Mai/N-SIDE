---
document_id: DOC-PRODUCTION
---

# 制作流程

| 阶段 | 工作与产物 |
| --- | --- |
| Concept | 核心体验、设定与范围 |
| Prototype | 镜头、交互、空间或战斗的可玩验证 |
| Vertical Slice | 一段完整内容，以及质量和工作量基准 |
| Production | 依据样板扩展任务、角色、场景与资产 |
| Alpha | 系统与主流程贯通 |
| Beta | 内容整合、平衡、兼容和体验调整 |
| Release Candidate | 候选构建与发布检查 |

## 内容制作

日常片段：情境与互动 → 灰盒 → 场景、表演与声音 → 试玩

敌人：定位与行为 → 灰盒 → 模型、动画与逻辑 → VFX / SFX → 数值与集成检查

委托按[叙事流程](narrative.md)制作。美术与声音方向从前期开始，正式资产随场景成熟制作，配音按稳定台词分批录制

## 开发任务

每项工作明确交付物和验收方式，任务状态见仓库 `todo/README.md`

每张任务卡维护唯一状态，依赖、证据与完成规则见[任务管线](../handbook/tasks.md)，Codex 的执行入口见[协作手册](../handbook/codex.md)。制作阶段描述目标，不以安装工具、编写文档或迁移目录折算游戏完成度

预估与实际工作量写在任务中，样板的实际成本用于后续排期。改动原因、涉及内容和结果记录在同一任务

## 输入与分工

先读任务卡的交付目标、依赖、规格与已有证据，再按问题选择专业 Skill。已有设计和普通技术细节直接沿用或实施；只有改变既定游戏设计、破坏性操作或新增付费授权需要作者判断。长期规则归所属 docs，当前结果归任务，素材身份和来源继续归原资产包

| 任务类型 | 制作与接入 | 对应交付与检查 |
| --- | --- | --- |
| design | 从已定事实与玩家行动形成规则、状态和恢复规格 | 设计数据与引用检查；需要体验判断时提交明确问题与真实反馈 |
| experiment | 写明假设、可推翻条件与最小实验，运行真实相关路径 | 参数、实际结果与结论；假设被否定也可完成实验，不等于功能完成 |
| feature | 接入现有系统、数据与生命周期 | 适用自动测试、运行状态及画面；恢复和失败路径可复验 |
| asset | 需求与参考 → 源素材 → 导出 → 实际接入 | 原清单、尺度/格式检查与实际表现，见[资产管线](asset-pipeline.md) |
| fix | 复现 → 定位共享原因 → 修复 → 回归 | 能失败的回归条件、受影响范围与实际复验；不暗改设计 |
| document | 对照事实 → 按职责组织 → 更新消费者 | 引用、对象、数据与发布边界检查；纯文档不强制游戏录像 |

## 执行闭环

1. 明确本轮输入、前提、交付和验收方式，恢复已有记录，不重复要求已获确认的选择
2. 实施最小可检查变化，专业方法分别见[叙事](narrative.md)、[美术](art-direction.md)、[模型与动画](blender-animation.md)、[UI](ui.md)、[声音](sound.md)、[VFX](vfx.md)与[参考分析](reference-analysis.md)
3. 先运行与改动直接相关的自动检查，修复明确失败；再依据[运行验证](../validation/runtime.md)选择实际入口、状态与画面，不将构建成功当作体验结论
4. 对需要图像或操作判断的交付实际观察，按[视觉验收](../validation/visual-review.md)区分自查、隔离评审与真实试玩，修复后复验受影响路线
5. 在任务卡关联结果与证据，将支持长期使用的结论写回规格或制作规范，再按任务完成条件更新状态与生成看板

## 输出与失败处理

交付始终能从任务追到实际文件、命令、结果和限制。结构化信息继续使用已有唯一源，不为本轮再造总清单。无环境的检查记为 NOT RUN；技术问题能在授权范围内修复就继续，依赖确实缺失时记录具体所缺结果，不笼统等待整个阶段

命令从仓库根执行，fish 可直接使用；先依据任务选择适用项

```fish
bun run check:docs
bun run test:tools
bun run docs:build
cargo test --manifest-path game/Cargo.toml --features viewer --locked
```

上述分别覆盖文档、工具、发布构建和工程测试；运行与操作命令由实际场景给出，当前唯一 Viewer capture 只证明其控制与渲染。任务生成和只读检查的命令见[任务管线](../handbook/tasks.md)，不在这里维护第二份状态规则

## 方法来源

基于 [Godogen 的运行闭环](https://github.com/htdt/godogen/blob/0b725bca053769a4727f76c332bf1f7b42e146ab/prompts/runtime.md)改编，固定 commit `0b725bca053769a4727f76c332bf1f7b42e146ab`，Copyright 2026 Alex Ermolov，[MIT LICENSE](https://github.com/htdt/godogen/blob/0b725bca053769a4727f76c332bf1f7b42e146ab/LICENSE.md)

Modified for N:SIDE：保留既有制作阶段，按真实任务类型选择证据深度，复用 Codex、Bevy、资产和任务入口，不采用空项目生成或上游发布脚本。完整许可与固定来源见仓库根 `THIRD_PARTY_NOTICES.md` 与 `third_party/skills/godogen/`
