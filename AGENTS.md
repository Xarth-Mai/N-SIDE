# N:SIDE 工作入口

先读 [Wiki 首页](docs/index.md)与[项目约定](docs/conventions.md)，再读当前任务涉及的主题正文、代码和 `todo/` 记录。

工作流程：明确交付 → 读取资料 → 制作 → 同步知识与实现 → 验证 → 记录结果。

长期事实维护在 `docs/`；当前进度维护在 `todo/`；可复用步骤维护在 `.agents/skills/`；游戏实现位于 `game/`。已定内容、候选方案和待定事项沿用各主题的实际状态。

新内容使用[模板](docs/templates/index.md)。文档变更执行[检查工具](tools/README.md)，Wiki 变更执行 `bun run docs:build`；工程变更执行 `game/` 对应的构建与测试。

## 资源与工具链

- 仓库内文档图片统一使用质量参数 `80` 编码的 WebP，转换后同步更新引用
- JavaScript 依赖管理、脚本、测试与 Wiki 统一使用 Bun，维护 `bun.lock`，不使用 Node.js 运行时；`node:` 标准库 API 使用 Bun 的兼容实现，带 Node shebang 的 CLI 通过 `bun --bun` 启动
