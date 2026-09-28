---
id: TASK-023
type: fix
status: active
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

第6轮，步骤6/6 记录与后续修订；第三处+410m停步台保持平台和眼位，通过三处近坡控制点恢复城区与河岸视线。地形射线回归修前FAIL、修后PASS，9机位和18秒连续转向已实跑并看图；1.31km短登高路线及450m唯一峰顶保留，短路线实际路幅候选仍为0。全城其余路幅候选由TASK-025接续；人物登山、最终美术与G1保持未验收

## 结果与证据

第6轮[近坡修复、连续运行与实际看图](../evidence/TASK-023/r6/review.md)保留源hash、修前失败、修后检查和机器／画面／试玩边界

后续TASK-033完整往返暴露下山局部大褐色地形折面，挡墙蓝洞已独立修复，地形折面与坡阶视觉过渡留待本任务精修，见[往返画面与最终挡墙对照](../evidence/TASK-033/r1/review.md)，不改变本任务阶段与验收状态

第5轮基线 `fcf92ee799d94775fcd26812f930e9217ed9db3f`，本轮证据见[r5源数据、运行与剩余问题](../evidence/TASK-023/r5/review.md)。第4轮基线 `8ae11b3164d64e1c026980d9fcc2f3c1a267f3cc`，历史证据见 [r4交付与自查](../evidence/TASK-023/r4/review.md)，数值和源保护见 [geometry.json](../evidence/TASK-023/r4/geometry.json)，大体积运行产物归 `output/terrain/2026-09-27/r4/final-*`。旧 [r3城市参照](../evidence/TASK-023/r3/review.md)、[r2单峰方案](../evidence/TASK-023/r2/review.md)与 [r1双峰方案](../evidence/TASK-023/r1/review.md)保留历史范围；本轮没有人物通行、电梯交互或登山实玩验收，没有放行G1
