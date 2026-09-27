# 住宅上街与温室入口接续

## 输入与改动

基线 `1002bf878f741b2e9e775de5394dde6a54f0dd0c`，修前地图 SHA-256 `791ae5548dece4227ec7363c72c12b6ae189d39e0b42c75f3fe8cab26043c9db`，本批源 `9435838bb2143c7dcee3844345a345b125cee98c3d45bbae8d4a9134724280ac`

- 住宅后街 `homes_high_m` 至 `homes_high_junction` 的两段台阶改为后场空地内的平行折返，各长12m、升高约4.747m，28级、踏面约0.429m；同高接入原75.316m后街与84.81m上街，中间保留同高转向平台
- 同一6m宽上街的四季温室、V-F59、V-F60三处入口先同高退出4m，再经过12m阶梯与2m门前平台到原门。分别35／38／39级，最小踏面0.308m，原门坐标与标高不变
- 增加12节点、8道路；原两处阶梯中间节点只改平面位置，高程与稳定ID全部保留。建筑、平台、入口、场所、生活路线、后勤与剖面保持原数据，本次变动的旧边没有额外路线或到达路径消费者
- 沿用已有地形烘焙，42个手工控制点和配方不变，样点8375→8390。摘星台单一450m最高点与短登高路线不变

[逐项记录](source-check.json)保存改动节点、道路、受保护数据、阶梯尺寸和建筑相交结果。本批未调整检查器阈值，未创建额外道路或地形实现

## 源数据检查

| 命令 | 实际结果 |
| --- | --- |
| `bun test tools/tests/road-width.test.ts`，修前 | 5 PASS、1 FAIL；[失败日志](road-width-before.log)命中本批12对原交叠 |
| `bun test tools/tests/road-width.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | 43 PASS、0 FAIL，[日志](tests.log)，覆盖阶梯、路幅、原短登高路线、第三停步台视线及全部入口／权限／生活路线 |
| `bun tools/terrain-shape.ts` | 重建8390样点，[日志](terrain-bake.log) |
| `bun tools/terrain-shape.ts --check` | PASS，无漂移，[日志](terrain-check.log) |
| `bun node_modules/typescript/bin/tsc -p tsconfig.json` | 退出0、无诊断，[日志](typescript-check.log) |
| `bun tools/check-road-width.ts all` | 预期 FAIL，退出1；[全城候选](road-width-all.json)83→71，移除12对、无新增 |
| `bun tools/check-road-width.ts hill-short` | PASS，退出0，[短登高路线](road-width-short.json)仍0候选 |

修改后的道路与所有建筑进行既有 `roadHitsBuilding` 检查，相交0；新增平台全部同高，阶梯满足当前每段最多48级、踏面至少0.28m的地图约束。日志仅清除行尾空白和多余末尾空行

## 源数据阶段边界

本节记录源数据与客观几何结果，实际GPU机位与看图由集成执行补记。全城仍有71对待逐项修复；人物碰撞、真实上楼操作、Windows与最终美术验收NOT RUN。现有Viewer自由相机能够提供道路与地形观察，不能代替人物可通行验收

## 集成运行与视觉自查

同一最终源完成CPU Viewer校验、2机位修前／修后GPU对照及1个补拍近景，命令与退出结果在 [before](before/render-checks.json)／[after](after/render-checks.json)，源与两次二进制hash见 [provenance](provenance.json)。补拍只增加检查相机，地图不变

- [修前住宅上街](before/inspect-road-upper-residential.webp)可见门前阶梯深入横街，[修后](after/inspect-road-upper-residential.webp)三条入口均在街外起阶，保留完整横街与门前平台
- [修前楼后](before/inspect-road-upper-stairs.webp)存在上口路面覆盖梯段的交叠；[修后反向视角](after/inspect-road-upper-stairs.webp)可见两端同高连接，阶梯进入独立折返范围
- 总览中下梯段接近侧视，补拍 [低处近景](after/inspect-road-upper-stairs-low.webp)核对下梯段、6m中段平台和上梯段连续可见，未见平台挡墙横穿踏面；内部陡切面和大三角支撑仍是灰盒表现，未作最终美术验收

`python3 tools/capture.py --script output/roads/2026-09-27/r5/residential-look.json --output output/roads/2026-09-27/r5/residential-look --binary game/target/debug/map_viewer` 实际PASS：30fps、540帧／18秒、8项断言通过、MP4生成，见 [run](capture/run.json) 与 [可复用脚本](capture/script.json)。复用既有左右观察／释放输入，不直接写相机结果；实际查看 [149帧](capture/frame00149.webp)、[359帧](capture/frame00359.webp)及连续 [90](capture/frame00090.webp)／[91](capture/frame00091.webp)，转向中楼梯与上街关系稳定可见。未直接播放视频，未把自由相机当成人物行走

构建命令均使用 `--manifest-path game/Cargo.toml --features viewer --locked`：`cargo build … --bin map_viewer` PASS；`cargo test … --lib --bin map_viewer` 22+4项PASS；补拍相机后4项Viewer再测PASS；`cargo clippy … --all-targets -- -D warnings` PASS；`cargo fmt --manifest-path game/Cargo.toml --check` PASS。见 [库与Viewer测试](rust-tests.log)、[最终构建](final-build.log)、[最终Viewer测试](final-viewer-tests.log)与 [最终Clippy](final-clippy.log)

`game/target/debug/map_viewer --project-root . --validate` PASS，见 [日志](viewer-validate.log)；`bun run docs:build` PASS（player 95页／dev 157页），见 [双构建](docs-build.log)。原PNG与MP4留在 `output/roads/2026-09-27/r5/`，归档图为质量80 WebP

源实现与集成差异复核无阻断问题，提交前复杂度审查：Lean already. Ship.

`bun run check:docs`、`bun run tasks:sync` 与只读 `bun run tasks:check` 均PASS；见 [文档检查](docs-check.log)，任务维持active，未新增G1验收
