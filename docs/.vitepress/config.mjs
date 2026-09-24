import { defineConfig } from 'vitepress'
import { fileURLToPath } from 'node:url'
import { buildSidebar } from './sidebar.mjs'
import { copyWikiData, wikiDataPlugin } from '../../tools/wiki-data.mjs'

const docsRoot = fileURLToPath(new URL('../', import.meta.url))

export default defineConfig({
  lang: 'zh-CN',
  title: 'N:SIDE',
  description: 'Null City 的世界、人物、日常与制作知识。',
  base: '/',
  markdown: {
    config(md) {
      const renderCode = md.renderer.rules.code_inline
      md.renderer.rules.code_inline = (tokens, idx, options, env, self) => {
        tokens[idx].attrSet('v-pre', '')
        return renderCode(tokens, idx, options, env, self)
      }
    },
  },
  cleanUrls: true,
  themeConfig: {
    siteTitle: 'N:SIDE Wiki',
    nav: [
      { text: 'Overview', link: '/vision' },
      { text: 'Universe', link: '/world/null-city' },
      { text: 'Game', link: '/gameplay/daily-life' },
      { text: 'Development', link: '/production/workflow' },
    ],
    sidebar: buildSidebar(docsRoot),
    search: { provider: 'local' },
    outline: { level: [2, 3], label: '本页内容' },
    docFooter: { prev: '上一篇', next: '下一篇' },
    returnToTopLabel: '回到顶部',
    sidebarMenuLabel: '目录',
    darkModeSwitchLabel: '外观',
  },
  vite: { plugins: [wikiDataPlugin(docsRoot)] },
  buildEnd(config) {
    copyWikiData(docsRoot, config.outDir)
  },
})
