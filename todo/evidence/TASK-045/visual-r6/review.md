# 坡面、近景植物、窗格与人物动作 r6

基线 `e6b398b5fac899ac4639dc90fae08c4274da7aaf`，持续推进完整游戏目标，当前先做地图、人物与画面；TASK-045、TASK-047、TASK-049 保持 active，没有增加作者审美通过或游戏里程碑完成数

## 交付与真实范围

- 自然地形以真实三角坡度分为苔土和超过 45° 的岩面，原位置、法线与碰撞保留；新增 CC0 Rock043L 颜色和法线图，来源与归一尺度进入原环境清单，详见[坡面记录](../terrain-surface-r3/README.md)
- 原创 6.5m 松树和 0.8m 灌木替换同一 `tree_pine`／`shrub` 角色，提供真实分枝、几何叶片、可编辑 Blender 主文件和检查入口；分别 6,510／5,824 三角，详见[松树](../../TASK-049/pine-r1/review.md)与[灌木](../../TASK-049/shrub-r1/review.md)
- 十栋住宅样板上层的 381 扇窗改为分离框、双片内缩窗面和窗台，玻璃共享十个建筑批次；窗壳 32,004 三角是未扣除原配件移除的计数，不当作净增量，详见[窗口记录](../../TASK-049/windows-r1/README.md)
- 曜 r5 完成鼻眼颌与发帽贴合修订；随后修正实际 GLB 走跑支撑脚反向、恒速段和半帧插值。最终资产 `86bc6a8ce380e40204f1baaddd19008d245ffb09248fceef7355121428450b9f` 保留 32 骨、绑定和灰阶图集；形体与动作分别见[模型 r5](../../TASK-047/model-r5/review.md)及[动作 r1](../../TASK-047/animation-r1/README.md)

源动作按当前 3.2／5.6m/s 平地速度编排，GLB 在 120 FPS 重采样的支撑段漂移峰值约 0.283／0.642mm，说明本轮规定速度下的源动画改善，不能推广为坡地、转向与碰撞停步零脚滑。Walk 是当前较快移动速度的实验步态；完整跳跃动作与动画过渡仍未制作

## 实际检查

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --manifest-path game/Cargo.toml --check` | PASS |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins` | PASS |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib` | 126 PASS，1 worker ignored，由父测试调用 |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer` | 3 PASS |
| `cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings` | PASS |
| `bun tools/export-environment.ts --check` | 31 个运行文件、69,565,322 bytes PASS |
| `python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py` | 1 PASS |
| `python3 source-assets/environment-kit/vegetation/pine-check.py` / `shrub-check.py` | 实际 GLB 检查 PASS；主文件重导出和坏 alpha 拒绝证据分别留在子项 |
| `bun run check:docs` | PASS，含 Skills 检查 |
| `bun run docs:build` | 玩家与开发两种构建 PASS |

人物 DCC 211 整帧检查、正式 GLB、静态属性保持及重导出字节一致通过，实际命令在模型与动作记录中；它们与 GPU 分开验收

首次全图地形检查失败于裁切细三角的 f32 叉积相消，修为在相同存储顶点上使用 f64 分类，未放宽 45° 阈值。复验曾被信号中断，随后链接缺少本包增量单态化符号；定点删除该测试的增量目录与目标文件后全测通过，清理清单见 `checks/cache-cleanup.json`，未清全部依赖。Python 首次模块路径调用不适用于本仓库，改用现有 unittest discover 后通过；原失败日志保留

## 真实运行与看图

共 9 次 GPU capture、1,620 个模拟帧全部 PASS，8 次 Viewer 各 6 项检查，正式角色路线 540 帧／26 项检查；`summary.json` 与各子目录保存原生退出码、真实命令、脚本、binary hash、运行日志、状态摘要和全部视觉文件 hash。逐帧状态保留在原 output 目录，截图与视频查看后按项目规则清理

旧切坡、窗格与院落对照使用冻结的基线资源和旧二进制；新 `planting-detail.json` 对照使用同一当前二进制、镜头、材质与地图，只将两种植物绑定回原模型，具体差异与文件 hash 见 `plant-comparison-assets.json`，不把这个受控对照冒充完整旧版本

Root 实际查看坡面前后44，窗格前后59及新60／62／64，植物旧59及新29／59／89／149和60／62／64，院落前后59，人物141／142／145／190／195／209；艺术审查补看植物60–64、人物Walk140–144、Run190–194、340／359／499，坡面作者补看40–44。70–74属于已停止输入后的稳定帧，未用于宣称移动验收。所有看图为 self-audit，暂停菜单遮住人物时以真实状态检查证明动画暂停，不以截图推定

真实替换可见：针叶树不再是层叠锥体、灌木有分枝空隙，住宅框厚和玻璃层次增强，陡坡绿色细长条减少；人物实际切换 Idle／Walk／Run、跳跃及暂停恢复正常。所看移动片段未发现整株消失、窗格跳位或明显网格破裂

现有主世界 ready 资源由 5,066 entities／43 images／51 materials／3,365 meshes 变为 5,068／46／53／3,367；同代码仅换植物的受控对照增加一张树皮 image。松树96实例与灌木15实例合计几何增量约691,176三角，没有逐窗或逐叶新增实体。capture 采样间隔包含读回与 PNG 工作，不代表正常帧率；全城 LOD、显存、原生帧时间预算仍需独立测量

## 留下的品质问题与下一动作

山体仍有大直切楔面、材质三角硬边、宽阔空坡和稀疏林缘；下一轮应结合真实坡形、山路和林缘体积解决，不能继续只调纹理。森林候选只做了位置与视线探测，见 `forest-candidates.md`，本批没有据此新增树群或声称场景已完成

新松树针叶偏细碎，未覆盖全距离与完整视频闪烁评审；其他黄色锥树和石块仍为灰盒。住宅窗仍均质重复，没有真实室内或局部反射。曜后发仍有叶片贴帽壳感，Run 上身与手姿偏僵，角色灰阶保持 needs_revision，正式配色与完整材质尚未通过。下一轮继续这些实际品质缺口，再扩展玲与安可，当前不计为 README 目标品质或 G2 放行

Windows、物理手柄、夜间多光源、作者审美与原生性能预算本轮 NOT RUN。最终独立复杂度审查 `Lean already. Ship.`；用户原有 AGENTS.md 与 docs/dev/validation/runtime.md 改动保留，不纳入提交，不推送远端

## 清理

各模型工作线已记录自有临时图、接触表和备份清理；root 在完成实际查看、hash 与文字结论后清理本轮 capture PNG、视频和两个隔离资源目录，共清理 1,694 个视觉文件、1,896,240,777 字节，两个隔离资源目录另释放 169,720,157 字节；九个 capture 目录仅保留 2,799,356 字节日志、脚本、原状态与运行信息，详情见 `cleanup.json`。正式资产、参考资料、源码、命令、日志、状态 JSON 和校验信息保留
