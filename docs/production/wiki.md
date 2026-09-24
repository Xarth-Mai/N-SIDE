# Wiki

`docs/` 同时是项目知识源与 VitePress 的页面目录。首页为 `docs/index.md`，导航按 Overview、Universe、Game、Development 组织

## 本地工作

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
