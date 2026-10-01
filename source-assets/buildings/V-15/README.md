# 镜厅 V-15 南、西立面

`BLD-V15-FACADE-001` 是实际使用本机 Blender 制作的可编辑外立面资产，状态为 `exported-candidate`；运行接入、真实街景和作者验收由本轮集成继续完成

主文件为 [mirror-hall-facade.blend](mirror-hall-facade.blend)，[build.py](build.py) 默认从已编辑主文件导出，`--extend-r2` 在主文件中更新南西立面，`--rebuild` 显式重建两轮完整设计，`--export-existing` 兼容既有导出命令，运行文件为 `game/assets/environment/buildings/v15-mirror-hall-facade.glb`；清单见 [asset-manifest.json](asset-manifest.json)，二进制检查见 [verify.py](verify.py)

## 设计与范围

参考 [镜厅与音乐街预览](../../../docs/public/images/cinema-music-street.webp) 的深色深檐、暖白字牌、竖海报框和混凝土收口，以南侧已有公共入口为中心制作 17.4m 宽的入口构架；采用大块灰泥、少量混凝土端柱、金属折边、木底板和真实门框进深，服务于白天街道视点的地点识别

原南侧入口构架、3m 出挑深檐、凹入木底板、灯具、排水、厚门框、凸起「镜厅」字牌和四只竖海报框保持；`Street programme 1 print` 接入安可竖版海报，另外三框仍为抽象几何内容占位

第二轮在南面 x=318.02..339.5 和西面 north=220..250 各制作五个整层结构开间，首层为高窗与门楣、二层为宽三分窗与木质楼层带、三层为较矮双窗加木格栅，顶端连续金属滴水与西南转角收口；不重复三层同尺寸小窗，南面末端 0.5m 保留旧墙以避让东电梯转角

新墙带距原外墙 0.02..0.28m，玻璃面距原外墙 0.045..0.065m，框前缘 0.32m、窗台前缘 0.35m；玻璃是实墙前的可见不透明凹窗面，既有建筑没有因贴附立面获得透明室内

东侧只制作 37m 公共平台底部的支承梁、斜撑、贴墙连接板和承托收口，保留平台以上通路；不导出建筑主体墙、室内、屋顶、公用电梯、地坪、平台面或第二套门玻璃，原地图稳定 ID、主体和基础碰撞继续由世界生成器负责，模型细件的实际三角由运行集成加入碰撞

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
| 碰撞 | 运行通过 `CollisionWorld::from_scene` 将本 GLB 实际三角按 node、模型缩放和实例变换加入结构碰撞；原主体与路线碰撞保留，被替换泛型细件同步裁剪 |
| 源总包络 | `[299.65,216.965,25]` 至 `[343.150005,250.08,36.83]` |
| GLB 总包络 | `[-0.35,0,-30.08]` 至 `[43.150005,11.83,3.035]` |

`V15_EXPORT_Facade` 是唯一导出 collection；`REVIEW_ONLY_Map_context` 包含 40×30×12m 主体参照、原门玻璃、1.7m 标尺和源多边形生成的 0.40m 厚平台，以及灯光、相机，均不导入运行 GLB

## 精确避让规则

以下均为源地图坐标，只限定 `V-15` 南侧边 `north=220` 和西侧边 `x=300`，东、北面保持原状

- 三层通用窗：南面及西面整面跳过通用窗框、玻璃与窗台；从 r1 的南面局部包络规则改为整面规则，避免旧窗穿过新柱、墙带与凹窗面；东面窗与北面窗保持现状
- 29m 与 33m 楼层带：南面和西面整段跳过，原 r1 南面保留的 x=318..340 段也移除；新分层墙带、滴水和完整转角负责收口
- `V-15` 已有 `RF` 楼层，世界生成器本来就不会为其生成泛型 roof coping，无需新增裁剪条件或计算 coping 移除数；r2 只增加自己的南西顶部收口，原主体墙、屋顶面保持
- `cinema_entry` 原雨棚单独跳过，原 box 为 `x=303.8..308.2、north=218.9..220、h=27.87..28.03`；保留原 3.6×2.75m 门玻璃、门框和中梃，北侧后勤与东侧二层餐饮门、雨棚保持现状
- 新入口横梁底为 27.92m、最低檐灯为 28.106m；南入口 3m 宽接近段 `x=304.5..307.5、north=216..220、h=25..27.75` 中无导出构件
- 东侧平台原顶 37m、底 36.60m；梁顶 36.80m、承托顶 36.83m，插入板底 0.20/0.23m，所有附加构件都在屋顶通行面以下
- 东侧支架墙板低 x=339.98，嵌入原墙 2cm；南块墙板 north=240.27..240.73，即按更宽的 `width+0.28` 包络检查，三层候选窗 north 上界仍为 240.150714，相距约 0.119m，因此不需额外跳过东面窗

