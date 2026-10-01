# V-55 包装修补工作室外立面

所属地图对象 `V-55`／场所 `55`，主文件为 [facade.blend](facade.blend)，可编辑制作脚本为 [build.py](build.py)，候选检查为 [check.py](check.py)，来源与实际导出数值见 [asset-manifest.json](asset-manifest.json)

该建筑对应[正式场所用途](../../../docs/player/locations/places/b05.md#place-55)与[本轮制作 brief](../../../todo/evidence/TASK-049/building-next-brief-r1.md)，制作状态保留为候选；已接入 Bevy 并完成本轮局部 GPU 画面自查，碰撞窄测与作者外观反馈分别记录

## 制作内容

在12×16m轮廓、+24m基底、9m总高和+24／27／30m三层工作空间不变的前提下，制作西面经营立面与南侧转角。暖灰泥与混凝土是基础，蓝紫色烤漆金属集中在首层作品柜、门檐与局部竖向遮阳片，米黄色包装和布料样卡显示经营用途；没有新增专有店名、住户阳台或室内房间

西面公用门仍在 `[148,202,24]`，净口1.8×2.35m；服务门在 `[148,210,24]`，净口1.3×2.35m。两门保留关闭门叶，公门有细中梃、下部色板和0.75m短檐，服务门使用较低强调的深色门叶与0.45m短檐；檐底均为+26.64m，源坐标及入口权限保持

展示位于两门之间的5.4m宽附墙柜，包含六组纸盒或折叠织物样品、四张带缝线的布料样卡和分层搁板。柜背挡住既有主墙，构件最大出墙0.29m；未增加透明玻璃覆盖，陈列并非可进入室内，也没有编造某位客户的剧情物件

16组工作间窗分布于楼上西／南两面与南面首层，玻璃表面位于旧墙外0.039m，外框到0.19m，形成0.151m框内层次；玻璃采用不透明PBR背板，未声称存在窗后房间。两组局部竖向遮阳片只出挑0.34m，保持工作间的采光节奏

用途字招使用原有字体的可编辑文字对象：`包装 / 修补`、`收件`、`样品`，不把整栋建筑烘成一张贴图。25个独立网格组和3个文字对象保留在源文件；运行导出只合并临时副本为单一mesh，四张PBR原图与字体均打包在源工程内

## 坐标与接入

| 项目 | 合同 |
| --- | --- |
| 源文件轴向 | X东、Y北、Z上，单位m |
| 地图锚点 | `[154,206,24]`，楼体基底中心 |
| 运行摆放 | `Scene0`、rotation identity、scale1，仅调用现有 `map_to_world` 一次 |
| model id | `v55_workshop_facade` |
| 运行派生路径 | `game/assets/environment/buildings/v55-workshop-facade.glb`，由编辑源脚本导出 |
| GLB边界 | X `−6.75…6`、Y `−0.55…9`、Z `−8…8.25` |
| 地图实际外包络 | X `147.25…160`、北向 `197.75…214`、高程 `23.45…33` |
| 低位饰面 | 常规出墙0.12m，样品柜0.29m；地脚下延0.55m，以真实坡地遮蔽 |
| 技术规模 | 7362三角、13008导出顶点、6材质、4张内嵌1024×1024图 |

### 旧构件替换

源楼体、屋顶、基础及东／北面泛型构件保持原样，仅接管 `V-55` 的西面 `x148` 与南面 `north198`：禁用这两面的泛型窗、腰线及重叠压顶；两扇西门的泛型门框、门叶和小雨檐由新模型替代。实际移除数量由 Rust 场景生成差分核对，本文件不以纸面估算当作运行结果

公／服务门支路分别来自 `/roads/452` 与 `/roads/453`，不新增地面平台、站牌、衣架或路侧花箱。关闭门的可接近性应止于门外：候选检查沿真实斜向路段到门前0.55m，用0.32m半宽、1.7m高的包络作三角AABB保守筛查；原0.45m停止距离触及公门中梃，已明确修正接近点，未移除门叶使测试通过。地面高度使用运行端 `Ground`；集成已导入实际 GLB 三角，最终 CPU 窄测确认两扇关闭门先于旧楼体阻挡胶囊，实际人物沿两条支路到门的行走仍待验证

## 公开素材与许可

- 自制建筑网格、UV、文字布局与脚本按项目现行 MPL-2.0；具体参考为[美术方向](../../../docs/dev/production/art-direction.md)、[镜厅预览](../../../docs/public/images/cinema-music-street.webp)及实际地图，不复制其他作品Logo或建筑身份
- 灰泥 `Plaster001` 与勒脚 `Concrete034` 原图来自 ambientCG，CC0-1.0，作者与下载原件哈希沿用 [AST-003清单](../../environment-kit/asset-manifest.json)；本包只登记引用关系和校验，不复制维护第二份来源总表
- Noto Sans SC 2.004 使用已有[字体源](../../ui-kit/README.md)，原 [COPYRIGHT](../../ui-kit/licenses/COPYRIGHT.txt) 与 [OFL 1.1](../../ui-kit/licenses/OFL.txt) 保持；字体原数据打包在 `.blend`，GLB仅包含当前三个用途标签的文字几何
- 色彩贴图sRGB、NormalGL为Non-Color；灰泥每2×2m重复、混凝土每1.1×0.55m重复，均为项目视觉尺度，不冒充上游物理测量。节点正常使用导出法线强度0.25，固有色从sRGB转换为线性后写入材质

## 操作

仓库根目录执行，fish可直接运行。默认从编辑后的 `.blend` 导出运行GLB，`--rebuild` 仅在需要按原始设计脚本重新建模时使用，会覆盖该主文件。进程数保持一个，当前批次CPU看图为640×480、12 samples、2线程，不启动GPU或后台常驻Blender

```fish
mkdir -p output/buildings/V-55
# 重建会覆盖此资产的编辑源
timeout --signal=TERM --kill-after=10s 180s env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-55/build.py -- --rebuild
# 常规编辑后的重新导出
timeout --signal=TERM --kill-after=10s 180s env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-55/build.py
python3 -B source-assets/buildings/V-55/check.py
timeout --signal=TERM --kill-after=10s 90s env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-55/check.py -- --source
# 仅在主任务分配的CPU预览窗口执行
timeout --signal=TERM --kill-after=10s 600s env ALSOFT_DRIVERS=null blender --background -noaudio --threads 2 --python-exit-code 1 --python source-assets/buildings/V-55/build.py -- --render
```

检查真实GLB顶点／法线／三角面积／绕序、PBR米制UV、原图与AST-003哈希、两条接近包络、地图契约和源对象；清晰诊断失败时停止接入。DCC看图是自查，场景截图与实际移动交给主任务验证。临时预览图在完成观察、记录哈希与结论后删除，源码、运行GLB、参数和日志保留

本轮源制作与检查记录位于 [v55-building-r1](../../../todo/evidence/TASK-049/v55-building-r1/handoff.md)。R3 的 `v55-west` 实机短片采集通过，主任务实际查看第49帧：用途字招与展示可辨，未见明显悬空；结果属于局部画面自查，见[视觉记录](../../../todo/evidence/TASK-049/blender-integration-r3/visual-review.md)与[集成记录](../../../todo/evidence/TASK-049/blender-integration-r3/review.md)。[最终碰撞窄测](../../../todo/evidence/TASK-049/blender-integration-r3/scene-models-final.log)为1项通过，包含两扇关闭门的真实三角阻挡；不替代人物行走或作者美术验收
