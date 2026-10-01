# 安可镜厅竖版海报 · 制作自查

日期：2026-10-01；任务范围为 AST-007 的既有 r2 画像派生排版，不重绘角色，不改变已确认角色或剧情

## 交付与来源

- 设计源：`source-assets/star-posters/anke-cinema.svg`，`1024 × 1620`，图片视窗、文字和几何装饰保持分层可编辑
- 运行图：`game/assets/environment/posters/anke-cinema.png`，不透明 sRGB，1,488,369 字节，SHA-256 `0aa6fcef6414b0a229c80cd2c935b54f606fc628b5a8ddc2805d328be43eb1fe`
- 原始画像：`source-assets/star-posters/anke-portrait.png`，保留 r2 全部原始字节；`rejected/` 的 r1 未参与本次导出
- 来源与工具：沿用 AST-007 的 image_gen 原创画像及既有提示词记录；本批只用 SVG 和已有 Bun / ImageMagick 导出器裁切、排版，无生图请求，无外部角色与素材，无新服务费用
- 字体：Noto Sans SC，沿 AST-005 固定版本、版权和 OFL-1.1；本批不修改字体
- AST-007 既有唯一登记入口为 `source-assets/star-posters/README.md`；新增版式写回该入口，不另建资产清单

## 实际检查

| 检查 | 结果 |
| --- | --- |
| 两版导出 | PASS，`bun source-assets/star-posters/export.mjs`，见 `export.log` |
| 两版重新导出只读比对 | PASS，`bun source-assets/star-posters/export.mjs --check`，见 `export-check.log` |
| 图片约束 | PASS，`asset_report.py --expect-size 1024x1620 --json`，见 `asset-report.json` |
| 原图及旧横版字节 | PASS，与制作前 SHA-256 相同，见 `inputs.json` |
| 新图 alpha | PASS，全部像素为 255，按不透明贴图接入 |
| 文档引用 | PASS，`python3 -B tools/validate_docs.py --root .`，见 `docs-check.log` |
| 文案 | PASS，仅安可、ENCORE、下一站晴天、NULL SITE / MUSIC、N:SIDE，无日期、票价或新剧情 |
| GPU / 模型安装 | NOT RUN，本批未改 V-15 master、GLB 或游戏系统，由建筑任务集成 |
| 作者验收 | 尚未接受，r2 原画像及新排版均保留候选状态 |

## 实际看图：self-audit

依次打开最终 `1024 × 1620` PNG、ImageMagick 等比例缩小的 `256 × 405` 与 `128 × 203`。脸、发夹、麦克风和持麦手保留，粉发的主形与清楚轮廓继续区别于已弃用 r1；上部裁掉横版左侧留白及部分外侧头发、衣袖，未拉长脸或改变姿势

下部大标题和曲名在 256 像素宽时可辨，128 像素宽时「安可」和脸仍为主信息；英文小标和底部 N:SIDE 在远距离不承担导航、任务或交互语义。文字位于稳定深色底板，没有穿过脸、持麦手或图片高对比细节

顶部留白与底部信息带是版式设计，图片原始顶端已有的发丝出画关系沿用 r2。当前未验证斜视、真实距离、Bevy 日光与压缩后的效果，不能凭平面自查宣布镜厅预览品质完成

## 接入约定与收尾

目标为 `Street programme 1` 的 `1.42 × 2.25 m` 画芯，正面 `0–1 UV` 完整显示，不旋转、不镜像、不重复；纹理宽高比相对物理框差约 0.16%。纸面采用 sRGB、非金属粗糙材质，不沿用大屏的 unlit 显示；实际受光与可读性由建筑接入后确认

建筑线保留原背框与稳定坐标，定向更新 print 并移除该框前方 horizon / composition / light stripe / footer，保存可编辑 master 后导出；源图和字体许可分别登记，不因建筑含 CC0 材质就将画像误标 CC0

独立 `/root/character_art_next/hair_review` 实际查看同一组三个显示尺寸，核对来源与像素、字体和导出差异，未发现需要修正的问题，结论 `Lean already. Ship.`；原图 `[648,0,1672,937]` 与成稿 `[0,82,1024,1019]` 像素逐个一致，底部四行由黄色分隔线覆盖

两张临时小图完成制作与独立复查看图后已清理；正式 SVG、PNG、日志、hash 与文字结论保留，本批没有生成游戏截图、视频或临时可执行文件
