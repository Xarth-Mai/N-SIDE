# BYTE BEAT 北立面

`AST-V35-FACADE` 为地图 `V-35` 的原创北立面，唯一主文件为 [facade.blend](facade.blend)，[build.py](build.py) 从编辑主文件直接导出 `game/assets/environment/buildings/v35-byte-beat-facade.glb`，同时保留可重建制作配方。源资产已批准进入 R4 场景接入，状态为 `exported`；原版 R4 实机暴露灰泥法线过强，本轮已完成源修与导出，修复版入口与总览 GPU 复验通过，支持保留本次局部修复，作者美术验收待完成

## 规格

- 源尺度为米，Blender 使用 X 东、Y 北、Z 上，锚点为源公共门 `game_entry = [-386,153,12]`；GLB 按标准 Y-up 导出，游戏仅执行一次 `map_to_world`，不再旋转或缩放模型
- 北面宽 32 m，总高 11 m；1F 高 6 m、2F 高 5 m，原地图轮廓、南侧后勤门、楼层与用途保持不变
- 包含完整北面附着表皮、2 m 公共门、1.5 m 连续雨棚、八组深窗、两个浅展柜和六台原创设备大形、粉紫与黄绿色的少量经营图形、门东侧候场边界
- 展柜为封闭浅景，设备没有玩法、动画或虚构游戏名；上层深窗与楼梯侧窗服务外观，不增加可进入室内
- `source-assets/district-scene/byte-beat.svg` 及其独立运行时招牌保持原样，预留 x ±3.08 m、z 3.17–4.83 m、北向深度 0.12–0.366 m；运行 GLB 不包含重复招牌
- 中央 6 m 来路保留，雨棚最下构件高于 2.64 m，候场边界位于门东 5–11.5 m；接入时按实际 GLB 几何验证碰撞
- 预算小于 8,000 三角面、七个材质角色、两张原颜色 JPG 与两张同尺寸衍生法线 PNG；无骨骼、动画、材质扩展或运行脚本

## 接入契约

运行模型 ID 为 `v35_byte_beat_facade`，加载 `Scene0`，地图锚点 `[-386,153,12]`，rotation identity、scale1；来源为 `buildings[V-35]/blender-attachment`。北面泛型窗、腰线、压顶、旧公共门构件及原北侧雨棚由本模型接管，原楼体、南侧后勤门与另外三面保持；既有 BYTE BEAT 独立招牌继续由 `business_signs` 生成

[candidate.glb](candidate.glb) 仅保留为 [v35-building-r1](../../../todo/evidence/TASK-049/v35-building-r1/review.md) 的冻结候选记录，SHA-256 为 `cca2fe302db5fce06565f67a03d12d8b3205f963988e4278c156a26a8288bd1b`。日常导出与 [check.py](check.py) 只面向正式运行路径，候选快照不随后续修改更新；来源、许可和正式派生关系由 [asset-manifest.json](asset-manifest.json) 维护

## 来源

模型、UV、图形和脚本为 N:SIDE 原创，适用项目 MPL-2.0。制作沿用 V-55 的米制 UV、标准 PBR 和导出副本方法，主文件中的网格仍可独立编辑

现有 `AST-003` 的 ambientCG `Plaster001`、`Concrete034` Color / NormalGL 各两张 JPG 保留为原授权输入，许可 CC0-1.0、来源与 SHA-256 沿用 [环境包清单](../../environment-kit/asset-manifest.json)。两张颜色 JPG 仍按原字节嵌入并使用 sRGB；运行法线改用下方衍生 PNG，使用 Non-Color。衍生图仍沿用 CC0-1.0，烘焙脚本按项目 MPL-2.0；没有新增生成服务或外部素材

视觉参照沿用 [已审查建筑参考](../../../todo/evidence/TASK-049/reference-r1/building-references.md) S14 金手指的经营层级和 S10 Box Galaxy 的厚檐、设备前后层次，未复制其商标、模型、贴图或游戏画面。粉紫、黄绿与深底读取本项目既有 BYTE BEAT 招牌

## 法线强度兼容

