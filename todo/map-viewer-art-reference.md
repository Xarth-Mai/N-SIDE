# Map Viewer 美术参考研究

核验日期：2026-09-25；本轮研究服务于 [TASK-003](map-viewer.md)，正式制作规则维护于[美术](../docs/production/art.md)

作者确定以绝区零六分街为主、光映广场为辅，首版通过强店面识别、图形、构件、材质和白天光照兑现环境方向。以下分别记录资料明确表述、实际画面观察与 N:SIDE 的制作选择，参考图仅保留来源链接

## 官方与直接采访来源

| 编号 | 来源与定位 | 可支持的内容 |
| --- | --- | --- |
| A1 | [制作人 Zhenyu Li 的 PlayStation Blog，2024-01-31](https://blog.playstation.com/2024/01/31/hoyoverses-action-rpg-zenless-zone-zero-is-coming-to-ps5/)，The urban fantasy world | 城市居民来自不同生活背景，电玩店与咖啡店经营者使街道具有具体用途与人物关系 |
| A2 | [制作人 Zhenyu Li 的 PlayStation Blog，2024-06-28](https://blog.playstation.com/2024/06/28/zenless-zone-zero-debuts-on-ps5-july-4-details-on-combat-new-area-and-characters-unveiled/)，The bustling cityscape of New Eridu | 光映广场定位为商业区，录像店楼上包含卧室、音乐和照片墙等生活功能 |
| A3 | [MMORPG.com 对制作人的直接采访，2024-06-21](https://www.mmorpg.com/interviews/zenless-zone-zeros-producer-talks-multiplayer-healing-characters-console-releases-and-more-interview-2000131984)，childhood memories 所在回答 | 街机、录像店、唱片店与旧收音机等复古元素联系共同的生活记忆 |
| A4 | [官方 Fairy Library: City Guide，2023-11-23](https://zenless.hoyoverse.com/en-us/news/113747) | 测试期六分街店铺图文，下面四张官方 CDN 长图已实际查看；用于画面语言研究，不作为当前版本逐店状态清单 |
| A5 | [PlayStation 官方产品页](https://www.playstation.com/en-us/games/zenless-zone-zero/)，城市截图 | 光映广场的道路、商业体量与公共设施；对应原图为黄昏，光色不作为首版白天参数依据 |

## 画面观察与转化

| 画面定位 | 实际可见内容 | N:SIDE 的制作选择 |
| --- | --- | --- |
| A4：[Random Play 长图](https://fastcdn.hoyoverse.com/content-v2/nap/113747/f92d1e8aba01216290372f4c15169d37_4349586996475971417.jpg)，首张街景 | 朴素楼体、黄色转角门面、门窗分段与连续横招牌，首层承担主要识别 | 建筑保持源轮廓与米制尺度，重点投入首层入口、橱窗、招牌及转角；楼上通过窗带和生活痕迹建立较安静的背景 |
| A4：[Store Introduction #2](https://fastcdn.hoyoverse.com/content-v2/nap/113747/b17507f91236bd9f6c99418153ae3323_4487646671909839363.jpg)，Bardic Needle、Turbo、Waterfall Soup | 唱片店采用平面化门脸与橱窗，维修区域有外露钢架、平台和设备，餐饮店使用开放柜台与凳位 | 按经营方式选择构件，小修理铺、兴趣店、食品店与生活杂货店形成不同界面，避免一套门窗套遍所有用途 |
| A4：[Store Introduction #4](https://fastcdn.hoyoverse.com/content-v2/nap/113747/6872debb1fe490234d22bb8e7e57b667_4176829308480928467.jpg)，141 与 Box Galaxy | 141 的大店标、蓝色雨棚、入口贴纸与外摆分层；Box Galaxy 用橙色立面、斜切门框和悬挑方标形成轮廓 | 每家重点店面安排主图形、主色与入口，次级菜单和陈列随靠近展开；原创标识从主要来路可读 |
| A4：[Store Introduction #3](https://fastcdn.hoyoverse.com/content-v2/nap/113747/508c1e1a98bc7572a84ccdaf6c83fcac_7171591894882661345.jpg)，报刊亭、电玩店与咖啡店 | 报刊亭前陈列、电玩入口立牌、咖啡店菜单与桌椅集中在可解释的经营位置；各店主色有区别 | 用物件表达候客、售卖、展示和休息，把细节集中在使用位置，保留经过与停留空间 |
| A4 四张店铺长图对照 | 店标、门框、檐口和大色块先于小细节被辨认，海报、拼缝与货架提供第二层信息 | 共用安静的墙面和铺地，店面局部使用醒目颜色；减少材质高频噪声，优先结构与图形层级 |
| A5：[光映广场城市原图](https://gmedia.playstation.com/is/image/SIEPDC/zzz-cityscape-screenshot-02-en-27jun24?fmt=jpeg&wid=1920) | 宽十字路口、连续路缘、人行护栏、架空交通层及较大影院体量；重复折面、圆角体块和广告框形成地标 | 站前和文娱中心采用更开阔的商业界面及清楚公共入口，与小店街形成空间节奏差异；道路宽度和体量仍由 N:SIDE 地图决定 |

上述构图、色块与构件关系属于图像观察，制作规则属于本项目选择。图片不能证明原作的 shader、贴图频率、曝光或 PBR 参数；本项目采用标准 PBR、克制的粗糙度与法线细节，通过自己的场景样板确定具体数值

六分街参考含不同明暗时段，能确认遮檐阴影、立面明暗和经营色块的组织；光映广场参考为暖色黄昏。首版白天光照以清晰体量、可辨阴影与入口为目标，单独完成实机验收

## 落地与检查

[正式美术规范](../docs/production/art.md)将参考提取为 N:SIDE 的建筑、经营图形、颜色、材质和生活物件规则；角色及 UI 只保留与环境协调的原则，完整游戏的昼夜方向与 Viewer 首版仅白天的范围分别维护

优先检查小店主来路、店前人眼视角、服务侧院、上坡回望和高处全景，确认外形与源数据一致、入口可辨、图形有层级、物件有用途。按同一材质和构件规则扩展相邻采购街，避免不同公开模型包各自形成一套风格

本轮仅查看官方公开宣传图与直接采访，临时研究图片位于仓库外；模型、纹理、品牌与图案没有作为项目资产接入。参考研究完成，N:SIDE 实际 3D 样板、渲染观感和性能均待后续阶段验证
