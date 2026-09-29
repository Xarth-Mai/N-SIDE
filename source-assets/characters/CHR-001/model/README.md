# 曜 · 可蒙皮灰阶建模候选 r1

本目录属于 `CHR-001`，继续使用人物包中的 r3 轮廓候选与镜厅预览左侧人物海报的日漫画法要求。实际年龄 20、外观年龄约 18，当前模型的灰阶材质只用于检查形体和部件关系，不确定发色、眼色或服装颜色

## 文件职责

| 文件 | 职责 |
| --- | --- |
| `yao-grey-study.blend` | 可编辑 DCC 主文件，含分件网格、UV、蒙皮、32 骨及三个动作；灯光、地板和相机是检查辅助 |
| `grey-study.png` | 原创程序绘制的 1024×1024 sRGB 灰阶候选图集，含服装面值、接缝与轻微织纹；同时打包在主文件 |
| `build.py` | 从明确轮廓、关节环线与发束曲线重现本轮初始建模；默认会重建主文件，手工编辑后仅用 `--export-existing` 导出 |
| `check.py` | 打开真实主文件，逐帧检查骨矩阵、循环端点和鞋底接地距离，断言失败由 `--python-exit-code 1` 返回非零 |
| `game/assets/characters/CHR-001/yao-grey-study.glb` | 主文件导出的运行候选，合并为一个共享图集网格；保留 32 骨与三个 clip |

构建脚本没有导入公开人体、衣服、发型或动作模型；几何、权重、灰阶图集与关键帧是本项目本轮原创制作。它解释 r3 设计轮廓，没有把参考图像烘到模型上。概念图和镜厅图的来源及待核许可沿用上层人物包与区域预览记录，不由建模代码重新声明权利

## 形体与接口

建模使用米制、Z 向上、正面 -Y，GLB 转为 Y 向上、正面 +Z；脚底位于原点平面，根节点 `Root` 在原点。当前实测高度约 1.744 m、头部含发束约 0.267 m，高度与头身都是可调整候选，未写回人物设定

夹克通过真实袖窿边界连接衣身与袖子，头颈由同一连续网格形成；内衫、裤装、鞋、面部及薄片发束可分别编辑，手部含独立五指网格和关节权重。衣服与头部保留细分修改器，导出副本应用表面修改器后合并，主文件仍保留分件。模型只有标准不透明材质及一张 base color 图集；实际游戏的光照与描边另在真实 Bevy 路径验收

UV 使用分区图集，镜像衣片、双鞋、重复发束按同类表面复用区域。这是当前灰阶候选的有意重用；最终独特印花、脸部绘制和局部磨损需要重新分配对应岛，不把本轮 UV 当作最终角色绘画展开

| Clip | 时长 | 当前内容与责任 |
| --- | --- | --- |
| `Idle` | 2 s，30 FPS | 呼吸与微量上身运动，循环；角色世界位置仍由控制器负责 |
| `Walk` | 1 s，30 FPS | 两腿交替抬脚、对侧摆臂，in-place；实际步速与动画速度尚未匹配 |
| `Run` | 2/3 s，30 FPS | 较大的腿部轨迹、肘弯和前倾，in-place；暂未按实际疾跑速度校准 |

骨骼 `Root → Hips → Spine → Chest → Neck → Head` 连接双侧锁骨、上臂、前臂、手、五指与大腿、小腿、足、趾。蒙皮最多四权重；真实 GLB 的名称、权重、inverse bind、纹理和动作必须通过项目预检

## 复现与导出

在仓库根目录执行，以下命令可直接用于 fish；当前可用版本为 Blender 4.5.14 LTS，正式环境可把可执行路径换为同版本 Blender

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --export-existing
python3 tools/validate_character.py game/assets/characters/CHR-001/yao-grey-study.glb --root-node Root --height 1.7441 --clip Idle --clip Walk --clip Run --require-texture
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/check.py
```

重建本轮初始模型并生成 CPU 检查图使用下述命令；它会覆盖主文件，手工修改前先保存另一个版本。检查图进入 `output/characters/CHR-001/model-r1/`，查看后按项目规则清理，保留文字与 JSON

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --render
```

## 当前验收边界

本轮交付仅允许通过显式 `--character-preview` 接入作技术候选，不替换正式人物，也未通过美术或作者形象验收。root 独立复看认为肩袖与眼片比前版改善，但脸颊→下颌→颈部仍显连续锥形、五官有贴片感、袖体仍偏膨胀、衣料边缘存在锯齿，距离参考硬边赛璐璐日漫仍远。这些是下一轮实际建模与材质修订项；灰阶不是已批准配色，模型不等于 README 预览质量

DCC 检查覆盖本轮 30 FPS 关键帧与循环端点；1.5 cm 鞋底距离只作为本候选的检测阈值，不是项目长期足滑预算。实际移动速度、转向、斜坡、动画切换、碰撞与 Bevy 材质表现需要后续真实路径检查，独立运行记录归 `todo/evidence/TASK-047/model-r1/`
