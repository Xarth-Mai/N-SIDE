# 站前、音乐北门与长山径接入

## 源变更

基线 `74102782c05eab0c4d1001c5989319a9a9eec8ea`，最终地图 SHA-256 `93d682bce484cb57f182577f504e98d5ff7c22b8f54acdfa252e967da864f315`；完整逐项变更、消费者、坡度、踏面和源保护见 [source-check.json](source-check.json)

- 站前广场的北、西、东支路先同高接出；北向采购街轻弯保留爬升长度，避免插入平段后把后续坡路拉陡
- 城西服务支路先退出15m主路，再以短折线接原岸线装卸路；音乐街北门以绕坡接既有门前道路，保持无台阶、原宽度与公共权限
- 长山径的山脚接入和两处高位弯道形成同高休息段，保留必要梯段；原两节点 `hill_east_arc_08_10` / `hill_east_arc_09_01` 调到弯道的434.444444m，摘星台450m不变
- 神龛原接近段从V-22底板下穿过约72.29m²平面轮廓，不能因低于底板就认定安全通行。正式用途为“登高步道旁短支路”，西侧唯一门点为 `fw_e_v_22_public0`；增加南侧两点绕到原西侧 `shrine`，保留6m门前支路、原门和楼体。改动道路全宽与全部建筑轮廓相交降为0，实际挡墙与地形仍须看图
- 16个新节点，2个原节点只调高程；道路870→874，节点1548→1564。所有原ID、建筑、平台、树、地块、权限及后勤记录保留，短上山路径与节点不变
- 相关public/service到达、往返路线、剖面和街道样段按原边同步插点；没有删除 `east_g → east_j` 等既有道路
- 正常调用既有 `bakeTerrain` 后保持42个手工点及8390样点数量，59个生成样点更新；第二次计算与源一致

## 实际数值检查

