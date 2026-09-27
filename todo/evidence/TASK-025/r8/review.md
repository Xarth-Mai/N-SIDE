# 第8轮：东台地、旧住区与食品滨水接入

基线 `3121c90`，最终源与二进制hash见 [provenance.json](provenance.json)，逐节点、道路与保留对象见 [final-source-check.json](final-source-check.json)。本轮为源几何与Viewer技术修复，TASK-025仍active，G1未放行

## 实际交付

全城路幅高差候选45→20，原25对消除、0新增；长短登高均0候选。新增31节点、17净新增道路，3个原梯道中间点平面调整，原标高保留；全部建筑、门、平台、地块、树与42个手工地形点保持原样。道路控制点改变后烘焙样点8389→8402，不能宣称整个地形数据逐值不变

东台地梯脚与木屋巷口先同高离街，再起阶；F63入口在原门外增加回折与顶部平段；两条相邻台地梯道分离并在同高上街真实相交。西侧兴趣巷改为直角接入，旧住区梯脚与内院保留平段。食品北街与镜厅路线沿现有门面绕开V-F39/F40/F41，原穿楼捷径退出；影院缓行往返888.49→961.69m，单程增加约36.6m，各段≤5%。渡口下行改在候船庭院东侧，旧楼梯穿入厚板的问题消除

## 失败与修正

新回归在旧源上实际8 PASS／3 FAIL，见 [red.log](red.log)。第一次集成真实49项回归出现3 FAIL：食品缓行路5.238%超限、两处同高交叉缺共同节点、横移后休息平台不足2m，见 [tests.log](tests.log)。修正为食品街缓弯、渡口/上街共用交点及完整2m平段，保持原检查阈值

滨水隔离报告原称45 PASS，但复制测试时误读了旧共享源，该结论撤回；[修订记录](water/review.md)说明原因，[真实旧试验失败](water/real-initial-fail.log)及[正确试验通过](water/tests-corrected-trial.log)保留。根集成测试直接读取正式源，不使用该旧结论。临时紧凑写入器首次遇已烘焙terrain行而拒绝写入，修为保留现有terrain再调用正式bake，未造成部分写入

类型检查发现测试JSON推断无string索引，显式使用既有District类型后通过；失败 [types.log](types.log) 与修后 [types-final.log](types-final.log) 分开保留

## 命令与结果

以下均在仓库根真实执行，命令可用于fish；capture输出使用新目录避免覆盖证据

| 命令 | 结果 |
| --- | --- |
| `bun test tools/tests/road-width.test.ts tools/tests/station-square.test.ts tools/tests/terrain-shape.test.ts tools/tests/district-map.test.ts` | 49 PASS／0 FAIL |
| `bun run check:types` | PASS |
| `bun tools/terrain-shape.ts --check` | PASS，8402样点 |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo test --manifest-path game/Cargo.toml --features viewer --locked --lib --bin map_viewer` | 22库测试＋4Viewer测试通过 |
| `cargo clippy --manifest-path game/Cargo.toml --features viewer --locked --all-targets -- -D warnings` | PASS |
| `cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer` | PASS；补拍只修改机位，再构建B2 |
| `game/target/debug/map_viewer --project-root . --validate` | PASS |
| `bun run docs:build` | PASS，player95页／dev157页 |
| `python3 tools/capture.py --script todo/evidence/TASK-025/r8/capture/script.json --output output/capture/roads-r8 --binary game/target/debug/map_viewer` | 已用同脚本在本轮dock-look运行：540帧、18秒、8状态断言PASS |

原始命令、运行秒数、源hash及退出码在各phase的render-checks.json；6修前＋15修后＋各2补拍，共25次GPU运行均退出0。人工看图结论独立于退出码

## 实际视觉自查

已查看修前/修后对照、东侧5图、西侧5图及滨水5图；[东侧记录](east/visual-review.md)与[滨水记录](water/visual-review.md)给出范围。西侧旧低巷直角接入、兴趣北口同高分流、西联络梯脚、西北梯上端与旧内院平段可见，未发现本轮新增的穿楼或路口互压；楼影下的踏面可辨，近处裸坡和楔形切坡仍为灰盒

首次neighbors机位被屋顶遮住，视觉FAIL；换北向南机位后的 [neighbors补拍](after-supplement/inspect-road-east-neighbors.webp) 能看到楼间台阶和平接点。原platform机位近坡遮去一段宽梯，[反向补拍](after-supplement/inspect-road-east-platform-reverse.webp) 可见两梯上下端与中部平段，未见互压。补拍保持同一最终场景源，保存相同新机位修前画面；B1/B2差别仅机位

连续画面实际检查90/91相邻帧及149/359关键帧，渡口梯道始终在候船板外侧，转头带来正常视野变化，未见突然跳位或资源丢失；状态确认真实FreeCamera响应转头、Escape释放后持有W不移动。已生成MP4，但本环境未整段播放视频，以实际查看的连续帧和关键帧限定结论

## 范围与后续

PASS：本批源路幅、踏面、消费者、建筑净距与实际Viewer场景。NOT RUN：人物碰撞、车辆、Windows、作者操作感受与最终美术。剩余20对转入小店公共台阶与东部街道；铺装楔形边、切坡三角面与栏杆留在场景品质工作，不以技术修复替代体验验收

复杂度检查：复用原数据、生成器、几何检查与Capture，无新生产工具或依赖，临时试验只留output。`Lean already. Ship.`
