# 安可画风修订 r2

作者实际查看 r1 后明确否定其画风，并指定 `source-assets/area-previews/cinema-music-street.png` 左上竖版角色海报的日漫感。r1 的原图与提示词已移入 `source-assets/star-posters/rejected/`，历史 CPU / GPU 记录继续保留；其既有技术通过不等于画风通过

本轮用内置 imagegen 实际重绘画像，输入为已打开的镜厅预览图和安可 r1 原图。工具、两张输入哈希、完整提示词、未知参数与实际输出见 `source-assets/star-posters/prompt.md`

## 实际看图自查

- 源图与最终排版都已实际打开；r2 的轮廓线更清楚，头发由细碎柔光发丝改为成组发束，衣褶以干净硬边色块组织，写实散景改为平面图形
- 粉长发、白发夹、琥珀眼、麦克风与轻快都市歌手形象保留，实际 19 岁、约 17 岁外观目标不变；没有将镜厅海报匿名角色换色复制
- 文字仍由原 SVG 提供，中文「安可」「下一站晴天」与人物分区清楚，运行图片完整不透明；r2 是制作自查后的候选，尚无作者接受

## 验证与边界

- `bun source-assets/star-posters/export.mjs`：PASS，重导当前 r2 图像
- `bun source-assets/star-posters/export.mjs --check`：PASS，见 export-check.log
- `asset_report.py ... --expect-size 1600x900 --json`：PASS，见 image-report.json
- 当前运行 PNG SHA-256：`06dc5c6f77d91573298bd0a6a6ccf98ba5c6dd67a8db86c447fbd62f6fe294fe`
- 屏幕几何、材料角色、朝向、位置和 capture 操作脚本没有修改，因此本轮不重跑无关编译与几何测试；仍需主代理对新纹理执行真实 GPU capture 并看图

本子任务没有留下临时截图、接触表或视频，导出临时目录已自动删除。原件、被作者否定的历史源图、SVG 和运行纹理属于正式资产与历史来源，按要求保留
