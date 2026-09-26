---
id: TASK-021
type: feature
status: blocked
blocked_reason: 完整站点及独立事件页已发布，浏览器桌面与手机交互验收仍未运行
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

第 5 轮已将 87 篇百科正文迁至 player 直属目录，原文保留并兼容旧地址；519 项线上检查覆盖全部当前页面、旧 URL 与资源。下一步仍是在可用浏览器中补齐桌面、手机与键盘交互验收

## 结果与证据

见[本轮结果](../evidence/TASK-021/r1/result.md)与[十八篇审查快照](../evidence/TASK-021/r1/audit.md)。技术检查 PASS，真实浏览器验收 NOT RUN；不登记完成验收，不增加游戏能力计数

第 2 轮[实际发布与清理记录](../evidence/TASK-021/r2/result.md)：47 项真实 HTTPS 检查通过，线上接入已生效

第 3 轮[完整站点恢复与事件拆分](../evidence/TASK-021/r3/result.md)：修正第 2 轮的发布受众选择，完整保留百科和开发资料

第 4 轮[Cloudflare 发布目录修复](../evidence/TASK-021/r4/result.md)：按用户要求恢复完整站的原有构建路径，显式维护部署配置，远端重试待执行

第 5 轮[玩家百科目录扁平化](../evidence/TASK-021/r5/result.md)：87 篇正文迁至 player 直属目录，更新引用并保留旧地址跳转，内容文字保留检查通过

第 6 轮[浏览器图标兼容](../evidence/TASK-021/r6/result.md)：补齐根 favicon.ico 与 Apple 图标，保留 SVG 图标及品牌原件
