# 山脚合流、西住宅落地与站前会合庭院

## 输入与交付

本轮基线 `389c309815e2a6235941b98a8f7b7c7986353c89`，最终地图 SHA256 `b1088fb91d9d58d84e97067d0bf92d2e967496a59e40579c6c813c1ed13a8039`。沿用真实地图、WorldScenePlugin、几何和Viewer输入，未更换渲染或道路检查规则

- 修复山脚住宅门口合流，西住宅台地梯脚、后排门路和顶部横街，增加同高落地再接斜路；建筑、门、原对象ID及450m单峰保留
- 站前会合庭院移至原地块内南侧8m平地，面积168→156m²；新3m平路接南北商业步道。北侧square仍9m，经原坡支路降至商业侧，再进入庭院；主路与庭院之间的草带和挡边不作为直接跨越入口
- 全城路幅候选61→45，移除16对、0新增；长环线hill及短登高hill-short均0候选。短路线全部节点与坐标保留，三维长度仍1311.187345m
- 最终1573节点、880道路、8389地形样点，新增9节点6段道路，无原ID删除；20段改动道路与建筑轮廓交叠检查通过。详细差异、稳定ID、地形及平台净空见[最终源检查](final-source-check.json)和[独立复核](final-source-review.md)

站前应用前的[源阶段](source-review.md)使用独立hash，保留红测和中间结果；本页及final开头文件对应最终集成输入。庭院与通路关系回写[站前规格](../../../../docs/dev/design/locations/district-station.md)，具体坐标继续只维护于地图源

## 实际检查

