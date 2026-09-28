# 完整上山运行验收

基线b0e6928，输入hash与binary见provenance.json；实际Linux／RX6650XT／Vulkan，960×540、30fps、450秒，13500张PNG和视频位于 `output/ascent/2026-09-28/r1/full/`

## 结果

PASS：58项库测试、3项Viewer测试，capture窄测10项，fmt与clippy -D warnings，默认正式入口build，文档与Skills、地图检查和双站构建95／158页；命令及输出分别见同目录log

PASS：真实录制8项检查，130节点全部到达；第12610帧即420.33秒到450.046m，零恢复，相机距离最小3.8m。路线驱动只发输入；地图道路关系、实际支撑、每节点水平距离和高度均核对，终点保持稳定。固定步与输入复现不等于跨GPU像素确定性

预期FAIL：missing-route探针明确退出1，保留原始命令和诊断；CPU还覆盖路段缺失、无进展、节点到达与死区逆映射。失败探针不是成功验收

## 画面自查

实际查看450、900、1800、2700、3600、5400、6300、7200、8100、9000、9900、10800、13499帧，另查看6458—6460、8549—8551、11361—11363、12609—12611四组连续帧。三处停步平台与峰顶能进入，代理持续可见，未见相机穿地；路线大部分仍是长梯段、硬切坡面及灰盒建筑，局部坡阶边缘锯齿和空旷轮廓需要后续美术处理，不能把可通行称为最终山城效果

这里是看图self-audit，未完整播放视频；机器run.json中的visual_review保留生成时状态。作者镜头手感与7分钟节奏、Windows和物理手柄NOT RUN，TASK-014与G1整体不据此放行。下一项沿同一路线下山回店，继续检查下阶与城市回望

## 复验

以下命令从仓库根执行，兼容fish，录制须选新目录

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-ascent-full.json --output output/capture/ascent-review
bun run check:docs
bun run docs:build
bun run tasks:sync
bun run tasks:check
```

独立只读审查核对数据源、系统顺序、输入与失败时序，无阻断；简化评审结论 `Lean already. Ship.`
