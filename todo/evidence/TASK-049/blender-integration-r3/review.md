# Blender 接入 R3：环境、角色与镜厅真实往返

2026-10-01，基线 `8effe5ddd7f7a3afc18a24482a05670d0fe5f1a6` 上的未提交艺术批次。本页记录已实际完成的接入、检查与复验；两项资产任务仍为active，作者审美与G2品质尚未放行

## 实际结果

九组真实capture均PASS，共11,310帧、75项原生检查；包装器另对两组角色核对真实加载ID，合计77项检查。逐项记录、命令、二进制／脚本／关键帧hash、入口与完整41节点访问见 [captures.json](captures.json)，原始 `run.json`、`script.json`、`state.json` 和 `runtime.log` 归档到 `captures/<name>/`；超过500kB的四份记录使用gzip无损保存，解压后逐字节核对一致，清单同时记录原件及压缩件hash

| 运行 | 入口与范围 | 帧数／时长 | 原生／包装器检查 |
| --- | --- | --- | --- |
| [v55-west](captures/v55-west/run.json) | Viewer，实际道路视点左右移动，看V-55西面 | 120／4s | 6／6 |
| [cinema-court](captures/cinema-court/run.json) | Viewer，新前场、绿岛、地面长椅近景 | 120／4s | 6／6 |
| [aircon-wall](captures/aircon-wall/run.json) | Viewer，上看挂机、下看实际管线与收水口 | 120／4s | 6／6 |
| [cinema-facade](captures/cinema-facade/run.json) | Viewer，镜厅南西立面与前场空间关系 | 150／5s | 6／6 |
| [shop-stairs](captures/shop-stairs/run.json) | Viewer，高侧向固定检查3段踏鼻 | 60／2s | 4／4 |
| [shop-stairs-eye](captures/shop-stairs-eye/run.json) | Viewer，小店人眼方向的台阶对照 | 60／2s | 4／4 |
| [yao-orbit](captures/yao-orbit/run.json) | 正式游戏，CHR-001真实Idle与锁鼠绕摄 | 240／8s | 9／10 |
| [yao-motion](captures/yao-motion/run.json) | 正式游戏，CHR-001实际Idle／Walk／Run／Jump | 540／18s | 27／28 |
| [cinema-gentle](captures/cinema-gentle/run.json) | 正式游戏，胶囊代理从小店走到镜厅门前再回家 | 9,900／330s | 7／7 |

所有运行实际使用 `--no-video`，没有生成视频；画面检查使用真实PNG关键帧及连续帧。原始 `run.json.visual_review` 与 `state.json.visual_review` 保持捕获工具写入的NOT RUN提示，单独的人工自查才记录看图结论，不能把机器PASS改写成作者批准

## 真实路线及角色边界

镜厅路线未启用 `--character-preview`，屏幕中的可控对象是正式人物控制器的胶囊代理。实际从小店 `home` 第59帧出发，41/41节点到达；第4563帧到 `cinema_entry_landing`，第9066帧回到 `home`，所有访问都有真实地面支承且恢复次数为0。最大节点水平误差0.058717m、支承高度误差0.023440m，末段保持站稳

这条路线水平长959.826124m，现有RouteDriver观察玩家与相机并发送普通轴/鼠标输入，没有改变出生点、移动速度、碰撞或到达容差。它证明既有到达与返回主路线，不证明镜厅室内、电梯、西侧长椅通道或绿岛横穿已经可玩。闭门锚和真实站位的分离、源数据迁移及1mm派生地形更新见 [cinema-landing.md](cinema-landing.md)

曜在两条短片中由包装器核对实际角色ID为CHR-001；motion样本实际出现四个具名动画，三次Jump以及Shift/右键Run均由现有输入触发。motion脚本的三次 `R` 是明示的动作分段恢复，不与长路线的零恢复证明混用。锁鼠和模拟手柄／焦点输入属于应用路由证据，未代替实体设备测试、跨平台验收或角色外观批准

八条短片使用地图hash `e1ab14e5f38a89611572e2153ab5136e0a3c18f4202220ba99858feb1b862d43`；之后只补镜厅门前站点、消费者与唯一1mm烘焙样本，长路线使用最终地图hash `a2f15c59246a57c1250ee76db54e0a12fa5ce766f17f15dc1535e5895128038f`。源输入与两份实际二进制hash见 [runtime-inputs.json](runtime-inputs.json)，此后scene.rs调整只在cfg(test)接近检查中，运行二进制保持

## 本批接入

- V-55包装修补立面按原地块、标高和门位接入，替换南／西重复泛型构件，北／东保留；真实GLB加入同一碰撞路径
- 镜厅前场使用3块真实surface，形成连续铺装、两条地栽绿岛与地面长椅；保留原服务路与斜来路，不以GLB假地板替代地图
- V-W10接入CC0空调及原创完整安装件，原模型、贴图和作者许可保留；低位收水口与长椅共同进入实际三角碰撞和支承检查
- 小店3段70级台阶添加420三角的视觉踏鼻，原踏面与结构碰撞保留
- CHR-001帽兜更新随真实骨骼和四动画接入；本批仍是待作者确认的灰阶角色候选

