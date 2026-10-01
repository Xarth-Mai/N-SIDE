# Blender 接入 R5：整合验收

基线 `5cd3037436e2fdd569569daa1a853f660d11d312` 加本轮工作区，完成镜厅北侧后勤前场接入、三个建筑共享法线修复及曜 r12 袖筒回归。6次成功原生运行共1,230帧、59项原生／60项包装检查 PASS；另保留1次启动前参数失败，不计入原生帧数。画面自查与作者品质验收分别记录，本报告不改变任务状态

## 本批交付与边界

- 后勤前场沿用 `cinema_loading → cinema_service_entry`，增加标高25m、198m²的 `cinema-service-court`，复用现有空调装配与路灯；[源布局及检查](../cinema-service-court-r1/README.md)记录3.6m门前净空与8×8.5m卸货净空。没有新增室内、车辆模拟或公共通行权限；实际 GLB、道路余量、脚点支承和闭门接近由 [CPU碰撞](collision.log)及[设施窄测](service-props.log)覆盖
- [共享法线源修](../normal-scale-shared-r1/README.md)把已使用的7种法线／强度组合集中到 AST-003 的单一烘焙入口，保留原授权 JPG。V-15烘入0.35、V-A08烘入0.28、V-55复用0.25，Blender Strength／glTF scale统一为1；V-35运行GLB保持原字节，独立屋檐未改。[导出保持](../normal-scale-shared-r1/runtime-preservation.json)记录几何、UV、节点、颜色／海报及非目标材质不变
- [曜 r12](../../TASK-047/sleeve-r12/review.md)仅调整袖筒中间五圈160个控制顶点，保留衣身、肩缝、袖口、拓扑、UV、权重、32骨与四动作；[21项正式提升契约](../../TASK-047/sleeve-r12/promoted-contract.json)通过，实际新GLB运行另见下表
- 环境反射候选B未获得足够的玻璃层次收益，已撤回；最终 `game/src/world/visual.rs` 与基线无差异。A/B各120帧的机器PASS、对照数据和候选patch保存在 [TASK-045独立归档](../../TASK-045/env-reflection-r1/captures.json)与[comparison.json](../../TASK-045/env-reflection-r1/comparison.json)，不计入本批1,230帧；实际负实验观察见 [画面自查](visual-review.md)。捕获通量包含PNG与读回，不作为游戏FPS结论

## 输入版本与真实运行

[runtime-inputs.json](runtime-inputs.json)冻结最终Viewer、地图和运行资产：地图为 `eb00044d2b2217594524d6ecd869dde42ff31463ddf6369038f32f3ae959b633`，Viewer二进制为 `e60fa2b82a4b20c690dbfb27d5df353e80ba01372341390c66cea8be5fb9c76a`。角色retry使用较早build-A的 `284df06444f9c362ad9bb208dfd58963ed27316e98afa8a15aa3a8ed2fef0d0d`，按实际run保存，没有替换成稍后构建的正式入口hash

最终V-15为 `b3617d3c57d8e466ce4e6ca3e5fe4dd3f2b55d596f39c43a3cbdd5405d313bdd`，V-A08为 `12e0fdef03f7a2ac2fcd22f2661302b0c29bc1932a0ac83d6ea7d8a02af1e09a`，V-55为 `597bc0b96477295efd6a743e2ebf3884ad8b94305e9ba2e93d232e03b258cff4`；V-35仍为 `1879fad3909afeaf32e2aa75c58e553252f0fec8168fa3663855a217259d441a`，曜r12为 `3fb220c6f073f003399acf25e0083aafb2f1bb18027064f49183bfbfe61370db`

| 实际尝试 | 帧数 | 原生／包装检查 | 入口与范围 |
| --- | --- | --- | --- |
| yao-motion | 0 | 未启动 | `--aa requires a district Viewer script`，wrapper校验在调用原生进程前失败；没有state、runtime.log或原生退出码 |
| yao-motion-retry | 540 | 27／28 PASS | 正式CHR-001，Idle／Walk／Run／Jump及暂停恢复；包装器额外核对角色ID |
| inspect-cinema-service-court | 120 | 6／6 PASS | Viewer真实左右输入，后场和服务来路总览 |
| eye-cinema-service-aircon | 180 | 8／8 PASS | Viewer横移及鼠标下看0.4rad再复位，覆盖设备与地面装配 |
| cinema-normal | 150 | 6／6 PASS | 修复后V-15，接近与横移 |
| va08-normal | 120 | 6／6 PASS | 修复后V-A08，本轮真实横移脚本，不使用旧60帧固定画面作为移动证明 |
| v55-normal | 120 | 6／6 PASS | 修复后V-55，真实横移与停止 |