本机 Bevy 0.19.1 的 glTF loader 尚未应用 `normalTexture.scale`，原导出的0.25强度在实机按完整法线图读取；Root 的同机位 `normal-off` 诊断仅移除灰泥法线后黑点消失、原阴影保留，依据见[归因记录](../../../todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/README.md)。此版本差异通过资产修正，不更改灯光、曝光或引擎全局材质

[共享 bake_normals.py](../../environment-kit/materials/bake_normals.py) 复用该轮诊断公式，将原 RGB 解码为 `[-1,1]`，仅使 `x/y *= 0.25`、保持 `z` 后归一化，再量化保存无损 RGB PNG；没有 sRGB 转换。`source-assets/environment-kit/materials/Plaster001-NormalGL-scale025.png` 为1024×1024，`source-assets/environment-kit/materials/Concrete034-NormalGL-scale025.png` 为1024×512，原图与衍生图的路径、哈希、许可和 Pillow 版本在同一 manifest 登记

主文件只替换这两张 packed 法线图并将 Normal Map Strength 设为1，glTF `normalTexture.scale` 缺省值为1，避免能正确读取强度的引擎再次衰减。两张 PNG 仍只有一级 mip，本次不代表远距采样和闪烁已经解决

## 导出与检查

普通更新直接编辑主文件并执行导出，随后检查正式运行 GLB；`--rebuild` 会按配方重建主文件，仅在明确需要重建时使用。R4 首次接入采用已审候选原字节；法线兼容修复随后在现有主文件原位换图并保存导出，没有执行重建

```fish
# 修改法线输入或烘焙配方时先生成衍生图，普通复验只用 --check
python3 -B source-assets/environment-kit/materials/bake_normals.py --check
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/build.py
python3 -B source-assets/buildings/V-35/check.py
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/check.py -- --source
```

首次制作和 CPU 预览使用以下命令，运行前与正在使用 GPU 的任务协调资源

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-35/build.py -- --rebuild --render
```

检查覆盖源地图契约、模型有限数值、法线与绕序、PBR 米制 UV、原授权输入哈希、两张颜色图原字节、两张法线的逐像素衍生关系与绑定强度1、招牌不重叠、入口保守 AABB 样本与可编辑主文件。CPU 图只作源模型 self-audit；实际 Ground/capsule、Bevy 材质及连续路线交由整合轮验收

首次原字节接入的 [Python 几何检查](../../../todo/evidence/TASK-049/blender-integration-r4/v35-runtime-geometry.json) 保留为历史结果；法线修复后的[正式检查](../../../todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/runtime-check.json)仍为3,612三角、7,176顶点、7材质、4图及41个保守入口样本通过。[源工程与 GLB 对照](../../../todo/evidence/TASK-049/blender-integration-r4/normal-scale-r1/source-repair.md)确认网格、UV、法线、索引、变换、其他材质值和两张原颜色图字节不变；两次实际 Blender 进程正常退出，保存后的主文件检查通过，修复版入口和总览实机结果见下方画面记录

R4 的 `v35-entry-final` 150帧／6检查与 `v35-facade-final` 120帧／6检查均 PASS；实际查看入口第59帧和连续40–42帧、总览第49帧和连续31–33帧，首轮突出的黑椒点消退，窗框及檐下投影保留，所看横移段未见大块颗粒跳动或立面覆盖切换，见[实机画面自查](../../../todo/evidence/TASK-049/blender-integration-r4/visual-review.md)。两处固定距离支持保留强度修复，不代表任意距离过滤稳定，也不替代人物真实行走或作者外观验收；设备、暗窗与外场仍有未完成的表现

预览输出到 `todo/evidence/TASK-049/v35-building-r1/`，查看、记录哈希及结论后清理 PNG；文字、状态 JSON、源文件与复现配方保留

共享整理将两张 PNG 原字节迁入 AST-003 材料目录，本目录 [bake_normals.py](bake_normals.py) 只转发兼容旧命令；主文件只更新 packed 图来源路径，运行 GLB 保持 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a` 原字节，R4 画面证据仍对应当前运行资产，见[共享迁移检查](../../../todo/evidence/TASK-049/normal-scale-shared-r1/README.md)
