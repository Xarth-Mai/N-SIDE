# N:SIDE 工作入口

先读 [Wiki 首页](docs/index.md)、[项目约定](docs/conventions.md)及当前任务相关资料

新内容使用[模板](docs/templates/index.md)。文档变更运行[检查工具](tools/README.md)，Wiki 变更运行 `bun run docs:build`，工程变更执行 `game/` 的构建与测试

影响玩法、场景、UI、视觉或动画的修改按[运行验证](docs/production/runtime-validation.md)执行真实路径、获取画面与状态证据、实际观察并复验；纯文档按其适用检查验收

## 项目推进

项目进度以 `todo/demo-progress.json` 为唯一编辑源，[看板](todo/demo-progress.md)由工具生成；阶段目标见[总路线图](todo/demo-roadmap.md)，循环与验收见[个人开发工作流](docs/production/solo-workflow.md)

查询位置和下一步使用 `n-side-guide`，授权制作或迭代使用 `n-side-work-loop`，核对交付和阶段放行使用 `n-side-review`；专业叙事沿用现有 Skills

专业 Skills 先读 [nside 适配入口](.agents/skills/nside/SKILL.md)，按任务选择原版方法与项目差异；来源、许可与固定版本见 [第三方声明](THIRD_PARTY_NOTICES.md)，不将上游模板当成更改设定、依赖或资产清单的授权

按当前任务读取相关设计与最近工作记录，简述真实进度、轮次、下一动作及需要作者判断的事项。只读查询保持数据不变，已有决定直接沿用；指定旧主题时读取归档与 Demo 承接，在新记录中续接。日常实现自主推进，体验判断和阶段放行依据真实反馈

## 资源与工具链

- Wiki 编辑遵循[内容规则](docs/production/wiki.md#内容规则)，按阅读视角分工，玩家正文采用发布态表述
- 资产遵循[资产管理](docs/production/assets.md)，唯一源保存在 `source-assets/`，Logo 保留 SVG；文档位图使用质量 `80` 的 WebP
- JavaScript 依赖、脚本、测试与 Wiki 统一使用 Bun，维护 `bun.lock`
