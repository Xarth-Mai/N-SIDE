---
id: TASK-021
type: feature
status: blocked
blocked_reason: 实际站点已接入并通过 HTTPS 检查，CUA 无可用浏览器，桌面与手机交互验收仍未运行
milestone: DOCS-PIPELINE
depends_on: []
specs:
  - docs/dev/design/catalogs/quests.json
  - docs/dev/design/story-graph.md
---
# 故事时间轴与任务关系

## 目标与范围

审查全部十条主线与八条街坊故事，建立单一任务关系源、条件推演、Wiki 时间轴和逐篇详情

## 验收条件

关系语义、引用与可进入性校验通过，十八篇接入，Wiki 双站与文档检查通过；桌面、手机与键盘路径观察另留证据

## 当前工作与下一步

第 2 轮已修复本机静态服务的旧产物指向和读取权限，旧故事入口可进入新时间轴；按作者要求清理旧构建与缓存。下一步仍是在可用浏览器中完成桌面、手机和键盘交互验收

## 结果与证据

见[本轮结果](../evidence/TASK-021/r1/result.md)与[十八篇审查快照](../evidence/TASK-021/r1/audit.md)。技术检查 PASS，真实浏览器验收 NOT RUN；不登记完成验收，不增加游戏能力计数

第 2 轮[实际发布与清理记录](../evidence/TASK-021/r2/result.md)：47 项真实 HTTPS 检查通过，线上接入已生效
