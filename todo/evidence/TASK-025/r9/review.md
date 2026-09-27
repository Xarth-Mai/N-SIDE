# 第9轮：小店公共台阶与东部街道

基线 `58725a3`，最终源 `df2e13f0630151ecf564588b1cb8f85491b89b55e1bcd47813dcbd15b5d805f8`；[逐项源检查](final-source-check.json)和[运行来源](provenance.json)记录范围。TASK-025继续active，下一步处理另行发现的道路与平台重叠；G1未放行

## 修改与保留

最后20对地面路幅高差候选消除，全城 [0候选](all-road-width.json)，未放宽面积0.5m²／高差0.35m检查阈值。增加全城回归，下一次地图修改会检查所有地面道路；这仍不覆盖平台厚板、建筑、挡墙、人物碰撞或最终美术

小店庭院与西门路先平出后上坡，公共台阶上下端各留平段，西旧住区联络梯顶部平接。东住宅坡路通过局部弯折保持坡度并从门面间穿过；上街在已有节点汇合。音乐街保留直达和经停music_west两条功能路径，合流后共用一段真实道路；诊所前路与后勤入口分别平接。东北林缘出路和上街平台先平出边界再上坡

24个新增节点、4条新增landing；原节点坐标、所有建筑、门、平台、地块、树和42个手工地形点保持原样。烘焙样点8402→8348，记录实际变化。长短登高分别5228.80m／1311.19m，节点、坐标与长度原样；影院缓行路线不变。修改路面按真实宽度与全楼体占地比较，0交叠

## 实际验证

新增全城回归在旧源真实FAIL，见 [red.log](red.log)；正式源 [50项PASS](tests-final.log)，覆盖路幅、地形、站前庭院、入口、路由、连续梯段与踏面。独立试验脚本最初把bakeTerrain返回的整份District当作terrain，且漏改动态import，产生harness失败；修正临时脚本后33项真实trial通过，正式源使用原入口烘焙，没有错误结构写入

| 已执行命令 | 结果 |
| --- | --- |
| `bun test tools/tests/road-width.test.ts tools/tests/station-square.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | 50 PASS／0 FAIL |
| `bun tools/check-road-width.ts all` | PASS，0候选 |
| `bun tools/terrain-shape.ts --check` | PASS，8348样点 |
| `bun run check:types` | PASS |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib --bin map_viewer` | 22库＋4Viewer测试PASS |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --locked --all-targets -- -D warnings` | PASS |
| `cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer` | PASS |
| `game/target/debug/map_viewer --project-root . --validate` | PASS |
| `bun run docs:build` | PASS，player95页／dev157页 |
| `python3 tools/capture.py --script todo/evidence/TASK-025/r9/capture/script.json --output output/capture/roads-r9 --binary game/target/debug/map_viewer` | 同脚本在本轮shop-look实际完成540帧／18秒，8断言PASS |

命令可直接在fish执行，capture使用新的输出目录。6修前＋11修后＋各1补拍，共19次GPU运行全部退出0；补拍仅改机位，B1/B2和源hash分别记录。PNG序列与视频在忽略目录 `output/roads/2026-09-27/r9/`；仓库保留WebP、命令、状态、日志与hash

## 实际画面自查

已查看全部修前后目标画面：小店庭院平出、上层楼梯三段落地、旧西联络梯顶部、东住宅绕行、上街合流、诊所接入与东北山脚接点可见。音乐街接近下街时两条路共用同段，原相互压过的宽度高差消失；铺装仍有三角拼接缝和局部薄边，邻近切坡保留灰盒大面，未认定最终品质

小店西巷初拍被V-A04屋顶遮住，视觉FAIL，保留原图；从街东侧 [补拍](after-supplement/inspect-road-shop-west.webp) 能看到店门支路与先平后坡的主巷。主峰全景及第三停步台回归保留单峰与城区/河岸视线

连续帧90/91及关键帧149/359已实际查看，转头期间上层台阶和转折稳定，无明显帧间跳变；8状态检查确认资源、帧数、Transform、转向以及Escape后持有W不移动。已生成MP4，未整段播放，视觉结论仅覆盖实际抽看帧

## 后续与验收边界

另行扫描发现20处道路与非架空平台高差重叠，包含小店休息台、住宅后步道、滨水广场、邻里庭院、站西庭院及峰顶边缘；历史对比显示在本轮之前已经存在，不能由road-road的0候选豁免。第10轮继续修复，保留平台用途、楼门与面积变化证据，并补相应运行图

人物移动/碰撞、车辆、Windows、作者操作与最终美术均NOT RUN；本轮没有创造新的玩法或代签验收

复杂度复查：源数据修复、沿用既有测试辅助函数与固定机位，无生产依赖或新框架。`Lean already. Ship.`
