# 城市与人物画面第 4 轮复现入口

本轮复用既有真实 Viewer 与人物操作脚本，不新增拍摄场景、相机控制器或验收框架。此页记录检查安排，运行前不预写结果；实际结果引用执行后的 `run.json`、`state.json` 和已查看帧号

## 最小检查集

| 对象 | 现有脚本与视点 | 时长与输入 | 主要查看帧 | 证据边界 |
| --- | --- | --- | --- | --- |
| 起坡段与山体材质 | [mountain-city.json](../../../../game/capture/mountain-city.json)，`eye-shop-mountain` | 18 秒，横移、仰视、回正、释放与恢复输入 | 0、74、149、239、374、539；73–75 连续帧 | 在小店 `home` 节点上方 1.7m、55°视场检查近中远层次，峰顶允许离开水平视场；相机飞行不证明人物可通行 |
| 山路切坡近处 | [descent-cut.json](../../../../game/capture/descent-cut.json)，`eye-descent-cut` | 3 秒，在已记录的约 290m 下山机位左右转头 | 0、29、44、59、74、89；43–45 连续帧 | 检查纹理缩放、法线、岩土与植被过渡、裁剪边缘；使用既有测得相机姿态，不改地形或路径 |
| 住宅上层 | [exterior-details.json](../../../../game/capture/exterior-details.json)，`inspect-residential-detail` | 5 秒，接近、左右横移和停止 | 0、29、59、89、119、149；61–63 连续帧 | 由 `east_mid_junction` 看 `V-A13` 西立面，检查窗台、遮阳、立面重复和悬空；只覆盖该住宅样板，非全城所有立面 |
| 街树 | [street-tree-detail.json](../../../../game/capture/street-tree-detail.json)，`inspect-street-tree-detail` | 5 秒，接近、左右横移和停止 | 0、29、59、89、119、149；61–63 连续帧 | 同一 `trees[68]` 实例与低台阶平台，检查树冠轮廓、枝叶密度、树皮与转头时的叶片表现；非全部树种 |
| 绑定人物 | [walk-character.json](../../../../game/capture/walk-character.json)，正式入口 `shop` | 18 秒，跳跃、行走、两种疾跑、转头、暂停与恢复 | 59、141–143、159、209、259、314、359、499；人物近脸另用源模型多角度检查 | 真实人物控制与 Idle／Walk／Run 动画；远景背面无法单独判断少年感、脸部表情和日漫画风 |

总计 1,470 帧、49 秒；山体脚本保留原有 960×540，其余为 1280×720。前后对照使用同一脚本、分辨率、帧号和光照配置，不以不同构图或缩放代替改进证据

## 执行

在仓库根运行，以下命令可直接用于 fish。先由统一构建产出本轮二进制，GPU 串行运行；每个输出目录必须不存在，重跑时改末级运行名

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/mountain-city.json --output output/capture/task045-visual-r4-mountain
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/descent-cut.json --output output/capture/task045-visual-r4-cut
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/exterior-details.json --output output/capture/task045-visual-r4-residential
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/street-tree-detail.json --output output/capture/task045-visual-r4-tree
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-character.json --output output/capture/task047-model-r3-runtime
```

需要补充大范围山坡观察时，先用现有 `hillside` 自由检查视点确认遮挡情况；只有上述山体与近切坡不能覆盖新材质时，再复用既有 18 秒输入写一份该视点脚本，不先扩展拍摄框架

## 机器条件与看图

现有 Capture 自动检查必需资产加载完成、全部帧异步落盘、每帧 Transform 有限；Viewer 脚本另查真实相机位移、转角或释放后稳定。人物脚本检查脚点高度、移动距离、疾跑、锁定、暂停与实际动画时间推进。条件和失败信息来自 [capture.rs](../../../../game/src/capture.rs)，它们不证明纹理自然、人物好看、碰撞全城正确或性能达标

执行后分别记录机器检查数量、失败条件及实际值；再写 self-audit 的帧号、可见改进、残留问题与未覆盖范围。截图间必须检查至少一组连续帧，避免静止构图掩盖移动时的穿插、叶片闪烁或纹理变化。人物灰度模型的风格与配色仍按 TASK-047 的真实反馈记录，不因播放动画成功而验收

`run.json` 自动记录二进制、脚本、Cargo.lock、提交与工作区状态；前后使用同一二进制而替换资产时，另保留相应源资产与运行资产 hash。比较过程中若混入其他人的素材或光照改动，应明确是综合结果，不能把整个差异归因于单一资产

查看并记录后，清理本轮 PNG、关键帧、接触表和录屏，保留日志、状态 JSON、运行参数与文字结论；在实际清理后记录范围与占用，不预写已清理

## 当前状态

- 源码与脚本核对：已读取现有视点、输入路径及断言实现，以上复用不改场景数据
- GPU 运行与画面自查：root 已串行执行，机器结果、失败迭代与实际观察见 [集成记录](review.md)
- 作者体验与画风验收：NOT RUN，本次计划不代替作者批准

实际执行后的状态、失败诊断与已查看帧见 [集成记录](review.md)；上方检查集保留复现用途，实际运行另包含修复与诊断批次
