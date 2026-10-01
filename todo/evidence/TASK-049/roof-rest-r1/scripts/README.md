# R6 屋顶运行复现

真实场景沿用正式 `PreparedScene`、源地图与 GLB；Viewer 脚本仅驱动已有相机输入，实际人物通行由真实场景碰撞检查分别记录，不把自由相机当人物控制器

| 脚本 | 画面与输入 |
| --- | --- |
| `roof-bench.json` | 沿用修复前 `eye-cinema-bench`、1280×720、30fps、60帧静态，核对同机位座椅、条栅与铺地变化 |
| `roof-city.json` | 人眼 `[316,241,38.725]` 朝 `[311,230,37.8]`，120帧，A/D各4帧横移、Escape停控，检查朝城视野及屋顶边界 |
| `roof-wide.json` | `[360,203,49]` 朝 `[322,238,37]`，120帧，真实左右横移，核对铺装、条栅、外侧平台与整体比例 |
| `roof-bearing.json` | `[348,220,35.2]` 朝 `[340.6,228,36.2]`，120帧，真实左右横移，核对新外半幅板底与主体承托 |

仓库根目录，fish 可直接执行，输出目录须不存在；先运行 `cargo build --manifest-path game/Cargo.toml -j1 --features viewer --bins`

```fish
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/roof-rest-r1/scripts/roof-bench.json --output output/roof-rest-r1/roof-bench
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/roof-rest-r1/scripts/roof-city.json --output output/roof-rest-r1/roof-city
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/roof-rest-r1/scripts/roof-wide.json --output output/roof-rest-r1/roof-wide
python3 tools/capture.py --binary game/target/debug/map_viewer --aa msaa4 --no-video --script todo/evidence/TASK-049/roof-rest-r1/scripts/roof-bearing.json --output output/roof-rest-r1/roof-bearing
```

机器检查包括资源、有限变换、全部帧落盘、移动距离与停止状态；视觉自查实际查看关键帧及连续横移帧。正常完成后按项目约定删除本次媒体，保留参数、hash、日志、状态与文字结论；没有生成视频时保留 `NOT RUN: --no-video`
