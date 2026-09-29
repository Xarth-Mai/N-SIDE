# CHR-001 造型源稿自查

历史记录：本页保留 r1／r2 当时的制作自查；作者随后否定其成熟感与配色，两版均已标为 rejected，当前造型探索见[r3 记录](../character-design-r3/review.md)，本页的候选选择不再有效

日期 2026-09-29，任务 TASK-047，本次只制作人物建模参考；剧情、地图、百科身份和游戏代码未改

## 输入与实际制作

读取 CHR-001／CHR-002 正式人物档案、美术方向、资产管线与现有区域预览；没有找到主角正式模型、外观设定图或已定发眼色。区域预览的匿名人物未作为身份源

使用内置 OpenAI image_gen 实际生成 r1，并实际查看完整 1536×1024 原图。随后作者追加“主要人物都是比较年轻潮流的二次元形象，不要做老气了”，以 r1 为唯一图像编辑输入生成 r2，再实际查看完整图。两张图都保存为资产参考，不属于临时 capture 截图

当前选用候选为 `source-assets/characters/CHR-001/turnaround-r2.png`，只代表本轮制作选择，未代作者确认外观

## 机器检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| r1 PNG 解码、1536×1024、RGB、完整文件 | PASS | asset-report.json，退出 0 |
| r2 PNG 解码、1536×1024、RGB、完整文件 | PASS | asset-report-r2.json，退出 0 |
| 两版源哈希与登记一致、generation 与 active_candidate 引用存在 | PASS | source-check.json |
| 全仓 Markdown 检查 | FAIL：并行文档仍在编辑 | `python3 -B tools/validate_docs.py --root .` 报 `docs/dev/production/narrative.md:59` 引用尚未存在的 `../design/quests/campaign-sequences.md`；未改该文件，交由整合时复验 |
| 真实游戏接入、骨骼、动作及镜头表现 | NOT RUN | 本次只交付 2D 建模参考 |

运行命令为 `python3 -B .agents/skills/create-game-assets/scripts/asset_report.py source-assets/characters/CHR-001/turnaround-r2.png --expect-size 1536x1024 --json`；图片没有 alpha 通道且本需求使用不透明底，不误报为去背景资产

## 实际看图 self-audit

| 项目 | r1 | r2 |
| --- | --- | --- |
| 20 岁动漫成年形象 | 清爽短发、普通身材，面孔未显老，但整体服装像普通维修人员 | 保留年轻面孔与短发，短款轻夹克、内衫层次和运动鞋更接近都市青年 |
| 服装轮廓 | 对称大贴袋与棉工作夹克过于职业化 | 去除大贴袋，肩袖色块清楚，裤侧条和鞋面分色可继续用于建模 |
| 三视一致性 | 头发、肩袖、左腰包与右裤袋方向一致 | 头发、肩袖、左腰包、内衫下摆与运动鞋主色关系一致 |
| 建模可解释性 | 完整头手脚，比例可作候选参考 | 完整头手脚，正背姿态接近，侧视接近正侧，适合统一几何后重建 |
| 尚存差异 | 鞋底与裤脚褶皱需统一 | 侧面衣摆斜度、鞋面分片与褶皱不是精确三视测量；以统一网格裁决 |

没有从这张图推断拓扑、UV、骨骼权重或动画质量。没有将生成图示意光照直接用作运行 base color，没有把试验身高写回故事设定

## 清理与下一步

本子任务未创建 output／tmp 临时视觉文件；保留 `source-assets/characters/CHR-001/` 两版图作为正式制作参考及历史候选，生成工具默认目录原件按工具要求未删除。下一步由建模任务解释 r2 成为单一网格并验证尺度、UV、骨骼、in-place 动作与真实街景镜头
