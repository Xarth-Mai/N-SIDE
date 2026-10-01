---
id: TASK-047
type: asset
status: active
milestone: G2
depends_on: [TASK-034]
specs:
  - docs/dev/production/art-direction.md
  - docs/dev/production/asset-pipeline.md
  - docs/player/characters/brother.md
---

# 人物制作管线与首个角色资产候选

## 目标与范围

覆盖完整游戏人物的外观制作需求，先以 CHR-001 月城曜建立设计参考、可编辑模型、贴图、骨骼、基础动作、GLB 导出与真实引擎检查的完整制作样板，再按既定角色身份扩展。造型参考、可计算模型检查和真实动画品质分别验收，不把概念图、技术代理或静态网格称为正式角色

本工作线负责人物、地图与画面；剧情与人物关系当前由 TASK-046 的并行 Agent 维护，读取其已确认资料，不自行改写故事。现有兄妹身份与年龄沿用；档案未指定的发色、体型和服装记为制作候选，交付真实参考与可运行结果后收集作者反馈

## 验收条件

- 原创候选有正面、侧面、背面及可辨认服饰层次，记录既定约束和本轮暂定参数，不把 README 匿名人物直接认定为主角
- 可编辑源工程保留真实 UV、材质与纹理、米制尺度和接地枢轴；运行 GLB 与源工程对应，引用可追溯
- 骨骼、关节引用、蒙皮权重和绑定姿态可检查；站立、行走、疾跑等本轮实际交付动作逐项记录循环、根位移和导出时间，不自动承诺整套动作已完成
- 实际引擎加载与播放动作，检查形变、接地、控制器位移归属和过渡；机器断言与画面自查分开，不以工具校验代替动作观感
- 必需资源缺失、静态模型冒充角色、无效骨骼或权重明确失败；适用格式、导出、工具和工程检查通过

## 当前工作与下一步

第 17 轮，步骤 6/6：曜 r12 只修两袖中段的截面与肘部折转，减轻圆鼓，衣身、肩环、袖口及其他网格保持；32骨、三图与四动作一致，提升及幂等21项检查通过。正式入口540帧走跑跳与暂停恢复通过，已实际查看连续动作，仍是 needs_revision 灰阶候选。下一轮转向能明显影响正常视距的服装裁片与头发层次，先比较候选再接入，不把局部修型等同作者品质认可

[R12 制作与保持检查](../evidence/TASK-047/sleeve-r12/review.md)、[R5运行结果](../evidence/TASK-049/blender-integration-r5/review.md)与[连续帧自查](../evidence/TASK-049/blender-integration-r5/visual-review.md)分别记录机器正确性与实际视觉边界。首次误用 Viewer 专用 `--aa` 参数在游戏启动前明确失败，去掉该参数后成功复验，并修正旧复现说明

## 结果与证据

工具证据见 [管线检查](../evidence/TASK-047/r1/character-pipeline-review.md)，36 位有人类年龄的角色外观目标及巧克力的空年龄见 [派生核对](../evidence/TASK-047/r1/appearance-age-targets.json)。源资产归 `source-assets/characters/` 下各 CHR 对象目录，当前造型状态以资产清单为准。[实际模型检查](../evidence/TASK-047/model-r1/review.md)与[运行接入](../evidence/TASK-047/runtime-r1/review.md)分别记录；已具备骨骼与动作候选，作者审美、正式彩色角色、转场混合及完整动作集尚未验收

本轮 [模型 r2 与实机复验](../evidence/TASK-047/model-r2/review.md)关联真实源工程、最终 GLB、保留的骨骼／动画契约、命令及运行摘要；完整游戏目标沿用[路线图](../roadmap.md)，剧情基线已提交，后续正文调整继续按 TASK-046 协调

[模型 r3 制作](../evidence/TASK-047/model-r3/review.md)与[真实走跑复验](../evidence/TASK-045/visual-r4/review.md)记录本轮实际改动、失败诊断、机器结果、已查看的画面与剩余问题

[第 5 轮集成与复验](../evidence/TASK-045/visual-r5/review.md)记录真实命令、失败修复、画面观察与仍未完成的品质项

[第 6 轮集成与真实复验](../evidence/TASK-045/visual-r6/review.md)记录本批代码、模型、实际运行、自查及尚未完成的品质项

[第 7 轮集成与实际复验](../evidence/TASK-045/visual-r7/review.md)记录坡形、连续材质、林群与人物的交付、失败诊断、最终检查和未完成的品质项

[第 8 轮集成与实际复验](../evidence/TASK-045/visual-r8/review.md)记录本批模型、材质、场景、实际运行及未完成的品质项

[第 9 轮集成与实际复验](../evidence/TASK-045/visual-r9/review.md)记录本批资产、真实运行、失败修复和停止位置

[第 12 轮社区模型接入](../evidence/TASK-047/community-reference-r1/review.md)与[哲官方包检查](../evidence/TASK-047/model-r8/official-wise.md)记录来源、失败修复、真实工具与运行证据

[Jump 资产与运行复验](../evidence/TASK-047/jump-r1/review.md)记录本批源工程、数据保持、实际命令、状态、看图、失败修复和清理

[曜后发 r9](../evidence/TASK-047/model-r9/review.md)与[实际游戏绕摄](../evidence/TASK-047/model-r9/runtime-review.md)分别记录 DCC 制作、保持项与真实加载范围；冠顶放射汇聚和大片感仍需修订，未获作者外观放行


[帽兜 r10](../evidence/TASK-047/hood-r10/review.md)记录选择更新、精确保留项、被否决的毛刺候选与最终源；[实际游戏复验](../evidence/TASK-049/blender-integration-r3/visual-review.md)承接其真实四动作和画面范围，帽兜侧缝接触、灰阶大面、脸部个性与配色继续待修

[上衣 r11](../evidence/TASK-047/jacket-r11/review.md)记录宽松轮廓候选、同机位对照、28 姿态诊断、正式回写与精确保留项；[R4 运行接续](../evidence/TASK-047/jacket-r11/runtime-continuation.md)引用真实 240／540 帧记录和实际看图结论，机器 PASS、外观仍 needs_revision
