# 曜 r9 实际游戏检查

PASS：使用已有正式游戏二进制运行 `character-orbit.json` 240 帧、30 FPS，显式选择 CHR-001；实际资源加载、原地绕摄、Idle 推进、无位移及地面状态通过，完整记录见 [run.json](runtime-orbit/run.json) 与 [state.json](runtime-orbit/state.json)

```fish
python3 tools/capture.py --binary game/target/debug/n-side --project-root . --script game/capture/character-orbit.json --output output/character-hair-r9/runtime-orbit --character-preview CHR-001 --no-video
```

Root 实际查看 Blender 前后侧后与背面、最终正面和冠顶共六图，以及游戏第 59、139、204 帧。背面发尾的长短差异较旧版清楚，游戏载入后未见额外脱落、显著裂开或骨骼错位；前脸与衣着维持灰阶候选。冠顶仍有放射式汇聚，正常跟随距离下发束细节占屏很小，模型、衣着与全场景仍远未达到 README 预览图的完成度

本轮改变的是头发静态网格；四动画数据保持由资产二进制比较证明。此绕摄只运行 Idle，不把它宣称为本轮重新实测全部四动作，也不把模型接入计作作者外观验收，继续保持 needs_revision

运行二进制为上一轮已编译的真实游戏，加载本次冻结的新 GLB；同时工作区扶手与下一版橱窗源改动尚未编译，画面不能证明这些场景新增内容。精确二进制、GLB 与源版本分别见运行记录及本轮模型检查

本轮未编码视频；以上为实际静态帧自查，未进行盲测或作者验收。视觉临时产物待本批统一记录 hash 后清理
