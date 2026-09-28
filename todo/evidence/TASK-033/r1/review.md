# 摘星台往返运行验收

本轮完成同一会话从月台杂货上山、到摘星台、原路回店的技术验证；实际失败定位、接地修补、挡墙修补与对应复验均保留证据，TASK-033技术交付完成

## 输入与证据边界

完整录制使用 Linux／RX 6650 XT／Vulkan、640×360、30fps、27000帧，共900秒模拟时间，原始PNG、状态与视频位于 `output/ascent/2026-09-28/r2/round-trip/`；机器执行耗时约588.64秒，不是实测玩家完成时间

基线提交 `c8c934b7c607ca621faa92a0e9f7285a5c3e70a5`，实际输入修订为 [provenance.json](provenance.json) 的 `8b7f209e48a1f46ffecd76b7eb9bd2f3aae6e2b0bd67bd18182b5d97d8a74799`；它记录完整GPU运行使用的文件，不用后续修改覆盖旧录制来源，二进制SHA见 [run.json](run.json)

最终交付修订为 [final-source.json](final-source.json) 的 `7b892b0445dac49218f169bf65f71c9e1f83ab18e2e6d3e5146ebabf0ab51787`，包含挡墙最后修补与同工作区并行的TASK-034 HUD代码；该快照记录最终回归的实际输入，TASK-033验收范围仍限往返与暴露问题的修补，原完整录制的来源另行保留

完整GPU录制发生在挡墙绕序修补之前；修补之后的证据是同位置Viewer转头比较与另行回归，不是再次完成900秒完整游戏录制

## 自动检查

PASS：完整录制7项检查，259/259节点实际到达、所有到达点接地、全程零恢复，保持同一世界实体集合；第12602帧抵达峰顶，第24035帧回到 `home`，实际足点约 `(100.0300, 28.0451, -255.0225)`，第26400–26999帧保持接地与位置稳定

路线复用源数据130节点，峰顶之后原路逆序返回，峰顶仅出现一次；259节点的逐次位置、目标、距离、高度误差、接地与恢复次数保留在 [state-summary.json](state-summary.json) 中，未归档全部27000条逐帧状态；脚本仅发送移动和看向输入，不写角色或相机结果

PASS：接地修补后的 `player::` 窄测8项，64Hz与30Hz分别在约803.16秒、799.47秒完成259节点，零恢复；见 [player-after-adjacent-treads.log](player-after-adjacent-treads.log)；当时全测59项库测试与3项Viewer测试通过，见 [tests.log](tests.log)，构建与Clippy见同目录日志

保留真实FAIL：[round-trip-cpu.log](round-trip-cpu.log) 中30Hz下山在约448m平台接缝处失去接地并停滞，静止落地检查不能复现，4mm低速移动可复现；最终在接触法线探针之外检查几何面的两侧相邻踏面，最多3次短射线均要求实际向上支撑，未放宽节点验收或直接设置接地；386m及448m处、两种步长与两种下落速度已加入回归，薄墙、高台阶和低顶阻挡原测试仍保留

保留真实FAIL：[after-wall-tests.log](after-wall-tests.log) 记录挡墙修补后的首次全测61项通过、1项碰撞测试失败，[wall-crossing-probe.log](wall-crossing-probe.log) 保留定位该假接触时的窄测失败；除原绕序反转导致背面剔除外，挡墙上下沿相交时旧quad自交，造成小店起点虚假接触；最终在交点拆分三角形并插值UV，原 `collision.rs` 与小店测试断言、参数均保持不变

PASS：最终挡墙交点修补后63项库测试，含原小店碰撞、10项几何与完整259节点往返；64Hz约802.36秒、30Hz约797.03秒，均零恢复，见 [wall-crossing-tests.log](wall-crossing-tests.log)；Viewer全部目标Clippy通过，见 [wall-crossing-clippy.log](wall-crossing-clippy.log)，两项几何窄测的原FAIL分别保留在 [wall-winding-before.log](wall-winding-before.log) 与 [wall-crossing-before.log](wall-crossing-before.log)

PASS：最终交点修补后另行运行90帧、3秒同位置Viewer转头，6项检查全部通过，见 [wall-final-run.json](wall-final-run.json) 与 [wall-final-state-summary.json](wall-final-state-summary.json)；这次复验覆盖最终挡墙画面，完整往返GPU仍以前述旧输入为准，最终代码的完整往返依据CPU测试

## 画面自查

主执行Codex查看下山关键帧12600、12800、14000、15000、16000、17000、19000、21000、23000，以及12938–12940、14517–14519、24034–24036三组连续帧；归档复核者实际查看12602、15000、24034–24036，峰顶到回店的位置变化与状态一致，代理始终可辨认；旧录制回店时按键提示发生键鼠／虚拟手柄切换，不把它视作物理手柄验证

下山画面暴露挡墙背面被剔除形成蓝色开口；修补前后各90帧、3秒的Viewer录制均通过6项检查，同一实际下山相机位置保持固定并向右、向左转头，参数见 [wall-before/run.json](wall-before/run.json) 与 [wall-after/run.json](wall-after/run.json)

归档复核者逐张查看修补前后19–21连续帧及44帧，另查看修后74帧；原蓝色开口被连续灰色挡墙覆盖，转头后的墙面仍可见；对照保留在 [修前20帧](wall-before/frame00020.webp)、[修后20帧](wall-after/frame00020.webp) 与两目录的连续帧中，实际观测未见这组帧内的闪烁

最终交点修补后的20帧由主执行Codex查看，归档复核者另查看44与74帧，左右转头后挡墙仍完整，未重新出现蓝洞，见 [最终44帧](wall-final-00044.webp) 与 [最终74帧](wall-final-00074.webp)

大块褐色地形折面、长梯段、空旷城市轮廓仍属于 TASK-023 的地形与灰盒品质工作，本轮只证明往返控制、恢复约束与挡墙修补范围，不将其称为最终山城效果；19张归档图保持原尺寸、WebP质量80，来源与SHA见 [archive-manifest.json](archive-manifest.json)

以上是实际看图的 self-audit，未完整播放视频；原始run/state中的 `visual_review` 保留工具生成时的值，由本记录补充观察结果；作者手感、陌生玩家路线理解、Windows与物理手柄均 NOT RUN，TASK-014和G1整体不据此放行

## 复验入口

以下命令从仓库根执行，兼容fish，capture输出使用新目录

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib player:: -- --nocapture
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-round-trip.json --output output/capture/round-trip-review
python3 tools/capture.py --script game/capture/descent-cut.json --output output/capture/descent-wall-review
```

往返驱动与接地修补的代码审查属于实现者自审，核对逆序节点、终点绑定、有限探针与原碰撞测试；挡墙修补由另一位实现者完成并回归，未发现额外实现阻断，两部分简化评审均为 `Lean already. Ship.`，最终画面按上文单独验收
