# Wiki

`docs/` 同时是项目知识源与 VitePress 的页面目录。首页为 `docs/index.md`，承载 Overview 与核心体验；内容按玩家与开发者的阅读需求组织，项目约定位于开发资料

## 内容规则

Overview 融合项目愿景与核心体验。游戏百科收录世界、人物、地点、故事、敌人与玩法，使用自然表述，直接说明设定、行动与结果；完整背景、章节和后续可以展开，故事入口标明包含结局。实现细节、创作要求、模板和检查项放入开发资料

按阅读层级组织主题总览、分组入口与具体内容，文档数量由内容决定。已纳入的设定归入所属主题维护，来源版本与接入检查留在任务证据中

Wiki 只记录跨机器、跨阶段仍然成立的项目知识。开发机器的主机名、绝对路径、端口、服务状态和本机配置不得进入 `docs/`；开发过程、排障过程、部署记录和一次性验证结果记录在 `todo/` 或对应的运维系统中

玩家正文不包含开发计划、制作备注、内部字段或验证要求；混有这些信息时，将其迁入对应开发文档，保留设计约束。发布态是写作视角，不代表功能已经实现，不补造尚未确定的操作、数值或内容

## 本地工作流程

在仓库根目录运行：

```sh
bun install
bun run docs:dev
```

构建与预览：

```sh
bun run docs:build
bun run docs:preview
```

构建输出：`docs/.vitepress/dist`

## 页面与资源

侧栏根据主题目录和 Markdown 一级标题生成，站内检索使用 VitePress 的 local search。正文、图片与数据的路径约定见[项目约定](../conventions.md)

JSON、CSV 保持源文件的相对路径，由 `tools/wiki-data.mjs` 提供开发访问与构建导出

## 工具文档

[VitePress](https://vuejs.github.io/vitepress/v1/guide/getting-started) · [Bun 锁文件](https://bun.sh/docs/pm/lockfile)
