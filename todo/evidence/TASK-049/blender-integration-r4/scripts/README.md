# R4 真实捕获输入

本页为运行前计划，GPU 捕获、实际看图与人物通行结论均为 `NOT RUN`；主任务补入对应 Viewer 机位并构建后执行。复用 [既有 capture](../../../../../tools/capture.py)，脚本只向真实 Viewer 发送输入，不改变场景、角色位置或受验收结果

| 脚本 | 实际机位与输入 | 重点观察 |
| --- | --- | --- |
| [inspect-v35-facade.json](inspect-v35-facade.json)，120帧/4s | `[-410,180,17] → [-386,153,17]`，自由诊断机位；30–33帧 A、60–63帧 D，90帧 Escape | V-35 北立面整体、用途招牌、橱窗与门檐的厚度，29/49/79/119关键帧及横移连续帧 |
| [eye-v35-entry.json](eye-v35-entry.json)，150帧/5s | `game_front_court=[-386,165,12]` 加1.725m眼高，看 `game_entry=[-386,153,12]` 加2.8m；40–43帧 A、80–83帧 D，120帧 Escape | 入口和招牌的阅读尺度、门把手/门框、玻璃及候场橱窗接合，29/59/99/149关键帧及横移连续帧 |

上述坐标使用地图 `[东西,南北,高程]`。V-35 原北墙为 `y=153`、`x=-402…-370`，建筑基底12m；入口视点在墙前12m，横移保持其前后距离，不向闭门前冲。两条脚本均先用 M 启用真实 FreeCamera，沿用前轮已执行的各4帧左右输入与1–3m位移边界，停止段检查相机位置、旋转、控制器关闭及实体数量稳定

有限 Transform、必需资产加载、截图数量由原生 capture 的 `finite_transforms`、`required_assets_ready`、`all_frames_saved` 检查；脚本再核对世界就绪与实际镜头变化。Viewer 没有人物碰撞，这两组画面只用于模型、材质、构图和相机输入；玩家接近与闭门阻挡另由主任务的真实 GLB 胶囊/支承窄测判断，本轮不额外复制长路线

角色直接复用 [character-orbit.json](../../../../../game/capture/character-orbit.json) 和 [walk-character.json](../../../../../game/capture/walk-character.json)，检查 CHR-001 的 Idle 环绕及 Walk、Run、Jump；后者的真实 R 恢复输入用于动作分段，不作为零重置路线证据

## 复现

在仓库根执行，以下命令可直接用于 fish；二进制须包含本轮机位及模型导入，输出目录必须尚不存在

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/blender-integration-r4/scripts/inspect-v35-facade.json --output output/blender-integration-r4/inspect-v35-facade
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/blender-integration-r4/scripts/eye-v35-entry.json --output output/blender-integration-r4/eye-v35-entry
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-001 --aa msaa4 --no-video --script game/capture/character-orbit.json --output output/blender-integration-r4/yao-orbit
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-001 --aa msaa4 --no-video --script game/capture/walk-character.json --output output/blender-integration-r4/yao-motion
```

保留版本/hash、日志、状态JSON和实际观察帧号，机器检查与视觉 `self-audit` 分开记录。`--no-video` 明确不编码视频，实际检查关键帧和横移/动作连续帧；完成观察后由主任务按项目约定清理本轮 PNG，保留复现与文字结论

准备阶段已用 Bun 对照当前 Rust `Script/InputSpan/Assertion` 字段核对2份JSON、8段有序输入、6条断言的字段与帧范围，并确认5个相对引用和2个源节点坐标，结果 `PASS`；原生 Serde、新增机位分派、GPU与视觉检查仍待主任务实际执行
