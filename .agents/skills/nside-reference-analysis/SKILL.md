---
name: nside-reference-analysis
description: Analyze reference images or videos for N:SIDE camera, composition, space, lighting, materials, motion and interaction feedback. Use when turning a visual reference into a Bevy implementation brief or comparing captured behavior; preserve project art direction and distinguish observed evidence from inference.
---

# N:SIDE 参考分析

先读[美术](../../../docs/dev/production/art-direction.md)与本轮修改边界。参考用于提取表现方法，正式人物、地点和既定空间仍由本项目设计决定

1. **取得可检查的来源**：定位文件或公开链接，记录来源、用途、许可和是否允许再分发。无法读取的片段明确标记缺失，不补造画面；继续分析已有部分
2. **检查媒体**：有 `ffprobe` 时记录视频时长、尺寸和帧率；有 `ffmpeg` 时先低频抽帧，再围绕转角、动作开始/接触/结束、交互响应和状态切换抽取连续帧。图像记录原尺寸与实际展示尺度。中间产物放运行证据目录或临时目录，不进入 `game/assets/`
3. **按时间与层次观察**：镜头距离/俯仰/运动、空间层次/入口/遮挡、光照方向/明暗、材质反差/色彩、动作节奏、输入后的可见反馈。每项附文件或时间点；区分直接看到的内容、估测值与实现推测，画面不能证明碰撞或隐藏逻辑
4. **写最小实现说明**：将观察转为本轮可交付的目标、保留项、建议变化、对应场景/系统、资产需求和检查方法。镜头或反馈涉及体验差异时给少量可比较方案；不复制参考角色、独有标识或网页架构，不因为参考有高差就改变地图主数据
5. **把制作接回既有流程**：资产用 `create-game-assets` 和[资产管理](../../../docs/dev/production/asset-pipeline.md)，工程以 `game/Cargo.lock` 及真实 Bevy 入口实现。视频、3D 生成与付费服务只有在当前环境确实可用且授权时才安排；普通模型可沿现有几何和 GLB 管线制作，不强制 image-to-3D
6. **回看实现结果**：按[运行验收](../../../docs/dev/validation/runtime.md)使用同场景、近似镜头和已记录输入比较。机器断言检查状态，连续帧检查方向、遮挡、动作与可读性。已读过设计的作者或 Codex 评图标为 `self-audit`，不当作盲测或陌生玩家反馈

交付随本轮规模组织：参考来源与代表帧 → 观察/推测 → 实现要求与对应文件 → 复验路线与剩余问题。写入已有设计或任务记录，不固定生成“superprompt”网站、通用视频分析平台或第二套资产清单

## Provenance

基于 [MengTo/Skills](https://github.com/MengTo/Skills/tree/a965851e27dc179e693fde1bee94457a64e1a7a5) 改编，固定来源版本 `a965851e27dc179e693fde1bee94457a64e1a7a5`

原文件：`agent-skills/codex/video-to-superprompt/SKILL.md` 与 `references/superprompt-template.md`；资产表示选择另参考 `agent-skills/game-development/build-hybrid-game-assets/SKILL.md`

Copyright (c) 2026 Meng To — [完整 MIT 许可](../../../third_party/skills/mengto-skills/LICENSE)

Adapted for N:SIDE：保留媒体检查、时序抽帧与分层分析；将网页滚动、CSS、GSAP、Three.js 指定技术改为镜头、空间、材质、运动和真实 Bevy 验收；brief 进入已有项目文档。原始参考见[来源清单](../../../THIRD_PARTY_NOTICES.md)
