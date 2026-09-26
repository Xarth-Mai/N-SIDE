---
id: TASK-020
type: fix
status: blocked
milestone: DOCS-PIPELINE
blocked_reason: 当前 CUA 环境无可用浏览器，真实地图交互、搜索界面与画面验收尚未运行
depends_on: []
specs:
  - docs/dev/handbook/wiki.md
---
# TypeScript 工具链与 Wiki 数据加载优化

## 目标与范围

迁移自有 JS 工具、测试、Wiki 配置与 Vue 脚本到 TypeScript，优化 Bun 与 CI，分离地图和全文搜索的大数据资源

## 验收条件

严格类型检查、工具测试、文档与双站构建通过；大包警告消失；地图、搜索与发布边界通过真实浏览器验证

## 当前工作与下一步

第 1 轮实现与自动检查已完成，下一步在可用浏览器中复验地图和搜索交互；技术改动按作者要求提交，任务等待运行证据后验收

## 结果与证据

见 [本轮结果](../evidence/TASK-020/r1/result.md)，技术检查 PASS，真实浏览器验收 NOT RUN，不登记完成验收