| 检查 | 结果与证据 |
| --- | --- |
| `bun test tools/tests/road-width.test.ts`，源修改前 | 6 PASS、1 FAIL，[原始回归失败](road-width-before.log)命中本批目标 |
| `bun test tools/tests/road-width.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | 44 PASS、0 FAIL，[日志](tests.log)；覆盖前几批、入口/权限/后勤、第三停步台、短登高与新接入 |
| `bun tools/terrain-shape.ts --check` | PASS，[8390样点无漂移](terrain-check.log) |
| `bun node_modules/typescript/bin/tsc -p tsconfig.json` | 退出0、无诊断，[日志](typescript-check.log) |
| `bun tools/check-road-width.ts all` | 预期FAIL、退出1，[全城候选](road-width-all.json)71→61，移除10对、0新增，阈值不变 |
| `bun tools/check-road-width.ts hill-short` | PASS、退出0，[短路0候选](road-width-short.json) |

低地消除7对、长山径消除3对；神龛修复的是另一个原有建筑穿越问题，不增加路幅候选的“完成数”。本批修改的非台阶道路最大纵坡10%；实际梯段最多33级、最小踏面0.3157m。全城仍有61对候选，不代表人物碰撞或车辆通行已通过

## 真实运行与画面自查

Linux原生GPU共完成14次固定机位运行，全部退出0：3次旧源对照、9次修改后检查及2次补拍。旧源取自基线提交的district.json，放在隔离目录，通过符号链接复用未改动资产；前后使用同一真实Viewer及世界系统。补拍版本仅调整站前机位并增加神龛西门机位，地图源未再改变。实际命令、源与二进制hash分别保存在[修前](before/render-checks.json)、[修后](after/render-checks.json)、[补拍](supplement/render-checks.json)，图片及视频hash见[provenance.json](provenance.json)

| 观察范围 | 实际观察与结论 |
| --- | --- |
| 长山径第六折返 | [修前](before/inspect-trail-turn-six.webp)上层路面压住下行梯段；[修后](after/inspect-trail-turn-six.webp)先同高转向再下降，原重叠消除 |
| 长山径第八折返 | [修前](before/inspect-trail-turn-eight.webp)端部叠层；[修后](after/inspect-trail-turn-eight.webp)转向段同高，梯段在其外接入 |
| 山脚东入口 | [实图](after/inspect-trail-east-entry.webp)可见同高连接与梯段；大面积灰盒切坡及挡墙棱角仍需后续处理 |
| 神龛接近段与西门 | [修前](before/inspect-trail-shrine.webp)道路消失在建筑底板下；[修后南侧](after/inspect-trail-shrine.webp)绕行清晰，[西门补拍](supplement/inspect-trail-shrine-door.webp)确认原6m门前支路及门口无新遮挡 |
| 站前路口 | [初次机位](after/inspect-road-station-square.webp)被楼体遮挡，视觉检查FAIL，即使进程退出0也不足以检查道路；[补拍](supplement/inspect-road-station-square.webp)看清新平接路口，同时确认既有平台8.5m与square 9m的半米边界仍未解决 |
| 城西服务路 | [实图](after/inspect-road-city-west.webp)显示支路先退出主路宽度，再接原装卸路径；车型与转弯半径NOT RUN |
| 音乐街北门 | [实图](after/inspect-road-music-north.webp)新绕坡及门前接入可见；旁边east_j旧路仍有叠层，留在61对候选中，不算整片通过 |
| 山形与第三停步台 | [山形](after/mountain-profile.webp)维持450m单峰；[第三停步台](after/eye-ascent-3.webp)仍能看见城区与河岸。本轮59个生成高程更新，最大约+28.365m，因此实际重拍这两处，没有把手工控制点不变写成全部地形不变 |

以上为Codex画面自查，检查构图、道路接合和遮挡，不是作者审美验收、独立盲评或人物行走测试

18秒连续输入录制完成540帧、30fps、1280×720，8项状态检查PASS，包括资源就绪、Transform有效、真实相机转向及释放控制后阻止按住移动。实际查看[连续帧90](capture/frame00090.webp)、[91](capture/frame00091.webp)及[关键帧149](capture/frame00149.webp)、[359](capture/frame00359.webp)：同高折返在转向中保持连续，没有在这组帧中发现路面跳变。机器记录见[run.json](capture/run.json)、[state.json](capture/state.json)、[运行日志](capture/runtime.log)

原PNG序列与MP4保存在忽略目录 `output/roads/2026-09-27/r6/trail-look/`；MP4编码PASS，直接视频播放NOT RUN，以实际抽帧检查补充。录制操作为Viewer观察控制，未伪造尚不存在的人物碰撞或登山玩法；run.json的自动visual_review字段仍为NOT RUN，本段另记实际人工看图范围

## 构建与复验命令

| 实际命令 | 结果 |
| --- | --- |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib --bin map_viewer` | [22项lib、4项Viewer PASS](rust-tests.log)，补拍机位修改后[4项Viewer再测PASS](final-viewer-tests.log) |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --locked --all-targets -- -D warnings` | [最终PASS](final-clippy.log) |
| `cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer` | [最终PASS](final-build.log) |
| `game/target/debug/map_viewer --project-root . --validate` | [PASS](viewer-validate.log)，1564节点、874道路、231建筑、82平台、72树、3248网格；范围为CPU几何及绑定 |
| `bun run check:docs` | [PASS](docs-check.log)，收尾记录后再检查见final-docs-check.log |
| `bun run docs:build` | [player 95页、dev 157页PASS](docs-build.log)，保留既有chunk体积提示 |
| `bun tools/check-road-width.ts hill` | [预期FAIL](road-width-hill.json)，仍有foothill_house_gate两对0.896m／0.665m高差，不能宣布长山径全线通过 |

以下命令可在仓库根直接由fish运行，复用已归档的输入脚本生成新证据

```fish
cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer
game/target/debug/map_viewer --project-root . --verify-headless output/roads/recheck/shrine --view inspect-trail-shrine-door
python3 tools/capture.py --script todo/evidence/TASK-025/r6/capture/script.json --output output/roads/recheck/trail-look --binary game/target/debug/map_viewer
bun tools/check-road-width.ts hill-short
bun tools/check-road-width.ts all
```

最后一项当前应返回1并列出61对候选；每次源修改后须重跑相关检查，不能用本轮结果替代新输入

## 剩余范围与下一步

TASK-025保持active，本批技术修复完成，未增加G1验收数。下一批优先处理foothill_house_gate两对、住宅西侧平台及站前平台半米边界；音乐街旁旧交叠与其余全城候选继续逐项处理。短登高路线保持原节点与路径，实际路幅检查仍为0候选；数值连续性和建筑平面净空不能替代人物碰撞、车型转弯半径、实机行走、Windows或最终美术验收，这些均NOT RUN

源与测试差异经独立正确性检查及ponytail-review，结论 `Lean already. Ship.`。日志仅清除行尾空白和多余末尾空行，保留真实失败、退出码、原始版本及画面范围
