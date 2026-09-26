# 参考图与视频分析

使用 `nside-reference-analysis` 把可观察的表现转为本轮制作要求。参考解释镜头、空间、光照、材质、节奏与反馈；本项目人物、地点、坡地关系与美术方向仍由[正式设计](../direction/demo-scope.md)和[美术方向](art-direction.md)决定

## 输入与前提

输入是可实际查看的图片、视频或游戏录制，以及本轮想解决的问题、当前场景和修改边界。记录来源文件或 URL、作者、用途和许可；可以分析不代表可以将原素材再分发，来源未知时不编造授权或生成参数

先确认可以读取媒体，再确认 `ffprobe`、`ffmpeg` 或图像查看能力。缺失片段和无法检查的时间段记为未覆盖；没有视频读取能力时使用真实解码帧，不把文字描述当成已观察证据。参考与中间帧保存在任务证据或忽略的 `output/`，不进入 `game/assets/`

## 分析步骤

1. 记录图片尺寸，或视频时长、尺寸、帧率、是否含声音；只为本轮问题抽取代表区间，保留时间与原始文件的对应
2. 先低频抽帧看整体结构，再围绕转角、动作开始、接触、结束或状态切换取连续帧；观察运动时同时查看前后文，避免用单帧推断速度和反馈延迟
3. 按镜头距离与运动、前中后景及入口、明暗与色彩、材料反差、动作节奏和输入反馈组织观察，每条附帧号或时间点
4. 将直接可见、估测和实现推测分栏记录；透视画面中的米制距离只能估测，穿墙、碰撞、隐藏任务条件与网络逻辑不能从外观推定
5. 写一个对应真实场景的改动 brief：问题、保持项、建议变化、负责文件或系统、资产需求、预期可见变化和复验方式；本轮只改变必要因素，未支持的数值留作待测假设
6. 制作后在相同场景与近似镜头、光照、分辨率和输入条件下对比；机器状态按[运行验证](../validation/runtime.md)，画面按[视觉自查](../validation/visual-review.md)分别记录

## 命令与输出

以下 fish 命令从仓库根执行，输入为已有 `viewer-tour` 录制示例；实际任务先按[运行入口](../validation/runtime.md)生成新输出，并将路径换为自己的记录

```fish
ffprobe -v error -show_entries stream=codec_name,width,height,r_frame_rate,nb_frames:format=duration -of json output/capture/viewer-tour/video.mp4
mkdir -p output/reference/viewer-tour
ffmpeg -n -i output/capture/viewer-tour/video.mp4 -vf fps=1 output/reference/viewer-tour/second-%03d.png
ffmpeg -n -i output/capture/viewer-tour/video.mp4 -vf 'select=between(n\,239\,242)' -fps_mode vfr output/reference/viewer-tour/transition-%03d.png
python3 -B .agents/skills/create-game-assets/scripts/build_preview_sheet.py output/reference/viewer-tour/second-*.png --out output/reference/viewer-tour/contact-sheet.png --columns 4 --cell-size 320
```

低频图只用于定位，连续帧 239–242 演示如何检查输入转折，不能成为其他动作的固定采样区间。`-n` 保留已有文件，重跑使用新目录；缺少视频时直接检查 capture 的 `frames/` 与 `keyframes/`

交付放在当前任务记录：参考与出处 → 带时间点的观察/估测/推测 → 改动 brief → 对应代码或资产 → 实际复验与限制。确认后的长期结论写回所属设计或制作规范，避免维护第二份独立视觉方向

## 检查与失败处理

逐条核对 brief 是否有观察依据、是否能由本轮真实场景检验，以及是否保留项目已定空间与人物。没有可见依据的效果留为建议，不写成参考的事实。媒体无法解码时保留诊断并更换可读取来源；画面不含关键变化时扩大原始时间区间，不能补画“证据帧”

参考变化若只影响文档，不要求游戏录像；落实为材质、镜头或动作修改后才运行对应路径。已读过设计和源码的评图属于 self-audit，不能将参考分析报告称为陌生玩家验证

## 来源与适配

基于 Meng To 的 [video-to-superprompt](https://github.com/MengTo/Skills/tree/a965851e27dc179e693fde1bee94457a64e1a7a5/agent-skills/codex/video-to-superprompt)及其 `references/superprompt-template.md` 改编，固定 commit `a965851e27dc179e693fde1bee94457a64e1a7a5`，Copyright (c) 2026 Meng To，[MIT LICENSE](https://github.com/MengTo/Skills/blob/a965851e27dc179e693fde1bee94457a64e1a7a5/LICENSE)

Modified for N:SIDE：采用媒体检查、抽帧、时序和分层分析，输出项目改动 brief，替换网页滚动、CSS、GSAP 与 Three.js 架构要求。完整保留声明与本地原文路径见仓库根 `THIRD_PARTY_NOTICES.md` 与 `third_party/skills/mengto-skills/`
