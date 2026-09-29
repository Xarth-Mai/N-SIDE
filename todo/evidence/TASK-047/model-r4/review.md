# TASK-047 · 曜眼部、连续鼻部与侧后发组 r4

2026-09-30，基线 `9503dfa`，仍为 CHR-001 的技术候选。本轮重新查看镜厅预览左侧竖幅日漫角色海报与 r3 未上色造型稿，并从已提交主文件生成四个同光照近景；没有重新选择配色、改服饰全身、改人物设定或改变动作

## 本轮形体

- 鼻梁与鼻尖进入连续 `Face_Head` 网格，沿鼻部增加两圈控制线并塑形，删除独立的四面尖角鼻片；侧面不再依赖贴在脸上的三角形表现鼻子
- 眼白保持约 0.65 mm 的贴面边界，中心形成约 2.84 mm 浅鼓面；虹膜沿同一曲面，顶部由上睑遮挡，避免整圆置于眼白中央。上眼睑与眉眼轮廓轻收，保留中性表情，未增加角色表情系统
- 发帽取消穿过头部的大封底面，侧后低缘按轮廓回收和变化高度；后发沿实际帽面设置主组。第一次三个主束仍同向且宽长接近，第二次只调转向、宽长与主次，不用多加碎发遮盖问题
- 单材质、不透明标准 GLB、灰阶图集、32 骨、inverse bind 与全部 Idle / Walk / Run 采样保持基线不变。正式 Bevy 分段光照由另一个工作线接入，DCC 的软光画面只判断本轮几何

## 实际检查

固定原有三盏灯、Standard 视图、CPU Cycles 24 samples、720×960、4 线程；正面、三分之四、侧面与后脑均为 Idle 第 1 帧，机位与输出参数见 `before-views.json`、`first-views.json`、`final-views.json`，脚本为 `render.py`

建模者实际查看三组四机位。最终鼻部尖角贴片消失，眼白与虹膜形成一致浅曲面、上睑覆盖关系可见；后脑的横切发帽改成不等高后发轮廓。侧后发组仍较大片，脸部还缺少人物个性，不能据此宣布约 18 岁日漫形象或 README 品质通过，继续 `needs_revision`

| 证据 | 实际结果 |
| --- | --- |
| `dcc-check.json` | PASS：113 个 Idle / Walk / Run 帧的骨矩阵、循环端点与鞋底接地；眼白边缘贴面及浅鼓体积、虹膜位于眼睑开口内且未穿眼白；鼻部为连续头面，鼻尖相对左右 20 mm 处面部约前突 10.45 mm |
| `glb-check.json` | PASS：1.7441906 m、32 骨、3 clips、17,754 蒙皮顶点、UV、法线、权重、嵌入图与 inverse bind |
| `preserved-contract.json` | PASS：从固定提交取 GLB，逐项比较实际骨架、bind 与全部动作采样、材质和图集；34,010 三角面，比 r3 增加 1,188，增面数不作为美术通过标准 |
| `export-existing.log`、`delivery.json` | PASS：从保存的可编辑主文件再次导出的 GLB 字节一致；Python 编译和 scoped diff 空白检查通过 |
| 首次 shader 运行输入 | root 的初始 shader 运行使用本轮 `first` 模型，精确 GLB、主文件与构建脚本哈希保存在 `first-input.json`，不与最终模型混用 |

眼白边界、鼓面与鼻部的距离阈值只约束当前候选的建模契约，不是全角色通用比例或审美评分。CPU 结果不代替 Bevy 中的材质、缩放、连续移动、实际帧率及用户形象反馈

独立 `art_review` 与 root 均实际查看最终四机位：眼部不再完整圆盘，鼻头接入连续脸面，后脑大束已有层次，未见本轮新悬空或穿脸回归；侧脸鼻额仍接近一条平线、后发帽低缘偏整齐，整体仍有通用玩偶感。本结论是已读规格的视觉自查，不是隔离盲审或作者形象验收。独立源码复查未发现明确正确性阻断，精简复查结论 `Lean already. Ship.`

## 复现

在仓库根目录实际执行以下命令，fish 可直接使用；重建会覆盖可编辑主文件，有手工修改时只使用 `--export-existing`

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/model-r4/render.py -- final
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
python3 todo/evidence/TASK-047/model-r4/check_preserved.py
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
```

本子任务未运行 Cargo、GPU、Windows 或手柄。最终 GLB 为 `0f16676224ab425aac27bcc6db05b068b0016dd3f862387674d26d9961fa42aa`，真实运行交 root 串行完成；作者形象验收尚未发生

## 产物管理

正式主文件、图集与运行 GLB 保留。建模者已查看三组共 12 幅临时近景，root 与独立评审均看过最终 4 幅；已清理全部临时 PNG、基线副本、`.blend1` 和本轮 Python 缓存，共 14,491,432 字节。保留参数、复现脚本、日志、状态 JSON、哈希和本结论，逐文件记录见 `cleanup.json`；本轮 output 仅余状态与校验文件 850 字节
