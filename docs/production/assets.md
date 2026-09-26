---
id: DOC-ASSETS
---

# 资产管理

## 分层

| 位置 | 用途 | 是否主文件 |
| --- | --- | --- |
| `source-assets/` | 项目级共享资产及 Blender、Krita、音频工程等源工程 | 是 |
| `docs/public/` | 仅供 Wiki 使用的图片与附件 | 否 |
| `game/assets/` | 游戏直接加载的运行资产 | 否 |
| `output/assets/` | 本轮参考对照、生成候选、裁切试验、预览与机器检查报告，Git 忽略 | 否 |

同一资产只指定一个主文件，统一放在 `source-assets/`。SVG、JSON 等开放格式可以直接作为主文件和交付文件使用。派生文件不得单独编辑，更新时从主文件重新导出

## 资产包

项目级资产按用途组织为 `source-assets/<package>/`。每个资产包使用 `README.md` 记录资产 ID、主文件、作者与来源、许可状态、导出设置、使用位置和派生文件

内容对象专用资产继续使用对象目录内的 `asset-manifest.json`。多个对象或多个交付面共同使用的资产提升为项目级资产包，其他清单通过资产 ID 引用

## 接入与更新

Wiki 通过配置中的显式发布清单，将允许公开的源资产映射为 `/project-assets/`，不复制主文件，也不公开整个 `source-assets/`。README 使用仓库相对路径引用主文件。游戏需要不同格式或尺寸时，将导出物放入 `game/assets/`，并在资产包说明中记录生成参数

更新资产时修改主文件，重新生成全部派生文件，然后运行文档检查、Wiki 构建及受影响工程的检查。在实际界面或镜头中完成视觉验收后，再将资产状态记为 `integrated`

## Codex 资产生产

采用原版 `create-game-assets` Skill 的需求、参考、生产、规范化与游戏内验收流程，项目差异集中在本页。生成调用使用当前环境的 `imagegen` Skill 与内置图片工具；先确认工具实际存在，再发起调用。原版 Skill 的 manifest 模板用作字段参考，实际记录仍进入既有资产包 README 或对象 `asset-manifest.json`

1. 读取[美术方向](art.md)、资产使用处与真实游戏截图，确定镜头、物件尺度或显示像素、状态和变体，复用已有 SVG、模型、材质及导出器
2. 在所属资产包 README 写本轮 brief：用途、轮廓、色彩、光照、材质、显示尺寸、保持项、变化项和验收镜头；先完成一个可比较的视觉样本
3. 同一角色或物件家族复用选定的参考图和比例，变体每次只改必要因素；先看源图，再做建模、绑定、动画或其他高成本下游操作
4. 按实际工具选择原生 alpha 或去背景，保留原图；确认尺寸、裁切、枢轴与命名，再做压缩、图集或派生导出。像素风用 nearest，现有动漫化图形缩小通常使用 Lanczos，最终以 Bevy 实际采样效果验收
5. 运行图片约束检查，查看统一预览和实际显示尺寸下的明暗对比底图；把选定原件移入 `source-assets/` 并更新既有清单，派生物进入 `game/assets/`
6. 在真实场景中检查材质、轮廓、遮挡、比例、动作及玩家反馈，按[运行验收](runtime-validation.md)保存证据。机器结果与看图结论分别记录到对应任务

生成能力以当前会话实际暴露的工具为准。内置图片工具可以请求透明图，仍需检查输出 alpha；没有视频、3D 或去背景工具时继续做参考分析、现有素材导出与验收，并明确记录缺口。新增付费服务或外部授权先由作者确认，当前配置不默认包含 Godogen 的 Gemini、Grok、Qwen、Tripo 或 Kimodo

## 记录字段

