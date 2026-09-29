# 站厅与音乐建筑室外立面 r1

本轮只修改 [scene.rs](../../../../game/src/world/scene.rs) 的立面生成及同文件窄测，输入版本与 hash 见 [run.json](run.json)。复用现有 `station`、`music` 建筑家族以及 `concrete`、`metal`、`trim` 材质，无新增下载、模型、纹理、许可或游戏设计

## 输入与落点

- [美术方向](../../../../docs/dev/production/art-direction.md)：先明确建筑轮廓、入口与成组结构细节，避免随机污渍或无用途装饰
- [街区建筑](../../../../docs/dev/design/locations/district-architecture.md)及[站前建筑](../../../../docs/dev/design/locations/district-station.md)：楼层保持水平，构件沿实际立面与楼层放置，现代站厅以成组窗面、入口和遮蔽形成识别
- [滨水建筑](../../../../docs/dev/design/locations/district-waterfront.md)：音乐场馆公共到达与设备服务分侧，演出体量具有自己的外皮而非复制住宅窗带
- [地图唯一源](../../../../source-assets/district-map/district.json)：V-01 N站真实公共门在东侧 `station_entry`，北侧已有长雨棚；V-79 AFTER 9 北侧 `live_entry` 为观众入口，南侧是设备服务入口，首层高 8 m

## 实际变化

| 家族 / 对象 | 构件 | 安装与尺度 |
| --- | --- | --- |
| station：V-01、V-32 | 首层站厅玻璃比例、贴壁柱与楼层顶收边 | 复用 4.2 m 窗格节奏及源层高；柱宽 0.32 m、厚 0.24 m，柱底至楼层顶下 0.25 m，避开真实门节点与门宽 |
| music：V-78、V-79 | 不透光上部金属板与竖缝 | 首层从 +3 m 起到层顶下 1.2 m，上层从 +0.35 m 起；保留现有高窗，2.4 m 左右一块板，板厚 0.10 m，竖缝宽 0.065 m |

这些面板是室外分层表皮，不宣称模拟隔声物理。现有门、楼层、建筑体量、雨棚深度和屋顶海报全部保留，未增加室内开放范围

新构件以 `buildings[ID]/derived-facade/skin` 追溯原建筑，并进入既有碰撞筛选。每栋按材质合并，四栋共增加 8 个网格批次、378 个立方构件、9,072 个顶点和 4,536 个碰撞三角形；材质继续复用，不增加运行纹理占用

## 检查与限制

- PASS：`cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib world::scene::tests -- --nocapture`，12 项通过，见 [日志](scene-tests.log)
- PASS：新窄测直接读取真实地图，检查新增构件有限坐标、正体积、非负有限米制 UV、高程不超原建筑、贴壁距离不超过 0.25 m、沿源边不越端点、与真实门口及窗面的包络无交叠，且全部新三角形进入碰撞
- PASS：现有 224 栋、15 类型、342 个独立门口、5 个采光庭覆盖检查，原小店室内边界和屋顶海报检查继续通过
- PASS：`cargo fmt --manifest-path game/Cargo.toml --check` 与 `git diff --check -- game/src/world/scene.rs`
- NOT RUN：本子任务未运行 GPU 或查看本轮实机截图，由主 Agent 串行复验；机器边界检查不能证明材质层次和远景细节密度已经合适

## 接续 capture

沿用现有 `game/capture/poster-station.json`、`game/capture/poster-live.json`，不增加复制场景；先重建当前 Viewer，再从相同机位比较楼体轮廓、入口辨识、金属板反差及海报与立面的主次

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-station.json --output output/capture/task049-facades-r1-station
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-live.json --output output/capture/task049-facades-r1-live
```

实际查看站前的北侧展示面后，再转到东侧确认真实入口；AFTER 9 重点查看首层窗带至上层高窗之间的板面，横移时检查竖缝闪烁和与窗框的关系。大屏色彩和主角画风仍由 TASK-048、TASK-047 分别验证

本子任务没有产生 PNG、视频、接触表或临时可执行文件，保留测试日志与来源参数；后续 capture 由执行者按项目规则查看、记录后清理