| 命令或检查 | 结果与范围 |
| --- | --- |
| 新路口回归，旧源 | [7 PASS／1 FAIL](road-width-before.log)，明确复现山脚与西住宅的全宽错层 |
| 新广场回归，旧源 | [0 PASS／1 FAIL](station-square-before.log)，明确复现斜路穿过水平庭院 |
| `bun test tools/tests/road-width.test.ts tools/tests/station-square.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | [46 PASS／0 FAIL](tests-final.log)，覆盖路幅、所有连续梯段与踏面、地形、庭院面积及到达、建筑入口和道路消费者 |
| `bun tools/check-road-width.ts all` | [预期FAIL，45对](final-road-width-all.json)，保持原阈值并列出剩余项 |
| `bun tools/check-road-width.ts hill`／`hill-short` | [长线0](final-road-width-hill.json)、[短线0](final-road-width-short.json)，均PASS |
| `bun tools/terrain-shape.ts --check`、`bun run check:types` | [地形无漂移](terrain-final.log)、[TS与Vue类型PASS](typescript-final.log) |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib --bin map_viewer` | [22项lib＋4项Viewer PASS](rust-tests.log)，抬高检查机位后[4项Viewer再测PASS](final-viewer-tests.log) |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --locked --all-targets -- -D warnings` | [最终PASS](final-clippy.log) |
| `cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer` | [最终PASS](final-build.log) |
| `game/target/debug/map_viewer --project-root . --validate` | [PASS](viewer-validate.log)，231建筑、82平台、72树、3259网格；仅CPU几何和绑定 |
| `bun run docs:build` | [player 95页／dev 157页PASS](final-docs-build.log)，保留既有chunk体积提示 |
| `bun run tasks:sync`、`bun run tasks:check`、`bun run check:docs` | 收尾见[任务检查](tasks-check.log)及[文档与Skills检查](docs-check.log) |

## 真实机位与画面观察

Linux原生GPU共完成14次固定机位运行：5次修前、7次修后及2次北路口补拍，全部进程退出0。修前使用基线提交的district.json及隔离项目目录，未改资产通过符号链接复用；各组前后使用同一Viewer二进制与同一镜头，不另造演示场景。源及二进制hash、命令和退出码见[修前](before/render-checks.json)、[修后](after/render-checks.json)、[修前补拍](before-supplement/render-checks.json)、[修后补拍](after-supplement/render-checks.json)

| 机位 | 实际画面自查 |
| --- | --- |
| 山脚住宅合流 | [修前](before/inspect-road-foothill-gate.webp)下行斜路压住横支；[修后](after/inspect-road-foothill-gate.webp)住宅支路先同高并入，再爬升。后方大面积切坡与挡墙仍为灰盒品质，不算本批完成 |
| 西住宅梯脚 | [修前](before/inspect-road-west-plateau.webp)底端梯段伸进横街；[修后](after/inspect-road-west-plateau.webp)水平落地接街，门口和上端短平台保持可辨 |
| 西住宅后排 | [修前](before/inspect-road-west-rear.webp)门前短路与斜段错层；[修后](after/inspect-road-west-rear.webp)路面平接，原楼体及门保持原位 |
| 北侧横街 | 初次[修前](before/inspect-road-res-north.webp)／[修后](after/inspect-road-res-north.webp)机位贴入坡面，视觉范围FAIL，不能据进程退出0验收；仅将观察眼高127→153m后，[修前补拍](before-supplement/inspect-road-res-north.webp)与[修后补拍](after-supplement/inspect-road-res-north.webp)清楚显示原错层消除，未改变场景来获得截图 |
| 站前会合庭院 | [修前](before/inspect-road-station-square.webp)8.5m板横切路口并露出裂缝；[修后](after/inspect-road-station-square.webp)庭院与南北8m步道平接，独立于9m主路边缘，原穿板问题消除。另由未修改最终源的代理独立看图复核；这是自查，不是盲评 |
| 山形与第三停步台 | [山形](after/mountain-profile.webp)仍为单峰，[第三停步台](after/eye-ascent-3.webp)仍可见城市和河岸。保留42个手工控制点不等于全部生成地形不变，实际重拍核对视线 |

18秒真实相机输入录制完成540帧、30fps、1280×720，资源就绪、有限Transform、相机转向和释放控制后停止等8项断言均PASS，见[运行结果](capture/run.json)、[状态](capture/state.json)与[日志](capture/runtime.log)。已实际查看[连续帧90](capture/frame00090.webp)、[91](capture/frame00091.webp)、[关键帧149](capture/frame00149.webp)、[359](capture/frame00359.webp)：转向中庭院、通路与路口保持清楚，这组帧中未见新的路面跳变

原PNG序列及MP4保存在忽略目录 `output/roads/2026-09-27/r7/station-look/`，编码PASS，直接视频播放NOT RUN；本轮用关键帧和连续帧检查，未将抽帧称作完整视频观看。机器run.json的visual_review自动字段保持NOT RUN，本页另记实际看图范围。图像转换为质量80的WebP，原文件与归档hash见[provenance.json](provenance.json)

## 重跑与后续

在仓库根由fish直接执行，输出到新目录保留历史证据

```fish
cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer
game/target/debug/map_viewer --project-root . --verify-headless output/roads/recheck-r7/station --view inspect-road-station-square
game/target/debug/map_viewer --project-root . --verify-headless output/roads/recheck-r7/res-north --view inspect-road-res-north
python3 tools/capture.py --script todo/evidence/TASK-025/r7/capture/script.json --output output/roads/recheck-r7/capture --binary game/target/debug/map_viewer
bun tools/check-road-width.ts hill
bun tools/check-road-width.ts hill-short
bun tools/check-road-width.ts all
```

最后一项仍应退出1并列出45对候选。TASK-025保持active，本批技术修复完成；下一批优先东侧住宅台地、西侧旧住区和食品街接入。人物碰撞、车辆转弯、Windows实机、最终美术和作者体验均NOT RUN；站前座椅、信息柱等原设定设施尚未制作，源连通和自由相机运行不证明这些功能已完成

按n-side-review核对输入与证据，G1验收数不变；代码及测试经独立正确性检查和ponytail-review，结论 `Lean already. Ship.`。日志仅清理行尾空白及多余末尾空行，保留原失败与版本
