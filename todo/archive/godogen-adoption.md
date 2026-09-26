---
id: TASK-005
status: done
depends_on: []
evidence: ["todo/evidence/godogen/runtime-validation.md", "todo/evidence/godogen/asset-validation.md", "todo/evidence/godogen/skill-validation.md"]
---

# 开发能力吸收与运行验收

2026-09-26 按本任务授权范围完成技术交付并归档；已知环境限制和后续玩法验收在文末保留

本轮基线为 `aa8971e`，开始时工作区干净。作者授权审查 Godogen 并扩展到七个指定 Skill 仓库，将可用原版完整保留、项目差异集中适配，完成工具、真实运行证据、许可与验证，按独立成果提交

当前主线仍为 M0-02，第 1 轮 review，已验收 1/61。本任务建设项目工具与工作流，沿用进度数据，不代替玩法或阶段验收

## 交付范围

- [x] 上游版本、文件、许可与采用取舍完整记录
- [x] 原版 Skills、适配入口、引用与 Codex 实际发现检查
- [x] 现有资产的报告、预览与导出检查，成功和失败路径
- [x] Viewer 真实输入驱动的离屏序列、视频、状态断言与失败诊断
- [x] 实际看图记录、开发闭环、非 GPU 与渲染 CI 分离
- [x] 适用检查完成并分批提交

## 记录归属

来源与复制文件清单在 `THIRD_PARTY_NOTICES.md` 和 `third_party/skills/manifest.json` 维护；长期操作方法写入制作文档；本轮命令、运行与视觉观察证据放在 `todo/evidence/godogen/`。大体积截图、视频与资产预览放在忽略的 `output/`，不进入运行资产

## 审查与取舍

七个仓库均固定完整 commit，[来源清单](../../THIRD_PARTY_NOTICES.md)负责版本、作者、目录映射、采用方式和许可，避免在此维护另一份来源表。Godogen 参考版本为 `0b725bca053769a4727f76c332bf1f7b42e146ab`，从独立临时 checkout 阅读全部目录、README、AGENTS、runtime、Bevy 指南、asset-gen 及引用工具、setup、publish、scripts 和变更记录；未执行上游发布器或接入其 Agent 宿主

