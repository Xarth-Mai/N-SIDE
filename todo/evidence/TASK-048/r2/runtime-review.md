# 安可城市大屏运行检查

## 实际输入与检查

2026-09-30 在 Linux / RX 6650 XT / RADV 运行当前 debug Viewer，源代码、图片与二进制 hash、完整输入序列、参数及结果见 [capture-runs.json](capture-runs.json)。两个公共视点各录制 300 帧、10 秒，1280×720、固定 30 Hz；真实 Viewer 接近、侧移、解除控制，无场景复制或直接改写检查结果

| 项目 | 结果 |
| --- | --- |
| N 站 r2 录制 | PASS，300 帧完整落盘、必需资产加载、Transform 有限、接近与侧移距离、解除控制后稳定，共 6 项 |
| AFTER 9 r2 录制 | PASS，同上 6 项 |
| Rust 格式与全部 bin 编译 | PASS，见 r1/fmt-final.log、r1/build-final.log |
| 屏幕几何与来源锚点 | PASS，1 项；初次支架低于屋顶的问题已修复，见 r1/geometry-test.log |
| Viewer 公共取景 | PASS，3 项，见 r1/view-tests.log |
| 源资产导出 | PASS，r2 重导与只读一致性，见 export-check.log；r1 的运行文件漂移测试正确非零失败 |
| 类型与 Skills 门禁 | PASS，见 r1/types.log、r1/skills.log |
| Wiki 玩家构建 | PASS，101 页，见 r1/docs-build.log |
| Wiki 开发构建 / 全库文档检查 | FAIL，并行 TASK-046 的 `campaign-sequences.md` 尚未落地；已报告其两处缺失引用，未修改该 Agent 的进行中内容 |

`r1/` 为相邻证据目录 `../r1/`，本轮没有调度远端 CI；运行结果只覆盖当前本机输入

## 实际看图：self-audit

实际打开安可 r2 原图与运行画面：N 站 179 帧，AFTER 9 的 119 帧，以及站前 60/63/66、会合庭 120/123/126 的连续移动抽样。海报正面朝公共观看位置，文字没有镜像，框体与后撑附着屋顶，查看范围内没有被立面遮住或明显闪烁

安可形象与「安可」标题在两处街景可辨，N 站斜视更压缩；小字不承担导航或任务信息。r2 相比被否定 r1 用更整块的发束、清楚线条和硬边明暗，风格方向取自作者指定的镜厅左侧角色海报。此为制作自查，作者尚未确认 r2 的最终造型和画风

当前屏幕使用 unlit 贴图，白天能稳定显示；没有实现向街道投光，也没有验证夜景发光效果。周围建筑、天空与街道仍是制作中的预览，海报接入不意味着城市整体已达到概念图品质

## 限制、复现与清理

本轮证明真实世界资产与自由镜头控制，不证明角色可以走到相机位置或碰撞、剧情、动画已完成。Windows、原生窗口连续操作、硬件手柄和作者审美为 NOT RUN

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-station.json --output output/capture/poster-station-next
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/poster-live.json --output output/capture/poster-live-next
```

完成查看和记录后清理 r1 与 r2 的临时画面，保留日志、状态、参数、复现脚本及正式源资产；数量见 [cleanup.json](cleanup.json)
