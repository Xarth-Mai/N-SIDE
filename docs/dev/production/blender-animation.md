# Blender、模型与动画接入

本页负责从可编辑模型到游戏内表现的交付，专业操作使用 `nside-blender-pipeline`，Bevy 播放接口按 `bevy-animation` 和当前 `game/Cargo.lock` 核对。资产身份、来源和派生关系归[资产管线](asset-pipeline.md)，风格归[美术方向](art-direction.md)

现有 `source-assets/environment-kit/` 与 `game/assets/environment/` 提供静态 GLB 样板；当前没有骨骼角色与动作播放路径。下述动画要求是接入契约，静态模型检查不能证明动画已完成

## 输入与前提

读取资产包 README、原 `asset-manifest.json`、所属人物或场所规格、实际使用镜头和对应任务。明确唯一主文件、输出路径、米制尺寸、正面朝向、放置基准、材质槽、碰撞责任和本轮需要的 clip；贴图遵循现有色彩空间与透明策略

先检查本机 Blender 是否可用，再安排 DCC 操作。缺少 Blender 时可以复验既有 GLB、导出和引擎引用，DCC 修改与绑定检查记为 NOT RUN。具体面数、骨骼数、权重影响数、帧率、过渡时长和足滑容差按资产与实际实验确定，未确定值留在任务中

## 制作步骤

1. 在统一标尺与光照下检查源模型轮廓、法线、UV、材质和透明边缘，再放到目标镜头判断细节是否必要；保留源工程，不以低清截图决定高成本绑定
2. 静态物件在导出副本中按放置方式处理尺度和 pivot：落地物件用底面，贴墙构件用贴附面，转动物件用转轴；地图 `[x,y,h] → [x,h,-y]` 由世界模块执行，模型不再次转换地图坐标
3. 绑定资产先检查父子空间、bind pose、inverse bind matrices、骨骼名称、蒙皮权重与附着点，再处理变换；修改骨架后一起检查 clip 和持物，不对源文件批量执行 Apply All Transforms
4. 每个必要动作记录名称、时长、循环方式、事件时点与根位移；在 DCC 中看开始、接触、结束和循环接缝，确认动作保存在源工程，导出范围只包含已需要的对象和动作
5. 导出 GLB 并核对实际场景、材质、图片及 clip；静态环境复用现有导出器，角色接入时由真实加载路径解析动画目标，不假定任何 GLB 天然包含 `Scene0` 或 `Animation0`
6. 在实际 Bevy 场景播放与切换动作，检查时间确实推进、关节与道具位置有效、打断和恢复可重复，再按[运行验证](../validation/runtime.md)记录状态和连续画面

## 位移与事件归属

每种动作只指定一个世界位移提交者，决定写入相应控制器规格与资产记录

| 方式 | 动画负责 | 控制器负责 |
| --- | --- | --- |
| in-place | 姿态、局部运动、步态节奏与脚接触提示 | 世界位置、速度、碰撞与地面约束；实际速度与播放速度的对应 |
| root motion | 根节点位移与旋转曲线、姿态及动作事件 | 读取本次根增量，经碰撞和约束提交一次世界移动；处理被阻挡、取消与重试 |

采用 root motion 不等于允许动画直接绕过碰撞写世界 Transform；采用 in-place 也不代表足滑自然消失。足滑、接地误差和动作取消用该动作的实测条件检查，先区分资产、播放速度和控制器原因，再改负责方

脚步、工具效果等表现可由动作事件触发；任务提交、物品发放、伤害判定仍由对应玩法状态负责。重播、跳转时间或读档不能重复提交一次性结果。表现对象的停止与清理遵循[声音](sound.md)和[VFX](vfx.md)

## 输出与检查入口

交付主文件、GLB、依赖纹理、导出命令及版本、clip 表与位移责任、资产清单更新和本轮证据。导出文件由主文件生成，不单独修补导出物。现有静态样板可从仓库根执行以下 fish 命令

```fish
command -v blender
bun tools/export-environment.mjs --check
python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --validate
```

`command -v blender` 只确认命令存在；其余检查覆盖已登记的静态模型、导出一致性、数值、米制高度、底部 pivot 和 Viewer 绑定，不包含蒙皮或 clip 播放。首个骨骼资产接入时，在实际应用中补充目标绑定、时间推进、过渡、重复进入及退出清理检查，再给出对应运行命令

缺失纹理、无效数值、错误场景或未解析动画目标直接退回接入检查；循环跳变和穿插记录具体帧与动作后修资产或控制器。无法播放时保留加载诊断，不用 DCC 预览冒充引擎验收。静态 GLB 检查失败时保留原件，从登记的源与导出参数重现

## 来源与适配

基于 Poorvith M P 的 [Blender 建模、动画和工具方法](https://github.com/poorvith-mp/skills-gamedev/tree/e8b87e9086fbf2322b1c216c2d2de85954bf4015/skills)改编，固定 commit `e8b87e9086fbf2322b1c216c2d2de85954bf4015`，Copyright (c) 2026 Poorvith M P，[MIT LICENSE](https://github.com/poorvith-mp/skills-gamedev/blob/e8b87e9086fbf2322b1c216c2d2de85954bf4015/LICENSE)。Modified for N:SIDE：按已有 GLB 与资产清单交付，区分静态与绑定变换，预算按项目实验，位移和任务提交由对应运行系统负责

Bevy 接入参考 Chris Gliddon 的 [bevy-animation](https://github.com/chrisgliddon/bevy-skills/tree/b1b4da5744ebbd5c526342b2351967411cd5ca61/skills/bevy-animation)，固定 commit `b1b4da5744ebbd5c526342b2351967411cd5ca61`，[MIT LICENSE](https://github.com/chrisgliddon/bevy-skills/blob/b1b4da5744ebbd5c526342b2351967411cd5ca61/LICENSE)。完整保留声明和本地映射见仓库根 `THIRD_PARTY_NOTICES.md` 与 `third_party/skills/`
