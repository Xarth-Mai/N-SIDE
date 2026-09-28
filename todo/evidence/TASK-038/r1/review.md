# 短登高路侧地形接合复验

本轮修正临城回望台至山腰林荫台之间的路幅与邻坡采样，保留原道路、台阶、建筑、入口、稳定ID和450m单峰；技术检查、最终同机位录制与完整人物往返CPU回归已通过，TASK-038技术交付完成，整座山体的最终视觉品质仍待TASK-023收敛

## 来源与改动

基线提交 `0c1445e3b0337c09c524a12bddb5b67014ec537d`，本轮 source_revision 为 `5e1913554958518e73b7b81054eef38a31d7bdf3e963bc9c14c586e8179f7c2c`，详细输入hash见 [provenance.json](provenance.json)；其范围为地形工具、源数据、测试和渲染相关文件，其他并行游戏功能不计入TASK-038验收

`hill-short` 中 `hill_short_rest1_departure` 至 `hill_short_rest2_arrival` 的23段道路复用既有6m路心与两侧控制；距这段路线30m内的平台边缘复用2m控制，自然山面使用12m局部网格和原高度函数。原8334样点变为8584；地图中terrain行之外的所有字节、terrain的42个手工前缀、recipe、水域均相同，生成点仍低于450m，见 [compare.log](compare.log)

本轮没有修改引擎几何、角色控制或场景设计；最终Viewer录制在8584样点源冻结后启动。修前和修后实际二进制hash分别保留在各run与provenance中，两次并非同一个二进制，当前map_viewer、geometry与相机视角代码没有本轮修改

## 机器检查

|检查|结果|证据|
|---|---|---|
|23段道路230个路缘点|旧版225点误差大于1cm，范围-3.37736至+1.95313m；新版全部小于1.1e-7m|[修前FAIL](edge-before.log)、[修后PASS](edge-after.log)、[最终量化](compare.log)|
|真实下山视角邻坡点`[160,740]`所在三角形|最长水平边60m降至16.97056m|[修前FAIL](slope-before.log)、[修后PASS](slope-after.log)|
|地形全部窄测|6/6 PASS，含平台边保护、单峰、重复生成与峰前视线|[terrain-final-tests.log](terrain-final-tests.log)|
|`bun run check:map`|54/54 PASS|[map-final-tests.log](map-final-tests.log)|
|`bun run check:types`|PASS|[types-final.log](types-final.log)|
|`bun tools/terrain-shape.ts --check`|PASS，8584样点且无烘焙漂移|[bake-final-check.log](bake-final-check.log)|
|最终Viewer capture|90/90帧保存，6项检查PASS，视频生成PASS|[run.json](run.json)、[state-summary.json](state-summary.json)|
|最终Rust回归|77项库测试与3项Viewer测试PASS；完整259节点往返在64Hz与30Hz均通过|[final-tests.log](final-tests.log)|

第一次只补路缘后，数值接合正确但同机位的大块折面依旧；随后补12m局部网格，现有测试发现临城回望台边缘受到三角化影响，保留 [真实FAIL](map-refined-before-platform.log)。最终将同一区域的平台边纳入已有边缘控制后复验通过，没有放宽断言

`player::tests::walks_complete_short_ascent_and_return` 从同一district路线取得130个节点，逆序返回时不重复峰顶，共259节点，使用实际PreparedScene与CollisionWorld，分别执行64Hz和30Hz两轮控制；两轮均通过位置、高度、接地、零自动恢复和进度约束。全测日志没有展开成功用例内部的逐节点打印，因此本轮不填造具体完成秒数；覆盖范围由实际测试源码与成功结果共同确认

库测试汇总另有1项`settings::tests::restart_worker`标记ignored，它是由已通过的`preferences_restore_in_a_second_process`以`--ignored --exact`启动两个隔离进程执行的helper，不是把未运行验证写成通过；该设置测试不扩充本任务的地形验收范围

## 实际画面检查

修前raw位于 `output/terrain/2026-09-28/r7/before/`，最终raw位于 `output/terrain/2026-09-28/r7/final/`；均使用实际 `eye-descent-cut` 机位、1280×720、30fps、90帧和真实转头输入。相机始终保持 `(119.85376,292.63010,-750.14526)`，右转、回转约0.075弧度，两方向均通过脚本断言

归档复核者逐张查看前后19–21连续帧，另看最终43–45连续帧；前一组是静止阶段，后一组包含右转结束。原路旁高灰墙被贴合路缘的土坡替代，上方粗折面具有更连续的明暗过渡，所看连续帧内未见旧蓝色开口或逐帧闪动。主执行Codex另查看最终44与74帧

前后对照：[修前20帧](before/frame00020.webp)、[最终20帧](frame00020.webp)、[修前44帧](before/frame00044.webp)、[最终44帧](frame00044.webp)、[最终74帧](frame00074.webp)。归档10张图片均保留原尺寸，WebP质量80，原PNG和归档图hash见 [archive-manifest.json](archive-manifest.json)

近景仍有褐色宽面与硬折线，山体材质、路侧坡形与植被需要继续在TASK-023收敛；本轮通过的是局部路缘接合与采样修补，不能把更小的三角形或成功截图称为最终山城效果。以上是实际看图的self-audit，没有完整播放视频；run中的工具生成`visual_review`保持原值，本记录单独补充人工观察范围

本轮Viewer只证明既有渲染与镜头路径，人物完整通行另有上述CPU回归支持；本轮没有重录完整人物往返GPU，Windows、物理手柄及作者实玩本轮NOT RUN，不以机器通行代签人工手感

## 复验入口

以下命令从仓库根执行，兼容fish，capture使用未占用的新输出目录

```fish
bun tools/terrain-shape.ts --check
bun test tools/tests/terrain-shape.test.ts
bun run check:map
bun run check:types
cargo test --manifest-path game/Cargo.toml --locked --lib player:: -- --nocapture
python3 tools/capture.py --script game/capture/descent-cut.json --output output/capture/ascent-terrain-review
```

实现者核对同一高度函数、稳定节点选择、去重与无漂移，不增加运行时系统或依赖；简化评审为 `Lean already. Ship.`，CPU通行与视觉品质按各自证据单独判断
