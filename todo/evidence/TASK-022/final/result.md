# 故事整合与命名迁移交付

TASK-022 文档整合验收 PASS，记录日期 2026-09-26。输入及逐文件 hash 见 `checked-input.json`，本轮证据与任务状态本身不进入内容 hash。基线为 `bbee921b284d30dedf20a4c62e9f2f160a61800e`，前两批提交为 `3083639`、`998fd27`；最后一批提交从本记录的 Git 历史追溯

## 完成范围

新命名覆盖作者输入，河流仍为白沙河。37 位人物、91 个场所、十条主线与十二条可选街坊故事进入既有百科和单一关系目录；未创建另一套活动草稿。十六条铺垫、各案结束状态、六段场景对白和终章十四个 Beat 分别归入开发规格

完整对应见 `source-coverage.md`；旧名与新名、地点归属见 `../naming/mapping.json`。旧 35 人的 ID、年龄、住处与场所关联保留；QST-101—108 保留，新增 QST-019—022。地图 22 项展示字段发生变化，几何、数值、对象 ID 和拓扑不变。旧 URL 可跳转到新页并保留 fragment

## 实际执行

以下命令从仓库根执行，可直接用于 fish

```fish
bun run check:docs
bun run check:story-design
bun run check:narrative
bun run check:types
bun run test:tools
bun run docs:build
bun tools/wiki.ts check player
bun tools/wiki.ts check dev
python3 todo/evidence/TASK-022/final/verify_migration.py
bun run tasks:sync
bun run tasks:check
git diff --check
```

| 检查 | 实际结果与边界 |
| --- | --- |
| 文档、Skills 与链接 | PASS，266 Markdown、90 ID、31 Skills；Skills 没有本轮修改 |
| 故事与场所 | PASS，91 场所、12 街坊、37 人物、22 故事、64 关联建筑；条件、来源、可达和可选支线边界通过 |
| 运行叙事数据 | PASS，1 份文件、22 个信息状态；基线语义比较只有展示名称变化，不代表运行实现 |
| TypeScript 与 Vue | PASS |
| 工具回归 | PASS，103 项 Python 与 97 项 Bun、221 个 expect；最后审查修正后重跑，见 `tests.log` |
| 双站完整构建 | PASS，玩家 95 页／547 文件，开发 156 页／782 文件；见 `build.log` |
| 实际发布产物检查 | PASS，528 个玩家 HTML／JS／JSON／CSV 中旧名零残留，37 个名字和 91 处地图标签一致；玩家包未导出完整 district 源，见 `published-artifacts.json` |
| 内容与身份不变量 | PASS，见 `semantic-checks.json` 与复现脚本；包括 16 条铺垫、14 个 Beat 和 6 段场景 |
| 真实地图解析 | PASS，首批执行 `cargo test --manifest-path game/Cargo.toml --lib world::map::tests::real_map_and_axis_contract -- --exact`，1 项通过；其后地图未再变化 |
| 任务管线与提交检查 | PASS，17 张卡片；生成稳定、check 只读，`git diff --check` 通过 |

首次故事检查发现正文新标题与目录来源锚点不一致，修正为真实标题并保留旧锚点；没有放宽检查器。最终独立审查发现的支线结果与人物关联问题已修复并复查，见 `review.md`。提交前简化审查：Lean already. Ship.

## 未执行与下一步

浏览器界面、地图拖动、搜索操作、桌面／手机时间轴交互 NOT RUN：当前 CUA 枚举没有可用浏览器，原始结果见 `../naming/browser.json`。自动构建和产物扫描不替代这些操作，TASK-020、TASK-021 继续保留真实界面验收缺口

游戏源码和运行资产未修改，没有安排新的 GPU、Windows、手柄或剧情分支实机验收。《最后的玩具》具体机制仍由 TASK-012 承接，Demo 不加入站前送行，不将本次文档完成计为玩法完成或叙事锁定

按本次授权提交后停止，不推送远端，不启动后续路线图工作。下次相关验收应先在可用浏览器补齐现有界面操作，再依据用户指定任务继续
