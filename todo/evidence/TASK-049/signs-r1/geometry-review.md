# 经营标識真实建筑接入

复用 `scene.rs` 的门面盒体、牌面、材质加载与真实世界入口，新增 `business_signs` 批次；每项关联现有公共门、原外墙和 `buildings[ID]/derived-facade/name-sign`，没有改变建筑、道路、入口或室内。源图与导出由本目录其他记录负责，准确源图 hash、贴图及高度见 [placement.json](placement.json)

| 建筑 / 入口 | 图形实际比例 | 实机尺寸 | 牌面底／顶高程 | 安装依据 |
| --- | --- | --- | --- | --- |
| V-01 / station_entry | 1200:256 | 7.5 × 1.6 m | +12.85 / +14.45 m | 东侧实际到达门上方，低于原 +15 m 屋顶，高于首层站厅玻璃 |
| V-35 / game_entry | 1024:256 | 6 × 1.5 m | +15.25 / +16.75 m | 北侧原雨棚上方，首层高 6 m 的不透明墙带 |
| V-36 / models_entry | 1024:256 | 6 × 1.5 m | +15.25 / +16.75 m | 北侧首二层窗间，保留原雨棚和二层玻璃 |
| V-39 / st_39_door | 1024:256 | 6 × 1.5 m | +15.25 / +16.75 m | 北侧窗间墙带，牌框顶低于二层窗底 +17.015 m |
| V-79 / live_entry | 1024:320 | 8 × 2.5 m | +11.15 / +13.65 m | 北侧演出大厅不透明外皮，首层入口与屋顶明星海报分别保持 |

每个框牌外沿比画面宽／高各多 0.16 m，框厚 0.24 m；牌面外挑 0.365 m，使用独立 0—1 UV 保留原图比例，材质 `tile_meters=[1,1]`。沿用普通受光店招材质，不把物理牌面默认改为发光屏

appearance.json 新增 `sign_station`、`sign_byte_beat`、`sign_frame`、`sign_playroom`、`sign_after9` 五个已有 PNG 的材质槽。没有把它们塞入旧五店 8:1 式门楣；旧五店和明星大屏保留各自职责

按 root 指令同批只把 `models.tree_a.file` 换为已验证原创街树 `environment/vegetation/street-tree.glb`，其余模型绑定保持；街树资产生产与运行验证由对应 Agent 和 root 记录

## 检查

- PASS：`rustfmt --edition 2024 game/src/world/scene.rs`、`git diff --check -- game/src/world/scene.rs source-assets/district-scene/appearance.json`、Python JSON 解析及五材质／街树路径核对
- NOT RUN：编译由 root 统一执行，新增 `business_signs_keep_source_aspect_arrival_direction_and_openings` 直接读取实际 PNG IHDR、真实地图及旧门窗雨棚几何，检查画面比例、正确朝向／绕序、门上净空、屋顶界限、门窗雨棚包络与 70 个新增碰撞三角形；移开 N站入口的负例须报源墙宽度错误
- NOT RUN：GPU、透视文字阅读、动态遮挡和夜景亮度，本子任务没有生成或保留临时视觉文件

实际画面至少查看 N站东侧入口、兴趣街三处北侧经营面和 AFTER 9 北侧前场。N站北广场海报机位不代表东侧门楣可读，按真实来路补证
