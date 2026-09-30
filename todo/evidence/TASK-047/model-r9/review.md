# TASK-047 · 曜后发流向与长短层次 r9

2026-10-01，基线 `f75387c19f98ad58aadb31f017ba80f35907aa22`，在已有可编辑主文件上只更新 `Hair_Cap` 与 `Hair_CrownBase` 的网格数据。没有全量重建人物、使用外部模型几何、选定新配色或修改骨骼与动作

## 交付与实际取舍

13 组后发分别指定扫向、长短、脊高与收尖位置，后脑形成两条较长主束和较短的相邻组；发根先落回原冠顶区域，再展开中下段流向。纵向曲线从 9 个样本增加到 17 个样本，减轻交叠线的折段；前刘海、鬓发、最高两根翘发及其余人物部件保持

`build.py --update-hair` 读取当前 `.blend`，保留对象、修改器、材质、权重接口与现有手工成果，只替换两个指定 mesh 后保存导出；同一 `hair_geometry` 也供生成脚本使用。实际 `Face_Head` 顶部为 1.7159999609 m，比生成稿名义值低约 18 mm，因此发面沿实际头顶定位，不能直接用生成稿坐标覆盖现有模型。完整重建仍会重建其他部件，本批复现应使用选择性入口

## 实际失败与修正

- `update-first.log` 记录替换 mesh data 后原 `Head` 顶点组不再存在的 `KeyError`；在替换前核对唯一 Head 组，替换后恢复该组及全权重，再执行保存。失败过程没有保存主文件
- 第一版四视角中，较低而外扩的发根像独立厚盖，遮住原翘发，俯视留下大片闭合面；本版未采用。`glb-first.json` 同时报告身高多出约 1.1 mm
- 恢复根部连续过渡后，`glb-revised.json` 仍检测到 1.748 m 高度；对比真实 master 与生成稿，确认上述 18 mm 头部差异，改为实际头顶参照。没有放宽身高容差或头发覆盖检查
- 高度修正后再加密同一曲线采样，形成最终 `final` 四视角；中间版日志、视图参数及 hash 保留，不能将其当作最终交付

## 检查

| 实际执行 | 结果与范围 |
| --- | --- |
| `build.py --update-hair` | PASS，见 `update-final.log`，仅替换两个指定网格 |
| 现有 `check.py --output todo/evidence/TASK-047/model-r9/dcc-check.json` | PASS，四动作、接地、脸部贴合、三图材质、发面 UV、实际细分面净距和冠顶覆盖；检查器自身的 r7 标签是既有标签，检查目标为本批真实主文件 |
| [内容保持比较](preserved-contract.json) | PASS，其余源网格坐标／面／UV／权重相等，32 骨 rest／层级、源动作曲线及其 handles／插值、全部四动作导出采样、inverse bind、材质、三张源打包图及 GLB 嵌入图字节相等 |
| `check-contract.py` 的生成器比较 | PASS，当前两块发面坐标与面列表匹配共用生成定义，比较严格只豁免两个指定对象 |
| [真实 GLB 检查](glb-check.json) | PASS，高度 1.7441905736923218 m，容差 0.00001 m；32 joints、24,149 蒙皮顶点、3 张嵌入图、4 clips，Root 世界运动为零 |
| `build.py --export-existing` | PASS，再次从当前 master 导出字节一致，见 [交付 hash](delivery.json)与 `export-existing.log` |

头发实际细分面最小净距约 5.73 mm，36 个后脑与冠顶覆盖样本通过；这些值只排查穿插和露头皮，不给日漫画风打分。GLB 从 1,498,780 bytes 增为 1,737,044 bytes，增量来自局部发面采样；没有增加材质槽或运行贴图

## 实际看图：self-audit

本工作线实际查看 before、after-first、after-offset、final 共 16 张图。四个角度为正面、后脑、侧后与俯视，沿用原灯光、Standard、CPU Cycles、2 threads、24 samples、720×960；机位、主文件 hash 见各 `*-views.json`，图片 hash 见 `delivery.json`

最终背面较基线呈现更清楚的两条长主束与相邻短组，扫向发生变化；侧后交叠线比低采样中间版平滑，正面眼眉、刘海外轮廓和脸部保持。已看四角没有新露头皮或独立悬浮发根，但仍有较宽大片感、局部尖锐交叠与俯视放射式汇聚。冠顶没有达到海报绘画品质，因此本批只交付后发层次改善，人物继续 `needs_revision`

参考输入实际查看了曜 `silhouette-r3.png`、镜厅左侧人物海报，以及本地哲官方和社区铃面部图集；图集仅帮助核对简洁五官的表现，不能从展开图推断最终脸型。本批没有下载、导入或复制这些外部图像到运行模型

独立代码与 Ponytail 审查见 [代码复核](code-review.md)。本子任务没有运行 Cargo、GPU、Windows 或手柄；真实 Bevy 连续动作和作者审美由主工作线分别记录，不以 CPU 图代替游戏验收

## 复现与交接

以下命令可在仓库根目录使用 fish 执行

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --update-hair
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py -- --output todo/evidence/TASK-047/model-r9/dcc-check.json
python3 -B tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441905736923218 --height-tolerance 0.00001 --clip Idle --clip Walk --clip Run --clip Jump --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python todo/evidence/TASK-047/model-r9/render.py -- final --master source-assets/characters/CHR-001/model/yao-grey-study.blend
```

前版本源文件与 GLB 保存在 `output/character-hair-r9/baseline/`，对应 `before.json`；`check-contract.py` 用这些真实快照做内容比较。临时图、基线和本批 `.blend1` 当前留给 root 复核，之后由 root 统一清理并记录；正式源文件、模型、脚本、日志与 JSON 保留
