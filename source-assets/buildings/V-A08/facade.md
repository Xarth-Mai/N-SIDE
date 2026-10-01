# V-A08 住宅外立面

`AST-V-A08-FACADE` 是月台杂货北邻三层商住楼的整片外立面样板，主文件为 [facade.blend](facade.blend)，运行派生物为 `game/assets/environment/buildings/v-a08-facade.glb`；登记见 [facade-manifest.json](facade-manifest.json)，制作状态为 `exported-candidate`，接入与作者外观验收另行记录

## 设计与保持项

依据 [小店与坡地住宅建筑](../../../docs/dev/design/locations/district-architecture.md)、[美术方向](../../../docs/dev/production/art-direction.md)和 [镜厅与音乐街预览](../../../docs/public/images/cinema-music-street.webp)，制作普通坡地住宅的成组窗、暗色金属框、细灰泥大面、混凝土收边和局部家用设备，沿用预览图的立面层次与可信构件尺度。公开空间、体量和入口以真实地图为准，不复制图中的匿名建筑布局

保留 `V-A08` 的18×13m轮廓、30.021515m基底、10.4m总高及1F／2F／3F的相对标高0／4／7.2m。既有 [深檐与雨槽](README.md)的 `.blend`、GLB、放置合同和材质保持不变，新立面接到它的下方，不重复导出屋檐

四面均制作薄贴附层：南面和东面承担主要街景，西面对应服务门，北面为与 `V-A09` 间的一米维护缝，采用闭实饰面。原北面泛型窗随新立面移除，这属于本栋立面布置调整，没有新增可进入室内或改变楼层用途

27组成组窗用真实金属框、窗台和外墙贴附层形成约0.135m进深，玻璃位于原墙外0.033…0.045m，主饰面外缘0.18m；玻璃为不透明标准PBR面，不宣称已经存在窗后室内。四组东面小窗栏属于固定窗栏，没有可站立阳台地板或阳台门

三台空调具有机体、百叶、贴墙托架和绕开窗洞的连接管，分置东、南、西立面二层；连接管从窗下转向窗侧，没有穿过玻璃。两处东门小檐分别覆盖经营门和住户门，替代原一整片13m长雨棚，最大出挑1.35m，檐底2.83m，保持人行头部净空

## 坐标与接入

| 项目 | 合同 |
| --- | --- |
| 主文件本地 | X向东、Y向北、Z向上，单位m，所有分组共用原点 |
| 地图原点 | `[79,277.5,30.021515]`，楼体基底中心 |
| 运行实例 | `Scene0`、rotation identity、scale1；`map_to_world` 转换一次 |
| 建议model id | `v_a08_facade` |
| appearance model | `{"file":"environment/buildings/v-a08-facade.glb","scene":0,"scale":1}` |
| GLB边界 | X `−9.657…10.35`、Y `0…10.06`、Z `−6.755…7.157` |
| 主门 | `v_a08_door [88,277.5,30.021515]`，1.8×2.35m |
| 住户门 | `v_a08_door_resident [88,273,30.021515]`，1.3×2.35m |
| 服务门 | `v_a08_service [70,277,30.021515]`，1.3×2.35m |
| 静态预算 | 6060三角、6材质／primitive、4张内嵌图（灰泥1024×1024、混凝土1024×512）、3,800,008 bytes |

三扇门在原节点外表现关闭门叶。V-A08原本不支持进入，主墙仍为封闭体量；模型没有开通新房间、互动或任务入口。源文件保留43个可编辑网格组；运行导出只合并临时副本为一个mesh，源组、packed贴图和UV不被覆盖

### 替换旧泛型构件

本模型覆盖四面，可以完整跳过 `facades()` 对 `V-A08` 的派生构件，其他建筑保持现有生成行为。继续保留 `geometry.rs` 的 V-A08 主墙、屋顶、基础，`v_a08_roof_eaves` 屋檐实例、地图、三入口、道路274／275／728和周边植被

| 旧面 | 本次替换范围 | 预估方盒／结构三角 |
| --- | --- | --- |
| 南 | 4扇首层简窗、8扇楼上详细窗、两腰线 | 66／792 |
| 东 | 首层北窗、6扇楼上详细窗、窗栏、空调、卷帘、腰线、雨槽／管／卡箍、两门、13×1.5m雨棚 | 142／1704 |
| 北 | 二层4扇简窗、三层4扇详细窗、两腰线 | 38／456 |
| 西 | 两扇首层简窗、6扇楼上详细窗、两腰线、服务门 | 52／624 |
| 合计 | 完整旧 `facades` 内容 | 298／3576 |

这些数量来自源码与Bun地形复算，集成必须用实际Rust生成结果确认。上一轮已删除的屋檐48三角不重复计入。V-A08没有display／shopfront绑定，本批没有额外店招、展架或花盆要清理

