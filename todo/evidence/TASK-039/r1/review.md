# 公共观察鼠标返回验收

公共观察使用真实返回按钮，鼠标左键命中后通过既有输入门槛关闭。空白点击保持原面板，关闭帧与持有输入期间不泄漏到人物或镜头；暂停恢复保留所选地点。当前仅为月台杂货与摘星台既有短公共信息提供该控件，未增加任务线索或滚动系统

## 运行与反例

三条原生Linux Vulkan运行均使用二进制 `309826c336b8488bc28cab1297725180b0080abff6a4e32685abfcfb740b1c32`，完整命令、实际脚本、版本与未提交范围见对应run。原始PNG序列在 `output/observation/2026-09-28/r3/`，本目录保存脚本、断言摘要、运行日志及有限WebP80；成功路径另生成视频

| 路径 | 结果与范围 |
| --- | --- |
| wide／pointer | 1280×720，600帧／20秒，13项检查PASS；空白点击、实际按钮命中、持有输入、释放后移动、手柄重开、暂停恢复、再次鼠标关闭及键盘返回 |
| narrow | 480×720，600帧／20秒，14项检查PASS；相同行为和全过程125%字号，使用独立 `narrow-settings.json` 夹具，镜头保持100% |
| miss-failure | 1280×720，300帧／10秒；将预期关闭的一次点击移至空白处，6项检查中 `mouse_hits_visible_return` 按预期FAIL，其余5项PASS；原生退出1、wrapper退出1 |

反例在269帧仍显示公共信息面板，没有将鼠标按下直接当成关闭成功；预期失败记录保持原始FAIL，不改为PASS。关闭与持有区间259—299逐帧检查位移≤0.001m、转角≤0.002rad、恢复次数为0；309—319真实移动≥0.8m证明释放后仍能继续操作

capture发送的是同一图像目标上的原生 `PointerInput`，输入经过 `UiPickingPlugin` 的布局与裁剪命中，再由按钮Pointer事件提交关闭请求；未写 `Interaction`、Observation.open或人物结果。CPU测试另检查事件请求只消费一次、等待释放、非主键无效、隐藏／暂停时拒绝以及未消费请求过期；该测试本身不声称实际布局命中

## 实际看图

本代理查看narrow259—261、409，wide439—441及miss269；主代理查看wide259—261和narrow-layout199。480×720／125%画面包含完整用途、开放说明、到达方式、返回按钮与操作提示，未见裁切；两组关闭连续帧中面板正常消失，代理位置与镜头构图保持；暂停恢复后原地点内容仍在，鼠标再次关闭正常。miss269的按钮没有悬停边框，面板保持打开，与失败断言一致

这是self-audit，未完整播放视频，不代签作者体验或G1放行。仅公共观察返回按钮获得图像目标上的鼠标验收，标题、暂停和其他Shell按钮的全部鼠标行为未由本脚本验证；Windows、实体手柄与桌面窗口指针集成NOT RUN

## 检查、来源与长期契约

共享全测77项库测试及3项Viewer测试、build与最终viewer全目标Clippy通过，见 [TASK-037 tests](../../TASK-037/r1/tests.log)、[build](../../TASK-037/r1/build.log)、[Clippy](../../TASK-037/r1/clippy-final.log)。该库测试包含本模块4项测试，原观察距离、高差、朝向和真实墙阻挡规则继续通过

GPU运行源码保留在 [TASK-037 runtime-source](../../TASK-037/r1/runtime-source.json)，input_revision为 `7a31648906709021c980f1476dfe078cc7b589d307117feb8d0c77c4d38e8bcf`；每条实际脚本在本目录独立归档，run记录脚本hash。最终交付输入由 `final-source.json` 记录，覆盖生产脚本、窄屏脚本、反例及独立字号夹具。GPU之后的Rust差异仅为两个if的等价折叠与观察查询Clippy说明；wrapper随后为TASK-041增加可指定runtime project-root，默认根路径不变，该版本不冒充本轮录制使用的旧wrapper

长期契约写回 `docs/dev/engineering/player-preview.md`，运行与复验入口写回 `game/README.md`。实现没有额外控件框架或滚动模块，沿用原生picking与已有观察状态；Ponytail自审为 `Lean already. Ship.`

## 复验

以下从仓库根执行，输出目录必须尚不存在；主脚本使用默认隔离设置。窄屏及反例输入保留在本目录，可分别传给同一capture入口，窄屏需使用独立目录内的125%字号夹具

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib observation -- --nocapture
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-observation-pointer.json --output output/capture/walk-observation-pointer
python3 tools/capture.py --binary game/target/debug/n-side --script todo/evidence/TASK-039/r1/miss-failure-script.json --output output/capture/observation-pointer-miss
```

最后一条是预期失败探针，正确结果为非零退出和 `mouse_hits_visible_return` 失败，不把它加入要求全部成功的批处理
