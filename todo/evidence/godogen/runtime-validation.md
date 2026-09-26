# 真实运行与集成验证

日期：2026-09-26；实现提交 `761f3251b474b76e93e3b979e0717282a98aae56`，录制时剩余工作树变更仅为操作入口与本轮文档。运行日志保留实际提交、工作树、脚本、锁文件与二进制哈希，不能用当前 HEAD 的变化抹去证据所属版本

## 环境与实现路径

本轮使用 Linux、Rust/Cargo 1.98.1、Bevy 0.19.1、Bun 1.4.2、Python 3.14.7、Pillow 12.3.0 与 FFmpeg 9.0.2。宿主 Vulkan 设备为 AMD Radeon RX 6650 XT / RADV NAVI23，Mesa 26.2.3；没有活动桌面，通过正常宿主授权执行离屏渲染，沙箱不可见 GPU 不代表机器没有设备

API 对照已安装 `bevy-0.19.1/examples/app/headless_renderer.rs`、`examples/window/screenshot.rs`、`bevy_render` 的 `view/window/screenshot.rs`、`bevy_time` 和 `bevy_camera_controller/src/free_camera.rs`。没有改 Cargo.toml、Cargo.lock 或 Bevy 版本，没有新增录制 crate

正式游戏入口仍是 Logo；当前可用的真实连续交互为地图 Viewer 自由镜头。捕获入口复用 `WorldScenePlugin`、真实城市与资产、`FreeCameraPlugin`、原有 `camera_focus` 和渲染设置，脚本发送按键与鼠标输入，不直接设置受测 Transform

## 实际命令

以下命令均从仓库根执行，可直接用于 fish；重跑 capture 时换一个新的输出目录

```sh
cargo fmt --manifest-path game/Cargo.toml --check
cargo clippy --manifest-path game/Cargo.toml --locked --all-targets --features viewer -- -D warnings
cargo test --manifest-path game/Cargo.toml --locked --features viewer
cargo build --manifest-path game/Cargo.toml --locked --features viewer
game/target/debug/map_viewer --project-root . --validate
bun run test:tools
bun run check:docs
bun run check:roadmap
bun run check:narrative
bun run check:story-design
bun run check:templates
bun run docs:build
python3 tools/capture.py --script game/capture/viewer-tour.json --output output/capture/viewer-tour-761f325 --binary game/target/debug/map_viewer
python3 tools/capture.py --script game/capture/assertion-failure.json --output output/capture/assertion-failure-761f325 --binary game/target/debug/map_viewer
```

| 检查 | 结果 |
| --- | --- |
| Rust 格式、clippy、构建 | PASS，无新增依赖或编译警告 |
| Rust 测试 | PASS，13 个 world 测试与 1 个 capture 脚本契约测试 |
| `map_viewer --validate` | PASS；995 节点、564 道路、231 建筑、74 地表、51 树、2591 meshes；保留影院上层平台某段无可用支柱位置的既有 Warning，CPU 预检不验收空间设计 |
| `test:tools` | PASS，117 项 Python 测试、51 项 Bun 测试；包含 PNG CRC/完整解码、丢帧、状态失败、超时退出 124、原图保护、许可漂移及 Codex 缺失时 NOT RUN |
| 文档、Skills、路线图、叙事、故事索引、模板 | PASS；31 个 Skills，25 个接入项；71 个稳定 ID，91 个场所、12 个街坊、35 个人物、18 个委托索引 |
| Wiki | PASS，VitePress 1.6.4 完成构建 |
| fish 命令语法 | PASS，`fish --no-config -n -c` 检查入口、游戏、工具、运行规范与本记录的 15 个命令块；不把语法检查当作再次执行 |
| 正常 capture | PASS，包装器和 Viewer 均退出 0，9 项机器检查通过 |
| 故意失败 capture | 预期 FAIL，包装器和 Viewer 均退出 1；没有把失败输出计入正常录像 |

## 机器运行证据

成功目录 `output/capture/viewer-tour-761f325/` 包含 540 张完整 PNG、7 张关键帧、18.000 秒 H.264 视频、`run.json`、`state.json`、`runtime.log`、原脚本与 ffprobe 结果。固定步长 1/30 s，640×360；资产和模型实例就绪后预热 30 帧，异步截图成功落盘后才继续时间线

9 项检查分别为：必需资产就绪、全部截图完整、540 次有限 Transform、真实横移、观察转向、Esc 阻断仍按住的输入、M 恢复移动、松开后摩擦停稳、最后释放保持稳定。代表值为横移约 12.0002 m、转向约 0.3000 rad、恢复后移动约 11.6000 m；各条件与逐帧样本在 `state.json`

失败目录 `output/capture/assertion-failure-761f325/` 保留 30 张 PNG 和状态／日志。期望 1000 m，实际约 11.6002 m，`intentional_failure_movement_does_not_teleport` 为 false；资产、帧数、Transform 三项仍通过，说明失败来自明确条件而非图形环境不可用

两个运行使用同一二进制 SHA-256 `3a4188d49930413b1b61a98f50afbeed8fa2833dba7d7d3e89d9a3a93f57e285`。此前 `viewer-tour-verified` 和 `assertion-failure-verified` 保留开发中证据；本记录以上述提交命名目录为交付证据

## 实际看图

Codex 通过 FFmpeg 从交付 MP4 解码帧 0、29、30、31、149、239、240、241、299、300、301、539，打开 `video-review/contact-sheet.png`，并打开原始 240、241 帧检查停止段。原始连续帧与视频解码帧都保留在成功目录。本次没有专用视频播放能力，采用关键帧与连续帧观察，标记为 `self-audit`

- 开场能看见小店立面、招牌、窗格、道路、台阶、天空和日光阴影，所查帧没有黑屏、空帧或加载失败占位
- 29—31 帧展示横移开始，149 帧展示转向后的建筑关系；239—241 与 299—300 帧保持机位，301 帧后恢复控制，539 帧已升到屋顶上方
- 640×360 中可辨小店的色带与入口，招牌副文字不足以做可读性验收；远处城市仍是现有样板，不由本次工具建设宣布美术完成
- 未见所查连续帧出现明显跳位；这只覆盖指定时段，不推导完整手感、碰撞、动画和跨设备品质

`run.json` 和 `state.json` 的 `visual_review` 保持机器默认 NOT RUN，上述文字单独记录实际看图，工具不伪造人工结论

## 修复、CI 与未覆盖项

独立复查发现并修复：截图门槛从只看文件头改为完整 PNG 校验／解码；Windows 二进制默认路径补 `.exe`；缺少 Codex 时发现探针记为 NOT RUN 并保持非零退出。最终测试和上述两条 capture 均在修复后执行

CI 分离 documents-and-tools、game-cpu 和手动 Vulkan 渲染任务，配置 YAML 已解析检查，本地命令通过。远端 Actions 未实际运行，自托管 `nside-vulkan` runner 尚未部署；没有把未调度记为 PASS

NOT RUN：Windows 实机、手柄、桌面鼠标捕获与失焦、人类操作手感、陌生玩家测试、角色动画／碰撞／任务，以及 Blender 和外部图片／视频／3D 生成。当前尚无这些玩法和生成能力时不搭空壳，也不增加 Demo 已验收数量
