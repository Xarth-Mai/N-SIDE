# Map Viewer 素材候选

核验日期：2026-09-25

本清单服务于全图基础外观及小店—采购街样板，采用[美术规范](../docs/production/art.md)中的轮廓、店面分段、生活细节和材质尺度要求，资产接入遵循[资产管理](../docs/production/assets.md)

首批 Kenney 与 ambientCG 子集已取得并导出，精确文件、原件哈希、米制和材质调整见[环境素材包](../source-assets/environment-kit/README.md)；可复现导出、Bevy 异步依赖加载及六镜头窗口渲染检查 PASS。全图基础外观与店面首轮画面已检查，样板美术验收仍待作者确认，替补与待核条目保持 `candidate`，性能及限制见[实际验证记录](archive/map-viewer.md#实际验证记录)

## 首批选择

先用两包少量模型与三种基础材质验证街景，建筑体量、入口、高差和道路轮廓由地图生成；公共模型负责绿植与街道小物件，原创店面构件和招牌负责 N:SIDE 的识别

| 优先级与候选 | 选用目标 | 官方许可、免费范围与格式证据 | 适配工作与待核项 |
| --- | --- | --- | --- |
| 首批：[Kenney Nature Kit](https://kenney.nl/assets/nature-kit) | 选两种阔叶树冠、一种低矮植物和一种石块，覆盖树点与种植带 | 产品页为免费单包，明确 CC0；[官方导入指南](https://kenney.nl/knowledge-base/game-assets-3d/importing-3d-models-into-game-engines)说明 Kenney 的 glTF 以 GLB 分发；本轮已核验包内 2.1 许可、GLB 与实际成员 | 逐项测量真实高度、冠幅和枢轴，统一到米制；控制树冠轮廓和色阶，近景检查低面数表面是否过于积木化；所需低矮植物若缺失则原创简单灌木，不扩展下载整套自然生态 |
| 首批：[Kenney City Kit (Roads)](https://kenney.nl/assets/city-kit-roads) | 路灯、路牌及少量护栏等街边构件 | 产品页为免费单包、CC0，更新记录明确加入路牌和交通灯；[作者的 itch.io 页面](https://kenney-assets.itch.io/city-kit-roads)列出 OBJ、FBX、DAE、STL、glTF，但该页显示较旧版本，本轮已核验当前 2.1 包内 CC0 声明、GLB 与依赖色板 | 只选小构件，保留地图定义的路面形状；校准灯杆、护栏高度和色板，替换需显示的文字与标识；已选路灯和空杆的贴图依赖、底部枢轴已记录，护栏待后续选取 |
| 首批：[ambientCG Concrete034](https://ambientcg.com/view?id=Concrete034) | 浅色混凝土、挡墙、平台边缘与建筑基础墙面 | 免费 CC0，页面提供 1K–16K JPG/PNG PBR 包，注明约 1.1 m × 0.55 m 的覆盖尺寸 | 首批已取 1K JPG，后续按镜头降低细碎法线和色差；UV 按实际尺寸铺设，不能把一张长方形纹理强制按正方形米制映射 |
| 首批：[ambientCG PavingStones092](https://ambientcg.com/view?id=PavingStones092) | 采购街与小店邻近的人行铺装 | 免费 CC0，页面提供 1K–12K JPG/PNG PBR 包，注明约 1.55 m × 1.55 m 的覆盖尺寸 | 首批已取 1K JPG；原材质为红色铺装，调整为街区约定的低饱和色域，保留铺装缝与尺度，近景检查扫描细节是否抢过店面 |
| 首批：[ambientCG Asphalt012](https://ambientcg.com/view?id=Asphalt012) | 车行路面 | 免费 CC0，页面提供 1K–4K JPG/PNG PBR 包 | 首批已取 1K JPG，后续按镜头控制深色路面与裂纹的对比；页面未标实物覆盖尺寸，导入时用道路米制宽度和参考颗粒尺寸校准并记录 UV 周期 |
| 替补：[Poly Haven Painted Plaster Wall](https://polyhaven.com/a/painted_plaster_wall) | 小店街旧粉刷墙面，仅在基础混凝土无法表达店面差异时选用 | 产品页标 CC0、作者 Amal Kumar、宽 2 m，提供 1K–16K、Blend/glTF/MaterialX/ZIP，单通道包含 PNG/JPG/EXR | 先选 2K PNG 的颜色、OpenGL 法线和粗糙度；压低污渍与破损对比，将经营痕迹集中在墙根和设备附近；不为这一材质引入位移渲染 |

表中选取目标用于后续补齐；当前实际子集为两种树、一种灌木、一种路灯、一种导视杆、道路色板及三套 1K JPG 颜色与 OpenGL 法线图，石块与护栏尚未选取，实际交付以环境素材包清单为准

Kenney 的[官方支持说明](https://kenney.nl/support)确认资产页素材采用 CC0，可用于商业项目；本轮只选独立免费包，不依赖 All-in-1 商店产品

[ambientCG 许可页](https://docs.ambientcg.com/license/)明确允许复制、修改、分发及在项目内包含原始素材；[Poly Haven 许可页](https://polyhaven.com/license)明确允许素材再分发，满足公开源仓库保留所选主文件的需求

## Quaternius 待核候选

两个产品页仍标示 CC0，但站点[许可页](https://quaternius.com/license.html)当前为 2026-08-28 更新的 QAL v1.0，允许素材进入游戏等产品，同时限制以素材形式单独分发原件或修改件；第 7 节称后续修改不追溯已按旧版本取得的素材

本项目尚未取得这些文件，当前网页证据不足以确认即将下载文件适用哪份授权，因此首批不选择这两个包，也不把其原始文件安排进公开仓库；这不是对历史 CC0 授权失效的判断

| 候选 | 产品页可确认的范围 | 若以后核清许可后的用途 | 风格与交付差距 |
| --- | --- | --- | --- |
| [Downtown City MegaKit](https://quaternius.com/packs/downtowncitymegakit.html) 免费子集 | 产品页写约 60–70% 免费、OBJ/FBX/glTF；Blend 源文件和完整引擎项目属于 Source 版本，不能将全包宣传数量计入免费交付 | 只考察窗框、檐口、屋面设备和少量街边构件，实际成员以免费包为准 | 整体是 Boston/NYC 建筑语言，与 N:SIDE 店面尺度有距离；替换立面组合、颜色和招牌；Source 版窗内景、磨损与倒角 shader 不属于标准 GLB/PBR 接入承诺 |
| [Ultimate Stylized Nature Pack](https://quaternius.com/packs/ultimatestylizednature.html) | 产品页列 FBX、OBJ、Blend、glTF、无缝纹理和法线，标 CC0 与免费商用；该页没有另列免费与付费子集 | 如果 Kenney 树冠无法通过近景验收，考察少量树、灌木和地被替换 | 逐项确认树种形态、树冠体积、叶片透明模式、纹理风格和米制尺寸；实际包内授权、依赖文件、网格成本均待核 |

后续只有取得与具体下载版本对应且允许计划分发方式的明确许可证据，才把上述条目转入接入清单；本轮使用首批 CC0 候选继续推进，不依赖这项核验完成

## 原创补缺

| 部件 | 制作方式与放置依据 |
| --- | --- |
| 建筑外壳、天井、屋顶、外部平台、挡墙、道路与台阶 | 按 `district.json` 的空间事实生成，外观规则独立关联源对象；公开模型不反向决定建筑尺寸与路网 |
| 门窗分段、雨棚、卷帘、空调外机简体、花盆与店外陈列 | 先制作样板街实际需要的少量米制构件，统一枢轴与材质；单个造型满足需求时直接制作，不提前搭建可配置构件框架 |
| 商店招牌、橱窗图形、价格牌和街区导视 | 使用项目原创文字与图形，保留可编辑源文件；绝区零画面用于构图和密度研究，品牌、图案与游戏资产不进入交付 |

## 后续接入顺序

1. 地图当前阶段交接后，从官方入口取得首批候选，保留所选主文件、包内许可及来源记录；只在实际取得文件后填写精确文件名和校验值
2. 先验证一株树、一个路灯、一段铺装及一面墙的 GLB/PBR、米制比例、法线与阴影，保留贴图依赖或导出为自包含 GLB
3. 将材质纹理周期、模型原始尺寸与转换参数记入资产包说明；导出的运行派生物放入 `game/assets/`，Wiki 位图另按 WebP 质量 80 导出
4. 在小店—采购街的白天镜头中审查轮廓、经营细节密度、PBR 反光和植物与建筑的协调，再决定是否需要替补材质或新增原创构件
5. 接入通过后记录实际采用的子集与修改，不复制整个候选包；真实 Viewer 画面通过视觉验收后再标记 `integrated`

网站宣传图只作为链接参考保留，Poly Haven 许可页将示例渲染等网站内容与 CC0 素材本体区分；本清单不下载或再发布宣传图
