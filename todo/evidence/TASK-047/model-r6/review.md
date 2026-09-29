# TASK-047 · 曜后发根部与连续分组 r6

2026-09-30，基线 `f93af27bfda9650ceb2687b822bda5367540ef1a`。实际对照镜厅预览左侧竖幅海报、人物 r3 轮廓与 r5 看图结论，处理后脑短发片像贴在光滑帽壳上的问题，沿用约 18 岁外观与当前灰阶，保持人物设定

## 制作与修正

原后发由完整光滑帽面、孤立短上层和长下层组成；本轮移除该覆盖关系，改成从共同发旋连续延伸的 13 组重叠曲面，宽度、转向与末端高度错开。根部仅用小面积闭合面，刘海与鬓发保持，未增加碎发片遮掩旧接缝

首个连续帽面候选虽然消除了孤立叶片，却呈南瓜式等距凹槽，实际看图后否定；第二轮改为独立重叠长组束，实际侧后图暴露发根间的头皮白缝，再增加小冠顶闭合面。最后俯视检查发现新闭合面的 UV 默认落到肤色区，形成白点；显式赋予头发图集 UV 后重新统一构建与渲染，白点消除

期间先在基线主文件的私有副本中只替换头发，避免覆盖并行制作的动作。最终由动作工作线统一重建正式主文件与 GLB，形体和动作共用同一份 `build.py`，没有第二套运行模型

## 实际检查

| 检查 | 结果与范围 |
| --- | --- |
| `check-crown-uv-failure.log` | FAIL，退出 1：新建冠顶面未分配头发 UV，断言定位 `Hair_CrownBase`；该候选不作为最终结果 |
| `check-uv-check.log` | PASS：私有修正模型通过 UV 区域、发束间隙、后脑与冠顶覆盖及原面部、骨架检查 |
| `dcc-check.json`、`check-final-master.log` | PASS：正式主文件的 211 个动作整帧样本、循环、5 mm 接地限、原眼口鼻贴合；发束实际细分面最小头部净距约 2.83 mm；36 条后脑射线均由头发覆盖，另查顶部闭合和 UV 区域 |
| `preserved.json` | PASS：与基线真实主文件逐项比较，63 个非头发网格的坐标、面、UV、权重和分组、32 骨父子关系与 rest 矩阵、灰阶图集字节完全保持 |
| `glb-check.json` | PASS：1.7441906 m、32 骨、3 clips、20,483 蒙皮顶点、38,898 三角面、单标准材质、单嵌入图集，权重、UV、bind 与根位移检查通过 |
| `export-existing.log`、`delivery.json` | PASS：从正式可编辑主文件重导出逐字节一致，最终源与运行文件哈希已记录 |

发束覆盖阈值只检测当前模型穿头、露头皮和图集错误，不给日漫画风或角色年龄打自动分。面数增加不是提升完成度的证据

最终主文件 SHA256 为 `298da4644c90adf0551afb7178fdee0087a5b1d7fce2339693c520a43cbdac54`，GLB 为 `5063920196cb1476e080f7d8acd6f8d16f0e55ab3cbfc94c30da9c1ad368a108`。并行的 Run 姿态修订见 [animation-r2](../animation-r2/README.md)，本轮不再声称 Run 与基线相同；Idle、Walk 和支撑链保留范围由该记录的实际 GLB 比对负责

## 看图与剩余问题

建模者实际查看基线正面、三分之四、侧面与后脑四图，并在中间轮次查看后脑、侧面、三分之四等问题视角；正式六视角全部实际查看。统一使用 CPU Cycles、2 线程、24 samples、720×960、Standard、同灯光和 Idle 第一帧，正式源哈希与机位见 `final-views.json`

后脑的小岛式短叶片已消除，发旋至发尾的连续关系清楚，侧后轮廓不再依靠浮在光滑帽面上的小片；冠顶没有此前肤色白点，嘴线与脸部没有本轮新增断裂或穿插。发组宽度与弧面仍较均匀，发根的疏密与脸部个性、衣料表现继续为待修订项，整体仍为 `needs_revision`

独立 `art_review` 实际查看正式六图，确认孤立叶岛与冠顶白点消除，侧面和侧后发束连续，没有本轮阻断回归；明确留下后脑 6—7 束的宽度与曲率过近、冠顶汇聚成光滑圆盘、侧顶接缝仍可读的问题。该反馈支持保留局部修复，不代表整个人物形象通过

root 实际查看正式后脑、侧后和俯视三图，确认冠顶白点修正；两方完成查看后才清理最终图片

这属于阅读规格后的形体自查，不是隔离盲审或作者验收。引擎材料与连续走跑见 [visual-r7](../../TASK-045/visual-r7/review.md)；本子任务未运行 Cargo、GPU、Windows 或手柄，不用 DCC 图代替实际游戏表现

## 复现

仓库根目录执行，fish 可直接使用。完整构建会覆盖主文件；已有手工修改时使用 `--export-existing`，与其他模型写入保持串行

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 -B tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/model-r6/render.py -- final --master source-assets/characters/CHR-001/model/yao-grey-study.blend
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
```

`preview.py` 与 `preserved.py` 用于本轮私有试制和基线比较；运行前创建 `output/characters/CHR-001/model-r6/`，再用 `git show f93af27bfda9650ceb2687b822bda5367540ef1a:source-assets/characters/CHR-001/model/yao-grey-study.blend > output/characters/CHR-001/model-r6/baseline.blend` 恢复精确基线，当前 `build.py` 的头发代码须与本轮交付版本一致。它们的中间 GLB 与主文件都写入该临时目录，不替代正式运行资产

`first`、`second`、`crown-open` 与 `pre-uv` 是依次被替代的中间候选，`pre-uv` 的旧 DCC 检查尚未加入 UV 区域断言，因此不能把该旧 PASS 用来覆盖后来的真实失败。一次预览脚本局部变量重名造成报告写入失败，保留 `build-second-metadata-failure.log`，改为 `hair_start` / `hair_end` 后重新执行通过，正式资产未受影响

本轮 5 个 Python 源码在不创建缓存的条件下编译通过，证据 JSON 解析、交付哈希与 scoped `git diff --check` 通过。形体代码的精简自查结论为 `Lean already. Ship.`；独立审查也没有发现需增加通用层或引擎依赖的部分

## 产物清理

本轮私有基线、中间模型、临时导出、重复探测脚本和全部检查 PNG 已在实际查看及记录后清理，合计释放 55,402,993 字节；输出目录仅保留 734 字节状态 JSON，逐文件哈希、字节数和最终六图查看范围见 `cleanup.json`。正式主文件、运行 GLB、灰阶图集、源参考、复现脚本、参数、日志和状态 JSON 保留；动作工作线产生的正式备份由其另行清理，没有跨工作线删除
