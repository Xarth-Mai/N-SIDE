# Wiki

`docs/` 同时是项目知识源与 VitePress 的页面目录。首页为 `docs/index.md`，导航按 Overview、Universe、Game、Development 组织

## 本地工作

在仓库根目录运行：

```sh
bun install
bun run docs:dev
```

`bun install` 生成或更新 `bun.lock`，锁文件与 `package.json` 一起进入 Git。构建与预览使用：

```sh
bun run docs:build
bun run docs:preview
```

`docs:build` 先按锁文件安装依赖，再构建静态站点。输出目录为 `docs/.vitepress/dist`

## 本机发布

Wiki 访问地址为 [ms.lzzz.ink:7777](https://ms.lzzz.ink:7777/)，由本机 Caddy 提供 HTTPS 静态文件服务

构建目录的发布副本位于 `/srv/n-side-wiki`，站点配置维护在仓库 `tools/caddy/n-side-wiki.caddy`。内容更新后重新构建并同步发布副本，核验页面、图片与数据链接；配置变更通过 Caddy reload 生效

## 页面与资源

侧栏根据主题目录和 Markdown 一级标题生成，站内检索使用 VitePress 的 local search。正文、图片与数据的路径约定见[项目约定](../conventions.md)

图片从 `docs/public/images/` 发布到 `/images/`。JSON、CSV 文件由 `tools/wiki-data.mjs` 按源路径导出到构建目录，供正文中的相对链接访问。开发服务从同一份源文件提供这些数据

## 工具文档

[VitePress](https://vuejs.github.io/vitepress/v1/guide/getting-started) · [Bun 锁文件](https://bun.sh/docs/pm/lockfile)
