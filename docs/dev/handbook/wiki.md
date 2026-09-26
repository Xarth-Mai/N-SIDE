# Wiki 制作与发布

玩家站发布 `docs/player/` 的游玩指南与完整游戏百科；开发站发布玩家资料和 `docs/dev/` 的设计、工程、制作、验收与开发手册。两站共用 VitePress 配置，各自从隔离的资料快照构建页面、导航、搜索与附件

## 输入与前提

输入是所属受众下的 Markdown、开发数据和明确允许公开的图片，目录职责与身份字段遵循[文档规范](documentation.md)。先判断内容的权威位置：玩家百科负责世界事实、人物和故事，开发资料引用百科并补充规格，某轮结果和排障记录保存在 `todo/`

使用 `package.json` 和 `bun.lock` 中的工具版本，在仓库根目录执行命令。Bun 负责依赖安装、脚本与测试，安装和构建分开执行；自动构建从 package.json 读取 Bun 版本，以 frozen lockfile 安装一次后运行检查与双站构建。开发原始 JSON、CSV 不因正文没有链接就视为公开，页面中也不通过直接导入、包含文件或静态附件绕过受众边界

## 编辑与检查

1. 在 `docs/player/` 或 `docs/dev/` 修改权威来源，使用相对 Markdown 链接；玩家操作指南只记录实际可玩版本，百科可以完整描述设定与故事，并在相应入口提示剧透
2. 侧栏读取选中目录和页面一级标题；新页面必须从适合的主题入口可达，主题总览、分组和具体对象按内容需要分层
3. 运行文档检查与两种构建，修复正文死链和导出边界错误；生成目录中的页面是快照，不在其中编辑
4. 检查实际产物与预览，确认导航、检索、数据和旧 URL 对应正确受众

```fish
bun install --frozen-lockfile
bun run check:types
bun run check:docs
bun run docs:build
bun run docs:preview:player
```

只检查某一站时可运行 `bun run docs:build:player` 或 `bun run docs:build:dev`，产物分别位于 `docs/.vitepress/dist-player` 与 `docs/.vitepress/dist`。`bun tools/wiki.ts check player` 和 `bun tools/wiki.ts check dev` 只读检查已有产物，不重新构建

## 类型检查与版本

自有工具、测试、Wiki 配置使用 TypeScript，Vue 与页面脚本使用 `lang="ts"`。`tsconfig.json` 严格检查工具与测试，`tsconfig.vue.json` 检查组件；两者不输出编译文件，历史证据、第三方 Skills 和构建快照不参与迁移

`bun run check:types` 使用当前 TypeScript 7 检查工具，通过 `tools/check-vue.ts` 调用 vue-tsc 与微软的 TypeScript 6 兼容 API 检查组件。vue-tsc 的 require 钩子需要 Node；Bun 负责启动，不能给该子进程强制添加 `--bun`。反向测试以真实 SFC 中的类型错误验证检查器没有静默跳过文件

直接依赖选用 npm `latest` 对应的正式版本，并将实际版本和传递依赖写入 bun.lock。Vue 兼容 API 是独立的 `@typescript/typescript6` 最新包，不将项目 TypeScript 降级；传递依赖遵守上游声明范围。升级后复验类型检查、错误反例、发布边界和双站构建

## 开发预览

```fish
bun run docs:dev:player
bun run docs:dev:dev
```

两条命令选择其一运行；默认 `bun run docs:dev` 选择开发站。开发与正式构建均由 `tools/wiki.ts` 生成独立来源目录，开发服务器只读取该快照和已安装依赖。编辑原始文档后重启命令即可刷新快照；不把原始 `docs/` 作为服务根目录

`docs:preview:player` 与 `docs:preview:dev` 先检查对应的已构建产物再启动预览。需要两个预览同时运行时，可以给第二个命令指定不同端口，例如 `bun tools/wiki.ts preview dev 4174`

## 数据、地图与附件边界

`tools/wiki-data.ts` 仅为开发站导出 `docs/dev/` 下的 JSON、CSV，并保留相对路径；玩家站没有通用原始数据导出。数据的内容权威仍是现有源文件，Wiki 快照不成为第二份编辑源

