# 故事关系系统第 1 轮

## 交付

十条主线与八条可选街坊故事全部接入 quests catalog 的 story 字段。三幕横向时间轴、单篇关系详情、主线与街坊目录、侧栏顺序均从同一源生成，正文不复制到结构化数据

强制条件使用 ALL / ANY 树，支持任务完成、节点完成、信息状态及状态成果；区分开始、必需阶段和可选片段。后续影响由引用反推，推演不读写玩家进度。已有 QST-002 narrative 是其 beat / information / state 的唯一维护源

[逐篇审查](audit.md)由同一 catalog 生成；持续维护规则与作者决定清单见[长期规格](../../../../docs/dev/design/story-graph.md)。重点风险包括重复求助、许遥提前出场、晚场时点、最终行动复习、旧处置持续有效和分场景后续对白；给出最小改动方案，未自行更改故事设定

## 验证

| 命令或观察 | 结果 | 证据范围 |
| --- | --- | --- |
| `bun run check:types` | PASS | TS 工具和真实 Vue SFC 类型检查 |
| `bun run check:story-design` | PASS | 91 场所、35 人物、18 任务及故事图校验 |
| `bun run check:narrative` | PASS | QST-002 的 22 个认知状态 |
| `bun run check:templates` | PASS | 叙事模板 |
| `bun run check:docs` | PASS | 245 篇 Markdown、83 个 ID、31 个 Skill |
| `bun run test:tools` | PASS | 103 项 Python、93 项 Bun 测试 |
| `bun run docs:build` | PASS | 玩家 88 页、开发 144 页；无超过 500 kB 的 chunk 警告 |
| `python3 todo/evidence/TASK-021/r1/http-probe.py` | PASS | [44 项真实预览 HTTP 检查](http.json)，十八篇详情、三个入口、开发审查与发布边界 |
| `bun run tasks:sync` / `bun run tasks:check` | PASS | 任务卡与生成看板一致 |
| `git diff --check` | PASS | 无空白差异错误 |
| ponytail-review | Lean already. Ship. | 无新增依赖，复用现有目录与 Wiki 发布流程 |
| 桌面、手机、键盘和真实点击 | NOT RUN | CUA 返回空浏览器清单，创建 iab 返回 Browser is not available: iab |

最终 SFC 类型复查修复了模板回调中 Condition 联合类型未缩窄的问题，来源任务查找改为 computed 后重新通过类型检查与双站构建

自动反例覆盖嵌套 AND / OR、部分满足、无入口循环、ANY 外部入口、阶段生产条件、可选支线无替代依赖、可选重访不阻塞主线完成、错误条件引用、缺失身份、显示位置冲突、错用推荐字段、遗漏正文与玩家投影

首次 HTTP 启动因沙箱绑定端口 EPERM 失败，随后对同一检查提权执行通过；探针退出时关闭自己启动的进程。HTTP 与服务端渲染只能证明交付内容和路由存在，不能证明客户端水合、交互或布局

## 输入与剩余验收

[输入散列](inputs.sha256)冻结本轮代码、结构数据、正文与审查依据；任务卡保持 blocked，无作者或玩家验收代签

浏览器可用后，在桌面和 390 px 宽度检查：十八张卡全部可达、时间轴自身滚动不撑破页面、主线与支线层次、每张卡选择状态、正文跳转、强制条件来源、推荐不阻塞、反向关联、条件推演与清空、尾声可选来信片段、Tab 与 Enter 操作、深色模式与刷新后无假存档。确认无客户端错误后补画面与操作证据，再评审任务状态
