# 局部坡形、草岩混合、林群与人物 r7

基线 `f93af27bfda9650ceb2687b822bda5367540ef1a`，完整游戏目标继续优先推进地图、人物和画面；本批没有改写剧情、扩展室内或新增玩法。TASK-045／047／049 均保持 active，作者美术及 G2 放行未发生

## 实际交付

- 下山镜头的巨型直楔来自道路吸附高度与稀疏外圈采样，将这一局部的高度包络及采样细化后重生成地形后缀；9,300 样点，原42个手工点及所有非地形地图数据保持。418点原生坡度 p95 从56.03°降至44.94°，25,770道路样本核对和真实短上山往返测试通过，详见[地形记录](../terrain-shape-r3/README.md)
- 自然地面回到单一真实网格，用扩展标准 PBR 按未扰动法线在35–55°试验范围混合 Ground037／Rock043L 颜色、法线及粗糙度；保留阴影、运动向量与法线预通道，见[材质记录](../terrain-surface-r4/README.md)
- 由三个已存在节点派生38棵松树，首轮实机发现幼树体量不足后，改为固定1.4–1.8倍变体，并重新按实际冠幅、根圈、门路净空及摘星台望城走廊筛选；实际仍为14／14／10，最大根部埋深0.447152m，见[林群记录](../../TASK-049/forest-r1/review.md)
- 曜 r6 用相连发束替换后脑短片，闭合冠顶并修正其灰阶图集 UV；32骨、63个非头发网格、图集及绑定保持。最终 GLB `5063920196cb1476e080f7d8acd6f8d16f0e55ab3cbfc94c30da9c1ad368a108`，20,483顶点、38,898三角，见[模型记录](../../TASK-047/model-r6/review.md)
- Run 调整腕／前臂方向、肘屈、上身前倾和摆动脚俯仰；Idle／Walk、控制器速度和 Run 支撑轨迹保持。真实源半帧及 GLB 120 FPS 检查、循环、脚底距离和再导出一致性通过，见[动作记录](../../TASK-047/animation-r2/README.md)

## 实际执行