玩家地图从 `source-assets/district-map/district.json` 生成明确投影：街坊名称、场所 `id/name/group/position/use/entry/page`，以及现有 `buildScene` 生成的绘图图元。页面 JS 只内嵌地点文字与交互索引，组件挂载后读取公开的 map.json 绘图；等待期间保留地图空间与地点列表，失败显示重试入口，离开页面中止请求。交互组件使用该投影完成原有浏览与场所选择，不向玩家打包建筑规格、制作备注、任务状态等完整源数据。开发站可以读取完整地图源

`docs/public/` 不整体复制。`tools/wiki.ts` 的 `PUBLIC_ASSETS` 明确允许文件和受众，并且只复制被选中正文引用的图片；Logo 与地图投影有单独的固定导出入口。新增静态图片时先审查其内容与受众，再登记允许项，运行边界测试并检查实际产物

页面脚本只使用已列出的 Vue、VitePress、地图数据和项目组件入口；Vite 最终模块加载再次核对隔离目录。跨受众 Markdown 引用、任意源文件导入、glob 导入以及 VitePress 的外部 include/snippet 指令会明确失败

源文档身份与设计 metadata 保留在仓库，玩家页面快照去除 frontmatter。百科直接位于 `docs/player/` 下的 world、characters、locations、story、enemies、gameplay；guide 保留游玩指南。旧 `player/encyclopedia/` URL 根据当前百科页面自动生成保留锚点的跳转，更早的玩家 URL 由迁移索引生成无正文重定向，保留锚点；旧开发 URL 仅进入开发站。它们是兼容入口，不复制旧正文或加入第二套内容维护

## 搜索资源

保留 VitePress 的全文检索与默认分包。生产构建将根语言的 MiniSearch 索引输出为带内容哈希的独立 JSON，打开搜索时读取；开发模式继续使用 VitePress 原有索引与热更新。`tools/wiki-search.ts` 只适配固定版本的搜索虚拟模块，接口变化时构建明确失败，升级依赖时同时复验

500 kB 警告保留默认阈值。把索引移出 JS 减少解析和转义开销，不表示索引正文消失；体积对比同时记录独立 JSON 与 gzip 大小。产物检查核对索引中的页面路径属于所选受众

## 静态服务接入

先确认站点现有内容范围：同时承载百科和开发文档的完整 Wiki 使用 `dev` 构建，只有明确需要独立玩家站时才使用 `player`。主页自动复用对应资料入口的项目介绍和导航，完整站同时包含两类内容

静态服务的 document root 使用完整站产物 `docs/.vitepress/dist`，独立玩家站使用 `docs/.vitepress/dist-player`。以服务运行用户检查父目录可遍历、公开产物可读取，干净构建后重新检查访问权限

切换路径前保留原配置，校验后平滑重载。核对主页项目介绍、百科与开发导航、原有入口及其实际目标、脚本和样式；发布记录保存在任务证据中

## Cloudflare Workers 发布

仓库根目录的 `wrangler.jsonc` 指定完整 Wiki 产物 `docs/.vitepress/dist`。Workers Builds 的构建命令使用 `bun run docs:build`，部署命令使用 `bun run docs:deploy`，根目录保持仓库根；部署脚本用 Bun 按固定的 Wrangler 4.141.0 版本运行，Wrangler 只发布已有产物，不再重复构建。404 使用生成的错误页，正常页面保留无扩展名与目录入口

本地先构建，再运行 `bun run docs:deploy --dry-run` 检查部署配置；去掉 `--dry-run` 才会发布，需要对应账号权限

## 输出与失败处理

输出包括两站独立构建包、各自的 `manifest.json` 和通过的产物检查。`tools/tests/wiki.test.ts` 覆盖缺少入口、错误受众、导入路径穿越、外部包含、public 附件、地图投影、意外输出与模块加载边界；实际开发服务器的 HTTP 验证按[运行验证](../validation/runtime.md)所需的证据范围记录到当前任务

发现 unpublished reference/import 时，先核对内容应属于哪个受众并修正来源或引用；需要公开的新资料要明确登记允许范围。发现死链时修正文档目标，不关闭 VitePress 的死链检查。产物出现开发内容时停止交付，修复相应入口后重新构建并检查输出

构建包位于已忽略的 `docs/.vitepress/dist` 与 `docs/.vitepress/dist-player`，开发快照与缓存继续隔离在 `docs/.vitepress/cache/wiki/`，一次性命令结果与验收证据归 `todo/`。构建成功证明文档发布路径可用，不代表游戏功能完成，也不代替画面检查或玩家体验验收
