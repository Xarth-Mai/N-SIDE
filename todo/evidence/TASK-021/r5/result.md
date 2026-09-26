# 玩家百科目录扁平化

按用户要求去掉 docs/player/encyclopedia 一层，将六个内容目录与 guide 并列，87 篇正文直接位于 docs/player 下；完整站仍使用 docs/.vitepress/dist

## 内容与引用

[migration.json](migration.json)记录每篇原路径、新路径及迁移前后 SHA-256。逐篇从迁移前 Git HEAD 读取正文，与新文件屏蔽链接目标后比较：87 篇文字全部一致，仅必要的相对链接发生变化

导航、地图入口、故事目录与校验器更新至新路径；历史证据只更新 Markdown 引用目标，保留原始迁移映射和历史 URL 快照。两个项目 Skill 仅更新正文入口路径，prompt-skill-authoring 与 prompt-skill-review 静态复核未发现行为契约变化

当前百科页面自动生成 87 个旧 player/encyclopedia 地址的跳转，并保留锚点；更早的 125 个旧地址通过原迁移索引直接指向新位置。导航与搜索只索引当前页面

## 验证

- PASS：文档与 Skills、叙事、故事关系、TypeScript 与 Vue 类型检查
- PASS：103 项 Python 测试、94 项 Bun 测试，覆盖旧映射转换、新旧故事 URL 与发布受众边界
- PASS：双站构建，玩家站 89 页，完整站 145 页
- PASS：87 篇正文文字保留检查
- PASS：519 项真实 HTTPS 检查覆盖 145 页、212 个旧入口、156 个资源及补充入口，结果见 [live.json](live.json)，检查脚本见 [check-full-site.py](check-full-site.py)
- NOT RUN：浏览器桌面和手机视觉与交互验收，原任务仍保留这一阻碍

简化审查（ponytail-review）：Lean already. Ship.
