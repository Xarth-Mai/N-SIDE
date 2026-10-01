# R3 真实捕获计划

本页保存运行前的输入计划与复现说明；计划编写时编译、GPU、实际看图为 `NOT RUN`，后续执行结果见 [R3汇总](../review.md)。复用现有 `tools/capture.py`、正式游戏控制器、Viewer `FreeCamera` 与真实世界资产，未改变出生点、模型位置、路线或受验收状态

## 最小画面集

| 脚本 | 目的与机位依据 | 重点查看 |
| --- | --- | --- |
| `v55-west.json`，4s | `eye-v55-west` 沿真实 `/roads/452` 在公门接近段35%处，眼高为道路标高+1.725m；真实左右输入各4帧，位移断言1–3m | 包装样品柜、用途字招、门檐、上层深窗与坡地收口；49/79帧及其相邻连续帧 |
| `cinema-court.json`，4s | `eye-cinema-court` 从 `[298,223,26.725]` 看向 `[295.8,233.5,25.8]`；沿西侧铺地接近/退回各4帧 | 新铺地与两条绿岛、地面长椅、植物根部、旧西侧服务路的分界；29/49/79帧 |
| `aircon-wall.json`，4s | `eye-aircon-wall` 从 `[565,95.6,14.391667]` 看真实 `V-W10` 西墙；固定位置向下看约28.6°后复位 | 29/89帧检查挂机格栅和背架；49/69帧检查贴墙管线与地面收水口，不能只看高挂机 |
| [既有影院全景](../../blender-integration-r2/captures/cinema-facade-retry/script.json)，5s | 复用 `inspect-cinema-facade` 的既有接近与横移输入 | 前场铺装是否连成入口空间，南面斜路裁角与实际建筑关系；近景不能替代该上下文 |
| [既有小店台阶](../../blender-integration-r2/captures/shop-stair-surfaces/script.json)，2s | 复用 `inspect-shop-stair-surfaces`，相机位于 `[103,279,40]` | 三段台阶共70级的鼻口是否接地、方向连续、有无亮白条纹和悬空；这是高侧向固定诊断，非人物通行证明 |
| [正式角色绕摄](../../../../../game/capture/character-orbit.json)，8s | `n-side --character-preview CHR-001`，真实锁鼠相机，人物保持Idle | 59正面、89侧面、139背面、204/219前三分之四；帽兜开口、颈肩、发梢和衣服接合，动作时间须推进 |
| [正式走跑跳](../../../../../game/capture/walk-character.json)，18s | 同一正式角色入口，实际键鼠/手柄Jump、Walk、Shift/右键Run、暂停/恢复 | 62–81和105–112的Jump、130–159的Walk、180–209及230–259的Run；检查帽兜穿插、肩颈拉伸与落地恢复 |

以上 Viewer 机位只证明真实场景渲染与相机控制；自由镜头位移没有人物碰撞。空调初始视野不能容纳整个3.43m高装配，所以用实际鼠标输入下看地面，不前冲进2.8m外的墙。其20帧×25dot×0.18/180=0.5rad旋转来自当前安装的 Bevy `FreeCamera` 实现，不沿用正式游戏的鼠标系数

角色原脚本已覆盖四个具名动作，不复制为第二套脚本。`walk-character` 的三次 `R` 是真实恢复输入，只用于动作分段测试；它不证明连续路线无恢复。若需复验台阶实际通过，复用 [walk-ascent-entry.json](../../../../../game/capture/walk-ascent-entry.json) 的35s真实小店出发路线，检查零重置与实际41m落点，不改出生点

## 镜厅缓行路线的真实时长

`cinema-gentle.json` 读取 `district.json` 中同名路线的全部41节点，该源路线已含 `home → cinema_entry_landing → home`，因此明确 `round_trip:false`

- 源路线水平总长 `959.826124m`，节点高程范围20–28m
- 当前 `player.rs` 步速3.2m/s，`RouteDriver` 发普通手柄轴与视角输入，没有疾跑；理想移动至少 `299.945664s`
- 30fps、从第60帧开始，理想完成约第9059帧；脚本给9900帧/330s，约30.1s余量供节点收敛与停止，墙钟超时3600s
- 自动 `route_all_nodes_reached` 要求41个实际支承节点全部到达；额外断言要求世界持续就绪、零重置，9870–9899帧在小店28m处站稳
- 4560等关键帧是按路径长度估计，不能当作实际到达时刻；完成后读 `state.json` 中 `checks[name=route_all_nodes_reached].route.visits` 或日志的 `[capture/route-node]`，按实测镜厅到达帧检查连续画面
- 门前站位 `cinema_entry_landing=[306,219.35,25]` 已写入真实道路，原 `cinema_entry` 建筑门锚不变；修订依据与消费者见 [门前站位迁移](../cinema-landing.md)。RouteDriver 的0.06m到达、支承与2s无进展规则保持；实际碰撞阻挡仍应失败，不传送、不扩大容差
- 该路线证明既有到达/返回主路径；未包含西侧座椅通道、绿岛横穿或电梯，后者由相应真实几何/碰撞窄测与近景分别覆盖

640×360用于长路线的状态和可达性证据，精细材质与模型由前述1280×720短片检查。捕获入口仍逐帧落盘，不能把少选关键帧写成少截图；不要为缩短录像改变固定步长、移动速度或路线

## 执行入口

仓库根目录执行，以下为fish可直接运行的复现命令；输出目录必须不存在，正式游戏/Viewer二进制须由主任务先完成本轮构建，统一保留同一MSAA4/默认成像对照

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --script todo/evidence/TASK-049/blender-integration-r3/scripts/v55-west.json --output output/blender-integration-r3/v55-west
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --script todo/evidence/TASK-049/blender-integration-r3/scripts/cinema-court.json --output output/blender-integration-r3/cinema-court
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --script todo/evidence/TASK-049/blender-integration-r3/scripts/aircon-wall.json --output output/blender-integration-r3/aircon-wall
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --script todo/evidence/TASK-049/blender-integration-r2/captures/cinema-facade-retry/script.json --output output/blender-integration-r3/cinema-facade
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --script todo/evidence/TASK-049/blender-integration-r2/captures/shop-stair-surfaces/script.json --output output/blender-integration-r3/shop-stairs
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-001 --script game/capture/character-orbit.json --output output/blender-integration-r3/yao-orbit
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-001 --script game/capture/walk-character.json --output output/blender-integration-r3/yao-motion
python3 tools/capture.py --binary game/target/debug/n-side --script todo/evidence/TASK-049/blender-integration-r3/scripts/cinema-gentle.json --output output/blender-integration-r3/cinema-gentle
```

记录实际二进制和源版本、退出码、日志、状态JSON与实际看过的帧号，机器断言与 `self-audit` 分列。检查后按项目收尾规则删除本轮截图/视频与临时可执行文件，保留脚本、日志、状态及文字结论。作者外观批准和实体设备操作不由本轮脚本代替

## 本次静态核对

Bun 已读取4份JSON并对照当前Rust的 `Script/InputSpan/Assertion/Wait` 字段：4文件、12断言、13输入段的字段与帧范围均通过；路线邻接、手动输入不与RouteDriver重叠、3个Viewer视点存在、5个相对文件链接均通过。该检查没有调用原生Serde校验或渲染，最终脚本合法性和阈值仍由真实入口裁决
