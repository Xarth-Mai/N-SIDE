# 显示设置恢复与本地试玩包

## 输入与交付

接续 main@8d24201a98f11f07c3b74bfeef7b528ab9560664 的 TASK-043 未提交实现，范围为显示确认、启动恢复与本地包更新；既有跳跃、疾跑、锁鼠和19项画质保留，未新增委托或路线图能力。原有 AGENTS.md 与运行验证文档改动保留，本轮未提交、未推送

窗口模式或分辨率调整后试用15秒，默认选择恢复；确认才保存新显示值，Esc、真实时间超时、失焦和手柄断连恢复原显示值与来源行。保存快照隔离未确认值，其他画质和通用偏好继续保留；`--reset-graphics` 只恢复默认画质，沿用原坏文件保护

## 实际检查

| 检查 | 命令或记录 | 结果 |
| --- | --- | --- |
| Rust格式与构建 | `cargo fmt --manifest-path game/Cargo.toml --all -- --check`；`cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins` | PASS，format.log、build.log |
| Rust回归 | `cargo test --manifest-path game/Cargo.toml --locked --features viewer` | PASS：91项库测试、3项Viewer测试；独立重启worker的1项ignored由父测试调用，all-tests.log |
| 真实显示确认路径 | `python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-display-confirm.json --output output/capture/task043-display-confirm --settings-dir output/capture/task043-display-settings` | PASS：880帧／29.33秒、13项检查 |
| 全部设置回归 | `python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-graphics-options.json --output output/capture/task043-display-options` | PASS：660帧、真实TAA／SSAO／阴影／Bloom及显示确认后返回 |
| 包内独立资源与恢复 | runs.json 中 package-restore 命令；使用包内二进制和 project-root | PASS：120帧，恢复已确认1600×900，保留125%字号 |
| 包内启动恢复 | runs.json 中 reset 命令；经 `run-walk-preview.sh --reset-graphics` | PASS：120帧，回到默认1280×720与画质，125%字号保留；文件前后对照见 settings-before-reset.json、settings-after-reset.json |
| 故意失败 | display-expected-failure.json，错误要求1920×1080 | EXPECTED FAIL：仅 intentional_wrong_resolution_is_rejected 失败，程序与外层退出1，基础资产／帧数／Transform检查通过 |
| 打包工具 | `bun test tools/tests/package-preview.test.ts` | PASS：4项、37次断言，package-tests.log |
| 包创建和清单复验 | `bun tools/package-preview.ts --binary game/target/debug/n-side --viewer game/target/debug/map_viewer --output 'output/packages/N-SIDE Linux Preview 2026-09-29' --strip`；同工具 `--check` | PASS：105个文件，package.log、package-check.log |
| 文档与Wiki | `bun run check:docs`；`bun run docs:build` | PASS：文档／Skills、地图检查、player与dev独立构建；日志见本目录 |
| 只读审查 | 独立检查确认、恢复、保存、重置与capture采样 | 无阻断缺陷；复杂度结论 Lean already. Ship. |

固定版本、输入文件和包hash见 inputs.json，完整运行命令、配置、状态和日志路径见 runs.json。录制使用本机Linux／Radeon RX 6650 XT／Vulkan，固定模拟输入通过正式入口和原有系统，不写受检结果

第140帧打开弹窗，持有Enter不立即确认；第180帧Esc恢复；第230帧明确保留；第260帧再次试用，第711帧超时恢复；第790帧模拟失焦恢复，第815帧手柄断连消息恢复。退出后设置文件只留下已确认resolution=1，恢复命令将其改回0，large_text始终为true

## 画面与限制

SELF-AUDIT：查看确认路线159、189、239、709、729、819帧及超时相邻710／711帧，核对125%字号下的标题、倒计时、恢复默认焦点、回到来源行；查看全选项568帧和包内恢复／重置29、119帧。所查页面文字完整，超时切回设置页，没有卡在确认界面；原有街区仍为灰盒和胶囊人物

图像目标保持脚本1280×720，configured_resolution／configured_borderless只声明真实菜单与设置资源状态，不能证明显示器实际尺寸。焦点和手柄断连是Bevy输入消息模拟；原生窗口、系统VSync刷新、实际鼠标约束、Windows和物理手柄仍NOT RUN，沿用r1的环境限制，不重复搭建Xvfb或安装图形栈

首轮窄测暴露entry_input超过Bevy支持的16个顶层系统参数，改为将两项设置资源作为tuple注入；随后发现测试夹具没有窗口实体，补充对应逻辑窗口后全测通过。没有通过降低生产断言规避失败，夹具失败日志保留于display-tests.log

## 包与下一动作

本地包为 `output/packages/n-side-linux-preview-2026-09-29.tar.gz`，解压目录为 `output/packages/N-SIDE Linux Preview 2026-09-29/`，含主入口、Viewer、透传参数的启动脚本、资源、源码和既有许可。包内说明已更新跳跃、疾跑、锁鼠与画质操作，清单记录工作区来源；这是本机Linux包，不作为跨发行版或公开发布验收

从仓库根可在fish执行 `./output/packages/'N-SIDE Linux Preview 2026-09-29'/run-walk-preview.sh`；需要恢复画质时在末尾追加 `--reset-graphics`。下一步是实际桌面检查跳跃／疾跑、鼠标锁定与释放，以及画质第5页改变窗口后的保留／恢复；作者手感和原生窗口检查前TASK-043保持review

## 清理

实际看图、核对状态并记录结论后清理本轮成功和故意失败录制的PNG、关键帧和视频，以及两份临时源码快照；保留正式包、复现脚本、参数、日志、状态JSON、hash和文字结论。进程核对与释放空间见cleanup.json
