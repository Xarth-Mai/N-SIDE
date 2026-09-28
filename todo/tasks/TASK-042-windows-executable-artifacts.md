---
id: TASK-042
type: feature
status: done
acceptance:
  role: codex
  revision: e4f99e6edf52e76f04a3d53dabe58093c0a14a13b8ad48bcbbc1d513ac967fd6
  record: todo/evidence/TASK-042/r1/review.md
milestone: G1
depends_on: []
specs:
  - game/README.md#github-actions
---

# Windows可执行产物与CI地形检查

## 目标与范围

扩充现有Windows Actions打包逻辑，收集本次Cargo产生的全部启用bin，沿用两个原生程序和三个启动入口；同时处理用户指出的上次CI全量地形检查超时，不新增游戏模式或更换构建配置

## 验收条件

核对本次Cargo产物收集、失败处理、三个入口及摘要逻辑，完成配置和文档检查与独立代码审查。按用户最新要求不额外下载PowerShell重演CI，Windows实际编译和启动由远端工作流另验

## 当前工作与下一步

第1轮，步骤6/6完成；工作流按本次Cargo结果收集程序并列出清单，YAML、文档、任务及独立审查通过。原CI唯一失败为全量地形检查超过5秒，单项预算改为30秒且保留全部断言，Python104项与Bun127项全部通过。用户要求停止额外本机PowerShell测试，Windows真实编译及cmd启动保持NOT RUN

## 结果与证据

[配置检查与范围记录](../evidence/TASK-042/r1/review.md)，保留既有AGENTS.md与运行验证文档的用户修改；本次不计游戏功能或阶段验收
