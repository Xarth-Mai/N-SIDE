# 小店室外材质样板第 1 轮

## 输入与范围

基线为 `f7357e68220756da3f855cbd12427a554008cf98` 的独立 Linux 预览包，本轮真实运行输入与二进制哈希见 [capture-runs.json](capture-runs.json)。中途作者明确本线先做人物、地图与画面，新增室内窗洞、家具与点光照已撤回；室内相关日志只保留试验历史，不作为当前交付

## 已交付

- 月台杂货招牌使用已有 OFL 中文字体重导出，SVG 保留可编辑文字；其余招牌未因字体处理变化
- 五处商店雨棚增加前低后高的坡度、边沿和墙托，招牌增加背箱，橱窗下沿增加木挂板，保留门口与公共通路
- 盒形构件改用按面实际米数的 UV，再由材质 `tile_meters` 控制重复，避免各类尺寸统一拉伸一整张纹理
- 补入 ambientCG CC0 的 Plaster001 与 WoodSiding009 颜色和 OpenGL 法线，来源、校验、尺寸与临时米制覆盖见 [material-intake.json](material-intake.json)及唯一资产包清单
- capture 增加真实墙钟间隔和资源数量，区分模拟步长、截图吞吐和原生运行性能

## 自动检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Rust viewer feature 库测试 | PASS，97 通过、1 个忽略 | [日志](exterior-rust-tests.log) |
| 游戏与 Viewer 编译 | PASS | [日志](exterior-build.log) |
| 招牌源导出一致性 | PASS，10 项 | [日志](sign-check.log) |
| 环境源导出一致性 | PASS，24 个运行文件 | [日志](material-check.log) |
| 静态环境 GLB 工具测试 | PASS | [日志](material-tests.log) |
| TypeScript/Vue 检查 | PASS | [日志](types.log) |
| 正式游戏 450 帧室外输入路线 | PASS，6 项状态检查 | [运行清单](capture-runs.json) |

首条橱窗路线因北侧栏杆阻挡而未到达预期位置，断言正确 FAIL；改为南侧公共窗口路线后复验 PASS。未通过瞬移或直接写位置绕过碰撞，复现输入保存在对应 run.json

## 实际画面自查

实际查看最终 59、222、330 关键帧，以及 314、319、325、330、340、345 连续转向抽样，对照基线 222 帧。中文店招可辨，墙面从颗粒混凝土调整为较细灰泥；雨棚前缘、支架和木窗裙形成可见层次，查看范围内未发现窗裙遮门、雨棚穿帮或转向时丢失材质

本轮仍明显是制作中的预览：角色为胶囊，树木和天空简化，大面积立面缺少局部细节，墙和招牌的色调偏浅；没有达到 README 概念图中的人物、城市密度、气氛与灯光品质。抽帧自查不代替连续人工操作或作者审美验收

## 成本与局限

运行新增纹理总大小 16,777,728 bytes，环境资产包共 44,879,370 bytes。最终 ready 状态为 4,982 实体、3,337 mesh、37 image、47 material；这里只统计主世界资源数量，不等于可见绘制次数或 GPU 内存

RX 6650 XT / RADV 的 debug 离屏录制中，世界截图间隔均值 25.57 ms、p95 29.50 ms，包含异步回读和 PNG 工作。观测到 loading→ready 为 0.044 s，不包含观测前的场景准备。旧包没有同一计时仪器，文件写入间隔也受调度与并行活动影响，因此本轮不据此宣布性能提升或稳定原生帧率；两次并行的室内试验尤其不用于成本比较

Windows、手柄实机、原生连续操作、GPU 帧时与作者视觉反馈为 NOT RUN。后续重点是完整城市立面、人物与天空植被，并继续在相同街景检验，不以此轮局部通过完成整项 TASK-045

## 复现

在仓库根执行，以下适用于 fish

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --binary game/target/debug/n-side --script game/capture/walk-shop-exterior.json --output output/capture/shop-exterior-next
```

运行日志与 state.json 留在对应 output 目录，关键参数及检查摘录已进入任务证据。临时画面查看后清理，清理数量见 [cleanup.json](cleanup.json)
