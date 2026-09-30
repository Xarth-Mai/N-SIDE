# 人物 Jump 接入与空中暂停复验

2026-10-01，基于 `b6a8c6d` 的当前批次；两位原创灰模增加真实蒙皮 Jump，接入现有控制器的离地与接地状态。本轮没有增加跳跃玩法、改变人物外观或通过作者审美

## 实际交付

CHR-001 与 CHR-002 的可编辑源工程、导出 GLB、生成和检查脚本均加入 40/60 s 单次 Jump；Root 不负责世界抛物线，Hips 和四肢表现压缩、收腿与展开。原网格、贴图、绑定和 Idle／Walk／Run 数据的保持对比见[资产报告](asset/review.md)，造型仍为 `needs_revision`

运行时根据真实 `grounded` 选择 Jump，播放一次后保持末姿态，实际接地后按水平速度回到 Idle／Walk／Run。暂停冻结动画时钟和控制器，恢复延续同次动作；下一次实际起跳重新播放。capture 的 clip 检查与角色加载共用同一四动作表，修复旧检查器拒绝 Jump 和原脚本误在整个空中区间要求 Idle 的问题，详见[独立代码审查](runtime/code-review.md)

`tools/prepare_belle_reference.py` 同步重定向 Jump，252 帧姿态及四动作 GLB 检查通过；参考 GLB 保存在本地隔离目录，没有迁入运行资产或替换原创角色，具体来源和分发边界沿用[社区模型记录](../community-reference-r1/review.md)

## 验证

| 实际命令或路径 | 结果与证据 |
| --- | --- |
| `cargo test --manifest-path game/Cargo.toml --locked --lib` | PASS，134 passed / 0 failed / 1 ignored，[日志](runtime/lib-tests.log)；ignored 为由父测试另进程显式调用的设置子进程测试 |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins` | PASS，正式游戏与 Viewer，[日志](runtime/build.log) |
| `cargo fmt --manifest-path game/Cargo.toml -- --check`、`git diff --check` | PASS；先修复新增 expect 链的 rustfmt 换行后复查 |
| Python `unittest discover` 的角色工具及 capture 窄测 | PASS，9 + 4 项；不以最初 dotted module 调用错误作为测试结果 |
| `bun run check:docs` | PASS；最终任务同步后再检查 |
| `bun run docs:build`，PATH 前置应用自带 Node | PASS，58 地图测试、player/dev 构建与受众产物检查，[最终日志](runtime/docs-build.log) |
| 曜／玲 `walk-character.json` | 两次 PASS，各 540 帧 / 18 s，27 原生断言及 1 指定角色检查，覆盖走跑、三次跳跃、手柄与键鼠、暂停和失焦 |
| 曜／玲 `character-jump-pause.json` | 两次 PASS，各 180 帧 / 6 s，8 原生断言及 1 指定角色检查，覆盖空中暂停、暂停中输入、恢复、落地和再次起跳 |
| 本地社区铃 `character-jump-pause.json` | PASS，180 帧 / 6 s，8 原生断言及 1 指定角色检查；只证明本轮隔离转换兼容 |

完整命令、二进制和脚本 hash、状态、日志及影片编码结果保存在 [运行汇总](runtime/summary.json)所对应子目录，均复用真实应用与输入路径。五段视频编码 PASS，本轮没有视频播放工具，实际审查使用关键帧和连续帧；不会把编码成功写作视频观看

Wiki 首次失败来自系统 Node 缺少 `libsimdjson.so.33`，Bun 直接执行 VitePress 又复现既有 esbuild service stopped；该入口试改已全部撤回。最终使用已安装的应用 Node，不安装或升级系统依赖，也不保留额外启动器，失败日志保留用于说明环境边界

## 实际画面 self-audit

Root 查看曜走跑的 62、69、70、71、78、392 帧，空中暂停的 72、95、108、129 帧，玲暂停路径的 64、108、129、155 帧，以及社区铃的 59、64、108、155 帧。玲完整走跑另有[独立连续帧观察](runtime/ling-visual-review.md)，实际文件 hash 分别留存，均不替代作者验收

画面中身体随真实控制器离地，膝部和手臂产生变化，落地后脚回到铺装面；已看帧未发现炸裂、明显关节脱离或落地后持续漂浮。暂停菜单遮住大部分角色，因此冻结结论主要来自逐帧实际状态：第 72–95 帧 Jump elapsed 固定为 0.2333333，foot 固定在 28.8859978 m，恢复后继续推进到第 108 帧 0.5 s，第 129 帧接地恢复 Idle

动作仍有直立和整体上提感，前段腿部变化偏小，硬切与连续动画混合尚未制作；灰模的服装、面部、材质和城市整体距离 README 目标仍明显。没有把状态 PASS、参考模型接入或小样数量计作 G2 正式品质放行

## 收尾

本批代码、资产与场景审查已处理有效发现，Ponytail-review：Lean already. Ship.；Windows 本批实机、正常窗口物理手柄、作者美术和连续手感验收为 NOT RUN

临时截图、PNG 帧与视频在实际查看和结论记录后清理，保留源码、资产、参考模型、脚本、日志和状态，明细见 [运行清理](runtime/cleanup.json)及[资产清理](asset/cleanup.json)
