---
id: TASK-049
type: asset
status: active
milestone: G2
depends_on: [TASK-024, TASK-034]
specs:
  - docs/dev/production/art-direction.md
  - docs/dev/design/locations/district-architecture.md
  - docs/dev/production/asset-pipeline.md
---

# 城市建筑外观家族与公共街景

## 目标与范围

按完整游戏城市的已定地图补足室外建筑、经营标识和街景层次，沿用建筑及场所稳定 ID、真实轮廓、楼层、入口与地形。首批以 N 站、AFTER 9 和主要来路为样板，将盒体灰盒逐步制作成用途可辨的建筑，后续依实际覆盖清单分批扩展

人物按统一日漫画法衔接环境，海报与角色分别沿用 TASK-048、TASK-047 的资产源。室内、战斗和新玩法仍后置；剧情与关系由 TASK-046 的并行工作维护，不因外观制作改写地图结构或故事

## 验收条件

- 每批真实关联源建筑、场所和门位，制作方法能复用于相同建筑用途，避免所有建筑共用同一种空盒立面
- 屋檐、门楣、墙板、窗与设备具有可信尺度和安装关系，保留公共通路、入口净空、既有房间和屋顶通行范围
- 外观所用材质与源资产可追溯，米制 UV、法线与颜色正确，运行资源与源文件一致
- 固定街景与连续移动检查轮廓、层次、遮挡、明暗及远近辨识；实际输入和状态证据与作者审美分别记录
- 适用代码、资源、工具及文档检查通过，记录本轮增量成本和未覆盖的建筑家族

## 当前工作与下一步

第 15 轮，步骤 6/6：镜厅北侧接入198m²后勤铺地，复用公共空调与路灯，保留原道路、入口和净空；四个 Blender 建筑包共用法线烘焙入口，修正 Bevy 0.19.1 忽略 glTF normal scale 的源资产差异，原几何、颜色、海报及对象关系保持。六组成功捕获共1,230帧，59项原生／60项包装断言通过，另保留一次参数失败；本轮包括曜动作，建筑和角色状态仍分别维护

完整 Rust 库142项通过、1项忽略，Viewer3项通过；实际 GLB 支承与通道、灯杆阻挡和地图62项检查通过，已实际查看后场、三处立面与连续动作。下一轮补镜厅公共屋顶可辨的边界、步行铺装与树荫座位关系，先核实完整路带的实际支承，不凭道路图判定整条路线成立

全城密度、玻璃层次、山地景观、完整屋顶与人物美术仍未达到 README 预览品质。共享法线只修正已有制作强度，未声称一级 mip 可覆盖所有距离，也未代签作者美术或 G2 放行

## 结果与证据

[全城覆盖核对](../evidence/TASK-049/coverage-r1/review.md)记录制作前的源对象与缺口；[公共外壳](../evidence/TASK-049/facades-r1/review.md)、[住宅外皮](../evidence/TASK-049/residential-r1/review.md)及[五处标识](../evidence/TASK-049/signs-r1/geometry-review.md)记录各批实际改动，[第 2 轮运行自查](../evidence/TASK-049/runtime-r2/review.md)承接 N站录制；[住宅与街树专项自查](../evidence/TASK-049/residential-r1/runtime-review.md)保存本轮机位、真实输入、状态摘要、hash 和实际观察边界

实际画面可辨 N站立柱、檐带、门楣标识和屋顶海报；V-A13 的三层住宅细部在短横移中未见整体跳位或明显互穿，首层门与小路可见。站前仍有大片浅色空地，住宅窗面平整深暗、构件重复，天空层次偏弱；住宅专项只覆盖 V-A13 正面，未验证人物碰撞与背侧入口。N站东侧真实到达路线、其他新招牌、其余住宅、全城外观、真人手感、作者审美和增量性能比较尚未验收，不计作 G2 品质放行

[第 4 轮集成与真实复验](../evidence/TASK-045/visual-r4/review.md)记录本轮实际改动、失败诊断、机器结果、已查看的画面与剩余问题

[第 5 轮集成与复验](../evidence/TASK-045/visual-r5/review.md)记录真实命令、失败修复、画面观察与仍未完成的品质项

[第 6 轮集成与真实复验](../evidence/TASK-045/visual-r6/review.md)记录本批代码、模型、实际运行、自查及尚未完成的品质项

[第 7 轮集成与实际复验](../evidence/TASK-045/visual-r7/review.md)记录坡形、连续材质、林群与人物的交付、失败诊断、最终检查和未完成的品质项

[第 8 轮集成与实际复验](../evidence/TASK-045/visual-r8/review.md)记录本批模型、材质、场景、实际运行及未完成的品质项

[第 9 轮集成与实际复验](../evidence/TASK-045/visual-r9/review.md)记录本批资产、真实运行、失败修复和停止位置

[建筑参照与全量对应](../evidence/TASK-049/reference-r1/building-references.md)记录官方、社区及媒体图源、实际观察、采用部位与覆盖边界

[月台上巷坡肩小样](../evidence/TASK-049/urban-bank-r1/README.md)记录真实实例、根部与净空、实际画面及输入隔离修复

[公共主梯扶手](../evidence/TASK-049/stair-rail-r1/README.md)与[公共砖面素材](../evidence/TASK-049/public-urban-r1/review.md)分别记录结构检查、来源与导出；各自未执行的实机范围明确保留

[Blender 建筑接入](../evidence/TASK-049/blender-integration-r1/review.md)、[实际运行自查](../evidence/TASK-049/blender-integration-r1/runtime-review.md)和[两棵街树](../evidence/TASK-049/tree-court-r2/review.md)记录源工程、替换范围、139 项库测试、Viewer 检查、实际画面与下一批缺口

[第 12 轮接入与复验](../evidence/TASK-049/blender-integration-r2/review.md)、[独立画面自查](../evidence/TASK-049/blender-integration-r2/independent-visual-review.md)与[V-A08 地形复核](../evidence/TASK-049/va08-facade-r1/ground-review.md)记录本批 Blender 与公共素材、真实三角碰撞、首跑失败修正、Wiki 环境边界及真实 Viewer 画面；台阶灰带和空旷前场继续作为未完成项，不将机器检查代签作者验收


[第 13 轮接入与复验](../evidence/TASK-049/blender-integration-r3/review.md)、[实际画面自查](../evidence/TASK-049/blender-integration-r3/visual-review.md)与[门前站位修正](../evidence/TASK-049/blender-integration-r3/cinema-landing.md)记录本批源资产、公共素材、真实输入、往返结果、失败修复和未完成范围

[第 14 轮接入与复验](../evidence/TASK-049/blender-integration-r4/review.md)、[近远景及连续帧自查](../evidence/TASK-049/blender-integration-r4/visual-review.md)与[法线源修](../evidence/TASK-049/blender-integration-r4/normal-scale-r1/source-repair.md)记录本批源资产、真实失败与修复、机器证据及未完成范围

[第15轮接入与复验](../evidence/TASK-049/blender-integration-r5/review.md)、[共享法线源修](../evidence/TASK-049/normal-scale-shared-r1/README.md)与[后勤场地整合](../evidence/TASK-049/cinema-service-court-r1/implementation.md)记录本批完整范围与实际结果
