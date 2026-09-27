# 音乐街配送与西侧步行接入

## 输入与改动

基线 `37c9576e3c379a83e97c7081e8d349801e505f45`，修前地图 SHA-256 `058d871a151bb2694de5f1b6888b99fae5a86501668631e9fedd319276e8bb6c`，本批源 `791ae5548dece4227ec7363c72c12b6ae189d39e0b42c75f3fe8cab26043c9db`

- 配送支路从既有 `fw_w_music_delivery` 同高退出12m主路，经过两段坡路到原器材装卸位；最大纵坡8.408%，小于原直接接入段约8.94%。这只是灰盒纵坡比较，尚未验证具体车型与转弯半径
- 西侧支路从 `music_west_walk` 同高横出后转向原 `fw_w_music_mid`；V-W10 从后者同高接门，保留原门、楼体与街坊用途
- 增加4节点，改3条道路的节点序列，道路数仍862；原1532节点坐标和ID全部保留，现1536节点。建筑、平台、入口、42个手工地形控制点及全部8375样点均不变，短登高路线保持原数据
- 场所77的服务到达路径与 `music-equipment` 车辆段沿真实新路同步；车辆仍在原卸货位转为同高推车路线

[逐项源保护与几何记录](source-check.json)保存改前/改后道路、路径消费者、坡度和建筑相交检查。坡路转折由3个新配送节点表达，未引入新道路系统或放宽原扫描阈值

## 实际检查

| 命令 | 结果与证据 |
| --- | --- |
| `bun test tools/tests/road-width.test.ts`，修前 | 4 PASS、1 FAIL；[真实失败日志](road-width-before.log)准确命中原2处交叠 |
| `bun test tools/tests/road-width.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | 42 PASS、0 FAIL，[结果](tests.log)；包括学校/采购街、第三停步台、路径权限和场所到达回归 |
| `bun test tools/tests/road-width.test.ts`，最终类型标注后 | 5 PASS、0 FAIL，[结果](final-road-tests.log) |
| `bun tools/terrain-shape.ts` | `Terrain is current`，无需改变现有烘焙样点 |
| `bun tools/terrain-shape.ts --check` | PASS，[8375样点无漂移](terrain-check.log) |
| `bun node_modules/typescript/bin/tsc -p tsconfig.json` | 退出0、无诊断，[日志](typescript-check.log) |
| `bun tools/check-road-width.ts all` | 预期 FAIL，退出1；[全城候选](road-width-all.json)85→83，移除2对、无新增，阈值保持重叠面积0.5m²/高差0.35m |
| `bun tools/check-road-width.ts hill-short` | PASS，退出0，[短登高路线0候选](road-width-short.json) |

已移除的原最大高差分别为 `music_cross` 2.3944m、`music_west_walk` 1.9511m。全城仍有83对待逐项处理，包含长登山路径转折与上层住宅接入；本批不代表全城道路已验收

## 源数据阶段范围

本源数据子任务未运行GPU，实际Viewer机位与观察由集成执行补记。数值检查覆盖地面路幅、地面建筑相交、到达节点与后勤路径，不能证明人物碰撞、车辆转弯或玩家体验。日志只清除行尾空白及多余末尾空行，原始诊断与退出结果保留

## 集成运行与实际看图

地图保持上述最终hash。`cargo fmt --manifest-path game/Cargo.toml --check`、`cargo build --manifest-path game/Cargo.toml --features viewer --bin map_viewer`、`cargo test --manifest-path game/Cargo.toml --features viewer --bin map_viewer`、`cargo clippy --manifest-path game/Cargo.toml --features viewer --all-targets -- -D warnings` 均PASS，4项Viewer测试通过；见 [构建](build.log)、[测试](viewer-tests.log)、[Clippy](clippy.log)

`game/target/debug/map_viewer --project-root . --validate` PASS，1536节点、862道路、3223网格，见 [CPU校验](viewer-validate.log)。两个固定机位真实GPU渲染均退出0，确切命令见 [render-checks.json](render-checks.json)，源和二进制hash见 [provenance.json](provenance.json)

- [配送支路](views/inspect-road-music-delivery.webp)：主路同高退出、两段坡路、卸货台和后场路连续可辨认，没有可见挡墙横切新路；卸货台处有road/surface材质三角交界，未见竖直高差障碍
- [西侧步行与门口](views/inspect-road-music-west.webp)：横出后下行与同高V-W10门支路可辨认，门口仍对应原门；主街原有高差挡墙和建筑灰盒保留

Root与源数据协作者均实际查看两张原尺寸图；这是知晓设计和源码的自查，不称隔离盲评，也不证明车辆转弯或人物碰撞

连续运行复用 `ascent-lookout.json` 的现有观察输入，仅更换检查机位和位置断言名称，归档 [本次脚本](capture/script.json)，没有新增第二套录制器。以下命令在仓库根可用于fish复现，输出使用新目录

```sh
python3 tools/capture.py --script todo/evidence/TASK-025/r4/capture/script.json --output output/capture/music-r4-recheck --binary game/target/debug/map_viewer
```

[运行结果](capture/run.json) PASS：30fps、540帧、18秒、8项断言通过，MP4生成。通过Viewer真实M启用、左右观察、Escape释放及释放后持续输入，验证固定位置、转向和释放后停止移动；只验证自由相机。实际抽看 [149帧](capture/frame00149.webp)、[359帧](capture/frame00359.webp) 及连续 [90帧](capture/frame00090.webp)／[91帧](capture/frame00091.webp)，左右观察中支路与主路连接保持可见，没有观察到跳变或挡墙穿入；视频文件未直接播放

`bun run check:docs` PASS；`bun run docs:build` PASS（player 95页、dev 157页），见 [文档检查](docs-check.log) 与 [双站构建](docs-build.log)。原始PNG、MP4和完整运行产物归 `output/roads/2026-09-27/r4/`，归档图为质量80的WebP

独立源/消费者/画面复核无阻断发现；提交前复杂度审查：Lean already. Ship.

本轮继续保持TASK-025 active；下一批按剩余报告处理住宅上街与长登山路折返，人物、车型、Windows与最终美术验收仍NOT RUN

任务收口：`bun run tasks:sync` 与只读 `bun run tasks:check` 均PASS（21卡），未新增已验收计数
