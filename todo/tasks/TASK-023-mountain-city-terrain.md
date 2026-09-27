---
id: TASK-023
type: fix
status: review
milestone: G1
depends_on: []
specs:
  - docs/dev/decisions/district-baseline.md
  - docs/dev/design/locations/district-plan.md
  - docs/dev/design/locations/district-space.md
  - docs/dev/design/locations/world-research.md
  - source-assets/district-map/README.md
  - docs/dev/validation/runtime.md
---

# 山城纵剖面与地形灰盒

## 目标与范围

按作者最新反馈重做河岸、坡地街区与单一主峰：山坡起点移到小店（河岸起算剖面约374m／+28m），之后沿人工改造的坡地街坊上升，峰顶与距离按参考比例和实际街坊关系协调；保持唯一最高点。保留既有场所身份、建筑层高和局部入口关系，重排登高路线及必要的台地过渡，同步地图、Wiki 和真实 Viewer

## 验收条件

源地形、道路、建筑基底、楼层、平台和入口一致；日常路线与登山路线分别校核坡度、连通和避让；影院高低层局部关系保留；摘星台是唯一最高点，台地、山脚与山顶层次连续。站前、镜厅前街、小店门前实际渲染，记录山脊、坡上住宅、遮挡和天空；机器断言与画面自查分开，不代签美术或人物通行验收

## 当前工作与下一步

第4轮，步骤5/6 体验评审；已修复院落／道路边缘误切坡、平台承托和两处异高通路冲突，补入4处沿台阶停步台地，完成同机位对照与18秒真实Viewer录制。待判断街道层次和灰盒比例；裸坡、长台阶节奏与街坊密度仍未通过品质验收，下一轮继续本任务，不推进玩法路线图

## 结果与证据

本轮基线 `8ae11b3164d64e1c026980d9fcc2f3c1a267f3cc`，当前证据见 [r4交付与自查](../evidence/TASK-023/r4/review.md)，数值和源保护见 [geometry.json](../evidence/TASK-023/r4/geometry.json)，大体积运行产物归 `output/terrain/2026-09-27/r4/final-*`。旧 [r3城市参照](../evidence/TASK-023/r3/review.md)、[r2单峰方案](../evidence/TASK-023/r2/review.md)与 [r1双峰方案](../evidence/TASK-023/r1/review.md)保留历史范围；本轮没有人物通行、电梯交互或登山实玩验收，没有放行G1
