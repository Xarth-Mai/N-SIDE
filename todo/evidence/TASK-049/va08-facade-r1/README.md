# V-A08 整片住宅立面制作

基线为 `0efb89c`，本轮仅制作V-A08的新Blender立面及导出、检查和来源记录；原地图、楼层、三门节点、主墙、屋檐源和屋檐GLB保持不变，场景接入与运行碰撞由集成批次记录

## 交付

- 主文件：[facade.blend](../../../../source-assets/buildings/V-A08/facade.blend)，43个可编辑网格组、27组成组窗、四张packed源贴图
- 运行文件：[v-a08-facade.glb](../../../../game/assets/environment/buildings/v-a08-facade.glb)，6060三角、6材质、1mesh／6primitive、4张1024²内嵌JPEG、4442332字节
- [放置／替换／碰撞合同](../../../../source-assets/buildings/V-A08/facade.md)和 [资产登记](../../../../source-assets/buildings/V-A08/facade-manifest.json)
- 源文件SHA256 `a31ee285b7bd832019db0421f6ecd0d8c9b26dae4a702dcf910e5a17d5361ee1`，运行文件SHA256 `aa2b9186b64d07a23ada84fa83f66fe0042d1dc5c8969d0b715638835e254808`

四面覆层、成组窗进深、固定小窗栏、两处门檐、三台空调及连接管共同替换本栋旧泛型立面，不增加室内、楼层、可站阳台、剧情或交互。主原点为地图 `[79,277.5,30.021515]`，运行Y向上转换一次

## 实际命令与结果

以下命令在仓库根目录实际执行，fish可直接使用

```fish
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-A08/facade-build.py -- --rebuild --render
python3 -B source-assets/buildings/V-A08/facade-check.py
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-A08/facade-check.py -- --source
env ALSOFT_DRIVERS=null blender --background -noaudio --threads 6 --python-exit-code 1 --python source-assets/buildings/V-A08/facade-build.py
```

| 检查 | 实际结果 | 证据与边界 |
| --- | --- | --- |
| 本机制作与四机位渲染 | PASS，退出0 | [build.log](build.log)，Blender5.2.2 LTS `d13f752e3b9c`，CPU Cycles28samples／固定六线程／1200×900 |
| GLB有效性／预算／有限Transform | PASS | [geometry.json](geometry.json)，Scene0单根，无动画、相机或扩展，预算低于12000三角和8材质 |
| 法线／绕序／UV米制 | PASS | 三角非退化、法线长度和绕序一致，换算贴图周期后UV／真实边长比为0.999992664…1.000007336 |
| 源图来源与内嵌数据 | PASS | 4张图与AST-003来源、CC0声明和SHA匹配，内嵌字节未改写 |
| 源文件结构 | PASS，退出0 | [source-check.log](source-check.log)，43网格组、43个静态可编辑对象、6060三角、4张packed图；身份Transform、封闭网格与正体积检查通过 |
| 默认主文件重新导出 | PASS，退出0 | [master-export.log](master-export.log)，GLB哈希与修正后首次导出完全相同，没有改写主文件 |
| 三门外接近体积 | PASS | 实际GLB三角AABB与门外0.30m开始／2.60m高的三个体积无交集；不等于人物胶囊或道路导航测试 |
| CPU模型画面自查 | SELF_AUDIT | [previews.json](previews.json)，四个最终机位均已实际看图，主任务另查看总景、东门和窗近景 |
| 实际Bevy加载／街道光照／连续通路 | NOT RUN by asset producer | 由主任务统一接入及运行；不能用离线渲染替代 |
| 作者体验／最终视觉验收 | 待反馈 | 本轮只交付可接入样板 |

源文件检查核对地图元数据、实际静态几何、三角数和packed图；它不声称逐顶点证明所有未来手工编辑与现有导出完全等价，编辑后必须重新导出并复验

日志中缩略图缓存目录写入失败不影响源文件保存或渲染；本机未提供的MeshOptimizer也没有被本资产使用，GLB无压缩扩展且实际导出检查通过。两条诊断保留在原日志中

## 画面发现与修正

初次近景发现空调连接管伸入窗面，因此把连接管改为从机体右侧沿窗下水平走管，再在窗侧向上收口。最终四张图重新渲染并实际查看：窗面已无连接管穿越，真实窗深、金属框、窗台、百叶、托架和门檐层次可辨。南东总景及西南总景确认三层比例、转角连续性和既有独立屋檐衔接

预览里的朴素楼体和原屋檐只是导出后的内存审查上下文，没有保存到新主文件或GLB。中性DCC光照下的自查不代表实际街景中颜色、反光与最终日漫画风已获作者认可

## 接入边界

建议完整跳过V-A08旧 `facades()`，保留主墙、基础、独立屋檐、稳定ID、道路和入口规则。旧泛型构件298方盒／3576三角为源码与Bun复算估计，实际Rust差分由集成确认；上一批移除的48三角压顶不重复统计

新覆层厚0.18m、角条最大0.255m，闭合门叶仍为不可进入外观。本次集成已采用 `CollisionWorld::from_scene` 将本立面实际GLB三角按视觉相同变换加入既有碰撞，保留原主墙，独立屋檐仍仅可视；三入口外侧及墙边的运行复验见 `todo/evidence/TASK-049/blender-integration-r2`；不能以从关闭门叶中心穿过作为成功条件，也不能把原门前命中全部忽略

独立只读复核核对了导出副本、预览顺序、来源、默认再导出和验收边界，发现的未使用导入已删除，证据链接已补齐，最终结论为 `Lean already. Ship.`。本地收尾重新运行GLB检查通过，两份Python AST、18个本地文档链接和两份冻结资产哈希检查通过，原屋檐六个文件无Git差异；没有改变冻结几何或材质

## 产物清理

四张最终CPU PNG已实际查看并记录尺寸、字节和哈希后删除，详见 [previews.json](previews.json)；本轮 `output/buildings/V-A08/facade-r1/` 已移除，保留可编辑源、正式GLB、脚本、来源、日志、数值JSON和结论。其他Agent的镜头JSON与产物保持原样
