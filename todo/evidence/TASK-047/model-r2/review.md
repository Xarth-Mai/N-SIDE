# TASK-047 · 曜脸颈局部修订 r2

2026-09-30，仅落实 [r1 下一轮建议](../model-r1/next-pass.md) 的第 1、2 项；实际查看镜厅左侧竖幅人物海报和 CHR-001 r3 黑白全身源图，保持实际年龄 20、外观目标约 18、未批准灰阶与现有服装轮廓

## 源文件与修改

修改前逐字节比较 `build.py`、可编辑 `.blend`、灰阶 PNG、运行 GLB 和两张参考图与 HEAD，全部一致，无未提交手工建模需要保留；原始 SHA-256 见 `input-hashes.json`

- 在原有连续头颈网格上调整颈上缘、颌下和外下颌四圈，分别改变前、侧、后顶点高度与半径，使前下巴回收后接入颈柱；保留原有头部整体下移 18 mm
- 眼角收尖、下眼线缩短，眼白至高光的离面距离从原 1.8–4.1 mm 收近至约 0.9–1.9 mm；鼻根与短口线复用真实面部 `face_y`，鼻尖仅保留小体积
- 首次收近眼白后，原三角扇内部穿入弯曲脸面，实际画面出现灰条；在既有 `patch` 增加一圈中间贴面顶点，复验最终图已消除灰条，没有通过移动灯光遮掩
- 主文件仍有可编辑分件、32 骨与 `Idle / Walk / Run`；运行文件保持一网格、一 primitive、一材质、一张嵌入 PNG。节点变换、全部动作采样值、材质与图集字节同 r1，见 `preserved-contract.json`

## 同机位实际观察

使用主文件原有三盏面积灯、灰阶、Standard 视图、720×960、CPU Cycles 24 samples；脸部三个机位 `ortho_scale=.39`，全身 `2.06`。精确相机位置与帧号见 `before-views.json`、`after-views.json` 和 `final-views.json`，复现脚本为 `render.py`

建模者实际查看 r1 正面、三分之四和侧面，确认脸颈连续锥化、口线悬空、鼻部突起；查看 r2 首次四机位后发现眼白穿面，再查看最终四机位确认修复。正面已有独立下巴，三分之四与侧面能读到颌下转折，嘴线不再悬空，鼻根更贴近脸部

root 实际查看 r1 侧面与 r2 最终四图，结论为“侧面颌下/颈柱分开、鼻嘴贴面确实改善，可保留这个局部修订”。这是局部几何改善的独立 self-audit，不是作者形象验收

root 同时指出整体仍像简化玩偶：眼白轮廓过圆平直、眉眼层次少、嘴固定笑弧、厚发片节奏重复，全身肩袖尤其夸张。本轮不把约 18 岁日漫形象记为成立，`AST-008` 继续 `technical-preview / needs_revision`；后续仍需脸部个性、发束主次、布料轮廓与硬边赛璐璐表现

## 实际检查

以下命令在仓库根目录实际执行，完整输出保存在本目录对应日志与 JSON

```sh
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/model-r2/render.py -- final
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
```

- `dcc-check.json`：PASS，Idle 61 帧、Walk 31 帧、Run 21 帧的骨矩阵、循环端点和鞋底距离；贴面检查覆盖五官顶点及眼片面中心，眼白最小离面约 0.583 mm、口线最大约 1.297 mm、鼻尖最大约 4.711 mm
- `glb-check.json`：PASS，实际 accessor、法线、UV、权重、inverse bind、32 骨、三个 in-place clip、一张嵌入图与 1.7440556 m 高度；三角形数 29182
- `preserved-contract.json`：PASS，实际解析 r1/r2 GLB 的节点及全部动画采样值后逐项比较，灰阶 PNG 的 SHA-256 保持不变
- `export-existing.log` 与 `delivery.json`：PASS，从现有可编辑主文件再次导出的 GLB 与本轮构建导出字节一致
- 真实 Bevy 近景、连续动作与贴面闪烁由 root 独立运行，本页 DCC 结果不替代运行记录；作者对约 18 岁外观、形象与材质的验收未发生

## 产物与清理

源 `.blend`、正式灰阶图集、运行 GLB、构建与检查脚本、输入输出哈希、日志、相机参数和本结论保留。r1 对照、r2 已修失败图与最终四机位暂存 `output/characters/CHR-001/model-r2/`，已交 root 完成查看后统一清理；清理记录由 root 补充，不将未清理写成已清理

## 正式入口运行复验

Root 使用 `--character-preview` 与 `game/capture/walk-character.json` 完成 540 帧／18 秒真实控制、骨骼动画及暂停／恢复，26 项机器检查 PASS，详情见 [runtime-summary.json](runtime-summary.json)。已实际查看141—142连续步行帧、159步行、209疾跑及314回转帧，确认修改模型仍由同一控制器与真实动画驱动，衣袖和发束的简化感仍可见；近脸判断另依据本轮前后 DCC 图，不把远距离背面录制当作面部审美验收。当前仍为 `needs_revision`，Windows、原生手柄与作者审美反馈未运行

本轮视觉清理已完成：确认无游戏、录屏或Blender进程使用文件后，root删除已查看的截图、连续PNG、关键帧与视频，以及本轮24468bb临时素材快照。正式模型、贴图、源工程、参数脚本、日志、状态JSON及文字结论保留；数量、字节与归属见 `todo/evidence/TASK-045/integration-r3/cleanup.json`，街树CPU图另见其 `cleanup.json`
