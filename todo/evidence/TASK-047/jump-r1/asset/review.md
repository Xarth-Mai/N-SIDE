# CHR-001／CHR-002 Jump r1 资产审查

2026-10-01，沿用曜 r7 与玲 r2 灰阶候选，在现有 32 骨、网格、绑定和材质上追加 `Jump`，两者继续为 `needs_revision`。本页记录资产检查和 DCC 姿态 `self-audit`；真实 Bevy 播放、暂停、落地衔接及作者验收由本轮运行证据分别判断

## 改动与保持项

两份 `build.py` 增加 `--update-jump`，打开当前主文件、只重作 Jump、保存并沿用现有 GLB 导出器；完整重建也包含同一动作。`--export-existing` 仍只导出已有主文件。Jump 使用 60 FPS、帧 1—41，共 `40/60 s`，包含起跳压缩、空中收腿、落地压缩与回到 Idle 首帧姿态；Root 静止，Hips 只作局部下压，世界跳跃轨迹归控制器

Jump 按非循环动作制作；GLB 没有本项目专用的循环开关，运行时是否单次播放须查实际播放器。源模型、导出物和 SHA-256 仍登记在两角色原有 `asset-manifest.json`，两份模型 README 已同步四动作契约、检查命令和更新方式

`before-hashes.json` 与 `output/jump-r1/before/` 保留本轮输入。两份旧 GLB 的字节均匹配固定提交 `b6a8c6d87b16bab17d4f05653f8e5420f06215b8`；必要时可在仓库根恢复对照输入：

```fish
mkdir -p output/jump-r1/before/CHR-001 output/jump-r1/before/CHR-002
git show b6a8c6d87b16bab17d4f05653f8e5420f06215b8:game/assets/characters/CHR-001/yao-grey-study.glb > output/jump-r1/before/CHR-001/yao-grey-study.glb
git show b6a8c6d87b16bab17d4f05653f8e5420f06215b8:game/assets/characters/CHR-002/ling-grey-study.glb > output/jump-r1/before/CHR-002/ling-grey-study.glb
```

## 命令与机器结果

Blender 为项目已有 `4.5.14 LTS`，本轮 DCC 使用背景模式与 2 CPU 线程。生成与 DCC 检查的本轮日志保持原件；收尾再次运行两份 GLB 预检与原数据对比，均退出 `0`

| 检查 | 结果与证据 |
| --- | --- |
| 两主文件更新及导出 | PASS，`build-CHR-001.log`、`build-CHR-002.log`；曜导出器仍提示多个 image node 共用 texture sampler，实际材质、纹理与图片对比保持相同 |
| DCC 原三动作与 Jump | PASS，`check-CHR-001.log`、`check-CHR-002.log` 及两份 `dcc-CHR-*.json`；Jump 41 帧有限矩阵、Root 静止、局部下压、首尾 Idle 与鞋底接地、起落压缩与收腿断言通过 |
| GLB 四动作、骨架、蒙皮、纹理与尺度 | PASS，`glb-CHR-001.json`、`glb-CHR-002.json`；Jump 导出时长 `0.6666666865 s`，根平移、首尾位移和旋转边界均为 `0` |
| 原有数据保持 | PASS，`compare.py`、`preserved-data.json` 与 `preserved-data.log`；节点、mesh accessor、索引、skin joint／inverse bind、材质、纹理、嵌入图片及 Idle／Walk／Run 通道值相同，动作集合仅增加 Jump |
| CPU 姿态渲染 | PASS，`render.py`、`render.log`、`render.json`；两角色各 4 张，Cycles CPU、12 samples、416×560、固定三分之四机位 |
| 文档与引用 | PASS，`bun run check:docs` 退出 `0`，日志为 `docs-check.log`；两份 README 的 `git diff --check` 通过 |

本轮命令如下，`--output` 显式写入本目录，避免覆盖旧模型轮次的 DCC 证据：

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --update-jump
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/build.py -- --update-jump
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py -- --output todo/evidence/TASK-047/jump-r1/asset/dcc-CHR-001.json
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/check.py -- --output todo/evidence/TASK-047/jump-r1/asset/dcc-CHR-002.json
python3 -B tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --clip Jump --require-texture
python3 -B tools/validate_character.py game/assets/characters/CHR-002/ling-grey-study.glb --root-node Root --height 1.65 --clip Idle --clip Walk --clip Run --clip Jump --require-texture
python3 -B todo/evidence/TASK-047/jump-r1/asset/compare.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/jump-r1/asset/render.py
```

| DCC 数值 | 曜 | 玲 |
| --- | --- | --- |
| 起跳阶段 Hips 最大下压 | 74.83 mm | 70.79 mm |
| 第 37 帧落地压缩 | 55.00 mm | 52.03 mm |
| 第 21 帧双鞋底高度 | 201.26 mm | 190.39 mm |
| 41 帧最小鞋底高度 | 约 −0.000023 mm，浮点误差 | 约 −0.000002 mm，浮点误差 |

这些高度相对 DCC 地板，仅用于局部姿态检查。空中脚底高度来自收腿，世界抛物线没有烘入骨架；中段鞋底数值不表示实机腾空高度

## 实际看图 self-audit

实际逐张查看 `output/jump-r1/CHR-001-jump-{1,6,21,37}.png` 与 `CHR-002-jump-{1,6,21,37}.png`，共 8 张原尺寸图，文件参数与路径见 `render.json`

| 帧 | 所见与范围 |
| --- | --- |
| 1 | 两角色均直立，鞋底接近地面，头、躯干、服装和肢体轮廓完整，能辨认各自发型与服装 |
| 6 | 双膝弯曲、身体下降，双臂稍抬，起跳压缩可见；曜裤腿与玲短裤／膝部在该机位未见明显网格断裂 |
| 21 | 双腿进一步收起、鞋底离地，双臂前摆，动作与起始姿态明确不同；所看角度未见明显手臂穿躯干或衣物破面 |
| 37 | 双膝仍有压缩，鞋底回到接近地面的位置，能辨认落地恢复阶段；数值表明此帧仍有约 6.4—6.7 mm 鞋底间隙，不能将其写成严格贴地 |

抽样姿态自查 PASS。两角色上身变化较克制，主要靠腿部收起和手臂摆动表达跳跃；这是当前灰阶技术动作的观察，不构成人物表演质量批准

本页未观看连续 DCC 动画，四帧不能证明中间插值、动作节奏、转场混合或动态穿插完全成立；未覆盖背面、极端相机与长发二级运动。实际跳跃高度、触地时刻、暂停恢复、移动跳与落地切换转由真实运行证据核对，本资产子项的作者／陌生玩家验收为 NOT RUN

## 精简审查与收尾

独立只读审查按 `ponytail-review` 检查本轮两个模型源码、manifest、README 和资产证据；未发现需删除的新增依赖、通用框架或重复台账。结论：`Lean already. Ship.`。复查建议已落实为四动作说明、`--update-jump` 用法、显式 DCC 输出路径和固定基线恢复命令

已先看图、记录结论并生成 [清理记录](cleanup.json)，其中保存 8 张 PNG 与两份本轮 `.blend1` 的删除前 SHA-256、大小和归属。两份 `.blend1` 均匹配本轮输入主文件哈希；宿主 `fuser` 对 10 个精确路径返回无占用，随后核对哈希并删除这 10 个文件，共释放 `9,921,552 bytes`。本资产子项的视觉产物已清理，源码、正式资产、`before/` 基线、复现脚本、日志和状态 JSON 保留；`output/jump-r1/` 剩余文件和大小见 `cleanup.json`