空间避让由实际检查的五个空体积支持：南侧 3m 入口、东侧二层门、北侧后勤门、东侧屋顶路径及上街接口；同时单独核对新增构件未占用本地 x=39.5..42、y=-2..0、z=0..3 的电梯转角接近段；这些实际评估网格 AABB 检查证明资产未占用所列空间，不替代人物控制器或原建筑可进入性验证

## 材质与来源

复用 [AST-003 素材清单](../../environment-kit/asset-manifest.json) 中 ambientCG 的 `Concrete034`、`Plaster001`、`WoodSiding009` 原始 Color/NormalGL JPG，均为 CC0-1.0；原文件与许可仍留在环境素材包，本资产不重复下载或登记为新来源

Color 为 sRGB，NormalGL 为 Non-Color；UV 按米展开，混凝土周期 1.1×0.55m、灰泥 2×2m、木饰面 2×1m，法线强度 0.35；其他材质为原创纯色标准 PBR，灯具和字牌仅使用标准 emissiveFactor，不加入额外灯光或材质扩展

脚本 `material()` 中的颜色三元组按 Blender 线性 RGB 写入 shader，r1 的纯色材质为本资产独立定义，未直接复用 `appearance.srgba` 数值；带贴图材质由 Color 纹理直接连接并覆盖 Base Color，不把未参与节点乘法的备用颜色称为贴图 tint

新增玻璃沿用 `appearance.json` 的 `glass` 值，sRGB 转线性后写入 Blender；GLB 材质是构建快照，不自动绑定运行时同名材质

安可竖版海报唯一源为 [anke-cinema.svg](../../star-posters/anke-cinema.svg)，运行源图为 `game/assets/environment/posters/anke-cinema.png`，1024×1620、全不透明；`Street programme 1 print` 正面为 1.42×2.25m，以整个 0..1 UV 显示，纵横比差约 0.16%；GLB 嵌图与源 PNG 逐字节相同，来源和许可沿 [AST-007 说明](../../star-posters/README.md) 与 AST-005 OFL 字体，整张图不标为 CC0

中文字形复用 [AST-005](../../ui-kit/asset-manifest.json) 的 Noto Sans SC 及既有 OFL-1.1 许可，GLB 导出字形网格；英文沿用 Blender 内置字体；原创建筑几何、抽象海报图形与脚本按项目 MPL-2.0 管理

## 重现与检查

已实际使用 Blender `5.2.2 LTS`；本机 OpenAL 在受限音频服务上会阻塞退出，因此离线命令使用 `ALSOFT_DRIVERS=null`，不修改用户系统音频设置

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --extend-r2 --render
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-15/build.py -- --export-existing --render
python3 source-assets/buildings/V-15/verify.py
```

主文件保留分件、可编辑文字、材质和打包源贴图；导出副本转网格、合并为一个 mesh 并按材质保留 10 个 primitives，移除字体三角化留下的两个零面积三角后删除副本，不覆盖主文件中的分件

GLB 预算上限为 30,000 顶点、16,000 三角面和 10 材质槽；当前为 21,482 顶点、12,362 三角面、10 材质、7 张内嵌贴图、7,812,440 bytes；硬边及 UV 面角使顶点数增长，三角预算仍保持 16,000，未为数字删原结构；没有新增建筑纹理，额外一图仅为安可海报，无骨骼、动画、相机、灯光、外部图片路径及 glTF 扩展

r1 共 58 件，其中 53 件评估后顶点、面、材质槽及 UV 的 hash 完全相同；变更仅为一块海报 print 的材质与 UV，以及删除其四个原几何图形覆盖层；原 8 材质导出描述和 6 张嵌入贴图逐项相同

视觉完成度为 `needs_revision`：南西街景已具备整栋墙窗层次，玻璃仍是平整暗面，开间节奏偏规则，另外三张抽象海报仍为占位；东、北其余墙面沿用泛型外观，本批不计作最终镜厅艺术验收

`verify.py` 实际读取二进制 POSITION/NORMAL/UV/indices，检查有限值、法线长度、索引、退化三角、声明包络、原点、场景、不透明材质、海报正面 UV 和嵌图原 bytes；`build.py` 检查评估后分件五处净空、新窗口深度及电梯转角；本轮结果及看图记录见 [self-audit](../../../todo/evidence/TASK-049/v15-building-r2/self-audit.md)，机器通过与离线看图均不代表作者审美验收

## 同机位运行验收

集成后先复查 `fw_w_cinema_front` 到 `cinema_entry` 的接近路线，再以约 1.7m 视点面向南立面，比较深檐厚度、入口净空、凸字和海报框；固定日光与曝光，不靠曝光改变材质读感

另从 `cinema_roof [340,245,37]` 到 `cinema_deck_turn [350,260,37]` 沿既有路线复查 4m 公共平台，再从东侧低层看斜撑是否贴墙并接合板底；同镜头检查三层窗不被墙板擦碰、新饰面无旧窗或楼层带半穿，保存截图与状态并实际查看
