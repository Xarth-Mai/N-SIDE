# 第三停步台视线修复与校园、采购街运行复验

本轮输入为 `2d6f374d8d41ba40cd992321e3f473ff13291cee`；最终地图 SHA-256 为 `058d871a151bb2694de5f1b6888b99fae5a86501668631e9fedd319276e8bb6c`，准确值和逐项变化以 [source-check.json](source-check.json) 为准

## 交付

第三停步台保留 `[330,880,410]`，相机仍在平台内朝城市偏移3m、眼高1.7m；增加三处近坡控制点 `[320,870,402]`、`[320,810,358]`、`[260,810,350]`，按原配方重烘焙地形。原39个手工点、450m唯一峰顶、平台和短登高路线保持不变，水平路长1213.93m、空间路长1311.19m

新增地形视线回归：从同一眼位向小店、车站、住宅上街、镜厅屋顶以0.5m间隔检查，保留 [修前真实FAIL](sightline-before.log) 和 [修后41项PASS](final-integrated-tests.log)。它只证明地形不遮挡射线，建筑、树木和实际画面另行观察

同批校园与采购街修复见 [TASK-025/r3](../../TASK-025/r3/review.md)。移除面包铺路径精确往返的重复路段，保留6m横街连接；全城实际路幅候选98→85，短登高路线仍为0

## 实际命令与结果

以下命令在仓库根执行，Cargo命令使用 `--manifest-path game/Cargo.toml`

| 命令 | 结果 |
| --- | --- |
| `bun test tools/tests/road-width.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | PASS，41项，见 [整合日志](final-integrated-tests.log) |
| `bun test tools/tests/road-width.test.ts` | PASS，4项；新增即时折返检查后重跑，见 [日志](final-road-tests.log) |
| `bun tools/terrain-shape.ts --check` | PASS，8375点，无漂移，见 [日志](final-terrain-check.log) |
| `bun run check:types` | PASS，见 [日志](types-check.log) |
| `cargo test --manifest-path game/Cargo.toml --features viewer --lib --bin map_viewer` | PASS，22项库测试及4项Viewer测试，见 [日志](final-rust-tests.log) |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --all-targets -- -D warnings` | PASS，补充相机后再跑通过，见 [日志](final-clippy.log) |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo build --manifest-path game/Cargo.toml --features viewer --bin map_viewer` | PASS，补充下段相机后的 [最终构建](final-build.log) 和 [4项Viewer回归](final-viewer-tests.log) |
| `game/target/debug/map_viewer --project-root . --validate` | PASS，1532节点、862道路、231建筑、82平台、72树、3223网格，见 [日志](viewer-validate.log) |
| `bun tools/check-road-width.ts all` | FAIL，85对仍待处理，见 [全城报告](../../TASK-025/r3/final-road-width-all.json) |
| `bun tools/check-road-width.ts hill-short` | PASS，0对，见 [短路线报告](../../TASK-025/r3/final-road-width-short.json) |

## 真实画面自查

9个固定机位均实际运行完成，见 [退出记录](render-checks.json)、[源文件与二进制记录](provenance.json)。前8机位后只增补一个学校下段检查相机，地图未再变化；记录分别保留两个二进制hash

- [第三停步台](views/eye-ascent-3.webp)：城区与河岸重新可见；原来近坡填满画面的遮挡消除，平台和眼位未抬高
- [前两处停步台与山形对照](terrain-contact.webp)：城市回望保留，摘星台仍为唯一450m最高点；灰盒山体棱角和大面积切坡仍需要后续美术处理
- [学校首个机位](../../TASK-025/r3/views/inspect-road-school-east.webp)：前景道路挡住下段视线，不能单凭此图验收下段；补拍 [反向下段机位](../../TASK-025/r3/views/inspect-road-school-lower.webp) 后，两段台阶、中间平台及下口均可见，无地形穿入；高挡墙及支柱仍属灰盒表现
- [采购街入口](../../TASK-025/r3/views/inspect-road-market-entry.webp)：水平接入离开主路后起步，台阶连接沿店平台
- [采购街转角](../../TASK-025/r3/views/inspect-road-market-turn.webp)：平层支路分别接入，既有相邻路段的小高差继续归全城剩余项

这些是Codex实际看图自查，未实施隔离盲评或作者美术放行

## 连续运行

`python3 tools/capture.py --script game/capture/ascent-lookout.json --output output/terrain/2026-09-27/r6/ascent-lookout --binary game/target/debug/map_viewer`

PASS：30fps、540帧、18秒，8项机器检查通过，连续截图及MP4均已生成。脚本通过真实Viewer的M键启用、左右观察、Escape释放、释放后继续输入路径执行；没有直接写入相机结果。固定位置、三段转向和释放后不再移动均有状态检查，见 [run.json](capture/run.json)、[状态](capture/state.json) 和 [原生运行日志](capture/runtime.log)

实际查看 [关键帧及连续90–93帧](capture/look-contact.webp)，转向期间城市视线保持；未用视频播放器播放MP4。另对480–539帧核对释放后的旋转只有一个唯一值，见 [后验检查](capture/release-rotation-check.json)，与脚本原生断言分开记录

原始PNG、视频和完整运行数据在 `output/terrain/2026-09-27/r6/`；归档图为质量80的WebP，日志仅去尾随空白，内容来源见provenance

## 未完成与下一动作

人物移动、碰撞、真实登山、手柄、Windows和最终美术验收均NOT RUN；Viewer自由相机不证明这些能力。TASK-023与TASK-025保持active，本轮没有新增G1验收；下一批处理音乐街配送路与西侧步行接入

文档与消费者收口：`bun run check:docs` PASS（294篇、31 Skills），`bun run docs:build` PASS（player 95页／dev 157页），`bun run tasks:sync` 与 `bun run tasks:check` PASS（21卡）；见 [文档检查](docs-check.log) 与 [双站构建](docs-build.log)

独立代码复核重跑41项Bun测试并核对稳定ID、来源hash及capture语义，无阻断发现；提交前复杂度审查：Lean already. Ship.
