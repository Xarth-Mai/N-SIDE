# N:SIDE 工作入口

先读 [Wiki 首页](docs/index.md)、[项目约定](docs/conventions.md)及当前任务相关资料

新内容使用[模板](docs/templates/index.md)。文档变更运行[检查工具](tools/README.md)，Wiki 变更运行 `bun run docs:build`，工程变更执行 `game/` 的构建与测试

## 资源与工具链

- Wiki 编辑遵循[内容规则](docs/production/wiki.md#内容规则)，按阅读视角分工，玩家正文采用发布态表述
- 资产遵循[资产管理](docs/production/assets.md)，唯一源保存在 `source-assets/`，Logo 保留 SVG；文档位图使用质量 `80` 的 WebP
- JavaScript 依赖、脚本、测试与 Wiki 统一使用 Bun，维护 `bun.lock`
