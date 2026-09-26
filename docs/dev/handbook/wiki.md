# Wiki 制作与发布

玩家站发布 `docs/player/` 的游玩指南与完整游戏百科；开发站发布玩家资料和 `docs/dev/` 的设计、工程、制作、验收与开发手册。两站共用 VitePress 配置，各自从隔离的资料快照构建页面、导航、搜索与附件

## 输入与前提

输入是所属受众下的 Markdown、开发数据和明确允许公开的图片，目录职责与身份字段遵循[文档规范](documentation.md)。先判断内容的权威位置：玩家百科负责世界事实、人物和故事，开发资料引用百科并补充规格，某轮结果和排障记录保存在 `todo/`

使用 `package.json` 和 `bun.lock` 中的工具版本，在仓库根目录执行命令。开发原始 JSON、CSV 不因正文没有链接就视为公开，页面中也不通过直接导入、包含文件或静态附件绕过受众边界

## 编辑与检查

1. 在 `docs/player/` 或 `docs/dev/` 修改权威来源，使用相对 Markdown 链接；玩家操作指南只记录实际可玩版本，百科可以完整描述设定与故事，并在相应入口提示剧透
2. 侧栏读取选中目录和页面一级标题；新页面必须从适合的主题入口可达，主题总览、分组和具体对象按内容需要分层
3. 运行文档检查与两种构建，修复正文死链和导出边界错误；生成目录中的页面是快照，不在其中编辑
4. 检查实际产物与预览，确认导航、检索、数据和旧 URL 对应正确受众

```fish
bun install --frozen-lockfile
bun run check:docs
bun run docs:build
bun run docs:preview:player
```

只检查某一站时可运行 `bun run docs:build:player` 或 `bun run docs:build:dev`，产物分别位于 `output/wiki/player/dist` 与 `output/wiki/dev/dist`。`bun tools/wiki.mjs check player` 和 `bun tools/wiki.mjs check dev` 只读检查已有产物，不重新构建

## 开发预览

```fish
bun run docs:dev:player
bun run docs:dev:dev
```

两条命令选择其一运行；默认 `bun run docs:dev` 选择开发站。开发与正式构建均由 `tools/wiki.mjs` 生成独立来源目录，开发服务器只读取该快照和已安装依赖。编辑原始文档后重启命令即可刷新快照；不把原始 `docs/` 作为服务根目录

`docs:preview:player` 与 `docs:preview:dev` 先检查对应的已构建产物再启动预览。需要两个预览同时运行时，可以给第二个命令指定不同端口，例如 `bun tools/wiki.mjs preview dev 4174`

## 数据、地图与附件边界

`tools/wiki-data.mjs` 仅为开发站导出 `docs/dev/` 下的 JSON、CSV，并保留相对路径；玩家站没有通用原始数据导出。数据的内容权威仍是现有源文件，Wiki 快照不成为第二份编辑源

玩家地图从 `source-assets/district-map/district.json` 生成明确投影：街坊名称、场所 `id/name/group/position/use/entry/page`，以及现有 `buildScene` 生成的绘图图元。交互组件使用该投影完成原有浏览与场所选择，不向玩家打包建筑规格、制作备注、任务状态等完整源数据。开发站可以读取完整地图源

`docs/public/` 不整体复制。`tools/wiki.mjs` 的 `PUBLIC_ASSETS` 明确允许文件和受众，并且只复制被选中正文引用的图片；Logo 与地图投影有单独的固定导出入口。新增静态图片时先审查其内容与受众，再登记允许项，运行边界测试并检查实际产物

页面脚本只使用已列出的 Vue、VitePress、地图数据和项目组件入口；Vite 最终模块加载再次核对隔离目录。跨受众 Markdown 引用、任意源文件导入、glob 导入以及 VitePress 的外部 include/snippet 指令会明确失败

源文档身份与设计 metadata 保留在仓库，玩家页面快照去除 frontmatter。旧玩家 URL 由迁移索引生成无正文重定向，保留锚点；旧开发 URL 仅进入开发站。它们是兼容入口，不复制旧正文或加入第二套内容维护

## 输出与失败处理

输出包括两站独立构建包、各自的 `manifest.json` 和通过的产物检查。`tools/tests/wiki.test.mjs` 覆盖缺少入口、错误受众、导入路径穿越、外部包含、public 附件、地图投影、意外输出与模块加载边界；实际开发服务器的 HTTP 验证按[运行验证](../validation/runtime.md)所需的证据范围记录到当前任务

发现 unpublished reference/import 时，先核对内容应属于哪个受众并修正来源或引用；需要公开的新资料要明确登记允许范围。发现死链时修正文档目标，不关闭 VitePress 的死链检查。产物出现开发内容时停止交付，修复相应入口后重新构建并检查输出

开发快照、构建包、缓存和临时预览均在已忽略的 `output/`，一次性命令结果与验收证据归 `todo/`。构建成功证明文档发布路径可用，不代表游戏功能完成，也不代替画面检查或玩家体验验收