| 命令 | 结果与边界 |
| --- | --- |
| `cargo test --manifest-path game/Cargo.toml --lib` | 128 PASS、1 ignored，含完整短上山往返；日志 `lib-tests.log` |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib world::scene::tests -- --nocapture` | 最终成熟林群20 PASS，另有首轮20 PASS；含缺 rock 槽即时失败及真实根圈 |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer` | 3 PASS |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins` | 最终 `build-mature.log` PASS |
| `cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings` | 最终 PASS |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `bun run check:map` | 56 PASS，含地形7项；生成器重复 `--check` 无漂移 |
| `bun tools/export-environment.ts --check` | 31文件／69,565,322 bytes PASS |
| `bun run check:types` | PASS |
| `bun run check:docs` / `bun run docs:build` | PASS，后者玩家／开发两种构建均通过；已有大chunk提示保留 |

本轮没有为修改迁移Bevy或引入新生成服务。地形独立复现用原生 `Ground` 与 `Map`，源码保留在 `terrain-shape-r3/probe.rs`；临时 example 和其探测可执行文件收尾清除

## 运行证据与失败诊断

`summary.json` 列出11次真实调用。9次成功录制共1,380帧；一次缺资源负例按预期退出1，一次人物调用遗漏显式候选开关失败，保留失败日志并修正命令后通过。各目录保存脚本、命令、binary hash、状态摘要、完整状态文件校验和、运行日志与每个视觉文件 hash；完整逐帧状态仍留在 output

普通 MSAA 与 TAA／SSAO 切坡各90帧／6项，成熟林群总览90帧／6项、根部150帧／6项，人物最终540帧／26项。人物路线实际驱动走跑、跳跃、暂停与失焦恢复，未直接修改角色结果

第一次准备缺shader负例时，隔离资源目录实际上仍含完整复制进来的新 shader，因此正确返回PASS；核对文件后只删除隔离副本中的该文件，正式资源未动，第二次明确报 `terrain slope shader shaders/terrain-slope.wgsl`，0帧、退出码1。首个结果只是含旧地形和新材质的有效正例，不计为故障拦截通过，修正依据记录在 `negative-fixture.json`

首次人物命令遗漏 `--character-preview`，启动了原有默认代理，六项真实人物断言失败；补上显式选项后最终全部26项通过，没有放宽断言。首次保存的540帧与失败状态保留索引，不归入成功帧数。模型另曾发现冠顶默认UV采到skin区，补显式hair UV与实际拒绝旧候选的检查后重新合并导出，未把中间版算作最终验证

## 实际画面观察

Root 查看旧／新切坡44、新 TAA／SSAO 44，以及旧地图加新材质44：只换材质仍保留大楔，最终形状明显回落，说明改善包含真实修形。艺术审查查看新切坡40–44连续帧，未见地面跳变；林群总览的灰岩向苔土连续过渡，远景仍有明显黄绿条状重复与偏软的TAA画面

林群两轮实际查看总览29／44、根部29／59与60–64连续横移：较大的冠形增加中景占比，所见根部接坡，未见整树消失或跳位；同机位对照包含真实避让导致的点位替换，不冒充同一棵树单独缩放。三组仍是稀疏树岛，缺灌木、中层、林下及树种变化，森林美术未通过

人物源画面实际查看完整20格Run侧视、最终后脑／侧后／冠顶及正面三分之四；艺术审查补齐六视角。最终实机Root查看141／190／195，艺术审查查看Walk140–144与Run190–194。后发连续且白冠消除，Run接入但手指、肩肘、躯干动势、规则发束和面部辨识仍需修订。以上均为知情self-audit，不是作者或隔离玩家验收

## 成本及限制

同一来源的 ready 资源由5,068 entities／46 images／53 StandardMaterial／3,367 meshes变为5,263／46／53／3,366；自定义 TerrainMaterial 不在原 StandardMaterial 计数器内，因此不能称总材质数未增加。模型实例增加38，新增247,380三角；缩放不改变单份几何量，却可能增加屏幕覆盖和阴影成本。capture 的读回／PNG耗时不能当正常游戏帧率，全城LOD、原生帧时与显存预算尚未测

更正上一轮漏算：r6松树实际100实例，96遗漏了r5四个院落组；当轮松树与灌木替换的增量应为716,400三角。原日志保留，计数依据归林群记录，当前总松树138

Windows实机、物理手柄、夜间多光源、作者美术与全城性能预算本轮 NOT RUN。下一轮继续地形大面的层次、地表重复和林下体积，并制作人物贴图与更多角色；现有改善没有达到README预览质量

## 本地复验入口

以下命令在仓库根目录可由fish直接执行；输出目录需尚不存在

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --script game/capture/descent-cut.json --output output/capture/cut-check --binary game/target/debug/map_viewer --aa taa-ssao
python3 tools/capture.py --script game/capture/forest-overview.json --output output/capture/forest-check --binary game/target/debug/map_viewer
python3 tools/capture.py --script game/capture/forest-root.json --output output/capture/root-check --binary game/target/debug/map_viewer
python3 tools/capture.py --script game/capture/walk-character.json --output output/capture/character-check --binary game/target/debug/n-side --character-preview
```

查看关键帧及移动段，核对state与runtime日志，记录范围后按项目要求清理临时视觉产物。用户原有 AGENTS.md 和 docs/dev/validation/runtime.md 修改保留，不纳入本次提交；无远端推送

## 收尾清理

实际看图、检查、hash与文字结论完成后，root清理本轮所有capture截图／录屏、隔离资源副本、临时地形example及其专属target文件，共2,166项、逻辑大小6,342,251,171字节；硬链接等可能影响实际释放量，不将此数当作磁盘净释放。11个capture目录剩余3,938,662字节日志、脚本、状态JSON，无剩余视觉文件，详见 `cleanup.json`。人物工作线另各自记录临时DCC图与副本清理，正式模型、贴图、参考图及复现脚本保留

最终独立复杂度审查：`Lean already. Ship.`；生成看板与只读任务检查均通过（46张任务卡），没有将本批技术交付计为游戏或美术验收完成
