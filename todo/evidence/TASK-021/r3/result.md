# 完整站点恢复与大共梦事件独立页

## 原因与纠正

第 2 轮错误地将原本包含百科与开发资料的站点切换为 player 受众，导致开发文档不再对该站可见；线上检查还把开发页面返回 404 当作通过，未验证用户原有完整站点的范围。这是发布选择与验收范围的错误，开发文档源码没有丢失

另外，工具链此前为根首页只生成一条进入链接，丢掉了项目介绍与主题导航。现已复用 player/index.md 与 dev/index.md 的权威内容生成完整主页，并将相对链接转换为根路径可用链接。独立玩家构建仍只包含玩家介绍和导航，完整构建包含两类入口

Caddy 已改为提供完整 dev 构建的独立快照，根目录使用 `output/wiki/published` 软链接，当前指向 `releases/full-20260926-190110`，上一份完整快照保留。构建目录与服务目录分离，后续构建不会清空正在提供的网页；快照准备完成并核对服务用户可读取后原子切换

## 大共梦事件

新增 `docs/player/encyclopedia/world/great-shared-dream.md`，收录事件现象、季闻与许遥的共同生活、持久梦境与旧日回放、三个条件相遇、独立验证与源头终止，以及今天的新联动

从原 history.md 迁移的 25 段事件正文逐段核对完整保留，[拆分记录](split.json)保存原文 hash。原页保留早期历史、兄妹成长、小店前史及相对时间线，并链接事件独立页。世界侧栏、百科首页、故事入口、Null City、两位当事人人物页和相关开发引用均已更新

## 验证

- `bun test tools/tests/wiki.test.ts`：11 项 PASS，新增根首页保留双类入口、玩家首页不混入开发入口、首页相对链接正确解析的回归检查
- `bun run check:types`：PASS，工具与 Vue 类型检查
- `bun run check:story-design`：PASS，18 条故事关系与原有对象目录
- `bun run check:docs`：PASS，248 篇 Markdown、84 个 ID 与 31 个 Skill
- `bun run docs:build`：PASS，玩家站 89 页、完整站 145 页；发布手册更新后完整站再构建 PASS
- `python3 todo/evidence/TASK-021/r3/check-full-site.py https://ms.lzzz.ink:7777`：432 项真实 HTTPS 检查 PASS，详见 [线上结果](live.json)
- 检查覆盖完整主页、145 个当前页面、125 个旧 URL、156 个脚本／样式／字体资源、原有无扩展名入口与开发 catalog；不再用单个新增功能可见代替完整站点验收
- `bun run tasks:sync`、`bun run tasks:check`、`git diff --check`：PASS
- ponytail-review：Lean already. Ship.

完整主页、开发资料与新独立页均已在线上生效。浏览器真实画面与交互仍未补齐，HTTP 通过只证明内容、链接和资源可访问，TASK-021 不据此宣告视觉验收完成
