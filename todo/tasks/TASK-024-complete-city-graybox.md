---
id: TASK-024
type: asset
status: done
milestone: G1
depends_on: []
specs:
  - docs/dev/design/locations/district-space.md
  - docs/dev/design/locations/district-plan.md
  - docs/dev/design/locations/district-architecture.md
  - docs/dev/design/locations/district-station.md
  - docs/dev/design/locations/district-waterfront.md
  - source-assets/district-map/README.md
  - docs/dev/validation/runtime.md
  - docs/dev/production/asset-pipeline.md
  - source-assets/environment-kit/README.md
acceptance:
  role: codex
  revision: 1d9cf4fba259e2734dcb2a3a74fc79e272538fda87ef63cb1b3dec76aa59bbf0
  record: todo/evidence/TASK-024/r1/review.md
---

# 最新地图的完整城市建筑灰盒

## 目标与范围

按作者“继续无人值守推进，包括最新地图设计的完整城市建筑灰盒”覆盖本岸12街坊、224栋建筑、7栋对岸背景及91个场所的外部空间。直接复用地图的类型、用途层、入口、院落、平台与街道设施，保留稳定ID、楼层尺度和人物场所关系；剩余规划容量按余量处理。按随后授权加入同家族CC0公共素材，补充公园与登高路线的植被、土石和现有街道设施，沿用唯一资产清单与真实场景

## 验收条件

逐街坊核对全部建筑体量、屋面、实际外露立面、源入口及公共／住户／后勤到达，图纸已有街道设施进入真实场景。数据与运行几何都有逐项覆盖证据；每个街坊具有实际渲染，关键街口有人眼高度检查，完整测试、失败路径与连续操作证据可追溯。室内、角色碰撞、电梯功能及最终美术品质另行验收，不以灰盒覆盖代替

## 当前工作与下一步

第1轮，步骤6/6 记录与交付；建筑外部、17项设施、5种免费模型与148个增量自然物件已完成技术验收，52机位截图和两段18秒真实Viewer操作可追溯。地形任务中的第三停步台视线及TASK-025路口问题继续保留，未放行作者美术、人物通行或G1

## 结果与证据

基线 `fcf92ee799d94775fcd26812f930e9217ed9db3f`，完整交付见[验收记录](../evidence/TASK-024/r1/review.md)，制作与检查证据归 `todo/evidence/TASK-024/r1/`，原始截图录像归 `output/city-graybox/2026-09-27/r1/`。地形与台阶的续作由TASK-023保留独立历史，本任务不重复登记游戏能力完成
