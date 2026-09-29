# 原创针叶树近景样板 r1

基于 `e6b398b` 的现有环境包制作 `pine-street`，范围限于新针叶树源工程、源程序、检查、GLB 与本轮证据；街树 `street-tree`、共享清单、地图、场景和游戏逻辑由原责任方维护。本记录是资产制作与 CPU self-audit，不代替根 Agent 的 Bevy 集成证据

## 现状与取舍

实际读取当前运行 `nature/tree_pineRoundA.glb`，其米制高度为 6.5m、水平最远半径为 1.694578m，204 三角，现有布置半径检查为 2.3m。已有树是一组明显分离的实心多面锥冠；同灯光、同尺寸、同三个机位重渲染了原模型，见 [inputs.json](inputs.json) 与 [render-before.py](render-before.py)

新模型用一根略弯主干、17 根不等高和不等角度主枝、68 根次枝与 426 组针叶簇形成偏心冠层。5,112 片针叶由不透明真实三角几何组成，不使用球团、锥体冠壳或 alpha 贴片。主干与分枝由原创顶点和轴向 UV 构造，复用现有原创树皮 PNG；没有借用 Kenney 几何或外部生成服务，来源、许可与正式导出方法见 [pine-README.md](../../../../source-assets/environment-kit/vegetation/pine-README.md)

新树高 6.5m、底部 Y=0、单位变换，最大水平半径 1.690000m，保持原模型最远水平圆内；这是一项保守的圆包络约束，不声称新树每个顶点均在原多面锥体内部。树冠从更高处分枝，原地图入口和净空校核无需放宽

## 制作中真实发现并修正的问题

- 第一候选 CPU 图出现洋红与蓝色针叶，检查 Blender 颜色层得到最大值约 1.449966；原因是新增 color 属性层后仍使用先前取得的 UV RNA 引用，底层层表重分配使写入落错位置。创建所有层后再按名称重新取得引用，颜色恢复四组常绿范围；不是通过改灯光掩盖错误
- 初版正确着色后枝叶过于稀疏，最终将主枝集中在较连贯上冠，把针叶簇从外端扩展到中外枝段，并调整针叶长度；仍保留自然透空和不对称分叉
- 新检查实际拒绝了倾斜根环导致的负底部高程；造型程序统一减去实际最低点后再归一化至 6.5m，最终检查通过

## 命令与机器证据

以下在仓库根目录执行，可直接用于 fish；本轮使用 Blender 4.5.14 LTS、4 线程、Cycles CPU 24 samples、720×840、AgX 与固定 seed `49031`

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --threads 4 --python source-assets/environment-kit/vegetation/pine-build.py -- --export --render-dir output/assets/task049-pine-r1/after
python3 source-assets/environment-kit/vegetation/pine-check.py
python3 todo/evidence/TASK-049/pine-r1/check-fixtures.py
```

`pine-check.py` 实际读取导出的密集 GLB accessor，有限数值、单位法线、非退化几何／树皮 UV、常绿颜色、OPAQUE 双面针叶、直接根节点、接地与包络全部 PASS，见 [geometry-check.json](geometry-check.json)。实际修改 GLB 的叶色为洋红、alpha 为 BLEND 的两个独立坏样本均以 exit 1 拒绝，见 [failure-check.json](failure-check.json)；临时坏样本由脚本自动清理

以 `--export` 打开正式可编辑主文件重新导出，GLB 与最终候选逐字节相同，见 [export-check.json](export-check.json)。所有新源文件与模型 hash 见 [outputs.json](outputs.json)，原树皮 hash 保持不变。运行派生物由根 Agent 按 [integration-entry.json](integration-entry.json) 合并共享清单后，使用现有 `bun tools/export-environment.ts` 生成；本分工没有旁路复制运行资产或关闭源 hash 验证

## 可计算成本

| 项目 | 原运行松树 | 新候选 |
| --- | --- | --- |
| 三角／实例 | 204 | 6,510 |
| 各 primitive 的 POSITION 顶点总数／实例 | 632 | 17,134 |
| GLB bytes | 14,524 | 855,836 |
| primitive／材质 | 2／2 | 2／2 |
| 内嵌纹理 | 0 | 1 张 256×512 sRGB PNG，60,358 bytes |
| 高度／最远水平半径 | 6.5m／1.694578m | 6.5m／1.690000m |

如替换此前核实的全部 96 个 `tree_pine` 实例，三角由 19,584 增至 624,960，增加 605,376；POSITION 顶点引用总数由 60,672 增至 1,644,864。7,000 三角只作为本候选的检查上限，未据此批准全城预算。CPU 渲染不能说明实际 GPU 时间、显存或移动稳定性；原模型仍可以保留为远景占位，根 Agent 根据近景、林缘效果与运行成本决定应用范围

树皮复用的是唯一源位图，不承诺不同 GLB 中内嵌同一 PNG 会被 Bevy 自动合并为同一 image 资源；也不把单份模型资源按 96 个实例重复计算文件存储

## 画面结论与后续

实际查看 before 两个角度和 after 三个角度，最终版的树干、分枝与针叶轮廓真实可见，偏心枝组替代了层叠实心锥体；侧后方仍能看见枝叶，没有以正面贴片伪装完整模型。背光角明显较暗，与同样灯光下的树干和地面一致，本轮没有为掩盖背光而重调灯光

目前是偏疏枝的修剪型树形，针叶中景略锐、枝组仍有可察觉的重复感，远景体积比原实心树冠轻；没有叶风和 LOD，不能当作完整林缘美术或最终日漫品质。真实场景需查看近景树干与叶簇、林缘整体体量、相机转动时细针的闪烁及阴影，根 Agent 随后运行既有 capture，当前不在此宣称通过

根 Agent 实际查看 before0 与 after0／1，额外 art_review 查看 before0 与 after0／1／2，CPU 自查未发现阻挡本候选集成的问题；真实 GPU 下的距离表现和细针闪烁仍待验证，不将这一轮自查等同最终美术验收

所有 CPU 视图路径、hash 与字节见 [views.json](views.json)。查看后已清理本轮 12 张临时 PNG，其中 6 张为 before／after，另 6 张是制作中候选；清理前 `ps -C blender -o pid=,comm=` 确认无 Blender 进程，没有 `pine-*.blend1`。正式源工程、纹理、GLB、复现程序、日志与 JSON 保留，根 Agent 的 capture 目录未改动；详细文件、字节与剩余占用见 [cleanup.json](cleanup.json)