| 上游文件或能力 | 价值与取舍 | 本地落点与验收 |
| --- | --- | --- |
| Godogen `README.md`、`AGENTS.md`、`prompts/runtime.md` | 改造采用开发→真实运行→画面→修复闭环，按任务影响分级；沿用既有状态和作者验收 | `AGENTS.md`、现有 work-loop/review、[运行验收](../../docs/production/runtime-validation.md)；真实输入录像与独立失败探针 |
| Godogen `engines/bevy.md` 离屏 target、ready、异步落盘 | 改造采用；扩展已有 Viewer，复用真实世界与 FreeCamera，和 bevy-skills capture 比较后只留一条实现 | `map_viewer/capture.rs`、`tools/capture.py`；540 帧、状态断言、视频及失败退出 |
| Godogen Bevy 依赖、asset features、glTF、winding、Visibility、构建 profile | 部分已有，保留针对当前版本的排错入口；不升级、不裁剪无证据的 features | `world::*`、[工程](../../docs/production/engineering.md)；当前锁文件、fmt、clippy、14 项 Rust 测试、CPU 地图预检 |
| Godogen `asset-gen/SKILL.md` 参考统一、源图先审、尺度登记 | 改造采用；由原版 create-game-assets 负责生产，nside 接现有资产包和导出器 | [资产管理](../../docs/production/assets.md)；原版图片报告、5 图预览、现有导出一致性 |
| Godogen `asset-gen/rembg.md` 对比背景检查 | 方法改编；按实际工具选择 alpha，不照搬透明禁令 | `tools/asset_preview.py`；合成像素、原件保护、错误尺寸与不透明 cutout 失败检查、实际看图 |
| Godogen `motion.md`、GLB 与数值 QA | 静态尺度检查落地；骨骼、接触、root motion 和动作约束暂缓实现，已有适配条件 | `test_asset_environment.py` 验证 10 个真实 GLB；动画方法见[工具取舍](../../docs/production/asset-tool-review.md) |
| Godogen `grid_slice.py`、`find_loop_frame.py`、`rembg_matting.py` | 暂缓：当前无 sprite sheet／精灵序列，重型去背景无实际需求；记录余像素、命名、循环相似度的失效边界 | [工具取舍](../../docs/production/asset-tool-review.md)逐项记录采用条件，不预建空工具 |
| Godogen `asset_gen.py`、`requirements.txt`、生成模型选择和价格 | 不采用多供应商平台，现有素材足够验证；没有自动开通服务 | Codex 现有 imagegen 能力按需调用；本轮未调用生成、未产生生成费用 |
| Godogen `publish.sh`、`setup.md`、`scripts/render_dir.py`、`generate_codex_metadata.py` | 不采用空仓复制、强制发布、模板替换和另一套配置生成器；原版 Skill 已含可用配置 | 原样 Skill + UPSTREAM + 本地适配；实际 Codex 发现，未运行 publish |
| Bevy 13 个基础 Skill | 原样采用，版本与 0.19.1 锁文件匹配；原文示例不是本地编译证明 | `.agents/skills/bevy-*`；来源哈希、宿主发现、实际使用 API 的构建 |
| qiuaoru 两个设计 Skill、abagames stress test | 原样采用；日常设计审查与专项规则攻击分别调用 | 完整目录、MIT 和第三方方法归属；静态检查和实际发现 |
| awesome 六个专业 Skill | 原样采用，资产脚本实测；工程和数据接口放项目适配层 | `create-game-assets` 等、`nside`；Apache LICENSE/NOTICE、脚本正反例 |
| poorvith Blender／内容、abagames 其余方法、MengTo 参考分析 | 3 个适配 Skill、19 个原样参考目录；不同时启用重复关卡入口，不将街机常驻读数或单平面规则覆盖本作 | 本地来源说明与方法映射；一次新上下文的前向使用探针，Blender 运行 NOT RUN |

未采用其他引擎指南、demo prompts、仓库 issue/PR 模板和 Claude 专用配置；保留内容的原作者版权行、完整 MIT/Apache 许可及必要 NOTICE。Godogen 仅是开发资料来源，不是运行依赖

## 实际交付

首个里程碑 `761f325 feat: integrate upstream skills and real runtime validation` 已提交全部能力实现与原版来源。第二个里程碑收口操作入口、检查记录与任务归档

- [运行证据](../evidence/godogen/runtime-validation.md)：真实 Vulkan 输入录像、机器条件、看图与未覆盖项
- [资产证据](../evidence/godogen/asset-validation.md)：原版脚本、现有图片与 GLB 的正反例
- [Skill 接入证据](../evidence/godogen/skill-validation.md)：172 个登记文件、31 个实际启用 Skill、兼容检查器差异
- [前向使用探针](../evidence/godogen/skill-forward-probe.md)：实际素材与录像生成可执行 brief，没有重复清单、额外服务或第二套 capture

## 检查与边界

本轮遵循 `prompt-skill-authoring` 编写项目差异，经 `prompt-skill-review` 独立复查保留人工验收、唯一状态源和 NOT RUN 边界。修复了宿主缺失时错记 FAIL 的分支；`ponytail-review` 结论为 `Lean already. Ship.`

原版 `bevy-ecs-systems/references/system-params.md:17` 含 Markdown 行尾双空格，按来源字节保留；本地变更的 whitespace 检查通过，不通过改原文来满足项目风格

远端 CI 尚未调度、自托管渲染 runner 未部署；Windows、手柄、桌面连续操作、作者手感与陌生玩家测试未执行。当前没有角色骨骼和玩法控制器，不虚报动画、碰撞或任务验收；保留既有影院支撑布局 Warning 与招牌比例观察，后续在相应场景／资产工作包处理
