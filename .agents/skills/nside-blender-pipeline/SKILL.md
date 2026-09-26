---
name: nside-blender-pipeline
description: Author or inspect N:SIDE models, rigs and animation exports using Blender when available and the existing GLB asset pipeline. Use for mesh scale, UVs, pivots, skeletal transforms, root motion and batch export; keep art generation in create-game-assets and do not replace existing source assets.
---

# N:SIDE Blender 制作与接入

读取[美术规格](../../../docs/production/art.md)、[资产管理](../../../docs/production/assets.md)、相关资产包 README 和现有导出脚本，先确认本机 Blender 与需要的操作能力是否实际可用

1. **定技术契约**：明确资产 ID、主文件、用途、画面尺寸或米制尺度、接地/贴附/转动枢轴、朝向、材质槽、纹理色彩空间及所需动作。预算按项目资产类别和实际镜头决定，不沿用上游固定三角面数、纹素密度、60 FPS、骨骼命名或完整动作套装
2. **先审源模型**：检查轮廓、法线、UV 接缝、材料与透明边缘；只为确实需要的高模细节烘焙。把看图结论和可计算的几何检查分开
3. **按类型处理变换**：静态道具可以在导出副本中按已定尺寸归一化；绑定角色、骨骼、关键帧和关联物体先检查 bind pose 与父子空间，再决定处理。不得对所有源文件批量执行 Apply All Transforms；保留可编辑主文件和可恢复的导出过程
4. **动作按行为验收**：记录 clip 名、时长、循环边界、根位移与旋转曲线，检查权重有效和目标骨可解析。root motion 或 in-place 由玩法控制器负责方决定，避免动画与控制器重复位移；导出范围只含本轮需要的动作
5. **导出与导入**：运行资产沿用米制 GLB、标准材质和既有引用；地图坐标转换由世界模块执行，不能在模型里再转一次。批处理优先复用现有脚本，使用 Blender 背景模式时避免依赖活动视口和选择上下文，不创建通用 Add-on
6. **实际复验**：使用[工程检查](../../../game/README.md)核对模型和必需贴图，再按[运行验收](../../../docs/production/runtime-validation.md)捕获真实加载与动作。模型无动画时不声称动画通过；没有 Blender 时仍可检查既有 GLB 与引用，并明确 DCC 操作未运行

按问题查原始方法：[建模](../../../third_party/skills/skills-gamedev/references/blender-modeling/SKILL.md)、[动画](../../../third_party/skills/skills-gamedev/references/blender-animation/SKILL.md)、[工具](../../../third_party/skills/skills-gamedev/references/blender-tooling/SKILL.md)。它们是保留原文的参考资料，项目参数与导出约束采用上述映射

## Provenance

基于 [poorvith-mp/skills-gamedev](https://github.com/poorvith-mp/skills-gamedev) 改编，原文件为 `skills/blender-modeling/SKILL.md`、`skills/blender-animation/SKILL.md`、`skills/blender-tooling/SKILL.md`，固定来源版本 `e8b87e9086fbf2322b1c216c2d2de85954bf4015`

Copyright (c) 2026 Poorvith M P — [完整 MIT 许可](../../../third_party/skills/skills-gamedev/LICENSE)

Adapted for N:SIDE：采用现有米制 GLB 管线和资产登记，按资产类别确定预算，区分静态与绑定动画变换，复用项目运行验收；未采用统一清理变换、固定骨架、FBX 默认输出与独立导出插件
