# 短登高真实碰撞与坡面修复

基线 `686cc9ccd23d78b694ec0b1487a4f2b1e4597399`；源地图及稳定ID未变。本轮输入hash、最终代码和构建对应见同目录 `provenance.json`，连续PNG及视频放在 `output/player/2026-09-28/r2/`

## 发现与修复

- 初次CPU探针在小店后的 `/roads/1` 缓坡停滞，9.5模拟秒、仅通过3段；真实30Hz录制也在同坡停止，角色仅移动23.593m。1050张截图、必需资产和有限Transform通过，4项推进断言失败，原生程序及工具均退出1，见 `cpu-ascent-first.log` 与 `ascent-entry-before/`
- 四轮实际扫掠复现投影后点积 `-2.3283064e-10`，不断得到零接触时间；修复只给剩余位移增加1µm向外分量，下一轮继续完整扫掠，不移动脚点或跳过最近障碍。坡上薄墙、高台阶、真实小店墙与两种时间步检查通过，见 `cpu-ascent-slide-diagnostic.log`、`controller-outward-requery.log`
- 修复后走到上街，`/roads/775` 桥侧栏杆横穿短登高接入路。几何现在按实际普通道路／台阶的三角面裁开同高接入口，保留异高道路上方护栏；斜接面仅移除高度差不超过0.17m的部分。新增窄测核对平接、低层路保留及部分斜接，见 `player-outward-requery.log` 与 `bridge-rail-test.log`
- 两个节点的精确中心射线落在网格接缝，错误地把目标判为无支撑；单点检测改为精确向上射线优先，只有失效才观察XZ邻近1cm的真实向上踏面，并约束相对节点高差≤0.195m。此前把胶囊接触作为所有目标首选，会优先命中旁侧结构，已弃用；原始失败保留在 `cpu-ascent-target-support-diagnostic.log`、`cpu-ascent-capsule-target.log`。这些仅改变目标高度观测，不给角色写入高度，也不放宽到点和停滞判据

## 整段CPU结果

`cpu-ascent-neighbor-target.log`：完整PreparedScene共557434个碰撞三角；每种时间步独立在home出生一次，逐段水平输入，不跳节点。每个目标要求XZ距离<0.06m、实际grounded、对实际目标踏面高差≤0.191m；2秒无1cm有效进展、900秒预算或任意自动恢复立即失败

| 时间步 | 结果 | 节点 | 模拟行走 | 峰顶实际脚点 | 恢复 |
| --- | --- | --- | --- | --- | --- |
| 64Hz，普通固定更新 | PASS | 130/130 | 423.421875秒 | (230.02438, 450.04562, -949.9872) | 0 |
| 30Hz，录制固定更新 | PASS | 130/130 | 419.733368秒 | (230.01239, 450.04593, -949.99347) | 0 |

每种时间步仅两个目标使用邻近踏面观察；实际角色均走过对应接缝。约7分钟是本原型3.2m/s参数下的输入驱动行走时间，不包含停留、迷路或真人操作，不能替代上山节奏体验结论

## 运行与画面

| 实际检查 | 结果 | 记录 |
| --- | --- | --- |
| Viewer feature库测试／Viewer测试 | PASS，44＋3项 | `rust-tests.log` |
| 格式、Clippy −D warnings、双binary构建 | PASS | `clippy.log`、`build.log`，机位补充另见 `viewer-view-test.log`、`viewer-view-build.log` |
| 小店到登高首段，35秒／1050帧 | PASS，10项；原生及工具退出0、视频生成 | `ascent-entry-after/` |
| 既有步行、墙体阻挡、离墙与恢复，650帧 | PASS，16项；视频生成 | `walk-regression/` |
| 上街固定机位 | PASS，4项，但原视角不足以展示桥口 | `bridge-junction/` |
| 补充真实登高入口回看桥口，60帧 | PASS，4项，开口已实际观察 | `bridge-mouth/` |
| 文档、Skills、双站构建与地图门禁 | PASS，玩家95页／开发158页 | `docs-check.log`、`docs-build.log` |
| 任务生成与只读检查 | PASS，23张任务卡 | `final-checks.log` |

GPU为RX6650XT／RADV Mesa26.2.3／Vulkan。首段修后沿同一输入从home走到 `(174.65173,40.62853,-310.98264)`，停止后位置不变，grounded且零恢复；数值阈值与原录制一致，两条断言名称改为首段爬阶和高度提升，避免暗示已走完第二组台阶

本轮为Codex视觉自查：实际查看修前449、修后449／959及连续799—801帧，检查坡面推进、台阶支撑、代理和镜头；查看墙边389帧确认相机近墙时隐藏代理、没有看入外壳；原上街机位59帧未正面展示接入口，因此增加 `eye-upper-bridge` 后查看新59帧，确认桥侧栏杆在交会处结束、登高方向路面敞开。完整视频已生成但未完整播放，不把抽帧自查称为真人试玩或盲测

## 检查命令

从仓库根执行，兼容fish；原始结果分别保存在对应日志，录制输出使用新目录

```fish
cargo test --manifest-path game/Cargo.toml --locked --lib walks_complete_short_ascent -- --nocapture
cargo fmt --manifest-path game/Cargo.toml --check
cargo test --manifest-path game/Cargo.toml --features viewer --locked
cargo clippy --manifest-path game/Cargo.toml --all-targets --features viewer --locked -- -D warnings
cargo build --manifest-path game/Cargo.toml --features viewer --locked --bin n-side --bin map_viewer
python3 tools/capture.py --script game/capture/walk-ascent-entry.json --binary game/target/debug/n-side --output output/capture/walk-ascent-entry-r2
python3 tools/capture.py --script game/capture/walk-preview.json --binary game/target/debug/n-side --output output/capture/walk-regression-r2
bun run check:docs
bun run docs:build
bun run tasks:sync
bun run tasks:check
```

## 证据范围

CPU整段结果验证真实mover与静态城市网格；GPU首段检查真实输入链和连续画面；Viewer路口图仅检查可见接合。Windows、物理手柄、作者手感、完整登高连续GPU画面、桥桥／桥平台接口、室内、电梯和角色动画均未在本轮验收。G1不据此放行

独立正确性及简化审查未发现阻断，结论 `Lean already. Ship.`；审查者未冒充运行检查或作者体验
