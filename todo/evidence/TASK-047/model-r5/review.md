# TASK-047 · 曜连续脸面与分层后发 r5

2026-09-30，基线 `e6b398b5fac899ac4639dc90fae08c4274da7aaf`。本轮读镜厅左侧竖幅日漫人物海报、人物 r3 灰阶轮廓和 r4 实际看图记录，从已提交主文件渲染相同四机位，保持灰阶图集、服装、32 骨与人物设定

## 形体与问题修复

- 在连续头面中增加鼻梁和鼻尖深度、浅眼窝与颧颊过渡，调整口鼻及下颌侧面；下颌收窄，眼开口稍内移并增高，上眼睑继续覆盖虹膜
- 后发改为三组较短上层、四组较长下层，两侧发尾各减一组，总发组数没有增长；下层根部收进头发体积，改变长短与转向
- 第一轮嘴线中间被遮断；原投影发生在头部重复缝点焊接之前，与最终细分面不同。修复为先焊接头面再投影，并沿实际嘴线逐段采样；检查从顶点扩展到面中心
- 降低后发帽时实际出现侧后穿头白块，控制点在头外仍不能保证细分面在头外。新增实际细分顶点与面中心的径向检查捕获最大约 1.58 mm 穿插；修正源控制面间距后最小净距约 1.45 mm，最终近景未见该穿插

## 客观检查与范围

| 证据 | 结果 |
| --- | --- |
| `dcc-check-second.log` | FAIL：扩展嘴线检查发现 2.74 mm 局部漂浮，不把断线图当作通过 |
| `dcc-check-cap-probe.log` | FAIL：细分后发帽约 1.58 mm 穿头，与实际白块一致 |
| `dcc-check-shape-freeze.json` | PASS：嘴线、眼白、虹膜贴合，连续鼻部，细分发帽间隙；原动作 113 帧骨矩阵、循环与接地仍通过 |
| `glb-check-shape-freeze.json` | PASS：1.7441906 m、32 骨、3 clips、18,011 蒙皮顶点，UV、纹理、法线、权重与 bind |
| `preserved-contract.json` | PASS：形体冻结时与基线逐项比较骨架、bind、全部动画样本、标准材质与灰阶图集，34,502 三角面，比 r4 增加 492，面数不是视觉质量证明 |
| `export-existing.log`、`shape-freeze.json` | PASS：从可编辑主文件再次导出的运行 GLB 字节一致，冻结版本保留精确哈希 |

以上形体冻结检查使用原动作，其 GLB 为 `1b0a31692a21098e6556115e73aa458c146d9298155bdfbc3e03eff1520b04b8`。随后 root 授权另一工作线修正实际行走脚滑，动画采样与时长已独立变化；这里的“动画相同”只针对该冻结输入，不对后续动作修订作同样声明

鼻尖相对两侧 20 mm 面部的前突由 r4 约 10.45 mm 调至本轮约 17.30 mm，眼白保持约 0.65—2.84 mm 贴面与浅鼓关系。这些是当前候选的形体量测，不是年龄或日漫程度的自动评分

## 动作修订后的最终文件

动作工作线完成 `animation-r1` 后，本子任务重新执行当前 `check.py`、GLB 校验及主文件重导出：60 FPS 的 Idle 121、Walk 49、Run 41 个整帧样本合计 211 帧通过，循环端点矩阵差为零；Walk / Run 源整帧最大支撑鞋底误差均小于 0.001 mm，Idle 呼吸位移的鞋底差约 2 mm，当前候选检查限收紧为 5 mm。整帧结果不代替半帧插值或实际脚滑测量，后两项由 [animation-r1](../animation-r1/README.md) 的动作证据负责

最终 GLB 为 `86bc6a8ce380e40204f1baaddd19008d245ffb09248fceef7355121428450b9f`，主文件为 `76b93aefb0b3ec9360ccde21cbb7771edad4c3944ed1e2e1efd8451548201861`，主文件再次导出逐字节一致，结果见 `dcc-check.json`、`glb-check.json`、`final-export-existing.log` 与 `delivery.json`。动作侧的 `preserved-geometry.json` 实际比较形体冻结输入和最终 GLB 的所有网格属性与 index、骨架 bind、材质及图集，均保持

## 实际看图

固定 CPU Cycles、4 线程、24 samples、720×960、Standard、同灯光与 Idle 第一帧，正面、三分之四、侧面、后脑共五组 20 幅均由建模者实际查看，机位详见各 `*-views.json`

最终侧脸的鼻额和下颌转折、较内聚的眼开口可以保留，嘴线已恢复连续；头后露白穿插消除，短上层与长下层开始分组。后发帽仍有整块感、发根较硬，脸部个性仍不足；本轮继续 `needs_revision`，没有作者形象验收，不宣称达到 README 画面品质

root 与独立 `art_review` 均实际查看最终四机位并确认嘴线恢复、鼻尖/唇/下颌较 r4 的改善，未见新的尖刺或穿头白块；后脑中央偏左的上层短束仍像贴在光滑帽面上的叶片，下组与根部衔接偏弱，作为明确未通过的形体问题保留。独立源码审查确认提前焊接、嘴曲线采样与细分帽面检查均直接处理本轮问题，没有新的正确性阻断，精简审查 `Lean already. Ship.`。此检查为已读规格后的自查，不是隔离盲审

## 复现

在仓库根目录执行，fish 可直接使用。构建默认覆盖主文件，已手工编辑的工程仅使用 `--export-existing`

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/model-r5/render.py -- final
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
```

形体相关构建与检查脚本的精简复查没有新增通用框架或外部依赖，结果 `Lean already. Ship.`；Python 源码编译与 scoped `git diff --check` 通过

本子任务未运行 Cargo、GPU、Windows 或手柄；引擎材质、连续走跑与动作速度由 root 及动作工作线按真实路径复验

## 产物管理

正式资产保留，全部 20 幅临时图、基线副本及本轮拥有的备份已清理，包括重复的临时排查脚本，共 19,900,446 字节，逐文件结果见 `cleanup.json`；复现脚本、参数、日志、状态 JSON 与哈希继续保留。动作工作线产生的后续备份由其负责，不覆盖或清理其他人当前输入
