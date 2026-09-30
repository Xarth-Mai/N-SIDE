# 第 9 轮地图与人物集成

本轮从 `13397c5626c8952511ceda52d3c4a8877b901acc` 接续，按用户要求完成当前批次后提交、推送并停止。TASK-045／047／049 保持 active，G2 和最终美术未放行；未加入新室内、战斗或剧情制作。既有 `AGENTS.md`、`docs/dev/validation/runtime.md` 未提交改动保留原状

## 本批交付

- [林缘浅沟](../terrain-shape-r4/review.md)：局部自然点加密、浅沟两肩及道路支撑，9752 点；单一 450m 峰顶、道路节点、建筑、对象 ID 和所有非 terrain 数据保留。原生 Spade 比较 25770 个道路采样，最大新增高程误差约 7.36e-8m，旧沟外源采样不变
- [草与石块](../../TASK-049/ground-props-r1/review.md)：原创可编辑 Blender 主文件、曲面草叶、不规则岩石、贴图与既有资产登记／导出；真实林下布点 46 件，未改变布点算法，未完成连续森林
- [树庭挡墙](../../TASK-049/retaining-r1/r9.md)：按真实生成墙顶制作墙冠、分缝与端头，13 个盒体、156 三角、2 个材质批次；真实道路余量与公共平台保留
- [曜 r7](../../TASK-047/model-r7/review.md)收紧颈口和肩袖过渡、前襟融入衣片、胸口图形贴合并错开发束；[玲 r2](../../TASK-047/ling-model-r2/review.md)改进下颌、眉眼和主次发束。均保留 32 骨、既有动作、尺度与灰阶贴图，仍为 needs_revision
- 更新既有资产说明和当前任务记录，清除路线图中已完成存档工作造成的过时描述；没有另外创建一套进度源

## 实际执行

Linux、RX 6650 XT RADV Vulkan；Blender 4.5.14 CPU 两线程；Bun 1.4.2、Rust 1.98.1、Bevy 0.19.1。最终输入内容 hash 见 [inputs-final.json](inputs-final.json)，基线从固定提交提取，复现入口为 [prepare-baseline.py](prepare-baseline.py)

