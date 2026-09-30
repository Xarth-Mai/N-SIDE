# 街区环境素材

| 项目 | 内容 |
| --- | --- |
| 资产 ID | `AST-003` |
| 状态 | `exported`，已生成运行派生物；加载与渲染结果见本轮任务证据，样板美术验收待作者确认 |
| 主文件 | 本目录中的 GLB、JPG、道路色板和原始许可文件，逐文件见 [asset-manifest.json](asset-manifest.json) |
| 作者 | Kenney、ambientCG；模型米制归一、植物材质调整与原创街树、松树、灌木、草及石块由 N:SIDE 制作 |
| 许可 | 第三方素材为 CC0 1.0，原创植物沿用项目 MPL-2.0，逐文件见清单；保留 [CC0 正文](licenses/CC0-1.0.txt)、[Nature Kit 包内声明](licenses/kenney-nature.txt)及[City Kit Roads 包内声明](licenses/kenney-roads.txt) |
| 获取日期 | 第三方素材取得与原创资产制作日期逐项见清单 |
| 使用位置 | 正式入口与 3D Map Viewer 共用的街树、灌木、公园与登高路旁植物土石、街灯、铺装与墙面；导视空杆已导出、尚未放置 |
| 运行派生物 | `game/assets/environment/`，由导出命令生成，运行期间无需联网 |

## 来源与取得范围

