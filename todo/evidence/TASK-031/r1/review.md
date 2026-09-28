# 手柄断连暂停与键鼠接管验收

基线 `1926c31`，输入完整hash与被测binary见 `provenance.json`。本轮延续真实入口、人物、碰撞、镜头与暂停菜单；地图、叙事、Demo范围和依赖版本未变。连续PNG及视频归 `output/gamepad/2026-09-28/r1/`

## 实现与失败复现

在固定循环前消费GamepadConnectionEvent，断开立即请求Paused，沿用既有物理及镜头门禁。加载中断连保留请求至World首帧，标题／失败／新加载清除；同帧断开后重连仍暂停。重连隔离同帧确认并等待输入释放，键鼠或手柄都能明确继续。菜单显示断连或重连提示，断开后先显示可用的键鼠提示；世界和代理仍为原会话

按当前聚合控制策略，任一手柄断开都触发保护，不追踪最后输入设备；接入设备本身不暂停已经运行的世界。没有新增设备所有权、重连服务或另一套暂停状态机

首次修前测试因fixture在App.finish后安装InputPlugin而失败，见 `cpu-before.log`，不算行为复现。调整组装顺序后，`cpu-before-valid.log` 的3项结果为2 FAIL／1 PASS：断连时NextState仍Unchanged；连接变化同帧导致标题／失败菜单导航。第3项旧测试带Start，旧实现本就会暂停，因此这项PASS不证明断连保护；随后增强为无按钮同帧断开／重连，再单独测持South／Start重连

修后CPU用真实InputPlugin处理连接与RawGamepad输入，覆盖0／1／3固定步、跌落冻结、镜头／复位、原代理保留、键鼠接管、多设备、加载及退出清理。Capture的连接事件在PreUpdate、InputSystems之前发出，由Bevy实际移除／重建Gamepad组件；驱动器不删实体或写游戏阶段。额外测试覆盖等待截图时不发消息、相同请求不重复、缺省保持连接状态、错误类型／Viewer输入拒绝，连接断言期待错误值时确实FAIL

## 执行结果

| 检查 | 实际结果 | 记录 |
| --- | --- | --- |
| cargo test --locked --lib | PASS，57项 | `cpu-after.log` |
| cargo test --locked --features viewer | PASS，57项库＋3项Viewer | `tests.log` |
| cargo fmt --check | PASS | `fmt.log` |
| cargo clippy --locked --features viewer --all-targets -- -D warnings | PASS | `clippy-final.log` |
| cargo build --locked --bin n-side | PASS | `build.log` |
| 文档与Skills | PASS | `docs.log`、`delivery-checks.log` |
| 地图门禁／玩家与开发站构建 | PASS，95／158页 | `wiki.log` |
| tasks:sync／tasks:check及重复生成 | PASS，26张卡，无漂移 | `delivery-checks.log` |

第一次Clippy报告UI缓存键的嵌套元组类型过长，见 `clippy.log`；命名为ShellSnapshot后通过，没有屏蔽lint。该类型别名是完整测试之后唯一源码调整，最终Clippy编译全部目标，运行binary包含此调整

## 实际渲染与看图

Linux／AMD Radeon RX6650XT／Vulkan，1280×720、30fps、固定输入和时间步。三条录制使用相同binary，程序及包装器退出0，视频生成；数值不代表跨GPU像素确定性

| 录制 | 结果 | 覆盖 |
| --- | --- | --- |
| gamepad | 540帧／18模拟秒／13项PASS | 加载期间断连、持键重连、行走中断连、键鼠接管、手柄明确继续、清理及重入 |
| focus | 600帧／20秒／14项PASS | 既有失焦、持键回焦与恢复路径 |
| pause | 600帧／20秒／14项PASS | 既有暂停、持有输入、继续与重入 |

gamepad中各冻结区间脚点、相机位置不变，旋转峰值在0.002弧度容差内；复位计数0。重新确认后的4段移动均约3.2m；没有手柄时键鼠也能继续。世界源对象数4572→标题0→重入4572。连接及game_page断言检查区间结束帧；运动／旋转上限检查整个区间，不混称为全过程状态断言。跌落冻结由CPU碰撞夹具验证，这条平地录像不覆盖跌落

Codex实际查看gamepad的34、89帧，断连／重连提示及中文完整，亮色继续焦点清楚；149—151连续帧显示断连同帧人物已停，下一帧暂停菜单出现；239帧键鼠接管后人物位置变化；294帧手动暂停后接回手柄且持键仍暂停；329、409、499帧分别为继续、清理到标题和重入。另查看focus的34、234帧及pause的272、419帧，两条既有路径的菜单、设备提示、代理和场景保持正常

上述为画面自查，已查看PNG按WebP质量80归档；视频未完整播放。工具run.json中visual_review保留生成时的NOT RUN，本段记录随后实际看图，不修改机器报告来冒充视觉结论

## 范围与下一步

技术任务依据CPU、模拟连接驱动及真实GPU渲染完成；USB／蓝牙拔插、物理按钮、Windows与作者手感为NOT RUN。TASK-030的桌面合成器验收缺口保持，本轮未重试桌面焦点探针。G1整体、TASK-014作者镜头和登高节奏不据此放行

下一项为完整短登高路线的连续渲染验证：沿已通过CPU的真实路线发送输入，保留人物与镜头，不以传送或直接写位置完成。该项本轮未开始，本批提交后停止

从仓库根复验，以下命令兼容fish，输出选择新目录

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-gamepad.json --output output/capture/gamepad-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-focus.json --output output/capture/focus-review
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-pause.json --output output/capture/pause-review
bun run check:docs
bun run docs:build
bun run tasks:sync
bun run tasks:check
```

独立只读代码及简化审查已核对消息顺序、加载标记、重连持键与旧脚本兼容，无阻断，结论 `Lean already. Ship.`；评审者未运行Cargo／GPU，不代签实机操作