六组成功录制均1280×720、30fps；5组Viewer共690帧，角色540帧。命令见 [脚本说明](scripts/README.md)，确切参数及原结果见 [captures.json](captures.json)。Viewer没有人物碰撞，本轮未录制新的服务来路人物长路线；角色脚本含3次实际R恢复输入，不证明无恢复往返。下一轮 `roof-rest-r1/baseline` 60帧不属于本批

## 检查与实际看图

| 已执行检查 | 结果 |
| --- | --- |
| 最终Rust lib完整测试 | [142 PASS、0 FAIL、1 ignored](lib-final.log)，覆盖真实场景、后场设施、碰撞及既有路线 |
| Viewer与最终构建 | [3项PASS](viewer-tests.log)，[最终开发构建完成](build-final.log) |
| 后场定向检查 | [模型支承／闭门碰撞PASS](collision.log)、[真实GLB脚点／路带／作业净空PASS](service-props.log)；这些窄测已包含于最终完整测试，不累加为额外功能数量 |
| 共享法线源修 | [源复读](../normal-scale-shared-r1/source-reopen.json)、[逐项导出保持](../normal-scale-shared-r1/runtime-preservation.json)、[独立审查](../normal-scale-shared-r1/independent-review.md) PASS；首次退出143与默认零引用Material消失触发的初次复读失败仍保留在该工作记录 |
| 曜源与运行数据 | [正式提升契约](../../TASK-047/sleeve-r12/promoted-contract.json) PASS；运行新模型540帧见上表 |
| 最终docs／Skills | [602份Markdown、129个ID、31 Skills、25 imports PASS](docs-final.log) |
| 最终地图／Wiki | [62项地图检查、player 128页／679文件、dev 223页／1,046文件 PASS](wiki-final.log)，dev大于500kB的chunk提示仍为非阻断警告 |
| 任务卡检查 | 主任务已执行 `bun run tasks:check`，47张任务卡通过；未由本归档子任务重复运行或修改卡片 |

[visual-review.md](visual-review.md)已记录全部实际看图结果与11个代表帧hash：环境A/B无足够收益，曜Run190–192、Jump68–70及落地79未见抽样范围内新增穿洞，79已是Idle而非Jump。后场总览与空调下看可见铺地、支架、管端与地面接合，所看横移未见新增悬浮或重叠；V-15／V-A08／V-55的关键帧及连续移动未见需要撤回法线修复的黑块、接缝闪烁或整片明暗跳变。该结论只支持本批接入与法线一致性，后场仍偏空、玻璃层次不足，未提升为建筑品质通过

单级法线PNG仍不能保证所有距离下过滤稳定；建筑暗窗、场外密度、角色衣料与脸发仍有品质工作。本轮不代签作者造型、概念图完成度或实体手柄验收。视频均明确 `NOT RUN: --no-video`，原run／state中的 `visual_review: NOT RUN` 保留，人工自查单独成文

## 归档与收尾

归档26个原文件，原始2,411,109字节、存储1,004,118字节；仅角色retry的state超过500,000字节，使用gzip、mtime0、空filename无损压缩。索引保留全部原始／解压hash与存储hash，逐份读回与原文件一致；启动前失败的state与runtime.log标为 `NOT GENERATED`，没有伪造空文件或退出码

实际看图与最终docs／Wiki检查已完成。主任务确认所有捕获进程退出后统一清理本批与独立A/B媒体，[cleanup.json](cleanup.json)逐文件保留hash：删除1,529文件、1,735,603,296字节，剩余媒体0。正式源资产、共享PNG、明确保留的源修前文件基线、日志和可复现输入继续保留；随后由root处理任务卡和本批提交

## 提交前最小性复审

实际复查本轮场景接入、GLB碰撞、道路反例、单一法线烘焙及角色局部修型；沿用现有入口与资产清单，没有新增录制器、反射框架或通用资产系统。共享法线与后场路径的独立审查已收口，最终差异复审：`Lean already. Ship.`
