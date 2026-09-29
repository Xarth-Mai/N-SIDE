# 曜 · 可蒙皮灰阶建模候选 r3

本目录属于 `CHR-001`，继续使用人物包中的 r3 轮廓候选与镜厅预览左侧人物海报的日漫画法要求。实际年龄 20、外观年龄约 18，当前模型的灰阶材质只用于检查形体和部件关系，不确定发色、眼色或服装颜色

## 文件职责

| 文件 | 职责 |
| --- | --- |
| `yao-grey-study.blend` | 可编辑 DCC 主文件，含分件网格、UV、蒙皮、32 骨及三个动作；灯光、地板和相机是检查辅助 |
| `grey-study.png` | 原创程序绘制的 1024×1024 sRGB 灰阶候选图集，含服装面值、接缝与轻微织纹；同时打包在主文件 |
| `build.py` | 从明确轮廓、关节环线与发束曲线重现本轮初始建模；默认会重建主文件，手工编辑后仅用 `--export-existing` 导出 |
| `check.py` | 打开真实主文件，逐帧检查骨矩阵、循环端点和鞋底接地距离，并检查五官贴面间距，断言失败由 `--python-exit-code 1` 返回非零 |
| `game/assets/characters/CHR-001/yao-grey-study.glb` | 主文件导出的运行候选，合并为一个共享图集网格；保留 32 骨与三个 clip |

构建脚本没有导入公开人体、衣服、发型或动作模型；几何、权重、灰阶图集与关键帧是本项目本轮原创制作。它解释 r3 设计轮廓，没有把参考图像烘到模型上。概念图和镜厅图的来源及待核许可沿用上层人物包与区域预览记录，不由建模代码重新声明权利

## 形体与接口

建模使用米制、Z 向上、正面 -Y，GLB 转为 Y 向上、正面 +Z；脚底位于原点平面，根节点 `Root` 在原点。当前实测高度约 1.744 m，r3 保持约 1.744 m 高度与骨架，修订眉眼、发组、耳部和肩袖、肘膝与下摆轮廓；高度与头身都是可调整候选，未写回人物设定

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

重建本轮模型并生成 CPU 检查图使用下述命令；它会覆盖主文件，手工修改前先保存另一个版本。检查图进入 `output/characters/CHR-001/model-r3/`，查看后按项目规则清理，保留文字与 JSON

```fish
output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python source-assets/characters/CHR-001/model/build.py -- --render
```

从现有主文件生成本轮固定正面脸、三分之四脸、侧脸与全身对照，使用 `output/tools/blender-4.5.14-linux-x64/blender -b --python-exit-code 1 --python todo/evidence/TASK-047/model-r3/render.py -- final`；它保持主文件的灯光、灰阶、720×960 分辨率与 24 个 CPU Cycles 采样，不保存相机改动；追加 `--motion` 生成侧身及 Walk / Run 三分之四检查图

## 当前验收边界

本轮交付仅允许通过显式 `--character-preview` 接入作技术候选，不替换正式人物，也未通过美术或作者形象验收。r3 沿用 r2 头颈修订，收眼睑、改短中性口线、收耳并组织主副刘海；发帽沿实际高度收深度，避开发际折回和穿出刘海。外套收肩袖，在肘、膝、裤脚保留局部折形，T 恤下摆在裤腰之外，袖口覆盖布边。灰阶图集、材质、骨架、inverse bind 与所有动画采样保持 r2 不变

本轮 DCC 和 GLB 数值检查、同机位观察及剩余问题见 [r3 记录](../../../../todo/evidence/TASK-047/model-r3/review.md)。脸部个性、侧后发帽切边、衣料细节和硬边赛璐璐表现继续为待修订项；灰阶不是已批准配色，模型不等于 README 预览质量

DCC 检查覆盖本轮 30 FPS 关键帧与循环端点；1.5 cm 鞋底距离只作为本候选的检测阈值，不是项目长期足滑预算。实际移动速度、转向、斜坡、动画切换、碰撞与 Bevy 材质表现由真实路径单独检查，不能由贴面距离或 DCC 渲染推定
