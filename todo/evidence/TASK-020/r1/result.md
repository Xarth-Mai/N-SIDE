# TypeScript 工具链与 Wiki 数据加载结果

## 输入与范围

本轮按作者要求迁移自有 JS 工具、测试、Wiki 配置与 Vue 脚本，并优化 Bun、CI 和大包资源；作者随后要求本轮 Bun／TS／Wiki 依赖全部采用最新正式版，Rust、Python 与第三方 Skill 代码保持原范围

源输入内容 hash 与逐文件摘要见 source-hashes.json，版本见 versions.json；体积基线取自本轮编辑前现存双站产物，不将其解释为实时网络或浏览器性能测量

## 交付

- 自有代码迁移为 TypeScript，严格检查工具与组件，CLI 和活动文档统一使用 .ts 路径
- TypeScript 7 检查工具；Vue 使用最新 vue-tsc 与微软 TypeScript 6 兼容 API，在 Node 子进程执行必要的 require 钩子；反向 SFC 测试证明类型错误实际被报告
- 地图 JS 保留地点索引，绘图从公开 map.json 加载，含加载占位、失败重试与离页取消；完整设计源继续仅发布到开发站
- 全文搜索索引作为带内容哈希的 JSON 按需读取，生产适配器核对上游模块契约，开发模式保留原生实现；产物校验核对搜索路由的受众范围
- Bun 版本统一由 package.json 提供，构建不重复安装依赖，CI 纳入类型检查；保留默认 500 kB 警告阈值

## 检查结果

| 命令或检查 | 结果 | 范围 |
| --- | --- | --- |
| bun install --frozen-lockfile | PASS | 冻结锁文件，无变更 |
| bun run check:types | PASS | TypeScript 7 工具与测试、Vue 脚本与模板 |
| bun run test:tools | PASS | 103 项 Python、82 项 Bun 测试，含 Vue 错误反例及搜索加载契约 |
| bun run check:docs | PASS | 文档、ID、链接与 Skills 本地完整性 |
| bun run tasks:check | PASS | 任务与看板一致性 |
| bun run check:narrative / check:story-design / check:templates | PASS | 叙事、内容关联与模板 |
| bun run docs:build | PASS | 88 个玩家页、143 个开发站页，无当前大包警告 |
| python3 todo/evidence/TASK-020/r1/http-probe.py | PASS | 26 项真实 HTTP 检查，覆盖两站开发与预览、JSON、搜索及受众隔离 |
| 实际 MiniSearch 索引查询 | PASS | 两站搜索“最后的玩具”返回正确正文与锚点，玩家站不含开发路由 |
| ponytail-review | PASS | 删除未使用的类型导入与重复构图计算后复查：Lean already. Ship. |
| 真实浏览器交互和画面 | NOT RUN | CUA 清单 browsers 为空，创建 iab 返回 Browser is not available: iab |

构建和类型检查日志、HTTP 记录与体积摘要保存在本目录。HTTP 启动需要沙箱外监听权限，测试仅访问本机；脚本结束后停止本轮启动的服务器

## 体积结果与解释

街区页面常规 JS 从 1,333,387 bytes 降至约 37 KB，lean 版本从 1,322,396 bytes 降至约 26 KB。两站现有 JS chunk 均低于默认 500 kB 阈值；地图和全文索引正文保留为独立 JSON，JSON 体积与 gzip 结果见 sizes.json

玩家包与开发包的 JS 总量包含普通与 lean 两份页面，不代表单次访问传输量；本轮未测浏览器首屏时间、FPS 或真实网络速度

## 尚缺验收

在可用浏览器打开双站，检查地图请求失败与重试、地点选择、缩放、拖动、键盘、全屏、直接访问与站内导航；检查搜索首次打开、重复打开、正文命中和锚点跳转，观察网络、控制台及画面。当前任务保持 blocked，不以静态检查代替真实交互验收
