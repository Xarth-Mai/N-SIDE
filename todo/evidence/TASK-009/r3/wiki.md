# R3 Wiki 双构建与真实发布边界验收

本轮使用仓库实际 Bun 1.4.2、锁定的 VitePress 1.6.4 和迁移后的完整文档；R2 的最小样板记录保留原范围，本记录补齐真实整站与开发服务器检查

## 实际执行

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `bun run docs:build` | PASS：frozen lock 安装、29 项地图检查、两站实际构建及产物检查 | [构建日志](wiki-build.log) |
| `bun test tools/tests/wiki.test.mjs` | PASS：9 项回归 | [测试日志](wiki-tests.log) |
| `python3 todo/evidence/TASK-009/r3/wiki-http.py` | PASS：实际两个开发服务器，共 24 个 HTTP 请求 | [请求结果](wiki-http.json)、[player 日志](wiki-http-player.log)、[dev 日志](wiki-http-dev.log) |
| `bun todo/evidence/TASK-009/r3/wiki-negative.mjs` | PASS：真实 VitePress 构建拒绝模块越界，exit 1；外部 include 在准备阶段拒绝 | [负例结果](wiki-negative.json)、[实际构建失败日志](wiki-negative-build.log) |
| `git diff --check` | PASS | 终端检查 |

整站构建：player 88 页、380 文件；dev 142 页、594 文件。实际文件清单摘要、重定向、原始数据及 source data 哈希见[产物记录](wiki-artifacts.json)

## 发布内容核对

玩家站只含玩家页面、其 local search、88 个旧玩家 URL 的无正文重定向、Logo 和地图投影；原始 JSON 仅有地图投影与 VitePress hashmap，没有任务／制作 JSON、CSV 或完整 district.json。开发站额外保留开发正文、125 个兼容重定向、8 份按源文件逐字节核对的数据，以及明确选中的开发图片与完整地图源

实际 HTTP 同时覆盖普通页面、编译 Markdown、localSearchIndex、旧地图 URL、地图投影、开发数据 GET/HEAD、编码后的 dev 路径、完整地图源与原 docs/source-assets 的 `@fs` 请求。玩家侧拒绝开发内容；开发侧正确返回所选数据；两侧的原始目录越界读取返回 403/404

玩家地图沿用现有 DistrictMap 与 buildScene，投影保留 91 个场所、绘图对象和地形，移除原始 brief、建筑规格与运行无关作者字段。源地图与游戏实现未改。本轮没有声称浏览器内地图点击或游戏玩法通过人工验收

## 审查发现与修复

独立审查实际复现两条 R2 边界缺口：alias 前缀允许 `../../` 导入开发 JSON，VitePress `@include` 可把开发正文包含进玩家页。两条路径曾在样板内完成构建，因此本轮没有把输出 marker 扫描当成完整隔离方案

修复采用精确的页面 import 入口、拒绝外部 include/snippet、隔离源目录，以及最终 Vite 模块实路径检查；带注释绕过前置 import regex 的变体仍由 Vite 真实构建明确拒绝。VitePress 已核对的 3 个虚拟模块、选中页面路由与已选 public 文件可正常工作

源 metadata 在玩家快照移除，仍保留原始文档身份；直接 `vitepress build docs` 明确失败，必须选择 player/dev。新增的 `check` 为只读，`preview` 先检查产物再服务

复杂度审查已完成：删除重复 project-assets 中间件，沿用 VitePress 的原生 public 输出；不新增 watcher、Markdown 解析器、发布框架或依赖。独立审查结论：`Lean already. Ship.`

## 限制与工作入口

两站地图相关 bundle 仍触发 Vite 的 500 kB chunk 提示，构建通过；本轮保持既有绘图功能，没有通过提高阈值掩盖提示。开发服务器使用隔离快照，修改源码后重启对应命令；规则和 fish 命令已写入[Wiki 手册](../../../../docs/dev/handbook/wiki.md)

首次 localhost 测试在沙箱内被 `listen EPERM` 阻止，随后按授权在可绑定 localhost 的环境中完成上述真实请求；脚本仅结束自己启动的进程。本轮为文档与发布工具改动，未运行游戏／capture，也未把构建与 HTTP 检查记为游戏功能完成
