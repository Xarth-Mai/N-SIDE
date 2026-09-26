# N:SIDE 工作入口

先读 [Wiki 首页](docs/index.md)、[项目约定](docs/dev/handbook/documentation.md)及当前任务相关资料

新内容使用[模板](docs/dev/handbook/templates/index.md)。文档变更运行[检查工具](tools/README.md)，Wiki 变更运行 `bun run docs:build`，工程变更执行 `game/` 的构建与测试

影响玩法、场景、UI、视觉或动画的修改按[运行验证](docs/dev/validation/runtime.md)执行真实路径、获取画面与状态证据、实际观察并复验；纯文档按其适用检查验收

## 项目工作

任务唯一编辑源为 `todo/tasks/` 卡片，[当前看板](todo/README.md)由 `bun run tasks:sync` 生成，`bun run tasks:check` 只读校验；[路线图](todo/roadmap.md)记录成果门槛，规则见[任务手册](docs/dev/handbook/tasks.md)

进度查询用 n-side-guide，授权执行用 n-side-work-loop，交付评审用 n-side-review。专业工作先读 [nside 适配入口](.agents/skills/nside/SKILL.md)，来源与固定版本见[第三方声明](THIRD_PARTY_NOTICES.md)

读取当前任务、相关规格与最近证据，说明真实状态、正在做哪一步、下一动作和需要作者判断的事项。技术实现自主推进，体验与里程碑放行依据真实反馈；长期结论回写 docs，本轮结果归 todo。旧记录在 todo/archive/legacy 中追溯，不把文档迁移或工具安装计作游戏能力

## 资源与工具链

- Wiki 编辑遵循[内容规则](docs/dev/handbook/documentation.md)，按阅读视角分工，玩家正文采用发布态表述
- 资产遵循[资产管理](docs/dev/production/asset-pipeline.md)，唯一源保存在 `source-assets/`，Logo 保留 SVG；文档位图使用质量 `80` 的 WebP
- JavaScript 依赖、脚本、测试与 Wiki 统一使用 Bun，维护 `bun.lock`
