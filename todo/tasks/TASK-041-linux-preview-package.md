---
id: TASK-041
type: feature
status: done
acceptance:
  role: codex
  revision: 15c67733e373c262bf8f95d35668bd2eb8585b9833afb5b4faf2e7d1500c811d
  record: todo/evidence/TASK-041/r1/review.md
milestone: G1
depends_on: [TASK-037, TASK-038, TASK-039, TASK-040]
specs:
  - docs/dev/engineering/player-preview.md
  - docs/dev/validation/runtime.md
---

# Linux独立室外预览包

## 目标与范围

复用现有发布资源布局形成可校验的Linux预览包，从独立解压目录启动与录制

## 验收条件

包内许可/运行依赖和hash齐备；无需源码路径读取实际世界、交互、设置恢复；脚本失败非零；不计完整Demo发布

## 当前工作与下一步

第1轮，步骤6/6完成；97文件本地Linux包在仓库外含空格目录通过实际启动、20秒交互和两进程设置恢复；移走包内源码仍可运行，缺地图与错误参数均返回非零，恢复文件后只读完整性检查通过

## 结果与证据

[独立包验收](../evidence/TASK-041/r1/review.md)保留输入、清单、实际动态库、真实运行及失败路径；本机资源独立性已验，跨发行版、Windows、作者体验和公开发布的完整依赖许可审计仍各按正式门槛验收
