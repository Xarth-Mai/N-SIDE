# Blender、模型与动画接入

本页负责从可编辑模型到游戏内表现的交付，专业操作使用 `nside-blender-pipeline`，Bevy 播放接口按 `bevy-animation` 和当前 `game/Cargo.lock` 核对。资产身份、来源和派生关系归[资产管线](asset-pipeline.md)，风格归[美术方向](art-direction.md)

现有 `source-assets/environment-kit/` 与 `game/assets/environment/` 提供静态 GLB 样板；`source-assets/characters/CHR-001/model/` 与 `source-assets/characters/CHR-002/model/` 分别提供曜、玲的可编辑灰阶骨骼研究，接入同一真实人物控制器。两者保持 `needs_revision`，外观与配色未获批准，静态模型检查与动作播放均不等于正式人物已完成

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

重定向外部骨架时，先区分骨骼局部轴、骨 tail 与解剖关节连线。转换器生成的 tail 可能只用于编辑显示，并不指向下一关节；不能将全部 head-tail 方向直接对齐到项目骨架。按实际肩—肘—腕和髋—膝—踝检查 rest 差异，逐段验证，再采样站立与步态的头、骨盆和脚位置。动画时间前进、根静止和 Transform 有限仍可能对应翻倒或扭曲的人物，必须检查真实姿态

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
bun tools/export-environment.ts --check
python3 -B -m unittest discover -s tools/tests -p test_asset_environment.py
cargo run --manifest-path game/Cargo.toml --features viewer --locked --bin map_viewer -- --project-root . --validate
```

`command -v blender` 只确认命令存在；其余检查覆盖已登记的静态模型、导出一致性、数值、米制高度、底部 pivot 和 Viewer 绑定，不包含蒙皮或 clip 播放。骨骼预览使用下面的真实入口；目标绑定、时间推进、暂停、重复进入及退出清理需按实际运行范围留证，尚未制作的混合过渡不计为通过

角色导出后先运行[角色 GLB 预检](../../../tools/README.md#角色-glb-导出预检)，按该人物规格传入根骨名称、实际高度和必需 clip。它读取真实 accessor 检查四权重、inverse bind 还原、米制边界、嵌入纹理与 in-place 根通道，失败返回非零；两骨方体夹具证明 Blender 多动作导出与检查路径，不证明正式人物建模或动作已经完成

两份角色研究GLB共用具名 `Scene` 与 `Idle` / `Walk` / `Run`、米制 Y-up、+Z 前向和脚底原点，32骨与in-place动作按各自源工程导出。玲的骨架与平移曲线同比缩放，运行播放速率相应补偿步幅；修改模型尺度后同时复验DCC接地与真实控制器下的足滑。具体加载、失败处理、尺度与播放速率由[角色预览契约](../engineering/player-preview.md#灰阶角色动作预览)统一维护

```fish
cargo run --manifest-path game/Cargo.toml --locked --bin n-side -- --project-root . --character-preview
cargo run --manifest-path game/Cargo.toml --locked --bin n-side -- --project-root . --character-preview=CHR-002
cargo build --manifest-path game/Cargo.toml --locked --bin n-side
python3 tools/capture.py --binary game/target/debug/n-side --character-preview --script game/capture/walk-character.json --output output/capture/walk-character
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-002 --script game/capture/walk-character.json --output output/capture/walk-character-ling
```

原参数不带值仍选曜，玲使用原生参数的等号形式；capture包装脚本接受空格或等号传值。两模型复用同一录制脚本，分别读取真实动画时间与切换、暂停状态，结合连续画面检查骨骼运动和接地；当前只实现Idle／Walk／Run直接切换，空中Idle占位，足滑校准、Jump与动作混合尚未完成。默认入口保留中性代理，具体运行结果归TASK-047，不把候选技术接入作为造型批准

缺失纹理、无效数值、错误场景或未解析动画目标直接退回接入检查；循环跳变和穿插记录具体帧与动作后修资产或控制器。无法播放时保留加载诊断，不用 DCC 预览冒充引擎验收。静态 GLB 检查失败时保留原件，从登记的源与导出参数重现

### 社区骨架的本地比较

`tools/prepare_belle_reference.py` 为已检查的 Ghost73 铃 glTF 提供单一转换入口：读取另行取得的 148 骨、无动画源包，归一到 1.65 m，同步网格和形态键，按实际关节位置修正上肢 rest 差异，重定向项目现有三段动作并创建本地资源覆盖目录。逐帧检查头／骨盆／脚关系和静止根，保留源文件与依赖 hash。工具不下载模型，输出目录必须是仓库 `output/` 中尚不存在的目录

此入口在 Linux、Blender 4.5.14 验证，资源覆盖目录使用符号链接，依赖当前仓库，不是可分发游戏包。`CHR-002` 只作为既有预览槽位，模型仍保留第三方铃的身份，不覆盖正式月城玲。该特定骨架映射不用于其他社区包；来源与条款先按资产管线登记，原模型和转换品均留在忽略目录。形态键只保留在源工程，本轮运行导出不包含表情动画，手指、裙摆与转场动作继续实际检查

从公开发布页取得并解压原包后，在仓库根执行以下 fish 命令，`--source` 指向实际 glTF；安装路径以本机 Blender 为准

```fish
output/tools/blender-4.5.14-linux-x64/blender -b -t 2 --python-exit-code 1 --python tools/prepare_belle_reference.py -- --source output/assets/zzz-reference/belle-player-share/model/belle.gltf --output output/assets/belle-local-test
python3 tools/validate_character.py output/assets/belle-local-test/reference.glb --root-node Root --height 1.65 --clip Idle --clip Walk --clip Run --require-texture
python3 tools/capture.py --binary game/target/debug/n-side --character-preview CHR-002 --project-root output/assets/belle-local-test/project --script game/capture/walk-character.json --output output/capture/belle-local-test
```

目录复用、源文件缺失、依赖越出源包或骨架不匹配时失败退出，不覆盖原件。运行后分别查看 `prepare.json`、GLB 预检、capture 状态和连续画面，再按根目录清理约定移除临时视觉产物

## 来源与适配

基于 Poorvith M P 的 [Blender 建模、动画和工具方法](https://github.com/poorvith-mp/skills-gamedev/tree/e8b87e9086fbf2322b1c216c2d2de85954bf4015/skills)改编，固定 commit `e8b87e9086fbf2322b1c216c2d2de85954bf4015`，Copyright (c) 2026 Poorvith M P，[MIT LICENSE](https://github.com/poorvith-mp/skills-gamedev/blob/e8b87e9086fbf2322b1c216c2d2de85954bf4015/LICENSE)。Modified for N:SIDE：按已有 GLB 与资产清单交付，区分静态与绑定变换，预算按项目实验，位移和任务提交由对应运行系统负责

Bevy 接入参考 Chris Gliddon 的 [bevy-animation](https://github.com/chrisgliddon/bevy-skills/tree/b1b4da5744ebbd5c526342b2351967411cd5ca61/skills/bevy-animation)，固定 commit `b1b4da5744ebbd5c526342b2351967411cd5ca61`，[MIT LICENSE](https://github.com/chrisgliddon/bevy-skills/blob/b1b4da5744ebbd5c526342b2351967411cd5ca61/LICENSE)。完整保留声明和本地映射见仓库根 `THIRD_PARTY_NOTICES.md` 与 `third_party/skills/`