| 来源 | 取得内容 | 证据 |
| --- | --- | --- |
| [Kenney Nature Kit](https://kenney.nl/assets/nature-kit) | `tree_detailed.glb`、`tree_oak.glb`、`plant_bushDetailed.glb`、`tree_pineRoundA.glb`、`tree_cone_fall.glb`、`rock_smallA.glb`、`grass.glb`、`flower_yellowA.glb` | 原包 `License.txt` 标 Nature Kit 2.1 与 CC0；所选 GLB 内嵌纯色材质，无外部纹理依赖 |
| [Kenney City Kit Roads](https://kenney.nl/assets/city-kit-roads) | `light-curved.glb`、`road-sign-empty.glb`、依赖的 `Textures/colormap.png` | 原包 `License.txt` 标 City Kit Roads 2.1 与 CC0；保留色板相对路径 |
| [ambientCG Concrete034](https://ambientcg.com/view?id=Concrete034) | 1K JPG Color、NormalGL，各 1024×512 | 官方产品页与[许可页](https://docs.ambientcg.com/license/)确认 CC0，可在项目中分发原始文件 |
| [ambientCG PavingStones092](https://ambientcg.com/view?id=PavingStones092) | 1K JPG Color、NormalGL，各 1024×1024 | 同上 |
| [ambientCG Asphalt012](https://ambientcg.com/view?id=Asphalt012) | 1K JPG Color、NormalGL，各 1024×1024 | 同上 |

所选文件通过官方 ZIP 的 HTTP Range 提取，ZIP CRC 校验通过，下载链接、包内成员路径及原件 SHA-256 记录在清单；仓库只保留实际选用文件，未保存完整包，因此没有完整 ZIP 的哈希

`road-sign-empty.glb` 是导视空杆，文字和牌面由项目原创；Quaternius 与 Poly Haven 候选尚未下载

## 米制、枢轴与材质

| 运行模型 | 原件几何高度 | 导出高度 | 三角面数 |
| --- | --- | --- | --- |
| `nature/tree_detailed.glb` | 1.332417 | 6 m | 402 |
| `nature/tree_oak.glb` | 1.226240 | 5.5 m | 196 |
| `nature/plant_bushDetailed.glb` | 0.360411 | 0.8 m | 104 |
| `nature/tree_pineRoundA.glb` | 1.3664372 | 6.5 m | 204 |
| `nature/tree_cone_fall.glb` | 1.43033278 | 5 m | 132 |
| `nature/rock_smallA.glb` | 0.191224009 | 0.65 m | 16 |
| `nature/grass.glb` | 0.254 | 0.35 m | 132 |
| `nature/flower_yellowA.glb` | 0.1925 | 0.35 m | 76 |
| `roads/light-curved.glb` | 0.675 | 4.5 m | 92 |
| `roads/road-sign-empty.glb` | 0.475 | 2.3 m | 42 |

运行 GLB 使用默认场景 `Scene0`、Y 轴向上、地面枢轴，默认实体缩放为 `1`；树木与道具造型本身保留原件，导出只修正场景根节点的缩放与底部高度，灯臂沿原模型的负 Z 方向延伸

Nature Kit 原件的场景节点含 `y=-0.05` 偏移，植物材质 `metallicFactor=1`；导出将底部置于 `y=0`，植物改为非金属、粗糙度 `1`，树叶与树皮按清单中的线性颜色调整为较克制的绿色与棕色

颜色图按 sRGB 加载，OpenGL 法线图按线性数据加载；JPG 主文件保持原件，运行纹理从主文件导出为完整 11 级 mipmap 的无压缩 BGRA8 DDS，原始分辨率不变，首次接入用固定粗糙度，无位移贴图；自然地形后续使用项目 `terrain-slope.wgsl` 连续混合，下述岩土段记录其边界

DDS 使用 ImageMagick 的默认缩小过滤，不作颜色空间转换；颜色图 mip 暂按编码值平均，法线 mip 按通道平均并由标准 PBR shader 在采样后归一化，保留首次验证通过的简洁导出链

Concrete034 的一周期覆盖 1.1 m × 0.55 m，PavingStones092 为 1.55 m × 1.55 m，均来自官方产品页；Asphalt012 页面未标尺寸，首轮暂定 4 m × 4 m，属于表现参数，需在真实镜头中校准

## 导出与检查

在仓库根目录执行：

```sh
bun tools/export-environment.ts
bun tools/export-environment.ts --check
```

导出需要 Bun 与 ImageMagick，本次验证版本为 `ImageMagick 7.1.2-31 Q16-HDRI`；[官方 DDS 格式说明](https://imagemagick.org/formats/)列出压缩与 mipmap 选项

导出先校验全部原件 SHA-256，再执行 GLB 根节点与植物材质调整；每张 JPG 使用 `magick <source> -alpha on -define dds:compression=none dds:-` 导出，附加不透明 alpha 明确选用 BGRA8，避免 RGB24 转码的通道解释差异，纹理未经 BC 压缩

导出器检查 DDS 头部的尺寸、像素掩码、完整 mip 层数和数据字节数，其余文件逐字节复制；生成的 `game/assets/environment/manifest.json` 记录派生物大小与 SHA-256，`--check` 重算派生结果并逐字节比较，不写文件

原件与导出参数为唯一编辑入口；新增素材时先记录来源、许可、原件哈希和尺寸，再更新清单与运行派生物

2026-09-25 已运行 DDS 导出及 `--check`，15 个文件共 28,054,454 bytes 校验通过；6 张纹理均含从原始尺寸至 1×1 的 11 级 mip，Concrete 法线图的第 0 级解码像素与 JPG 主文件一致，旧运行 JPG 已移除

上述结果验证文件完整性、mip 数据与可复现导出，最新 GPU 材质效果、相机近景和场景性能由 Viewer 验收记录承接

## 全城与登高预览的同家族补充

按作者加入更多免费公共素材的要求，2026-09-27从同一Nature Kit 2.1选取针叶树、秋色树、小石、草丛和黄花，原件路径、用途、SHA-256、包内成员及取得日期记入既有清单；包内许可证与此前留存文件逐字节一致，获取费用为0。原件保留，运行导出沿用米制归一、底部枢轴及非金属粗糙材质，叶色和土石颜色与现有环境统一

已有植树点按海拔和有限的秋色比例使用变体；新增自然小簇放在公园与登高路线两侧，按实际模型范围避让楼体、路面、水体与公共平台，保留停步和看景空间。空间与外观生成沿用现有主数据和 `world::scene`，不把装饰植被另存为一份城市地图

这里的用途与尺寸是预览制作规格，运行与看图按[运行验证](../../docs/dev/validation/runtime.md)记录在对应任务；灰盒资产的风格适配不代表最终美术已获作者验收

## 墙面与外饰板补充

新增 ambientCG [Plaster001](https://ambientcg.com/view?id=Plaster001) 和 [WoodSiding009](https://ambientcg.com/view?id=WoodSiding009) 的 Color / NormalGL，按[官方 CC0 许可](https://docs.ambientcg.com/license/)使用。此次完整取得 1K-JPG ZIP，核对 ZIP CRC 与每个原件 SHA-256；整包 SHA、文件 SHA、尺寸、来源与获取日期记录于现有 asset-manifest.json，不沿用早期分段下载的完整包哈希缺失说明

林缘坡地使用 ambientCG [Ground037](https://ambientcg.com/view?id=Ground037) 的原始 1K-JPG Color／NormalGL，稀疏苔草与裸土共同形成地表，按官方约 2.1×2.1m 周期导出 DDS。完整 ZIP 的 CRC 与原件 SHA 已核对，来源、许可、尺寸与获取日期保存在同一清单；颜色和法线分别按 sRGB 与线性数据加载。地形位置与法线保留，UV 按三角面的主轴投影到世界米制平面，避免顶视投影在陡坡压缩成条纹；同主轴共享坐标，主轴切换处有纹理方向边界，不是无缝三向混合。固定世界坐标色斑及坡度调色补充远景层次；这是缓坡苔土地表，陡面由下述裸岩材质分工；两者尚未覆盖完整山林制作，平铺重复与投影交界通过真实近景检查

Plaster001 供米白与青灰外墙使用；WoodSiding009 供样板店外侧木饰板使用。源图分别为 1024×1024 与 1024×512，运行 DDS 保留原尺寸和完整 mip 链。当前以 2×2m 和 2×1m 作为视觉试排周期，官网没有提供米制覆盖范围；比例需在真实街景与近处镜头校准

## 原创街树样板

新增[初秋街树](vegetation/README.md)，以真实分枝与 1,452 片阔叶替换 `tree_a` 的块状冠，6m 高、11,686 三角、2 个材质与一张内嵌树皮纹理，保持既有树点、根部枢轴与放置半径。可编辑 Blender 主文件与源码在同目录，GLB 通过既有导出器复制；项目原创来源与 MPL-2.0 单独登记，原 CC0 素材的许可保持不变。全城植被、风动与 LOD 尚未完成，运行成本与实机观感按任务证据验证

当前清单列出 35 个运行文件，包含 3 份许可文件，并非新增 35 种素材；原创植物单独标为 MPL-2.0，其余条目保留 CC0-1.0。`appearance.models.tree_a.file` 已绑定 `environment/vegetation/street-tree.glb`，原 Kenney 树保留来源与导出条目；Blender 主文件、造型／导出程序、GLB 和几何检查入口都在 vegetation 目录，实际制作与运行覆盖分别见 [CPU 制作记录](../../todo/evidence/TASK-045/vegetation-r1/review.md)和[第 2 轮实机记录](../../todo/evidence/TASK-045/runtime-r2/review.md)

## 陡坡岩面与原创植物

ambientCG [Rock043L](https://ambientcg.com/view?id=Rock043L) 的 1K-JPG Color／NormalGL 用于自然地形陡面的裸岩土，按官方约 `1.8 × 1.8 m` 尺度导出，CC0 许可、完整 ZIP 校验和及原件 hash 沿用同一清单。地形使用单个扩展标准 PBR 材质：依据未扰动的插值世界法线，在暂定 35–55° 范围内连续混合 Ground037 与 Rock043L 的颜色、法线及粗糙度，普通渲染和法线预通道保持一致。原几何、碰撞与主轴米制 UV 保留，主轴切换处仍需检查纹理方向边界；连续草岩覆盖不代表已完成全山植被或无缝三向贴图


## 草与石块细化

`appearance.models.grass` 与 `rock` 使用本目录的原创 [草石主文件](vegetation/ground-props.md)，保留原 Kenney 文件及来源。草使用真实曲面叶片和原创不透明颜色图，石块使用原创几何与既有 ambientCG Rock043L 颜色／法线；它们复用既有植物布点与地面适配，尺寸和许可逐项登记，不代表已形成连续森林

地形材质使用固定世界坐标的连续旋转／相位采样和 2m 三角格三顶点混合，法线同步逆旋转，普通渲染与预通道共用采样。它改善明显的同向平铺，仍保留主轴投影交界和三角地形轮廓的制作边界

## 公共砖面素材

ambientCG [Bricks057](https://ambientcg.com/view?id=Bricks057) 补充暖棕砖面与可辨认的砖缝，供镜厅相邻音乐街、店屋侧墙或围墙的局部面试排，避免把全部外墙变成统一旧砖。来源发布日为 2021-04-11，官网标一周期约 `1.05 × 1.05 m`，原件及许可在同一 AST-003 清单登记；[CC0 许可](licenses/CC0-1.0.txt)允许随游戏分发，来源为 Lennart Demes / ambientCG

原始 1K JPG Color／NormalGL 保存在 `materials/public-urban/`，不修改原件。沿既有导出器生成 `game/assets/environment/materials/public-urban/brick-color.dds` 与 `brick-normal.dds`，各 `1024 × 1024`、11 级 mip，颜色以 sRGB 采样、OpenGL 法线以线性数据采样。`brick_music` 只绑定 V-W10 西面首层的局部饰面，按 1.05m 周期连续展开 UV，保留门框与 2.2×2.55m 门洞；其他建筑的 `wall_brick` 保持现状

饰面在原墙外 12mm、原门框后方，不另加碰撞，旧墙继续承担阻挡；三块平面共六个三角形，位置与 UV 由真实建筑、楼层和门节点派生。源图、ZIP CRC、派生 DDS、平铺预览及实际引擎检查范围记录在[本轮证据](../../todo/evidence/TASK-049/public-urban-r1/review.md)，导出继续使用上方两条既有命令；当前仍是局部美术候选