| 命令／检查 | 实际结果 |
| --- | --- |
| `cargo fmt --manifest-path game/Cargo.toml -- --check` | PASS，[最终格式日志](fmt-final.log)；初次格式差异已修正 |
| `cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins` | PASS，[构建日志](build-final.log) |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --lib` | PASS，133 passed、0 failed、1 ignored，[最终日志](lib-test-final.log)；ignored 是由另一测试在两个隔离进程调用的设置助手 |
| `cargo test --manifest-path game/Cargo.toml --locked --features viewer --bin map_viewer` | PASS，3 项，[日志](viewer-test.log) |
| `cargo clippy --manifest-path game/Cargo.toml --locked --features viewer --all-targets -- -D warnings` | PASS，[日志](clippy.log) |
| `bun run check:map`／地形幂等烘焙 | PASS，58 项；完整证据见地形记录 |
| `bun tools/export-environment.ts --check` | PASS，33 文件、73,332,914 bytes，[日志](environment-check-final.log)；可编辑主文件再次导出逐字节一致 |
| `python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py` | PASS，既有米制导出测试；新增草石另由实际 GLB 契约与原生场景检查覆盖 |
| `bun run check:types` | PASS，[日志](types.log) |
| `bun run check:docs` | PASS，含全部 31 Skills 与 25 导入记录，[最终日志](docs-check-final.log) |
| `bun run docs:build` | PASS，地图检查及玩家／开发独立站点；构建器已有 bundle size 提示，[日志](docs-build.log) |

首次草石集成在真实启动前返回 1，加载器拒绝 `KHR_materials_specular`；修复主文件高光参数并重新导出，保留 [失败运行](after-understory/run.json)，没有绕过材质校验。初版贴地断言把石块近底侧面当成底部，修正真实底面筛选后保持原容差通过；首版草图显黑，改为明确 sRGB 色值后重新实机检查。完整失败与修复日志保留，未把失败记录改成成功

## 真实运行与画面自查

所有运行使用真实 map_viewer 或 n-side、1280×720、30 FPS 固定模拟步长、seed 0；Viewer 使用 TAA+SSAO，角色入口沿用 MSAA4。脚本驱动真实输入，不替换角色结果或场景状态。[汇总](summary.json)记录每次运行、帧数、原生与包装器断言数，目录内保存命令、二进制 hash、运行日志、脚本、状态摘要和所有临时图像 hash。视频均由真实 PNG 序列编码

| 最终路径 | 时长 | 断言 |
| --- | --- | --- |
| `after-understory-final` 林下近景 | 5s | 6 原生 PASS |
| `after-forest` 山体总览 | 3s | 6 原生 PASS |
| `after-planting` 既有公共种植区 | 5s | 6 原生 PASS |
| `after-retaining` 树庭横移与转向 | 5s | 7 原生 PASS；在草色最后修订前录制，墙体和全部几何与最终一致 |
| `after-yao`、`after-ling` | 各 8s | 各 9 原生 + 1 角色 ID 校验 PASS |
| `after-yao-motion`、`after-ling-motion` | 各 18s | 各 26 原生 + 1 角色 ID 校验 PASS |

角色路径实际覆盖加载、Idle／Walk／Run 时间推进、疾跑、跳跃、暂停、焦点／输入恢复及脚本要求的返回／复位；末尾恢复 Idle，无资源加载错误。脚本注入手柄状态不等于实体手柄实测。全短登高往返由本轮原生人物控制器测试覆盖；本批没有再录制整条登高路线

Root 实际查看基线林下／森林／挡墙与两角色环绕关键帧；最终查看林下 59–61、总览 44、种植区 59、挡墙 59–61、两角色环绕 204、两套动作 190–191 与 209–211。动作 209–211 正好跨停止输入，包含既有的立即回 Idle，不能拿这些帧声称过渡混合已经打磨。DCC 的实际查看记录分别由角色与草石证据承接

观察结论为 self-audit：草石轮廓与纹理比原粗模明确，挡墙增加了真实厚度和细分，角色前襟／脸型有局部改善，所看连续帧未见整体跳位或新增大块互穿。山体仍有大片空坡和折面，树群稀疏且较整齐；公共庭院仍有粗模树与重复住宅；人物发根、冠顶、长发分束及脸部个性仍不够成熟。普通第三人称距离下细部改善较小，未达到 README 概念图品质

材质替换后的 Viewer 就绪资源为 entities 5439、meshes 3383、images 49、materials 52；同机位基线为 5465／3382／46／53，资产合批与地形支持造成实例变化，不能解释为纯地形增量。林下 capture 帧间隔 P95 从 41.25ms 到 43.06ms，包含回读、PNG 工作和更新等待，也有同时进行的 CPU 检查；它是本轮录制吞吐记录，不是原生 FPS 或受控 GPU 性能对照

Windows 实机、实体手柄、作者审美、陌生玩家试玩、完整森林与正式彩色人物验收均 NOT RUN／未完成。固定步长与 seed 不代表跨 GPU 像素级确定性

## 复现与交接

以下为 fish 可执行命令，输出目录须未存在。其余机位只替换 script 与角色 ID；`run.json` 保存本轮完整实际参数

```fish
cargo build --manifest-path game/Cargo.toml --locked --features viewer --bins
python3 tools/capture.py --script game/capture/understory.json --output output/r9-local-understory --binary game/target/debug/map_viewer --aa taa-ssao
python3 tools/capture.py --script game/capture/tree-court-retaining.json --output output/r9-local-retaining --binary game/target/debug/map_viewer --aa taa-ssao
python3 tools/capture.py --script game/capture/walk-character.json --output output/r9-local-ling --binary game/target/debug/n-side --character-preview CHR-002
```

本批技术交付可提交；未给角色或 G2 代签作者验收。下次恢复时从真实连续林缘、城市空间密度与人物脸／发束品质继续，而非增加新系统；本轮按用户要求到提交推送为止

临时视觉产物及原生探针已在实际查看后清理；冻结源资产副本同步删除，正式资产和参考图保留。逐文件 hash、字节及剩余日志／状态占用见 [cleanup.json](cleanup.json) 与 `cleanup-files.jsonl`；运行目录中已无截图、关键帧和视频，复现命令保留

提交前独立 ponytail-review 复查当前差异：`Lean already. Ship.`；实际 GLB 合同输出与最终记录一致、8 项输入 hash 一致、本批 131 个本地 Markdown 链接无缺失。任务看板同步与只读校验通过 46 张卡片，保留现有状态分工
