---
id: TASK-023
type: fix
status: review
milestone: G1
depends_on: []
specs:
  - docs/dev/decisions/district-baseline.md
  - docs/dev/design/locations/district-plan.md
  - source-assets/district-map/README.md
  - docs/dev/validation/runtime.md
---

# 山城纵剖面与地形灰盒

## 目标与范围

按作者最新反馈重做河岸、坡地街区与单一主峰：城区上缘约80m、山脚90m，摘星台设为450m最高点；收紧城区到山脚的空缓坡。保留既有场所身份、建筑层高和局部入口关系，重排登高路线及必要的台地过渡，同步地图、Wiki 和真实 Viewer

## 验收条件

源地形、道路、建筑基底、楼层、平台和入口一致；日常路线与登山路线分别校核坡度、连通和避让；影院高低层局部关系保留；摘星台是唯一最高点，台地、山脚与山顶层次连续。站前、镜厅前街、小店门前实际渲染，记录山脊、坡上住宅、遮挡和天空；机器断言与画面自查分开，不代签美术或人物通行验收

## 当前工作与下一步

第2轮，步骤5/6 体验验收；已交付450m单峰、小店至山脚约460m路线及固定种子自然山面细化，完成三处水平人眼视点、山侧、全景与18秒连续录制。等待作者判断山体高度、街坊上升和水平紧凑程度，具体画面、启动方式与技术检查见[第二轮记录](../evidence/TASK-023/r2/review.md)；不放行G1或自动推进后续玩法

## 结果与证据

基线 `8365df245375fb9aa70ff5feccf2273e5cacf5ec`，期间新增的 `85f2ecb` TypeScript配置提交完整保留；本轮结果归 `todo/evidence/TASK-023/r2/`，双峰方案和作者否定反馈保留在 [r1](../evidence/TASK-023/r1/review.md)，最终大体积渲染产物归 `output/terrain/2026-09-27/verified-*`。当前不是人物碰撞或登山操作验收，摘星台两面夜景的成因继续单独管理
