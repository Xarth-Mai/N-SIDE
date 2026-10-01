# 公共长椅

沿用 AST-003：Poly Haven 的 [Modular Street Seating](https://polyhaven.com/a/modular_street_seating)，原作者 Stuart Attenborrow，[CC0 许可](https://polyhaven.com/license)，完整许可正文复用 [CC0-1.0.txt](../licenses/CC0-1.0.txt)。官网发布日 2023-09-08；本次取得 2026-10-01，费用 0，逐文件来源、官方 MD5、SHA-256 与尺寸在[唯一清单](../asset-manifest.json)

## 用途与制作边界

镜厅前街需要可近看、有背靠和扶手的公共座椅，木条与金属框架补充建筑之外的使用尺度。该候选保留暖棕木条、深灰支架及原作者的旧化纹理，先在一处公共停留位置检查，不批量覆盖全城座位。既有 V-15 主数据没有 bench fixture，本轮单件置于既有镜厅屋顶公共平台，地图点 `[312,237,37]`，不修改地图设定；运行通过 `CollisionWorld::from_scene` 把 `street_bench` 实际 GLB 三角按视觉实例变换加入碰撞，足底与原屋面支承、座前空间和道路净空由集成核对

模型不包含人物、坐下动作或交互逻辑。CPU 制作和画面自查支持装配完整性，Bevy 的实际光照、尺度及场景观感按[运行验收](../../../docs/dev/validation/runtime.md)另行记录

## 主文件与原件

- `street-bench.blend` 是可编辑主文件，内嵌所用的 9 张原始 1K JPEG，包含一件已装配座椅
- `street-bench.glb` 是主文件导出的中间交付，由既有环境导出器复制为 `game/assets/environment/street-furniture/street-bench.glb`
- `original/` 保留上游原始 glTF、共用二进制和本件实际使用的 9 张贴图；官网 glTF 是零件展示布局，不直接加载为游戏成品。未采用的弧形连接件贴图没有下载，不能将原始全套 glTF 误作完整离线包
- `assemble.py` 复现首次装配：筛出直座部件，移动靠背支架、端腿与扶手，复制并旋转对侧端腿和扶手，保留原几何、UV 和材质；只剔除从原扶手继承的两个共线零面积三角。不用的弧接件、双端腿与悬吊支架不进入成品。该脚本会从原件重建主文件，后续手工修改主文件时只执行导出步骤

## 运行契约

| 项目 | 实测值 |
| --- | --- |
| 场景与变换 | `Scene0`，1 个根节点、1 个网格、实体缩放 1 |
| 三角形／材质 | 8,906／4 个原材质槽 |
| 宽 × 深 × 高 | `1.920000 × 0.670394 × 0.866914 m`，坐面高约 `0.45 m` |
| 坐标 | GLB：长轴 X、上方 +Y、坐者面向 +Z；Blender：上方 +Z、坐者面向 -Y |
| 枢轴 | 底面高 0、水平包围盒居中；未统一缩放，沿用原始米制几何 |
| GLB 包围盒 | `[-0.960000, 0, -0.335197]` 至 `[0.960000, 0.866914, 0.335197]` |
| 足底 | 两个立柱足底；X 分别约 `[-0.96,-0.88]` 与 `[0.88,0.96]`，Z 均约 `[-0.062846,0.133239]` |
| 纹理 | 9 张内嵌 `1024 × 1024` JPEG，原字节保留；颜色按 sRGB、OpenGL 法线及粗糙度／金属度按线性数据使用 |
| AO | 原 ARM 贴图保留 AO 红通道，但上游与本次材质均未绑定 `occlusionTexture`，不宣称 AO 已实际启用 |
| 运行放置 | `Vec3(312,37,-237)`、identity rotation、scale 1，坐者朝地图南；世界包络 `[311.04,37,-237.335197]` 至 `[312.96,37.866914,-236.664803]` |
| 运行碰撞 | 仅明确列入的 `street_bench`、`v_a08_facade`、`v15_mirror_hall_facade` 导入实际 GLB 三角；本椅两足与原屋顶支承由 CPU 窄测核对，游戏画面与路径由集成另验 |
| 运行文件 | `5,925,368 bytes`，无需联网或外部贴图 |

## 导出与复验

在仓库根目录执行，命令可直接用于 fish；首次装配已使用 Blender 5.2.2 LTS

```fish
# 仅在需要由原件重建初始主文件时运行
blender -b -t 4 -noaudio --python source-assets/environment-kit/street-furniture/assemble.py

# 从当前可编辑主文件导出，保留后续修改
blender -b source-assets/environment-kit/street-furniture/street-bench.blend -noaudio --python-expr 'import bpy; bpy.ops.export_scene.gltf(filepath=bpy.path.abspath("//street-bench.glb"), export_format="GLB", export_animations=False, export_yup=True, export_tangents=True, export_image_format="AUTO")'

# 变更后先将源 GLB 的实际 SHA-256/尺寸更新到既有清单，再导出运行包
bun tools/export-environment.ts
bun tools/export-environment.ts --check
python3 -B todo/evidence/TASK-049/public-street-r2/check.py
```

检查入口读取实际 accessor、法线、切线、UV、索引、底面、原件哈希与内嵌 JPEG，比较源／运行 GLB；CPU 预览脚本和实际记录见[本轮证据](../../../todo/evidence/TASK-049/public-street-r2/review.md)
