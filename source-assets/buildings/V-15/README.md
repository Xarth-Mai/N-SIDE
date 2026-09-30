# 镜厅 V-15 外立面样板

`BLD-V15-FACADE-001` 是实际使用本机 Blender 制作的可编辑外立面资产，状态为 `exported-candidate`；运行接入、真实街景和作者验收由本轮集成继续完成

主文件为 [mirror-hall-facade.blend](mirror-hall-facade.blend)，[build.py](build.py) 重建初始设计并导出，`--export-existing` 从已编辑主文件导出而不重建，运行文件为 `game/assets/environment/buildings/v15-mirror-hall-facade.glb`；清单见 [asset-manifest.json](asset-manifest.json)，二进制检查见 [verify.py](verify.py)

## 设计与范围

参考 [镜厅与音乐街预览](../../../docs/public/images/cinema-music-street.webp) 的深色深檐、暖白字牌、竖海报框和混凝土收口，以南侧已有公共入口为中心制作 17.4m 宽的入口构架；采用大块灰泥、少量混凝土端柱、金属折边、木底板和真实门框进深，服务于白天街道视点的地点识别

南侧构件包括薄贴附饰面、端部收口、3m 出挑深檐、凹入木底板、灯具、排水、入口两侧厚门框、凸起「镜厅」字牌和四只竖海报框；海报里的几何排版由本项目原创，仅标示 `MIRROR HALL`，不新增影片、上映日、票价或人物设定

东侧只制作 37m 公共平台底部的支承梁、斜撑、贴墙连接板和承托收口，保留平台以上通路；不导出建筑主体墙、室内、屋顶、公用电梯、地坪、平台面或第二套门玻璃，原地图稳定 ID、主体和碰撞继续由世界生成器负责

参考图用于材质、构件和比例分析，实际 footprint、楼层标高、门位和平台以 `district.json` 为准；镜厅本次仍是外立面，原 V-15 主体墙未开真实门洞，不能据此宣称影院内部可进入

## 坐标与接入

| 项目 | 契约 |
| --- | --- |
| 源建筑 | `V-15`，源地图 x=300..340、north=220..250，基底 25m，屋顶 37m |
| Blender 本地 | `(x-300, north-220, height-25)`，米制、Z-up |
| GLB | 标准 Y-up，导出自动得到 `(dx, dh, -dnorth)`，`Scene0`，无额外根变换 |
| 运行 anchor | `map_to_world([300,220,25]) = Vec3(300,25,-220)` |
| 实例变换 | scale=1、rotation=identity；不再次翻轴 |
| model id 建议 | `v15_mirror_hall_facade` |
| appearance model | `{"file":"environment/buildings/v15-mirror-hall-facade.glb","scene":0,"scale":1}` |
| 实例化 | 沿用 `Appearance.models` 与 `props()` 的单个 `PropPlacement`；仅登记 model 不会产生实例 |
| 碰撞 | GLB 是视觉附加件，不加入 collider；原 V-15 主体、路线与碰撞保留 |
| 源总包络 | `[300.58,216.965,25]` 至 `[343.150005,250.08,36.83]` |
| GLB 总包络 | `[0.58,0,-30.08]` 至 `[43.150005,11.83,3.035]` |

`V15_EXPORT_Facade` 是唯一导出 collection；`REVIEW_ONLY_Map_context` 包含 40×30×12m 主体参照、原门玻璃、1.7m 标尺和源多边形生成的 0.40m 厚平台，以及灯光、相机，均不导入运行 GLB

## 精确避让规则

以下均为源地图坐标，限定 `V-15` 南侧边 `north=220`，不扩大到其他建筑或另外三面

