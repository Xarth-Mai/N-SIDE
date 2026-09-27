# 正式入口、场景清理与共享录制

按技术范围通过，基线 `5defc08`，输入与二进制见 [provenance](provenance.json)。正式入口从 Logo 展示接到真实城市，完成标题、异步加载、固定镜头街区、返回、重入及取消；人物、碰撞、任务、存档、作者体验与 G1 整体尚未验收

## 实现与边界

`app.rs` 使用 Bevy States 与原生任务池准备既有地图和资产，复用 `WorldScenePlugin`、DaylightSettings、字体与 UI tokens。准备任务不直接改 World，取消时移除其结果接收资源；轮询先检查已有状态请求，避免加载完成覆盖同帧返回。场景资源每轮启动一次，失败由正式入口显示恢复页，由 Viewer／capture 返回非零

`clear_scene` 移除真实 MapSource 根及后代和加载资源，宿主镜头、光照、UI 保留。录制实现从 Viewer 子模块移到共享 `game/src/capture.rs`，没有另造城市场景；脚本输入经过真实 Bevy 状态路径。新增 wait 只等待真实页面并暂停录制时钟，异步准备及墙钟超时持续运行

Windows 既有打包脚本改为从包根显式传项目路径，并核对运行字体、OFL 与版权文件；未在本轮运行 Windows workflow。附带 rustfmt 只折行已有楼梯回归的一条 assert，未改几何逻辑；地图和资产数据保持原 hash

## 实际机器检查

| 命令，均从仓库根执行且 fish 可直接使用 | 结果 |
| --- | --- |
| `cargo check --manifest-path game/Cargo.toml --locked --all-targets` | 默认无 viewer 特性 PASS |
| `cargo test --manifest-path game/Cargo.toml --locked --lib app::tests` | 3 PASS，真实输入动作、失焦、取消、失败及重试 |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib --bin map_viewer` | 31 库＋3 Viewer PASS，含场景与录制合同 |
| `cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings` | PASS |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo build --manifest-path game/Cargo.toml --locked --bin n-side` | PASS，正式入口不要求 viewer 特性 |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer` | PASS |
| `python3 -B -m unittest discover -s tools/tests -p test_capture.py` | 2 PASS |
| `bun tools/export-ui.ts --check` | 3 文件 PASS |
| `bun run check:docs` | PASS，含31 Skills／25来源记录 |
| `bun run docs:build` | player 95／dev 157页 PASS，含52地图检查 |
| `bun run tasks:sync`、`bun run tasks:check` | PASS |

CLI 的帮助、缺少成对参数、未知参数和不匹配录制场景共4条检查见 [cli-checks](cli-checks.json)。CPU 鼠标测试构造 Interaction，验证按钮动作路径，不是实际指针操作证据

真实GPU为 RX 6650 XT／RADV／Vulkan，使用当前源码和运行资源。正式录制命令如下，重复执行时使用新输出目录

```fish
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/game-entry.json --output output/game-entry/2026-09-28/r1/verified
python3 tools/capture.py --binary game/target/debug/map_viewer --script game/capture/ui-signal.json --output output/game-entry/2026-09-28/r1/viewer-ui-regression
```

最终正式入口540帧／18秒，11项检查全部通过，视频编码成功；[状态](verified/state.json)与[执行记录](verified/run.json)保存实际结果。frame119和329均有4572个 MapSource 实体，返回标题为0；frame480发进入、481发取消，481仍是Loading/0实体，482已回标题。取消在首次Loading帧触发，避免用固定等待猜测异步完成速度

前两次录制 `main`、`final` 均机器通过，分别保留原提示和提示修订后的结果；最后 `verified` 同时使用最终提示与更早取消输入。三次都保留记录，未覆盖旧证据。录制等待不计入模拟18秒，也不当作加载时长或正常游戏性能成绩

Viewer 回归840帧／28秒／25检查全部通过，[状态](viewer-ui-regression/state.json)保留原菜单、调查记录、焦点恢复、缩放、滚动、输入切换和释放后的相机移动结果

## 真实失败探针

[failure-checks](failure-checks.json)记录实际命令与退出码。缺地图项目仅在 output 中链接现有字体、光照和外观，故意缺少 `district.json`，原仓库数据不变；原生入口返回1，capture 保存具体缺文件原因而非等待超时，见[错误状态](missing-map/state.json)

另一条[脚本](impossible/script.json)通过真实输入进入世界，要求至少100000000个实体，实际120帧完整保存、资产和Transform检查通过，只在该不可能断言失败；原生和wrapper均返回1，见[状态](impossible/state.json)。错误场景会提前结束录制，因此不据此声称失败页画面已验收

## 实际画面自查

本节为 self-audit，实际打开原PNG关键帧与连续帧，非盲测或作者试玩；归档图为同源质量80 WebP，原PNG序列和视频留在忽略的output目录

- 首轮[街景](main/frame00119.webp)的键鼠提示拥挤换行，标题还提示无效的Esc返回；按页缩短提示，修后[街景](verified/frame00119.webp)文字完整、固定镜头身份明确，[手柄页](verified/frame00329.webp)使用对应返回提示
- 最终标题、加载、街区、返回及重入保留清楚焦点；实际检查初录frame0／61／119／329／481／483，以及最终119／329和连续480–482。连续480–482显示标题→加载→标题，世界没有残留到标题底板之后
- Viewer实际看frame150及778–780，长中文与第二条空记录的当前项、焦点、正文一致；这些仍是排版样例，不是已获得的任务线索
- 保留既有 Parley 中文分词告警，字形和换行实际看图；未将告警消失或图片生成成功当作语义、手感或性能验收

## 评审与后续

独立代码审查核对真实退出、取消优先、错误保留、旧页按钮隔离、capture调度和包路径，未发现阻断；复杂度审查结论 `Lean already. Ship.`。其建议的首次Loading帧取消已落实并实际复录

NOT RUN：Windows图形启动与打包任务、实体手柄和热插拔、人工鼠标连续操作、失败页视觉、作者手感和陌生玩家反馈。下一步复用该入口开展中性代理人物的人尺度技术实验，不据此选择最终操作兄妹或锁定镜头参数
