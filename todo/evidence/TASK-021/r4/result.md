# Cloudflare 发布目录修复

用户日志中的双站构建已通过，部署自动识别 `docs/.vitepress/dist`，该目录在迁移后不再输出，导致部署失败；用户明确要求恢复原有路径

完整 Wiki 直接输出 `docs/.vitepress/dist`，独立玩家站输出 `docs/.vitepress/dist-player`；构建、产物检查与预览共用路径函数，来源快照和缓存隔离到 `docs/.vitepress/cache/wiki`。Wrangler 显式使用完整站目录，不配置二次构建；按用户要求，本机静态服务恢复直接读取 `docs/.vitepress/dist`

仓库已删除 GitHub Actions 工作流，同步移除工具手册中的失效链接，保留本地检查入口

## 验证

- PASS：Wiki 边界测试、类型检查、文档与 Skills 检查
- PASS：双站构建，玩家 89 页、完整站 145 页，地图 29 项测试通过
- NOT RUN：Cloudflare 远端重新部署，需要推送本轮提交并重试；本轮不更改远端账号设置

Cloudflare 构建命令 `bun run docs:build`，部署命令 `bun run docs:deploy`，根目录 `/`

## 清理与本机验证

Caddy 配置校验与平滑重载 PASS，真实 HTTPS 全站 432 项检查通过，覆盖主页、145 个页面、125 个旧入口、156 个资源及补充入口。项目根目录的 output 已移至临时备份 `/tmp/nside-output-backup-20260926`，其中旧截图和资产检查记录保留；临时发布副本已清理

递归清理 docs 下 18 个空目录及根目录下 .github/workflows、.github 两个空目录。Git、依赖、第三方 Skills 和构建工具管理的目录不纳入源码空目录清理；其他资产与运行验收工具仍可按其原有约定生成 output，Wiki 不再生成它

Wrangler 最新正式版本经 npm 官方 registry 核实为 4.141.0，部署脚本通过 Bun 固定调用该版本。本机依赖下载多次超时，因此 Wrangler dry-run 为 NOT RUN；未引入未完成的依赖锁变更。仓库显式配置同时兼容 Cloudflare 现有的 npx wrangler deploy 命令

简化审查（ponytail-review）：Lean already. Ship.
