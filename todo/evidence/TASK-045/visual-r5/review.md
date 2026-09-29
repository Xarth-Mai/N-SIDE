# 地表诊断、人物明暗与坡地构件 r5

基线 `9503dfa`，本轮目标沿用完整游戏路线图，优先地图、画面与人物制作；TASK-045、TASK-047、TASK-049 均保持 active，未产生新的作者审美通过或 G2 放行

## 实际交付

- CHR-001 r4 完成连续鼻面、浅鼓眼白与上睑覆盖、侧后发组修订，34,010 三角、单材质、32 骨，原灰阶图集与 Idle／Walk／Run 全部采样保持；源工程重导出与 GLB 检查通过，详见[模型记录](../../TASK-047/model-r4/review.md)
- 正式 `--character-preview` 接入主方向光分段实验，继承真实蒙皮、StandardMaterial 光照／阴影／雾；同一最终模型、参数与二进制仅开关 N 重映射的两次 240 帧对照显示背部衣服、帽沿与裤腿明暗分区改变，详见[材质自查](../../TASK-047/shading-r1/review.md)
- 两处有真实露高的台地增加错缝混凝土面、墙冠与泄水口，共 4 批、121 盒、1,452 碰撞三角；小店南边退开道路转角，实际净空测试通过，详见[挡土记录](../../TASK-049/retaining-r1/README.md)
- 三处既有花园／林缘增加 4 组、24 个植被实例，新增 2,768 三角；保留原实例与地形、全部派生植被为 176/200，详见[院落记录](../../TASK-049/courtyards-r1/review.md)
- capture 增加真实 tracing ERROR 门禁及 Viewer `--aa` 转发；修复完全相同四元数被 acos(dot) 浮点误差误报成旋转的问题，保留原阈值，新增实际数值回归

## 失败、诊断与撤回

初选的两处台阶平台边缘实际仅有约 0.1–0.18m 外露，未生成预期排水构件，窄测失败；核对 Ground 后改用小店南边与东侧后场，实际最大露高为 0.779m／0.958m。最初挡土录制的 130–149 帧四元数完全相同，旧角度测量却返回 0.000690534rad；修复观测算法后同一路线通过，未把误报归因于构件或放宽阈值

隔离坏 WGSL 的真实运行完成 90 帧、原生退出 0、状态 5 项检查全部通过，却有 pipeline_cache 编译 ERROR；wrapper 正确返回 1，保留该失败。资产 loaded 与 shaded_meshes 只证明接入，不能证明 GPU 已编译成功

地面采用相同切坡路线做原版、2倍周期、三相位混合、TAA+SSAO、去纹理、各向同性过滤与 10m 棋盘格共 7 次实测。相位扩展的收益不足，已完整撤出运行代码，正式仍使用原 StandardMaterial、8×各向异性、2.1m 图对。棋盘格与源地形射线显示左坡局部仅约 3.2° 掠视；不能继续把所有长条都归为 UV 退化，也不能将模糊当作美术改善，详见[完整取舍与可复现实验](../terrain-surface-r2/README.md)

## 命令与结果

`summary.json` 记录 16 次实际录制：14 次 wrapper PASS、2 次 FAIL，共 2,970 个模拟帧；失败分别是故意坏 shader 与已修复的静止角度误报。各子目录保留真实命令、脚本、二进制 hash、runtime.log、状态摘要、原始 state.json hash 与关键帧 hash；原始逐帧状态继续留在各 output 目录。次数与帧数不是游戏完成比例

| 实际命令 | 结果与范围 |
| --- | --- |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins` | PASS |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer` | 124 个库测试与 3 个 Viewer 测试 PASS；1 个 worker 忽略，由父测试调用 |
| `cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings` | 首次常量 chunks_exact lint 失败，按本机版本改 as_chunks 后 PASS；挡土窄测再运行 1 PASS |
| `python3 -m unittest discover -s tools/tests -p test_capture.py` | 4 PASS；实际 GPU 失败另有独立证据 |
| `bun run check:docs` | PASS，含 Skills 来源与引用检查 |
| `bun run tasks:sync`、`bun run tasks:check` | 生成并只读检查当前看板，未增加任务完成数 |

最终正常路径：挡土 150 帧／7 检查、共享花园 150 帧／6 检查、山脚院落 540 帧／9 检查、角色走跑 540 帧／26 检查均 PASS。角色环绕对照各 240 帧／9 检查 PASS。全部直接使用正式世界、模型、真实输入与控制器；Viewer 的自由飞行不作为人物可通行证据

## 实际看图与剩余工作

Root 查看模型四机位、七组地表的44帧、角色正／侧／背及三分之四，最终走跑141–142、159、209、314，挡土59／89、花园59／61和山脚89。艺术审查、模型与相应制作 Agent 另查看连续转头、横移和 Walk／Run 帧，其帧号、hash 和结论分别记录在子项；均为 self-audit，不是隔离盲测或作者反馈

人物分段明暗比球面渐变更清楚，所看动作没有新增袖腰穿插，但脸部仍通用、后发偏大片、衣料和完整动画待制作，配色未批准。院落的树—灌—草层次可辨且接地，现有锥树和块石仍是低模表现；山脚中庭的草石根部被建筑遮挡，未作视觉通过。挡土墙冠与格栅可辨，墙材质和竖缝层次仍弱，东侧后场的单独近景尚未覆盖

城市仍有宽阔空坡、陡直切面、同高长挡墙、重复外壳及黑平窗，尚未达到 README 预览质量。下一轮优先处理真实坡面材质分区、坡形与林缘体积，并替换近景低模植被；人物继续沿海报的日漫脸部与头发语言修订，再扩展正式贴图和更多角色。Windows、物理手柄、夜间多光源、作者审美及帧时间预算验收本轮 NOT RUN；capture 墙钟与截帧间隔不作为正常游戏帧率

独立复杂度审查结论为 `Lean already. Ship.`。工作区原有 AGENTS.md 与 docs/dev/validation/runtime.md 改动保留，不纳入本轮提交；未推送远端

暂存检查识别到原始日志末尾空行、tracing 空消息的行尾空格与 patch 的空白上下文前缀。为保留已记录 hash 的原始字节，`.gitattributes` 仅对证据 `.log` 的这两类空白及证据 `.patch` 的行尾前缀关闭空白提示，源码和文档检查保持

## 产物清理

实际查看、hash 与结论记录完成后，已清理本轮 3,111 个原始帧、关键帧与视频，共 3,287,412,515 字节，并删除两处隔离 shader 资源目录；16 个 capture 目录仅保留日志、原始状态、脚本及运行信息，共 5,561,354 字节。源码、正式模型／贴图与实验 patch 保留，具体数量见 [cleanup.json](cleanup.json)。模型 Agent 自有 DCC 清理另见 [model-r4/cleanup.json](../../TASK-047/model-r4/cleanup.json)
