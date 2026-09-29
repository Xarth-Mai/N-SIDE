# 自然地面采样诊断 r2

基线 `9503dfa`；TASK-045 仍在进行中，本轮没有通过整片山地的材质品质验收

实际制作并运行了地面三相位混合，也验证了放大周期、纯色、各向同性 LOD 和 10m 棋盘格。三相位没有解决目标视角的长斜条，增加的 shader、纹理采样和预通道维护不值得保留，已撤回运行接入。正式地面继续使用 `StandardMaterial`、8×各向异性、2.1m Ground037 图对及 r1 主轴 UV

## 实际执行与看图

主 Agent 串行执行全部 Cargo/GPU 操作，记录位于 `output/capture/` 对应目录。`diagnostic-runs.json` 保留逐次命令、二进制 SHA-256、退出码与帧数，`diagnostic-configs.json` 保留隔离配置 hash 与地面参数；每组均为真实 `eye-descent-cut`、1280×720、30fps、90 帧，保留既有镜头转动和状态断言

| 运行目录 | 唯一针对地面的诊断变量 | 机器结果 | 实际看图结论 |
| --- | --- | --- | --- |
| `task045-cut-before-r2` | 原配置 | PASS / 90 帧 / exit 0 | 29、44 帧左坡有明显长细条 |
| `task045-cut-scale2-r2` | 周期由 2.1m 改成 4.2m | PASS / 90 帧 / exit 0 | 29、44 帧细节和条带变大，重复未消除，未采用 |
| `task045-ground-phase-r2` | 同源图三相位颜色/法线混合 | PASS / 90 帧 / exit 0 | 44 帧部分平地重复弱些，左坡长斜条仍明显，未采用 |
| `task045-ground-solid-r2` | 移除地面颜色图与法线图 | PASS / 90 帧 / exit 0 | 44 帧长细条消失，说明它来自纹理表现而非纯几何着色，诊断后恢复纹理 |
| `task045-ground-isotropic-r2` | 由最大 UV 导数计算显式 mip LOD | PASS / 90 帧 / exit 0 | 44 帧颗粒变糊，宽斜带仍在；模糊不是解决结果，未采用 |
| `task045-ground-checker-r2` | 同一 UV 的 10m 棋盘格 | PASS / 90 帧 / exit 0 | 44 帧左坡是连续、受透视压缩的长菱形格；远处平地格规整，未见同面上 UV 突发收缩 |
| `task045-ground-ssao-r2` | 三相位试验加 `--aa taa-ssao` | PASS / 90 帧 / exit 0 | 用于验证法线预通道的真实运行，不据此宣称视觉通过 |

上述看图是实际截图自查，未冒充作者验收或陌生玩家测试。基线与 2×周期使用同一个二进制；solid/isotropic/checker/SSAO 使用同一个稍后构建，确切 SHA 已记录，不把整个并行工作区称为只有一个变量的性能对照

## 根因证据与范围

r1 的真实地形测试已经证明 16,130 个地形三角的投影面积比至少为 `0.5868630943`，排除了近退化顶视 UV；关闭法线的上一轮实测也未消除条纹

本轮 `ray-probe.ts` 使用现有 `tools/district-map.ts::buildGround`、真实 frame44 相机及 55° FOV，对源地形做射线测量。结果保留在 `ray-probe.json`，包含源地图 SHA-256。它使用 Delaunator 的未裁切源面，可能与 Bevy Spade 的共面三角划分不同，因此作为交叉核对，不能代替 GPU mesh 检查

| 图像像素 | 距离 | `abs(N·V)` | 相对表面的观察角 | 主轴 chart |
| --- | --- | --- | --- | --- |
| (80,160) | 52.74m | 0.0554 | 约 3.2° | XY |
| (120,260) | 37.58m | 0.1329 | 约 7.6° | XY |
| (220,350) | 37.92m | 0.1741 | 约 10.0° | XY |

第一点的相邻屏幕像素沿投影 U 变化约 0.68m、V 仅约 0.04m，正是强烈掠视条件。GPU 棋盘格与此一致：正常世界尺度的纹理也会受这种透视压缩。三相位平移能打散周期，不能改变真实表面的观察角，也不能代替坡面分区、植被体积与坡形

原 Ground037 颜色图已再次直接查看，是苔土、细枝与斑点，并非整幅平行条纹图。结论限于这条实际捕获路径；本轮未证明所有位置都没有图案重复，也未据掠视诊断批准整座山的画质

## 被拒绝的实验与复现

- `rejected-phase.patch` 保存已撤出的完整地面材质扩展、WGSL 与真实场景接入，供复查本次实验；不属于游戏的活动资产或正式渲染路径
- `diagnostic-isotropic.patch`、`diagnostic-checker.patch` 是相对于实验 WGSL 的单项诊断修改；前者按最大 UV 导数选 mip，后者输出 10m 格与平直法线
- 三相位方案对照本机 Bevy 0.19.1 的 `extended_material.rs` 示例、`pbr_fragment.wgsl`、`pbr_prepass.wgsl`、`prepass/mod.rs` 与 `render/mesh.rs`；颜色与三通道 OpenGL 法线共用相位、权重和原 UV 导数，前向及预通道共用函数
- 成本由一组颜色/法线读取增至最多三组，没有增加纹理内存；实际收益不足后完整移除，不留未启用的框架或配置开关

以下只读测量可直接由 fish 执行；输入为本轮保留的 `output/capture/task045-ground-phase-r2/state.json` 与当前源地图，输出会标出当前源地图 hash

```fish
bun todo/evidence/TASK-045/terrain-surface-r2/ray-probe.ts
```

## 下一动作与清理

后续把材质任务拆到真实坡面类型：林缘、苔土缓坡、裸露岩土与人工边坡，并在常用街景和登高路径检查分区、体积及重复。任何修改仍保留真实碰撞与稳定 ID；先比较正常街景与这条极斜视路径，再决定是否需要专用采样

本子任务不改任务状态、不运行 Cargo/GPU、不提交。实验接入已撤除；最终统一构建由主 Agent 负责。已检查没有进程引用隔离项目后，移除 `output/assets/terrain-r2/` 全部配置与链接，源资产保留；主 Agent 在实际看图与记录后统一清理其 capture PNG、关键帧和视频，保留日志及状态 JSON
