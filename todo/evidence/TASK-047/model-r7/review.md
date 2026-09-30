# TASK-047 · 曜颈肩、前襟与发束局部精修 r7

2026-09-30，基线 `13397c5626c8952511ceda52d3c4a8877b901acc`。实际查看镜厅预览左侧海报、游戏基线 `output/visual-r9/before-yao/keyframes/frame00059.png` 与 `frame00089.png`，再对照同灯光 DCC 上身正面、三分之四、侧面与后脑。范围是现有曜模型的局部几何，不改角色身份、实际年龄 20／外观约 18 的目标及未定配色状态

## 实际改动

- 将圆领上沿从 1.416 m 提到 1.443 m，连同 T 恤上胸、夹克领口和帽兜开口调整，收窄肩袖连接；脸部和头颈网格保持，缩短露颈观感
- 删除两根独立拉链黑管，将黑色拉链带并入夹克衣缘的共享顶点、厚度和蒙皮；帽绳缩短、减细，避免多条长黑线在胸前叠成框架
- 胸前双点与横线投影到真实细分 T 恤面，横线增加必要的曲面分段，使用与衣面一致的脊柱／胸部权重；去掉原来的悬浮牌片阴影
- 后发保持从发旋开始的 13 个连续组束，改变分区宽度、偏转和尾端高度，降低等幅中脊；刘海与鬓发改为宽根逐渐收尖，中央刘海略侧偏。两束最高翘发保持原形，身高精确不变
- 完整保留 r1 三张图的源字节、主文件打包字节和 GLB 嵌入字节，以及单材质参数、32 骨 rest/bind、三个动作的全部实际采样

## 检查与失败记录

| 入口与证据 | 结果与范围 |
| --- | --- |
| `check-first.log` | 首版 FAIL：不等宽发束在 1.61 m、angle 1.8 的侧后覆盖出现空隙；未采用该版，恢复 13 组并加宽相邻组束的覆盖区后复验 |
| `check-coverage.log` | 初始 1.6 mm 印花检查界限在 Run 第 6 帧测到 1.661 mm，FAIL；该界限是本轮新设的候选值，不是既有项目契约。最终明确采用 2 mm 检测上限，并查看两侧 Run 胸前图，保留实际最小／最大值 |
| `check-final-master.log`、`dcc-check.json` | PASS：211 个动作整帧样本、循环、5 mm 接地检查、原脸部贴合、发束实际细分面净距与覆盖、UV 和三图材质；新加 10 个 Idle／Walk／Run 图形贴合姿态 |
| `dcc-check.json` | 胸前图形对衣面有向净距 0.0885—1.8652 mm；头发实际最小净距约 4.60 mm，36 个后脑／冠顶覆盖样本通过 |
| `check-contract.log`、`preserved-contract.json` | PASS：未授权改变的源网格逐坐标／面／UV／权重相等，32 骨层级与 rest 相等；真实 GLB 节点、inverse bind、所有动作采样、材质及三图字节相等 |
| `glb-check.json` | PASS：实际高度 1.7441905736923218 m，使用 0.00001 m 高度容差；32 骨、3 clips、20,479 蒙皮顶点、3 嵌入图 |
| `export-existing.log`、`delivery.json` | PASS：从正式可编辑主文件再次导出的 GLB 字节完全相同；源码与所有正式资产哈希记录在 delivery |

最终 GLB SHA256 为 `7020ed499f8feb76e987fe7997fcb126bea9680fa5cda9a56943a857b4fdee4d`，主文件为 `a8cdaa87548b3df5c7bb244c70e832536c98c3a325558b09e0162eb2675dc098`。局部净距阈值检测穿插与漂浮，不给年龄或日漫画风打自动分

## 实际看图：self-audit

固定 CPU Cycles、2 threads、24 samples、720×960、Standard、原灯光与灰阶。最终主文件的六个静态角度、Run 第 6／26 帧胸肩和 Walk 第 8 帧整身分别记录在 `final-master-views.json` 与 `final-master-motion-views.json`；只有这两份视图清单对应最终交付哈希，`first` 和 `final` 为中间候选

正面及三分之四中，领口更靠近下颌，肩袖过渡较原先集中；前襟黑边成为连续衣缘，胸前图形不再产生此前厚片阴影。侧面仍可看到帽兜较硬的折面和 T 恤下摆的厚重外扩，保留为后续衣料形体问题

两侧 Run 胸肩图中，拉链边、圆领和胸前图形连续，未看到新的穿插或脱离；Walk 整身保留既有步幅与接地关系。该图组仅检查实际骨架下的局部形变，持续走跑和实际环境材质由 root 的 Bevy 路径单独判断

后脑发尾高低与分组宽窄较基线错开，刘海收尖减轻整齐叶片感；后脑仍有较宽的平滑组束，侧后重叠边缘仍有折线化，冠顶仍可辨规则汇聚，未达到海报中的绘画完成度。脸部个性、帽兜体积、最终配色、完整动作集继续为待修订项，整体保持 `needs_revision`

root 已实际查看最终上身正面、三分之四与 Run 第 26 帧，确认前襟和胸标贴合改善，同时保持灰阶候选 `needs_revision`

没有运行 Cargo、GPU、Windows 或手柄，也没有把 CPU 渲染当作真实游戏验收。实际游戏对照与作者反馈由主工作线记录；不更新人物百科或擅自将候选转为正式角色

## 复现

仓库根目录执行。重建会覆盖可编辑主文件；已有手工编辑时只执行 `--export-existing`

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
python3 -B tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441905736923218 --height-tolerance 0.00001 --clip Idle --clip Walk --clip Run --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/model-r7/render.py -- final-master --master source-assets/characters/CHR-001/model/yao-grey-study.blend
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/model-r7/render.py -- final-master --master source-assets/characters/CHR-001/model/yao-grey-study.blend --motion
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
```

基线精确比较前创建 `output/characters/CHR-001/model-r7/`，将 `git show 13397c5:source-assets/characters/CHR-001/model/yao-grey-study.blend` 保存为其中 `baseline.blend`，将 `git show 13397c5:game/assets/characters/CHR-001/yao-grey-study.glb` 保存为 `baseline.glb`，再运行 `output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/model-r7/check-contract.py`。`render.py -- before` 可重现本轮静态基线机位

## 收尾

Python 源码编译、JSON 与限定范围 diff 检查通过；本子任务运行 `bun run check:docs` 返回 FAIL，唯一报错是并行制作中的 `source-assets/environment-kit/vegetation/ground-props.md:19` 链接尚缺 `todo/evidence/TASK-049/ground-props-r1/review.md`，保留 `docs-check.log`，不修改其他工作线文件。全仓文档复验与 Wiki／工程检查由 root 集成批次记录

建模者已查看最终九张图，root 已查看上述三张；本轮 PNG、临时基线和源目录 `.blend1` 在查看后清理，正式源文件、贴图、运行 GLB、复现脚本、日志及 JSON 保留，文件哈希、释放字节与剩余占用见 `cleanup.json`