- 三层通用窗：若整个窗框范围与 `x=300.58..318.02` 相交则跳过该窗，按含窗台的最大构件半宽 `(width+0.28)/2` 保守计算；整窗跳过避免在新饰面边缘留下半扇旧窗，东侧窗保持现状
- 29m 与 33m 楼层带：原长 40m、中心 `[320,219.96,29/33]`、高 0.13m、深 0.18m；南面只保留 `x=318..340`，对应中心 x=329、长 22m；新端柱负责左段收口
- `cinema_entry` 原雨棚单独跳过，原 box 为 `x=303.8..308.2、north=218.9..220、h=27.87..28.03`；保留原 3.6×2.75m 门玻璃、门框和中梃，北侧后勤与东侧二层餐饮门、雨棚保持现状
- 新入口横梁底为 27.92m、最低檐灯为 28.106m；南入口 3m 宽接近段 `x=304.5..307.5、north=216..220、h=25..27.75` 中无导出构件
- 东侧平台原顶 37m、底 36.60m；梁顶 36.80m、承托顶 36.83m，插入板底 0.20/0.23m，所有附加构件都在屋顶通行面以下
- 东侧支架墙板低 x=339.98，嵌入原墙 2cm；南块墙板 north=240.27..240.73，即按更宽的 `width+0.28` 包络检查，三层候选窗 north 上界仍为 240.150714，相距约 0.119m，因此不需额外跳过东面窗

构件碰撞边界由实际检查的五个空体积支持：南侧 3m 入口、东侧二层门、北侧后勤门、东侧屋顶路径及上街接口；这些 AABB 检查证明新资产未占用所列空间，不替代人物控制器或原建筑可进入性验证

## 材质与来源

复用 [AST-003 素材清单](../../environment-kit/asset-manifest.json) 中 ambientCG 的 `Concrete034`、`Plaster001`、`WoodSiding009` 原始 Color/NormalGL JPG，均为 CC0-1.0；原文件与许可仍留在环境素材包，本资产不重复下载或登记为新来源

Color 为 sRGB，NormalGL 为 Non-Color；UV 按米展开，混凝土周期 1.1×0.55m、灰泥 2×2m、木饰面 2×1m，法线强度 0.35；其他材质为原创纯色标准 PBR，灯具和字牌仅使用标准 emissiveFactor，不加入额外灯光或材质扩展

脚本 `material()` 中的颜色三元组按 Blender 线性 RGB 写入 shader，仅为本资产的纯色材质独立定义，未直接复用 `appearance.srgba` 数值；带贴图材质由 Color 纹理直接连接并覆盖 Base Color，不把未参与节点乘法的备用颜色称为贴图 tint

中文字形复用 [AST-005](../../ui-kit/asset-manifest.json) 的 Noto Sans SC 及既有 OFL-1.1 许可，GLB 导出字形网格；英文沿用 Blender 内置字体；原创建筑几何、海报图形与脚本按项目 MPL-2.0 管理

## 重现与检查

已实际使用 Blender `5.2.2 LTS`；本机 OpenAL 在受限音频服务上会阻塞退出，因此离线命令使用 `ALSOFT_DRIVERS=null`，不修改用户系统音频设置

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --render
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --export-existing --render
python3 source-assets/buildings/V-15/verify.py
```

主文件保留分件、可编辑文字、材质和打包源贴图；导出副本转网格、合并为一个 mesh 并按材质保留 8 个 primitives，移除字体三角化留下的两个零面积三角后删除副本，不覆盖主文件中的分件

GLB 预算上限为 20,000 顶点、16,000 三角面和 8 材质槽，当前为 9,552 顶点、7,154 三角面、8 材质、6 张内嵌贴图，约 5.9MB；无骨骼、动画、相机、灯光、外部图片路径及 glTF 扩展

视觉完成度为 `needs_revision`：抽象几何海报仍是内容占位，源墙面的大面积留白和真实街道里的构图还需后续观察；本批冻结建筑构件供 GPU 接入，不计为已达到镜厅预览的最终完成度

`verify.py` 实际读取二进制 POSITION/NORMAL/UV/indices，检查有限值、法线长度、索引、退化三角、声明包络、原点与场景约定；本轮结果及实际看图哈希见 [self-audit](../../../todo/evidence/TASK-049/v15-building-r1/self-audit.md)，机器通过与离线看图均不代表作者审美验收

## 同机位运行验收

集成后先复查 `fw_w_cinema_front` 到 `cinema_entry` 的接近路线，再以约 1.7m 视点面向南立面，比较深檐厚度、入口净空、凸字和海报框；固定日光与曝光，不靠曝光改变材质读感

另从 `cinema_roof [340,245,37]` 到 `cinema_deck_turn [350,260,37]` 沿既有路线复查 4m 公共平台，再从东侧低层看斜撑是否贴墙并接合板底；同镜头检查三层窗不被墙板擦碰、新饰面无旧窗或楼层带半穿，保存截图与状态并实际查看
