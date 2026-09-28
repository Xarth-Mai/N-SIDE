# TASK-040 室外预览游玩指南验收

## 输入与范围

基线提交为 `72eaf124e3f990858fb4244289613c1179cae164`，本次验收的[玩家指南](../../../../docs/player/guide/index.md) SHA-256 为 `887731362ea52ea63f3c8a5f5ca2397bdd80eeda162b748834c0eb8ceb01c0f6`；[输入清单](inputs.json)逐文件记录实际读取的入口、人物、地点观察、设置、地图与打包工具 hash

前置结果已分别在[TASK-037 设置保存](../../TASK-037/r1/review.md)与[TASK-039 鼠标返回](../../TASK-039/r1/review.md)验收。本轮属于文档工作，不重复宣称其运行结果，也不增加玩法或代替玩家体验验收

## 语义核对

| 指南内容 | 核对来源与结果 |
| --- | --- |
| 源码启动与步行入口 | `app.rs` 的 `--project-root`、`--walk-preview` 与标题动作；省略步行参数时仍是固定街景 |
| 移动、镜头与回起点 | `player.rs` 的 WASD、左右摇杆、右键、Q/E、R/Select；镜头灵敏度作用于三种观察输入 |
| 地点查看与鼠标关闭 | `places.rs` 仅为 04 月台杂货及 23 摘星台提供公共信息；`observation.rs` 与入口处理 F/南键、Esc/东键、真实返回按钮和关闭输入隔离 |
| 暂停、失焦与重连 | `app.rs` 优先处理焦点/连接保护，Tab/Start 可在观察中暂停，继续后恢复原观察；指南提醒先释放旧输入 |
| 设置与保存边界 | 两档文字、两档灵敏度、来源菜单焦点与 `settings.rs` 实际保存状态；明确用户配置与行走进度分开 |
| 短登高与往返 | 地图 `hill-short` 实际节点从 home 经 east_low_junction、east_mid_junction、bend、upper、hillgate、三个 hill_short_rest 到 summit；指南使用玩家地名，不要求读对象 ID，也不把长神龛路线混入短路线 |
| Linux 包入口 | 打包工具生成 `run-walk-preview.sh`，指南采用「收到预览包后」的条件式说明，不提供虚构下载链接，不宣布公开发布；包的实际运行由 TASK-041 独立验收 |
| 当前开放范围 | 保留室外、白天、简化人物与灰盒边界；未写入室内、任务、潜梦、战斗、夜景已开放或任务存档已实现等承诺 |

## 实际命令与结果

从仓库根执行，以下命令可直接用于 fish

```fish
bun run check:docs
bun run docs:build
python3 -B todo/evidence/TASK-040/r1/check-build.py
bun -e 'import { readTasks } from "./tools/tasks.ts"; const result = readTasks("."); console.log(JSON.stringify({ok: result.ok, issues: result.issues, count: result.tasks.length})); if (!result.ok) process.exitCode = 1'
git diff --check -- docs/player/guide/index.md todo/tasks/TASK-040-preview-player-guide.md todo/evidence/TASK-040/r1
```

- PASS：[文档检查日志](check-docs.log)，包含 Markdown/对象 ID 检查与 31 Skills、25 条导入记录检查
- PASS：[双站构建日志](docs-build.log)，地图相关 54 项测试通过；玩家站 95 页、547 个文件，开发站 158 页、791 个文件，分别通过受众与产物检查
- PASS：[产物检查](build-inspection.json)直接读取玩家 HTML 的 main 正文、导览链接和本地搜索索引，核对真实操作文本与 10 个指南章节索引，保留 HTML 与搜索文件 hash
- PASS：[任务卡检查](task-card-check.log)通过现有 `readTasks` 检查 36 张卡片的字段、依赖与证据引用；看板留给主任务统一生成，本次没有修改看板
- PASS：最终差异空白检查；Ponytail 审查指南/卡片/证据职责、重复维护与新增工具范围，结论为 `Lean already. Ship.`

开发站仍有单个输出 chunk 大于 500 kB 的构建提醒，不影响本轮构建成功。产物检查脚本初次把生成的相对链接当作绝对链接比较，已按页面地址解析再检查，指南链接本身没有修改

## 未运行与边界

NOT RUN：浏览器中的桌面/手机排版、实际搜索交互和阅读体验。本轮实际执行 `mcp__cua_repl.js` 的 `await cua.getState();`，返回 `apps: []`、`browsers: []`，见[宿主清单](browser-inventory.json)；构建产物检查不替代浏览器操作

NOT RUN：本轮未重新启动游戏、测 Windows 或调用物理手柄，相关游戏行为引用各自已验收任务的覆盖范围；也没有作者或陌生玩家可读性批准

## 结论与下一步

PASS：当前操作、范围与已有实现一致，引用与双站实际产物通过，TASK-040 可按技术文档任务记为 done。Linux 预览目录继续由 TASK-041 独立核对包内资源和隔离运行；公开发布与 G1 阶段放行不在本次验收内
