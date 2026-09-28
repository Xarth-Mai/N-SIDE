# 设置跨启动保存验收记录

完成两项偏好的独立用户文件保存、下次启动恢复、坏档保留与保存失败反馈；技术任务通过，任务／世界进度存档未实现，G1整体与作者手感不据此放行

## 自动检查

PASS：全测77项库测试、3项Viewer测试，见 `tests.log`；设置窄测5项通过，见 `settings-tests.log`。测试列表中的 `restart_worker` 标为ignored，父测试 `preferences_restore_in_a_second_process` 显式在两个独立子进程中调用它完成写入／读取，不把默认跳过项当成单独通过。覆盖版本与字段、过大或损坏文件保留、已有临时文件冲突、正常退出等待最新选择、capture默认隔离与跨进程恢复

PASS：正式入口build与3项Python capture工具测试，分别见 `build.log`、`capture-tools.log`。Clippy首次发现两个可合并if及观察模块查询类型复杂度，原失败保留在 `clippy-before.log`；两处if等价折叠、观察查询使用有原因的局部expect后，viewer全目标 `-D warnings` 通过，见 `clippy-final.log`

## 实际运行与失败路径

Linux Vulkan实际GPU运行，四条录制使用同一个二进制 `309826c336b8488bc28cab1297725180b0080abff6a4e32685abfcfb740b1c32`，命令、平台、脚本、源提交与未提交范围分别保留在对应 `*-run.json`。原始连续PNG和视频在 `output/settings/2026-09-28/r2/`，本目录保留脚本、全量断言摘要、日志与有限WebP80画面，未复制全量采样

| 运行 | 范围与机器结果 |
| --- | --- |
| write | 600帧／20秒，17项检查PASS；标题与暂停设置、字号、真实镜头倍率、设备切换、标题清理及重入，最终保存125%／65% |
| restore | 第二个进程240帧／8秒，7项检查PASS；从同一独立设置目录恢复125%／65%，进入真实街区，65%档一秒实际镜头转角1.040000rad |
| bad | 180帧／6秒，9项检查PASS；损坏的settings.json触发读取失败，默认启动后本次仍可调整，UI显示仅会话有效 |
| unwritable | 180帧／6秒，9项检查PASS；备份目标settings.json.bak被目录占用，实际rename失败，UI显示保存失败，调整继续作用于本次运行 |

write中100%档一秒真实转角1.599996rad，与65%档分开检查；未用设置变量代替镜头效果。两条失败GPU的PASS表示预期退化行为通过，不表示设置已写入。`bad-runtime.log`与`unwritable-runtime.log`保留真实错误原因，`failure-files.json`分别核对坏档 `{broken` 与合法默认文件的预期原字节，两者均原样保留

## 画面自查

本代理直接查看write69、99，restore49、179，bad99与unwritable99；主代理另查看write509。保存成功提示、125%／65%复原及键鼠焦点可见；坏档和保存失败提示均完整换行，未伪报成功，选项保持可操作。restore179显示真实小店场景和125%地点／操作HUD，不把该帧当作人物美术验收

这是self-audit，未完整播放视频。当前连续帧机器完整性由capture检查；Windows实机设置目录、实体手柄、桌面合成器退出、断电恢复与作者体验NOT RUN

## 来源与交付边界

GPU源码快照沿用未修改的 `runtime-source.json`，input_revision为 `7a31648906709021c980f1476dfe078cc7b589d307117feb8d0c77c4d38e8bcf`。之后仅两个if折叠及观察模块Clippy说明发生变化，不改运行行为；最终快照 `final-source.json` 的input_revision为 `bfe826e7bb936280f3690d0a0fe45ba1015fd99ca36148f73e75acce4c138264`，保留运行时快照以免将后续版本冒充实际录制版本。快照同时包括本轮并行的坡面与鼠标返回实现，本任务验收范围只覆盖设置读写、反馈及其真实输入路径

长期接口写回 `docs/dev/engineering/player-preview.md`，操作命令写回 `game/README.md`。独立正确性审查核对Last汇总、Drop等待、临时文件、备份、状态反馈与默认capture隔离，未发现未处理问题；Ponytail结论为 `Lean already. Ship.`

## 复验命令

以下从仓库根执行；两个输出目录与设置目录使用新路径，首个脚本要求初始默认值

```fish
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-settings.json --output output/capture/settings-write --settings-dir output/capture/settings-user
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-settings-restore.json --output output/capture/settings-restore --settings-dir output/capture/settings-user
```
