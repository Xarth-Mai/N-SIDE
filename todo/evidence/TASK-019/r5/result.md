# TASK-019 R5 资产工具实测

## 范围与输入

2026-09-26 使用 n-side-work-loop、nside、create-game-assets 执行，按 n-side-review 核对证据边界；任务保持 active，交根会话复核。前置 TASK-010 已完成，读取其 r4/result.md；未启动其他任务

输入完整提交、Python/Pillow 版本、SVG、PNG、显示尺寸实现和两个工具 SHA-256 见 [inputs.json](inputs.json)。执行前工作区已有 game/README.md、todo/README.md、TASK-011 及其证据变更，以及未跟踪的 TASK-019 卡片，保留这些工作

## 工具结果

实际 argv、退出码与 stderr 见 [commands.json](commands.json)，原始 stdout 见 [asset-report.json](asset-report.json) 和 [logo-contrast.json](logo-contrast.json)

- PASS：asset_report 退出 0，PNG 为 512×512 RGBA、50938 bytes、256 色，alpha min=max=255、透明像素 0，尺寸与通道检查无 problems
- PASS：预览工具退出 0，Lanczos 保持比例缩至 205×205，白色 #ffffff、深灰 #151515、洋红 #ff00ff 三底色，输出 output/skills/r5-host/asset/logo-contrast.png（615×229，含 24 像素标签区）
- 显示尺寸依据 game/src/main.rs 的 Sprite 缩放 0.4：512×0.4=204.8，预览取整 205；这是近似显示样本
- 未使用 require-cutout：SVG 自带黑色背景矩形，当前 Logo 全不透明符合既有约定；require-alpha 成功不代表存在透明区域

## 视觉自查 self-audit

使用 view_image 先查看 game/assets/branding/n-logo.png，再运行工具并查看 output/skills/r5-host/asset/logo-contrast.png

原图为黑底白色立体 N 与两个冒号块，留白完整。205×205 三个预览中的主轮廓与冒号可辨，未见裁切或缺块；细斜线缩小后略有灰阶抗锯齿，仍可见。三块图形一致且背景都为黑色，与全不透明 alpha 一致，三种合成底色被原图覆盖，不能据此评价透明抠图边缘

PASS：本轮源图与预览已实际查看，未发现上述范围内明显异常。工具原始 JSON 中 visual_review 的 NOT RUN 是自动生成占位，保留原文，本段另记真实看图结果

NOT RUN：真实游戏窗口、GPU 采样与 DPI 下显示、作者审美、素材许可验收；本轮只执行授权资产工具工作，不改变图片、许可状态或游戏

## 交接

长期约定仅写回 source-assets/branding/README.md 的检查与预览段，本轮日志留在本目录。根会话下一步核对机器报告、预览和输入 hash 后决定任务状态

## 收尾检查

- PASS：输入文件执行前后 SHA-256 一致，含 SVG 与 PNG，详见 [integrity.json](integrity.json)
- PASS：`bun run tasks:sync` 退出 0，14 张卡片，看板已同步
- PASS：`bun run tasks:check` 退出 0，见 [tasks-check.log](tasks-check.log)
- PASS：`bun run check:docs` 退出 0，见 [docs-check.log](docs-check.log)
- NOT RUN：Wiki 构建，本轮未更改 Wiki 页面，且构建输出超出用户指定写入范围
- 未提交或推送；未修改图片、许可状态或游戏，未将任务设为 done

## 根会话复核

已独立读取输入与机器报告并重新核对全部输入hash，实际查看同一205×205三底色PNG：三个N与冒号完整可辨，黑底全不透明，与报告一致。小型阅读证据另存[质量80 WebP](logo-contrast.webp)，原PNG与哈希继续由原记录追踪

本任务的处理、检查、视觉自查与长期约定回写满足既定技术验收，Codex记录done；不更改AST-001的working/许可unverified状态，不宣称真实游戏或作者审美通过
