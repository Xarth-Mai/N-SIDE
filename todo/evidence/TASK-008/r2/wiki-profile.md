# R2 Wiki 发布 Profile 样板

输入为 R1 迁移索引与既定 player/dev 职责，本记录只覆盖隔离发布工具和最小真实构建样板；全量迁移后双站及开发服务器验收在 R3/R5 另记

## 实现

`tools/wiki.mjs` 在忽略的 `output/wiki/{player,dev}/source` 准备互相独立的源码投影，构建与开发服务器复用该投影。VitePress 共用 `docs/.vitepress/config.mjs`，各自产出 `output/wiki/{player,dev}/dist` 与 cache

玩家页、导航、搜索只读取 player；原始 JSON/CSV 仅允许 dev/design 等开发目录的数据。public 使用具体图片及受众允许清单，并且只复制被选中正文引用的图片。跨受众本地链接和源码 import 明确失败，不使用 ignoreDeadLinks

玩家地图保留现有 91 场所、风貌/道路切换、取景、缩放、地点详情与索引所需字段。`buildScene` 在构建侧读取唯一地图源，玩家只得到生成的 SVG 形状、groups 和 places 的 id/name/group/position/use/entry/page；完整建筑规格、brief、运营与线路主数据不默认发布。源地图未改

原玩家 URL 生成无正文的静态重定向并保留 location.hash；旧开发 URL 只在 dev profile 生成。开发服务器使用隔离快照，修改源文档后重启同一命令刷新；VitePress 默认加入 workspace root 的访问范围改为隔离目录和依赖目录

## 实际检查

| 检查 | 结果 |
| --- | --- |
| `bun test tools/tests/wiki.test.mjs` | PASS，8 项；涵盖导航顺序、profile 非空、原始数据隔离、symlink、跨目录链接/import、未批准 public 附件、地图字段投影及产物注入负例 |
| 已安装 VitePress 1.6.4 最小 player 构建 | PASS，2 页、34 个输出；包括真正的地图组件 SSR 和客户端 bundle |
| 已安装 VitePress 1.6.4 最小 dev 构建 | PASS，4 页、42 个输出；包含开发页与真实原始 JSON |
| 产物检查 | PASS，player 未输出最小样板的开发页、DEV_ONLY_SENTINEL 数据及未引用 public，dev 数据按原字节复制 |
| 全仓 player/dev 构建与搜索检查 | NOT RUN，本时点旧 docs 还未完成迁移；R3 实際落位后执行，不把最小样板当整站通过 |
| 实际 HTTP 开发服务器边界 | NOT RUN，R3/R5 使用真实服务器请求补齐；当前为 handler 与隔离目录检查 |

最小构建源与产物位于 `output/wiki-profile-smoke/`；执行脚本为本轮临时 `/tmp/wiki-profile-smoke.mjs`，由现有文档、地图和工具建立 2/4 页样板。正式入口为 `bun tools/wiki.mjs build player` 与 `bun tools/wiki.mjs build dev`

直接在 Bun 进程中调用 VitePress build 时，当前 Bun 1.4.2 的 esbuild 子进程返回 `The service was stopped`；保持既有 `bun run vitepress` CLI 调用后两次构建通过。没有升级依赖或改用新构建框架。现有地图 bundle 有大于 500 kB 的构建警告，保持真实画面，不为文档重构先做渲染优化

## 简化审查

已删除迁移后没有生产调用者的旧 project-assets 中间件和复制 API，统一由隔离投影准备并交给 VitePress 原生 public 复制。没有第二套服务或数据库，测试继续使用已安装 Bun

`ponytail-review`：Lean already. Ship.
