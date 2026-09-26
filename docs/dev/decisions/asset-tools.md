# 资产工具取舍

资产生产使用原版 `create-game-assets`，项目接口由[资产管理](../production/asset-pipeline.md)维护。此页记录方法来源与采用条件，实际检查结果进入对应任务证据；技能来源、完整版本和许可汇总在仓库根目录 `THIRD_PARTY_NOTICES.md`

## 暂缓或不采用

| 上游文件或能力 | 结论 | 采用条件或原因 |
| --- | --- | --- |
| Godogen `tools/grid_slice.py` | 暂缓复制 | 当前没有 sprite sheet；原脚本整除会丢弃余像素，名称未限制输出路径，数量错误也未非零退出。首个真实图集出现时，基于实际网格添加严格尺寸、命名及失败检查后再迁入 |
| Godogen `tools/find_loop_frame.py` | 暂缓复制 | 当前没有逐帧精灵动画；32×32 RGB 相似度会被背景和静态画面主导，候选相似不证明循环连续。未来只作为候选提示，检查重复端帧、接缝速度和连续播放，不自动删除源帧 |
| Godogen `tools/rembg_matting.py` | 暂缓复制 | 依赖 BiRefNet、rembg、numpy、ONNX 与模型下载；现有素材已有有效 alpha。需要复杂去背景且环境具备模型后再试一张源图，核对边缘和前景保留，再批量处理 |
| Godogen 动画链“参考→姿势→视频→抽帧→循环→去背景” | 选择性保留顺序，暂缓服务链 | 先有实际短动画需求及视频工具，再使用已装 ffmpeg 抽帧；记录帧率、保持统一基线、避免重复循环末帧，并按实际播放时间验收。视频生成不因具备 ffmpeg 而自动可用 |
| Godogen `tools/asset_gen.py`、模型与价格表 | 不采用 | Codex 使用环境现有图片工具；不新增 Gemini/Grok SDK、API key 自动选择或固定价格估算，不引入多供应商平台 |
| Godogen 对透明图的一律禁令 | 不采用 | 原生 alpha 能力取决于实际工具，当前内置图片工具支持请求透明输出；通道、边缘和对比底图检查仍是必须的验收证据 |
| Godogen Tripo、Kimodo、外部动画 workspace | 暂缓 | 没有已授权生成服务、角色骨骼或自定义动作集；模型、编码器、GPU 缓存和计费链不进入当前项目依赖。未来需要时先核对服务、素材和模型各自条款 |
| Godogen root-motion bake、手脚约束、夹持与末端 QA 实现 | 暂缓工具，采用检查思想 | 实际角色骨架、控制器、道具与动作确定后再实现相对接触和过渡条件，不能用别的引擎或独立采样路径替代真实 Bevy 播放 |
| poorvith 固定面数、texel density、骨名、60 FPS、全动作套装、FBX 默认 | 不作为项目统一要求 | 依据当前米制 GLB、资产类型和玩法范围决定；不为静态植物增加角色规范，也不把完整跑跳动作包变成 Demo 承诺 |
| poorvith 无条件 Apply All Transforms | 不采用 | 当前环境 GLB 已由导出器用根节点缩放归一，盲目应用变换可能破坏绑定和动画；改动应在源工程上做可验证导出 |

## 来源与改编说明

此页及资产流程中的对比底图、尺度登记、动作接触与连续帧 QA 方法，基于 [htdt/godogen](https://github.com/htdt/godogen/tree/0b725bca053769a4727f76c332bf1f7b42e146ab/asset-gen) 改编，原作者 Alex Ermolov，版本 `0b725bca053769a4727f76c332bf1f7b42e146ab`，MIT，[上游许可](https://github.com/htdt/godogen/blob/0b725bca053769a4727f76c332bf1f7b42e146ab/LICENSE.md) 完整保留在 `third_party/skills/godogen/LICENSE.md`。本地修改是使用 Codex 图片入口、项目唯一资产清单和 Bevy 运行证据，去背景工具与生成服务未复制

原版资产 Skill 来自 [gamedev-skills/awesome-gamedev-agent-skills](https://github.com/gamedev-skills/awesome-gamedev-agent-skills/tree/44888f28ff918357ad82c4473352c60a1c5bde5b/skills/disciplines/create-game-assets)，版本 `44888f28ff918357ad82c4473352c60a1c5bde5b`，Abhishek Barali and the awesome-gamedev-agent-skills contributors，Apache-2.0，[LICENSE](https://github.com/gamedev-skills/awesome-gamedev-agent-skills/blob/44888f28ff918357ad82c4473352c60a1c5bde5b/LICENSE) 与 [NOTICE](https://github.com/gamedev-skills/awesome-gamedev-agent-skills/blob/44888f28ff918357ad82c4473352c60a1c5bde5b/NOTICE) 保留在 `third_party/skills/awesome-gamedev-agent-skills/`；asset_report 的颜色预算边界修复已在文件中显著标注，其他未修改文件继续按原样核对；具体补丁与来源比较见 third_party/skills/manifest.json

Blender 约束比较基于 [poorvith-mp/skills-gamedev](https://github.com/poorvith-mp/skills-gamedev/tree/e8b87e9086fbf2322b1c216c2d2de85954bf4015/skills)，原作者 Poorvith M P，版本 `e8b87e9086fbf2322b1c216c2d2de85954bf4015`，MIT；参照 `blender-modeling`、`blender-animation` 与 `blender-tooling`，具体保留文件和许可见根来源清单
