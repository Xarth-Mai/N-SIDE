# 玲 · 可蒙皮灰阶全身候选 r2

本轮精修 `CHR-002` 的独立全身技术候选，实际年龄 19、外观约 17。库内没有玲已批准的造型图，因此衣装、身高和灰度分区都是试制选择；长发、图样夹、随身绘画小物依据[正式人物](../../../../docs/player/characters/sister.md)，成组发束与概括五官参考[镜厅左侧角色海报的画法](../../../area-previews/cinema-music-street.png)，不复制海报中的匿名人物

## 轮廓与来源

侧分刘海、肩胛下方长发、双片图样夹、弧形下颌、收尖眉弧和受上睑遮盖的眼开口，形成与曜不同的头部。短箱形外套与较长内衫叠穿，搭配宽松膝上裤、长袜、轻便鞋和小笔袋；绘画身份通过小物表达，衣服没有职业制服或新剧情含义

`build.py` 明确改编自 N:SIDE 自有 `source-assets/characters/CHR-001/model/build.py` 在 `b948cabe717c1c6081055267a4b0c063b7de855d` 的版本，保留已验证的 32 骨拓扑、导出方法和基础走跑计算。玲的脸、长发、发夹、服装与比例在本文件中独立制作，不读取或修改后续版本的曜源文件，也不建立通用人物生成框架

没有导入外部人物、服装、骨骼或动作模型；灰阶图集沿用该固定版本的项目原创绘制规则。参考图的来源与权利范围沿用[区域预览资产包](../../../area-previews/README.md)，它不是运行纹理，也不由本模型重新声明许可

本轮收窄肩胸并抬高衣领，保留原颈骨与上肢绑定；头部下颌改为弧形过渡，刘海按七个不同宽度的主次组侧分，背发分组宽度、转向和收尖高度分别调整。眉弧由连续采样收尖，上睑加厚外半段并留单个外眼角细节；灰阶色板没有变化

## 文件与技术契约

| 文件 | 职责 |
| --- | --- |
| `ling-grey-study.blend` | 可编辑主文件，分件网格、真实 UV、权重、32 骨和三个循环；相机、灯光与地板只用于检查 |
| `build.py` | 重建本轮候选，`--export-existing` 只导出已有主文件，保持手工编辑 |
| `check.py` | DCC 骨矩阵、循环、接地、眼口鼻贴合、头发覆盖与 UV 检查，失败返回非零 |
| `grey-study.png` | 1024×1024 sRGB 不透明灰阶图集，角色区域的面值不代表已定配色 |
| `game/assets/characters/CHR-002/ling-grey-study.glb` | 合并为一个共享标准材质的运行网格，保留骨架、权重和具名动作，内嵌同一图集 |

源文件采用米制、Z 向上、正面 −Y；GLB 为 Y 向上、正面 +Z，脚底在原点，`Root` 为单位缩放根。当前目标约 1.65 m，实测完整导出高度约 1.641 m；该差异来自长发头顶轮廓，没有曜原候选的翘发，不写回设定身高

全套网格、骨架 rest 坐标和动画位移曲线以同一系数 `0.9459975445842896` 缩放，对象缩放仍为 1。UV 不随米制尺寸变化；最大四权重，普通不透明单材质与一张 base color 图集，后续引擎光照使用项目既有角色管线

| 动作 | 时长 | 运动责任与自然速度 |
| --- | --- | --- |
| `Idle` | 2 s，60 FPS | 呼吸与微动，根保持原地 |
| `Walk` | 0.8 s，60 FPS | 每脚前 1/3 周期支撑，自然速度 `3.027192142669727 m/s` |
| `Run` | 2/3 s，60 FPS | 每脚前 1/5 周期支撑，自然速度 `5.297586249672022 m/s` |

世界位移由真实人物控制器负责；使用其他移动速度时，动画播放倍率取实际速度除以上表自然速度。不能把曜的同名动作倍率原样套用给缩小后的玲，实际导出足轨另以 120 FPS 核验

长发目前由 `Head` 与 `Chest` 混合权重带动，发夹附着 `Head`；未增加独立发骨、布料求解或二级动态。手指保留原 10 根指骨，只有基础动作姿态，不声称已具备拿画笔或对话表演

## 制作与检查

在仓库根目录执行，命令可直接用于 fish；现用 Blender 4.5.14 LTS，CPU 检查固定 2 线程

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/build.py
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/check.py
python3 -B tools/validate_character.py game/assets/characters/CHR-002/ling-grey-study.glb --root-node Root --height 1.65 --clip Idle --clip Walk --clip Run --require-texture
python3 -B todo/evidence/TASK-047/ling-model-r1/measure.py --fps 120 --gait --check --output todo/evidence/TASK-047/ling-model-r2/exported-foot-tracks.json
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/ling-model-r2/render.py -- final --master source-assets/characters/CHR-002/model/ling-grey-study.blend
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-002/model/build.py -- --export-existing
```

完整重建会覆盖主文件；手工编辑后改用 `--export-existing`。渲染命令追加 `--motion` 输出完整 Walk / Run 八个等距相位和三个跑动正视角，实际看图后按项目规则清理；保留源工程、脚本、参数、状态、哈希与结论

## 验收边界

候选保持 `needs_revision`，未批准配色、服装与人物最终形象；脸部个性、发组疏密、根部接缝、衣料细节与手部表现仍待迭代。首次整模和循环检查不等于完整人物动作集，也不替代真实 Bevy 播放、移动、转向与碰撞

本轮来源、失败与修正、DCC / GLB 数值检查及实际看图范围见 [ling-model-r2](../../../../todo/evidence/TASK-047/ling-model-r2/review.md)，唯一资产登记位于上级 `asset-manifest.json`
