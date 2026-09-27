# 全城建筑灰盒与公共素材交付

本轮从 `fcf92ee799d94775fcd26812f930e9217ed9db3f` 开始，免费素材先以 `44d8792` 提交。实际运行输入、二进制和资产清单哈希见 [inputs.json](inputs.json)，其输入集合 SHA-256 为 `1d9cf4fba259e2734dcb2a3a74fc79e272538fda87ef63cb1b3dec76aa59bbf0`；源地图为 `49f46aed0e9d3a578304f0a99d7174864e7a9c2dcc9382c7ffcd1c2ca402acf0`

## 实际交付

- 224个本岸楼体与7个对岸体量沿用源地图，12街坊和91场所保持身份；396入口角色合并为342个实际门口，区分公共、住户、后勤与屋顶通路；15种建筑用途类型参与外部生成
- 门窗按实际外露墙面及用途派生，保留天井朝内立面，省略被贴坡或共墙遮住的窗，真实入口被阻挡时返回诊断；17项既有座椅、柜、状态屏及排水带进入场景并检查接地
- 从同一Kenney Nature Kit 2.1补入松树、秋色树、石块、草簇、黄花5个CC0模型，原件、包内声明、下载成员、SHA-256、用途、尺度、枢轴和材质参数保存在既有AST-003清单；原包声明与已保存许可证逐字节一致
- 增量植被与土石148个，公园53个、登高及长山径95个；既有72个树点保留，按海拔使用树种变体。生成复用真实源道路与平台，按实际导出模型半径避让，不增加另一份城市地图；见[素材放置记录](free-assets-use.md)
- 新增小店至摘星台短登高路线，水平约1213.935m、三维约1311.187m、累计爬升422m，设置三处8×8m停步台；长观景路线保留。源拆段、设施校准与路线保护见[TASK-023第5轮](../../TASK-023/r5/source-review.md)

## 机器验证

完整命令与结果见 [checks.json](checks.json)，测试原始文本包括 [Bun](bun-tests.log)、[Rust](rust-tests.log)、[Clippy](clippy.log)与[几何加载](validate.log)。最终Bun为107项、Rust为19项库测试及4项Viewer测试，模型导出20文件共28101642字节可重算一致，Python资产5项与capture包装器2项通过；类型、故事关系、玩家／开发Wiki构建和实际发布产物检查通过

首次并行Bun执行有1项地形测试超过默认5秒，原结果保留在 [bun-first.log](bun-first.log)；串行重跑全部107项通过，未修改时间上限。Clippy全targets检查发现3处新增测试的固定分组迭代写法，改用 `as_chunks` 后通过

完整短登高路线的路幅扫描为PASS／0候选；全城扫描为FAIL／108对候选，最大约9.05m，原始结果与共享节点负例见[TASK-023路幅记录](../../TASK-023/r5/source-review.md#实际路宽复核)。未把全城FAIL计为本轮通过，也未把道路中心连通等同于人物可通行；剩余处理见[TASK-025](../../../tasks/TASK-025-road-junction-widths.md)

## GPU与连续操作结果

[52机位记录](render-checks.json)与[原始渲染日志](render.log)保留逐项样本；所有PNG已解码检查为2560×1440。Linux Vulkan离屏、dev构建的固定机位帧间隔P95最高5.45ms，各机位均满足现有16.67ms检查门槛；这是应用帧间隔，不是独立GPU耗时，也不替代Windows、窗口VSync或实际玩法预算。资源就绪为3213 meshes、260模型实例、25依赖，失败0、降级0

首次整批运行在完成47个机位后退出143，日志未说明终止原因，保留该中断结果；最后5个设施机位独立补跑均退出0。汇总结论来自52份实际文件及对应机位采样，不声称首次批处理成功退出

[连续操作记录](capture-checks.json)包含城市与山城两段960×540、30fps、540帧／18秒MP4，均完整生成1080张连续帧并通过真实横移、转向、释放、恢复和稳定状态断言。使用已有FreeCamera系统，没有角色行走、碰撞或任务完成的声明；录像时长及帧数由ffprobe独立复核

城市脚本首轮实际转角0.0742rad低于预设0.1rad，断言正确失败；保留[原状态](city-scan-first-state.json)与[原脚本](city-scan-first-script.json)。将鼠标输入由每帧1.2／0.3增至2.0／0.5，再反向返回，复验达到0.1237rad；未降低门槛或修改结果。独立的[故意失败样例](expected-failure-state.json)要求1000m位移，实际未达到，状态FAIL且进程退出1，资产就绪与其余条件正常

实际查看了[城市关键帧](city-keys.webp)、[山城关键帧](mountain-keys.webp)及各自144—151连续帧；看到横移、向山峰抬头、返回及停稳，没有把MP4生成等同于已观看完整视频。完整视频留在本地output，未声称视频逐帧人工审查

## 实际画面自查

此处是Codex知情自查，已阅读代码和设计，属于self-audit。实际打开了[12街坊总览](views/contact-blocks.webp)、[17设施总览](views/contact-fixtures.webp)、[街景总览](views/contact-streets.webp)、[登高视点](views/contact-mountain.webp)及整城、山侧、桥下原图，并放大检查三条排水带；固定帧性能独立于画面判断，截图成功不能代表所有空间问题解决

已实际打开整城、街景及三个停步台原图：楼体、用途层、门窗和地面材质进入真实Viewer；山形保持摘星台单一最高点，短登高线和三个平台均可辨认。前两处停步台能俯瞰街坊及白沙河，第三处+410m停步台被近坡遮住大部分城区，临城视线目标未通过，保留在TASK-023；没有通过抬高相机伪造行人视线

街景仍可见部分道路交接的尖角和高差，地形侧壁及远山折线也保留灰盒形态。建筑外部与公共素材接入可单独验收，整座城市的最终美术、所有道路衔接和登山体验尚未验收

## 日常复验入口

以下命令在仓库根可由fish执行，capture输出目录须为新目录

```fish
bun tools/export-environment.ts --check
bun tools/check-road-width.ts hill-short
cargo test --manifest-path game/Cargo.toml --features viewer --locked
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --view city-overview
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --view ascent-overview
python3 tools/capture.py --script game/capture/city-graybox.json --output output/capture/city-graybox-review
```

全城候选复现使用 `bun tools/check-road-width.ts all`，当前预期退出1并输出逐项JSON。逐街坊视点为 `block-B01` 至 `block-B12`；完整原始PNG、MP4和日志归忽略的 `output/city-graybox/2026-09-27/r1/`，小型证据随本任务保存

## 验收边界与下一步

此任务只验收全城建筑外壳、设施与免费模型接入，不放行G1。NOT RUN：Windows、手柄、人物碰撞与登山实玩、室内、电梯功能、作者美术验收、陌生玩家测试；Viewer录像仅证明已有自由镜头控制。后续先按TASK-025收敛真实路口，再在TASK-023调整第三停步台视线和山坡表现，最后安排实际人物路径与节奏验收
