# 品牌资产

| 字段 | 值 |
| --- | --- |
| ID | `AST-001` |
| 状态 | `working` |
| 主文件 | `source-assets/branding/n-logo.svg` |
| 作者与来源 | 用户提供的 N:SIDE 主 Logo |
| 许可状态 | `unverified`，用户指定用于本项目，作者与许可信息尚未提供 |
| 使用位置 | README、Wiki 顶栏、Wiki 首页、浏览器图标、游戏启动画面 |
| 游戏派生文件 | `game/assets/branding/n-logo.png` |
| 游戏导出设置 | `512 × 512` PNG |

主 Logo 保留 SVG。README 直接引用主文件，Wiki 通过显式发布清单使用主文件；游戏从主文件导出 PNG

## 检查与预览

处理前先查看现有运行 PNG 或源图，保留 SVG 主文件与 PNG 派生关系；记录输入 SHA-256，结束后复核原件未变，命令与本轮结果进入对应任务证据

使用 `asset_report.py --expect-size 512x512 --require-alpha --json` 检查运行 PNG；`--require-alpha` 仅要求存在 alpha 通道。Logo 保留源 SVG 中的不透明黑底，不使用 `--require-cutout`，也不因通道存在判断图片透明

使用 `tools/asset_preview.py` 以 Lanczos 生成白、深灰、洋红三底色预览；显示尺寸按实际使用处缩放计算，当前启动画面 `512 × 0.4` 取近似 `205×205`，实现变化后重新核对。预览进入 `output/`，逐图观察轮廓、细线、裁切和边缘，将机器结果与视觉自查分开记录；实际窗口表现按[运行验证](../../docs/dev/validation/runtime.md)另验

## Wiki 图标派生

`docs/public/favicon.ico` 包含 16、32、48 像素图层，`docs/public/apple-touch-icon.png` 为 180 像素。两者由现有 `game/assets/branding/n-logo.png` 使用 Pillow 导出，Apple 图标采用 Lanczos 缩放；保留黑底、构图与 SVG 唯一源，更新 Logo 时同步重新导出

Wiki 显式发布根路径 favicon 和 Apple 图标，同时保留 SVG 图标，供不同浏览器与收藏入口选择
