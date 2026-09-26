---
name: nside
description: Apply N:SIDE project constraints when selecting imported game-design, Bevy, art, UI, dialogue, audio or AI Skills. Use for specialist production and runtime verification in this repository; progress queries and milestone acceptance stay with the existing n-side-guide, n-side-work-loop and n-side-review Skills.
---

# N:SIDE 专业工作适配

先读根 `AGENTS.md`、本次涉及的正式设计与实现入口，沿用已确认范围。原版 Skill 负责专业方法，此处只说明项目差异；来源和版本见[第三方清单](../../../THIRD_PARTY_NOTICES.md)

## 按问题选择

| 当前工作 | 入口 | 本地输入与边界 |
| --- | --- | --- |
| 把设定变成可玩机制 | `gameplay-mechanism-designer` | [Demo 范围](../../../docs/dev/direction/demo-scope.md)、相关 `docs/gameplay/`、当前委托；从指定问题开始，不重做整个游戏 |
| 日常设计审查、找最小实验 | `game-design-reality-check` | 实际提案与已有证据；结论写入当前任务记录 |
| 专门寻找无解状态、支配策略、规则漏洞 | `stress-testing-game-concepts` | 只攻击待验证规则；需要修复时保留失败轨迹；不重复上一项整份报告 |
| ECS、资产、镜头、渲染、动画、UI、测试 | 对应 `bevy-*` Skill | [工程入口](../../../game/README.md)、`game/Cargo.toml`、`game/Cargo.lock`、当前安装的 Bevy 源码和同版本示例 |
| 视觉资产生产 | `create-game-assets` | [资产管理](../../../docs/dev/production/asset-pipeline.md)、[美术](../../../docs/dev/production/art-direction.md)及已有资产包 |
| Blender 模型、绑定、动画、导出 | `nside-blender-pipeline` | 按资产类别核对变换、尺度与导出契约 |
| UI、对白、声音、关卡、NPC | `game-ui-ux`、`dialogue-systems`、`audio-design`、`level-design`、`game-ai` | 读取下面的工程映射；具体角色和剧情沿用现有 `create-case`、`create-content`、`review-narrative` |
| 参考图或视频的表现拆解 | `nside-reference-analysis` | 解释观察与推测，形成可实现、可检查的制作要求 |

## 工程映射

- Bevy 参考针对 0.19；本项目以锁文件为准。上游的 compile-check 声明不等于本地检查通过，使用到的 API 以当前源码和实际编译验证，不为安装 Skill 升级引擎
- 13 个 Bevy 基础 Skill 的完整原文保留。未安装的可选兄弟 Skill 在各 `UPSTREAM.md` 提供固定版本链接；内置 Rapier、Hanabi、Seedling、输入插件和 `bevy_capture` 示例不构成本项目选型
- 渲染和交互验证统一走[运行验收](../../../docs/dev/validation/runtime.md)，复用真实系统、操作路径与场景。ECS 单测、离屏渲染、人工试玩各自说明覆盖范围
- UI 使用 Bevy 的布局、交互与焦点系统；按值变化更新可用变更检测或现有消息，不为上游的 Godot signal 示例另建总线。键鼠与手柄沿用项目范围
- 对白以[现有数据契约](../../../docs/dev/engineering/content-contracts.md)中的 `narrative.json`、稳定 ID 和 `dialogue.csv` 为输入；Ink/Yarn 仅是上游比较材料，不自动引入。任务事实、玩家认知和一次性事件分开检查
- 关卡沿用 `source-assets/district-map/district.json` 与源对象关联，区分道路连通、人物碰撞可通行与 NPC 导航。先测真实移动尺度，再扩展空间
- NPC 按需求选择最小状态机；Agent 是游戏内角色，使用游戏逻辑，离线运行。声音按[声音制作](../../../docs/dev/production/sound.md)与现有实现选择，FMOD/Wwise 和上游目标响度不自动成为依赖或验收预算

## 资产与验收边界

`create-game-assets` 的 brief 信息写进已有资产包 README，资产登记沿用对象 `asset-manifest.json` 或项目级共享资产包。其模板用于核对字段，不复制成第二套台账；源文件、参考关系、许可、实际尺寸与可获得的生成参数依[资产管理](../../../docs/dev/production/asset-pipeline.md)记录

生图调用读取当前环境实际提供的 `imagegen` Skill；Codex 组织流程不表示已配置视频、3D 或外部付费服务。优先复用既有素材，先看源图再做下游处理，alpha 路径服从工具真实能力

仅在任务需要时读[内容与验收方法适配](references/content-and-review.md)。进度、循环和阶段验收继续使用原有三个项目 Skill，不因工具安装、编译或截图成功增加游戏已验收数量

## Provenance

N:SIDE 项目适配层，基于 `gamedev-skills/awesome-gamedev-agent-skills` 的 `skills/disciplines/{create-game-assets,game-ui-ux,dialogue-systems,audio-design,level-design,game-ai}/` 工作流改编，固定版本 `44888f28ff918357ad82c4473352c60a1c5bde5b`

Copyright 2026 Abhishek Barali and the awesome-gamedev-agent-skills contributors — [Apache-2.0 LICENSE](../../../third_party/skills/awesome-gamedev-agent-skills/LICENSE) · [NOTICE](../../../third_party/skills/awesome-gamedev-agent-skills/NOTICE)

Modified for N:SIDE：仅增加本项目的数据归属、Bevy 接口、资产路径和验收映射；原版目录保持不变，未采用其他引擎示例或创建重复账本。其他被引用项目的完整来源与许可见[第三方清单](../../../THIRD_PARTY_NOTICES.md)
