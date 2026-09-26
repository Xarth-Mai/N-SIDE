# 资产流程验证

日期：2026-09-26；范围：现有源素材、运行派生物与新接入图片检查工具。采用来源和长期条件见[资产工具取舍](../../../docs/dev/decisions/asset-tools.md)，本记录不增加新的资产清单，也不表示 M0/M1 工作包已验收

## 机器检查

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `python3 -B -m unittest discover -s tools/tests -p 'test_asset*.py' -v` | PASS，4 项测试；包含原版报告与 contact sheet、本地实际尺寸合成、原件保护和失败退出、5 组源／运行 GLB | `output/assets/tool-tests.log` |
| `python3 -B .agents/skills/create-game-assets/scripts/asset_report.py game/assets/branding/n-logo.png --expect-size 512x512 --require-alpha --json` | PASS，512×512 RGBA；alpha 最小／最大均 255，实际为不透明黑底 | `output/assets/logo-report.json` |
| 同一报告命令将尺寸改为 `--expect-size 64x64` | 预期 FAIL，退出码 1，明确报告实际 512×512 与预期 64×64 | `output/assets/wrong-size-failure.json`；单元测试断言退出码 |
| `python3 -B tools/asset_preview.py game/assets/branding/n-logo.png --display-size 205x205 --require-cutout --out output/assets/logo-cutout.png` | 预期 FAIL，退出码 1，拒绝完全不透明图；说明仅有 alpha 通道不足以证明透明 | `output/assets/logo-cutout-failure.json` |
| 同一预览命令移除 `--require-cutout`，输出 `logo-contrast.png` | PASS，保留原件，生成 205×205 显示尺寸的三底色预览 | `output/assets/logo-contrast.json`、`logo-contrast.png` |
| `python3 -B tools/asset_preview.py game/assets/environment/signs/shop.png --display-size 512x64 --require-cutout --out output/assets/shop-contrast.png` | PASS，原件 1024×128，alpha 6—255，半尺寸三底色预览 | `output/assets/shop-contrast.json`、`shop-contrast.png` |
| `python3 -B .agents/skills/create-game-assets/scripts/build_preview_sheet.py game/assets/environment/signs/*-display.png --out output/assets/displays.png --columns 5 --cell-size 192` | PASS，5 张陈列图，统一顺序、比例与棋盘底 | `output/assets/displays.png` |
| `bun tools/export-environment.mjs --check` | PASS，15 份运行派生物，28,054,454 bytes；源哈希、GLB 根变换与材质、DDS 完整 mip 和派生字节一致 | `output/assets/environment-export.log` |

所有命令在仓库根目录执行，单行命令可直接用于 fish。`output/` 为忽略目录，提交的是可复验脚本与本记录，图像和 JSON 检查产物不进入运行时 assets

## 静态模型数值检查

`test_asset_environment.py` 针对当前环境素材的单根静态场景检查，不充当任意 glTF 验证器。它读取 GLB 内实际 accessor 数据，检查 buffer view 范围、有限数值、POSITION 声明 min/max、有限节点变换与均匀正缩放，再对运行文件验证米制高度和地面枢轴；源文件保留原始偏移，不要求它也落在 y=0

| 模型 | 原几何高度 | 运行高度 | 运行底部 | skin／animation |
| --- | --- | --- | --- | --- |
| tree_detailed | 1.332417 | 6 m | 0 m | 0／0 |
| tree_oak | 1.226240 | 5.5 m | 0 m | 0／0 |
| plant_bushDetailed | 0.360411 | 0.8 m | 0 m | 0／0 |
| light-curved | 0.675000 | 4.5 m | 0 m | 0／0 |
| road-sign-empty | 0.475000 | 2.3 m | 0 m | 0／0 |

5 个源文件与 5 个运行文件全部通过，浮点容差为 1e-6；变换与边界摘录保存在 `output/assets/model-checks.json`。这些检查不证明三角网格的美术品质、碰撞或骨骼动画有效

## 实际看图

Codex 已用图片查看工具打开 `logo-contrast.png`、`shop-contrast.png` 与 `displays.png`，以下是人工式看图记录，不是机器断言，也不是隔离盲测

- Logo 在约 205 px 尺寸下保留 N 与冒号的白色轮廓、黑色立体边和细线，三个底色栏相同，符合不透明黑底数据；没有据此修改 Logo 许可状态
- 小店招牌在 512×64 下主字和黄色边框可辨，三种底色未出现明显洋红溢边；小字在半尺寸下较细，真实街景距离是否可读仍交给运行 capture
- 五张陈列图共享克制绿色、框线和斜向玻璃高光，面包、杯子、包材、果蔬与杂货有可区分的大形；棋盘仅出现在 contact sheet 留白中，不是原图透明背景证据

## 能力与未测项

本轮实际环境为 Pillow 12.3.0，可运行 ffmpeg；未发现 Blender、Qwen CLI、Tripo CLI 或 rembg Python 包。会话暴露内置图片生成工具与 imagegen Skill；没有调用生图、视频或 3D 生成服务，没有发生生成费用，不检查或打印 API key

NOT RUN：Blender 导出、视频生成、3D 生成、去背景模型、骨骼与 root-motion 动作验收；当前素材没有骨骼动画。Bevy 真实场景的加载、运动与画面由本轮运行验收证据承接，这组静态图片检查不代替它
