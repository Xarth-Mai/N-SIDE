# R2 双样板与发布边界

《最后的玩具》先在现有路径完成职责分离：百科完整故事保留并标记结局，开发 README 改为引用与制作入口，当前验收和下一步移入本次证据。可玩行为、信息顺序、四种调查路径和 narrative.json 没有改变；`python3 -B tools/validate_narrative.py --root .` PASS，1 文件、22 个设计状态

资产样板见 asset-sample.md：先读源图、检查原件与派生 hash、运行修复后的图片工具、生成并实际查看三底色预览，10 个现有招牌/橱窗导出检查通过，没有改动源图与运行图

发布工具使用独立 output/wiki/player 与 output/wiki/dev 源投影、缓存和产物。8 项 Bun 窄测通过，覆盖原目录之外的数据、public 附件、Markdown/import 入口、地图字段投影和实际模拟构建产物中的污染负例；真实 VitePress CLI 另通过最小 player 2 页 / dev 4 页构建，检查实际搜索与数据产物；完整仓库构建尚未在本阶段执行

为避免同一切换点混入全部文档移动，本阶段先验收发布工具和样板，R3 路径落位后再执行完整双站构建与服务器请求。本记录不把导出 fixture 当成 VitePress 通过，不启用 ignoreDeadLinks

Ponytail 差异复核：复用现有 VitePress/Bun、场景绘图与图片导出器，无新框架、数据库或重复源资产。隔离源投影负责同一受众规则进入构建与开发服务器，属于本次要求的真实发布边界

Lean already. Ship.
