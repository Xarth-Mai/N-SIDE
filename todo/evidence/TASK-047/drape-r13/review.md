# 曜 r13 下背衣片垂坠候选

基于 R5 提交 `60cfed4` 的正式 r12 主文件，仅在 `output/yao-drape-r13/` 制作独立候选；未改正式源、运行 GLB、生成器或清单，输入完整提交及五个文件 hash 见 [冻结记录](source-baseline.json)

## 本轮制作

`Jacket_ContinuousShoulders` 的下背原来是从侧腰到下摆的大块平滑曲面；本次沿两侧从约 z1.17 到 z1.045 的方向形成两道宽褶面，使中央垂面与侧腰转折可区分，保持灰阶和全部原造型组成

为承载折面，仅将衣身原 row2→3 的 32 个四边面各分成四段，插入 99 个支撑点；原 519 个控制点中只改下背两圈内的 36 点，新增点中 46 点产生褶面位移。最大向外位移约 20.12 mm、向内约 4.47 mm，没有整体缩放或扩大人物

保持原门襟顶点、衣摆边界、肩袖、帽兜贴靠的上背及 r12 两袖；旧点的 UV／权重、其他物体、三图、32 骨与四动作保持。新增 UV 和权重由原衣片两端线性插值，不引入衣骨、布料模拟或新材质

## 检查与失败记录

- 首次替换 mesh 后，Blender 清空了对象的顶点组定义，赋权时立即报错，未保存候选；[原失败日志](candidate-initial-failed.log)保留。显式恢复原组名、顺序与锁定状态后，候选重建成功
- [保持与姿态比较](preserved-contract.json)：20 项 PASS，非目标网格、图片、骨架、动作、GLB 材质／绑定／全部动作样本精确一致；原边界与旧权重保持，其他面 UV／材质不变，无新增非流形边；正式文件 hash 保持
- 四动作 28 个实际蒙皮姿态中，头、发、领口与手掌 overlap 为 0；帽兜／T 恤／袖口原有最大接触对保持 269／180／68，不把接触统计当作视觉验收
- [GLB 检查](glb-check.json)：PASS，蒙皮顶点从 24381 增至 25197，32 骨、四动作、三图与 1.7441905736923218 m 身高保持
- Bevy 运行：NOT RUN，候选尚未接入正式运行路径

## 实际看图

先实际查看固定 CPU 灯光的 before/candidate 正面、侧前、后侧与背面八张图，640×480 输出、人物约 300 像素高，参数/hash 见 [静态图记录](views.json)

背面与后侧能看见两道斜向折面，较 r12 大块平滑后背更可读；前襟、袖子与人物外部比例基本保持。折面仍可能读成圆鼓而非充分成立的衣料垂坠，需要独立复查；本轮不宣称整套服装、日漫形象或作者外观已通过

随后实际查看 Run6、Run26 和 Jump21 的 before/candidate 六张背面图，参数/hash 见 [姿态图记录](motion-views.json)：左右摆臂及收腿时后背折面仍可读，所看姿态没有新开洞、T 恤穿出或袖口分离。对称弧形仍有软鼓感，这组离散 CPU 图不证明所有动作插值或真实游戏材质效果

这一阶段未提升 A，继续独立看图；普通全身显示尺寸下的可见净收益是接入依据，机器检查通过不自动接入

## 独立复查与唯一一次 r13b 修正

art_review 独立实际查看 A 的 back／back-quarter／Run6 三对六图，认为普通全身尺寸下差异可见，但两侧弧形亮脊更像两块软鼓包，未清楚读成主布片折转；Root 查看 candidate-back-quarter 后同意，未提升 A

Root 只允许一次针对性修正：保留 A 的拓扑和下摆固定，将 +X 一侧改成窄的内凹主折及浅外侧面，另一侧强度为 23%，不增加随机小褶。候选 B 从 r12 基面恢复坐标后一次形变，最大内收约 15.87 mm、向外约 6.80 mm；[形变记录](geometry-change-b.json)、[20 项保持检查](preserved-contract-b.json)和 [GLB 检查](glb-check-b.json)保存实际结果，28 姿态接触最大值未增加

B 只新增渲染 back／back-quarter／Run6 三张图，使用已冻结的 r12 同机位、同灯光原图组成三对，参数/hash 见 [B 图记录](views-b.json)。制作方实际查看后判断：两块对称鼓包已消失，但普通全身尺度主要剩下一点单侧明暗，尚未清楚建立有价值的衣片大形

## 最终选择

Root 实际查看 A／B 后拒绝接入，正式模型保留 r12；art_review 的[独立画面自查](independent-visual-review.md)分别实际查看 A 与 B 的三对同机位图，结论一致。有限姿态未见新开洞或穿插，但不能因此收录没有明确美术净收益的候选

本轮实验到此完成，不再追加参数迭代；没有正式资产增量，也不增加人物或游戏能力完成数。TASK-047 保持 active，日漫外观与作者审美验收继续待评；[下一项只读建议](next-art-brief.md)转向前额发组与额头留白，不继续微调下背衣片

## 复现与临时产物

[candidate.py](candidate.py) 只打开本轮 before 副本并保存 candidate，使用冻结生成器的既有 exporter；[check.py](check.py) 复用既有精确保持和 28 姿态检查；[render.py](render.py) 沿用固定双线程 CPU 相机与 12 samples，未将摄影设置保存回模型

临时输入已清理，复现时先由 `source-baseline.json` 中完整提交 `60cfed49aa5f9fc31034184c084d625596314261` 的 CHR-001 `.blend`、GLB 和 `build.py` 恢复为 `output/yao-drape-r13/before.blend`、`before.glb` 和 `generator-before.py`，核对记录 hash 后再执行下列命令；不能直接使用后续变化的正式源冒充本轮输入

```fish
env ALSOFT_DRIVERS=null output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/drape-r13/candidate.py
env ALSOFT_DRIVERS=null output/tools/blender-4.5.14-linux-x64/blender --background -noaudio --threads 2 --python-exit-code 1 --python todo/evidence/TASK-047/drape-r13/check.py
```

实际查看与最终决定记录完成、相关进程全部退出后，已清理本轮 17 张图与 7 份临时候选／输入副本，共 24 个文件、24,032,601 字节，`output/yao-drape-r13/` 已移除；完整路径、大小、hash、正式输入保持与可恢复来源见 [清理记录](cleanup.json)。保留脚本、运行日志、JSON 和文字结论，正式源、运行资产与参考图未变

本轮只增加这一处衣片的受控候选，没有新增通用衣服系统、模拟器或配置层；按 `ponytail-review` 自查：Lean already. Ship.

最终 `bun run check:docs` PASS（610 Markdown、129 IDs、31 Skills／25 imports），见 [检查日志](docs-check-final.log)；任务卡差异检查通过，五个正式输入重新逐字节核对 hash 保持，临时输出目录不存在。TASK-047 已记录第 18 轮实验结论，看板同步由 Root 统一执行