沿用[模板资产关系](../templates/index.md#资产关系)中的资产 ID、路径、制作状态和许可状态，共享资产仍按 ID 引用。按素材类型在原有记录中补充以下信息；未取得的值写 `null` 或“未知”，不把估价写成实际花费

| 信息 | 内容 |
| --- | --- |
| 需求与参考 | 用途、所属对象、参考资产 ID 或路径、保持项与变体关系 |
| 来源与许可 | 作者、源 URL 或生成工具、原文件哈希、适用条款与署名；开发 Skill 的许可不替代素材许可 |
| 尺寸与导出 | 原图与运行尺寸、显示像素或米制尺寸、朝向、枢轴、色彩空间、滤镜、图集边距、导出命令 |
| 生成记录 | 实际可获得的工具与版本、提示词、输入参考、任务 ID、种子、生成参数和实际成本；工具未返回的项目留空 |
| 验收 | 机器报告路径、真实场景与构建、看图或操作结论、尚未完成的检查 |

已有 `AST-003` 在 `source-assets/environment-kit/README.md` 与同目录清单中按原件哈希和导出参数管理米制 GLB、纹理尺寸及许可，继续作为环境素材唯一清单；不另外维护一份“生成资产总表”

## 图片检查入口

以下命令在仓库根目录执行，fish 可直接使用。contact sheet 脚本保留原版，asset_report 的颜色预算边界按本地补丁修复；两者依赖 Python 3.10+ 与 Pillow；缺少 Pillow 时按 Skill 中的 `scripts/requirements.txt` 安装到项目使用的 Python 环境

```fish
mkdir -p output/assets
python3 -B .agents/skills/create-game-assets/scripts/asset_report.py game/assets/branding/n-logo.png --expect-size 512x512 --require-alpha --json > output/assets/logo-report.json
python3 -B .agents/skills/create-game-assets/scripts/build_preview_sheet.py game/assets/environment/signs/*-display.png --out output/assets/displays.png --columns 5 --cell-size 192
python3 -B tools/asset_preview.py game/assets/branding/n-logo.png --display-size 205x205 --out output/assets/logo-contrast.png > output/assets/logo-contrast.json
python3 -B tools/asset_preview.py game/assets/environment/signs/shop.png --display-size 512x64 --require-cutout --out output/assets/shop-contrast.png > output/assets/shop-contrast.json
python3 -B -m unittest discover -s tools/tests -p 'test_asset*.py'
```

Logo 的 `205×205` 预览来自当前 `512×512` 图与启动画面 `0.4` 缩放的近似显示尺寸，仍需在真实窗口中核对；Logo 是不透明黑底，因此不要求 cutout。招牌 `512×64` 是半尺寸可读性样本，透视场景中的实际覆盖像素由运行 capture 检查。上游 `--require-alpha` 只检查通道是否存在；本地 `--require-cutout` 进一步拒绝完全透明或完全不透明的图片。预览保留宽高比，浅色、深色、洋红底用于发现灰边、色溢和缺口，工具输出不会自动声称视觉通过

## GLB、骨骼与动画

当前环境模型复用 `bun tools/export-environment.mjs --check` 与 `world::assets` 的 Bevy 加载预检，保留 `Scene0`、材质槽及外部色板依赖。`test_asset_environment.py` 对现有静态环境模型核对实际二进制 accessor 的有限数值与声明边界，以及节点变换、米制高度和底部枢轴。源 GLB 的节点缩放与运行 GLB 的米制归一分开记录；当前导出器有意保留根节点缩放，放入场景时实体缩放为 `1`，无需为满足通用“应用全部变换”规则破坏现有数据

Blender 源工程进入 `source-assets/`，运行交付仍使用项目已支持的 GLB。面数、贴图密度、碰撞代理和 LOD 依据[美术](art.md)及实际镜头预算确定。绑定后的角色调整变换时，应同时核对 bind pose、inverse bind matrices、动画和附着点，不能沿用静态道具的批量清理方法

接入首个骨骼角色时，再按真实播放路径检查 clip 名称、时长推进、关键动作和过渡帧。位移由控制器或 root motion 的一个明确责任方提交；脚接地、持物接触、攻击时机和相对速度使用适合该动作的数值条件，并配合连续录制检查。当前没有骨骼动画资产，不预建动作生成服务或声明已通过此类检查

工具选择、适用条件与保留来源见[资产工具取舍](asset-tool-review.md)