来源、导出文件与碰撞接入复核见 [代码审查](code-review.md)，具体资产制作分别留在各源包与TASK-047／049已有证据，不在本页另建资产台账

## 测试与构建

| 实际检查 | 记录与结果 | 范围 |
| --- | --- | --- |
| Rust库首次完整测试 | [lib-tests.log](lib-tests.log)：140 PASS、1 FAIL、1 ignored | 原服务路净空探针起点距铺地不足skin；首次失败保留 |
| 修正后碰撞窄测 | [collision-test.log](collision-test.log)、[scene-models-final.log](scene-models-final.log)：各1 PASS | 最终导入模型真实接地、碰撞及服务通道；没有把窄测拼成完整库重新通过 |
| 最终资产接近窄测 | [attachment-final.log](attachment-final.log)：1 PASS | 真实GLB包络、楼体与7条接近段，含新的门前站位 |
| Viewer现有检查 | [viewer-tests.log](viewer-tests.log)：3 PASS | 固定视点与地图框架条件 |
| 游戏构建 | [build.log](build.log)：dev构建正常完成 | capture实际使用两份已记录hash的二进制 |
| 地图最终检查 | [check-map-final.log](check-map-final.log)：61 PASS | `bun run check:map`，包含本轮门前站位与前场窄测 |
| 门前站位专项 | [cinema-landing-test.log](cinema-landing-test.log)：4 PASS | 到达链、原路幅、站位、缓行路线，已包含于最终地图检查，数量不重复累加 |
| 环境导出一致性 | [export-check.log](export-check.log)：37文件PASS | 现有AST-003导出核对 |
| TypeScript／Vue | [types.log](types.log)：命令完成 | 本轮TS地图检查已通过；后续新增文档未改变TS类型契约 |
| 文档／Skills | [docs-final.log](docs-final.log)：584 Markdown、129 IDs；31 Skills／25 imports通过 | 包含本批最终汇总、源文档和任务链接 |
| Wiki双构建 | [wiki-build.log](wiki-build.log)为早先结果；[wiki-final.log](wiki-final.log)为最终地图版本：61项地图测试、player／dev均PASS | 最终player128页／679文件，dev223页／1046文件；保留原先记录，不互相覆盖 |

曾调用不存在的 `map:check`，错误保留于 [map-final.log](map-final.log)，之后使用真实 `check:map` 入口获得上述61项结果。早先Wiki dev构建有包体超过500kB提示，构建仍完成；没有把它写成运行性能验收

原始状态记录中的性能数值包含离屏读回与PNG落盘开销，不能当成正常游戏帧率或GPU耗时。Windows、实体手柄、人工手感与作者审美在本轮均为NOT RUN

## 视觉自查与待处理项

[实际画面自查](visual-review.md)由art_review记录53张实际查看的PNG与SHA-256，包含设施近景、连续走跑段、Jump样本、门前4562–4566帧和回家9064–9068帧。该评审者了解设计和实现，因此是self-audit；本文维护者核对录制数据与归档，并引用该观察，不将其冒充自己的看图或隔离评审

已看局部未见需要撤回本批接入的新可见阻断：V-55用途、前场铺地／长椅／绿岛、空调整套安装关系和近处70级踏鼻可辨。但平玻璃、空墙／空草地、低面花簇、远处深灰梯以及曜灰阶衣身仍未达到README预览目标；曜Jump样本主要正视，不能证明帽兜背面全部相位无穿插。下一轮应优先完善成片街面、材质陈设层次与人物上衣轮廓，而不是据本批机器PASS宣布品质完成

## 证据保存与收尾

36个非媒体原文件共44,414,431 bytes，四个大文件使用gzip减少仓库占用，其余原样；解压后逐文件SHA-256一致。每条capture保留原关键帧hash与原路径，PNG／视频均未复制进todo。实际查看后，Root确认capture已退出，删除本轮 11378 张PNG／关键帧，共 4,288,642,463 bytes；output剩余 44,414,431 bytes 均为脚本、日志及状态，无剩余媒体，见[清理记录](cleanup.json)。正式源、参考资料与运行资产保留


## 本轮复验命令

在仓库根执行，以下可直接用于fish；完整capture入口见[脚本说明](scripts/README.md)

```fish
bun run check:map
bun tools/export-environment.ts --check
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib -j 1 blender_building_attachments_fit_existing_shells_and_routes
cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib -j 1 scene_models_have_real_support_and_block_the_player_outside_the_original_shell
bun run tasks:check
bun run check:docs
fish -c 'set -px PATH /home/lzzz/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin; bun run docs:build'
```