不能通过把V-A08改成其他ID恢复测试基线，因为 `residential_sample` 使用明确ID白名单，改名会使详细窗、窗栏与雨水系统一起丢失。沿用实际旧构件快照或可关闭本栋接管的窄测；10栋旧住宅样板断言应对V-A08改用此GLB合同，保留其余9栋检查

### 碰撞与净空

本次运行集成通过 `CollisionWorld::from_scene` 导入 `v_a08_facade` 的实际 GLB 三角，按节点矩阵、模型缩放和实例变换与可见模型对齐；原主墙仍负责基础阻挡，被移除的旧 `derived-facade` 装饰从结构碰撞同步移除。新覆层突出原壳0.18m、角条突出0.255m，运行窄测比较贴墙命中位置，三入口外侧与连续街行复验由集成证据记录；独立 `v_a08_roof_eaves` 仍保持可视附件，不在此次三模型碰撞名单内

三门的测试终点应在门外可接近处，并单独核对门叶与节点对齐。模型检查已验证门平面外0.30m开始、2.60m高的三个接近体积无导出三角；此AABB排除检查不等于胶囊导航。不要要求角色穿越关闭门叶，也不通过删除门叶或忽略所有门前命中让测试通过

保留或窄测原来的入口邻楼遮挡与 `Ground ≤ entry+0.35m` 检查；实际道路七段按本次碰撞状态复验。北侧覆层最远north=284.255m，距V-A09原墙0.745m；这条窄缝仍是维护边界，不能作为新公共路线

## 材质和来源

复用 [AST-003](../../environment-kit/asset-manifest.json) 已登记的 ambientCG `Plaster001` 与 `Concrete034` Color／NormalGL原图，许可CC0-1.0。保持登记的原图字节与SHA，GLB颜色图仍与原 JPG 逐字节比较，法线使用共享无损 PNG 并检查衍生关系；没有二次下载、生成或新授权费用

| 材质 | 来源／色彩空间 | 周期与用途 |
| --- | --- | --- |
| Plaster | AST-003 Plaster001，Color=sRGB，NormalGL=Non-Color | 2×2m，四面墙层 |
| Concrete | AST-003 Concrete034，同上 | 1.1×0.55m，勒脚、窗台、腰线、转角 |
| Metal | 项目原创纯色标准PBR，sRGB输入转线性 | 暗金属窗框、托架、窗栏、雨管 |
| Glazing | 项目原创纯色标准PBR，sRGB输入转线性 | 不透明玻璃样板，不含新增室内 |
| WarmPanel | 项目原创纯色标准PBR，sRGB输入转线性 | 门叶、卷帘、空调壳及檐底 |
| Ochre | 项目原创纯色标准PBR，sRGB输入转线性 | 两处小檐收边 |

贴图直接连接base color，没有未生效的tint；法线强度0.28已烘入共享 PNG，节点 Strength 与 glTF scale 均为1，UV按实际面坐标展开，检查换算周期后的每条边与真实米制长度。源贴图packed在主文件中，GLB使用四张内嵌图，不依赖DCC中的绝对路径。几何、UV、脚本按仓库MPL-2.0管理，贴图继续采用其CC0许可

## 编辑、导出与检查

以下命令从仓库根目录执行，fish可直接使用。默认导出现有主文件；只有重新制作初始网格时添加 `--rebuild`，会覆盖手工源修改。渲染限制CPU六线程，`ALSOFT_DRIVERS=null` 避免本机无音频设备的退出阻塞

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-A08/facade-build.py
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-A08/facade-build.py -- --render
python3 -B source-assets/buildings/V-A08/facade-check.py
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-A08/facade-check.py -- --source
```

本轮实际使用本机Blender5.2.2 LTS。机器检查与四机位自查记录在 [本轮证据](../../../todo/evidence/TASK-049/va08-facade-r1/README.md)；独立运行截图、街道连续操作与作者验收由集成记录，不把离线模型预览算作最终游戏画质完成

## 共享法线兼容修复

本机 Bevy 0.19.1 glTF loader 忽略 `normalTexture.scale`；本包沿用[共享烘焙入口](../../environment-kit/materials/bake_normals.py)，将上列原强度按 `xy *= scale`、保留 z 后归一化写入 CC0 无损 PNG，原授权 JPG、颜色、UV、几何与其他在用材质保持。主文件原位换图后保存并在宿主 Blender 复读，普通导出与逐像素检查通过；证据及前后语义见[共享修复记录](../../../todo/evidence/TASK-049/normal-scale-shared-r1/README.md)。当前修复版已完成 [R5 真实场景复验](../../../todo/evidence/TASK-049/blender-integration-r5/review.md)，机器断言通过且关键帧已查看，覆盖范围见[画面自查](../../../todo/evidence/TASK-049/blender-integration-r5/visual-review.md)，作者验收仍待完成
