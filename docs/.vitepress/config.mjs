import { defineConfig } from 'vitepress'
import { fileURLToPath } from 'node:url'
import { buildSidebar } from './sidebar.mjs'
import { copyProjectAssets, copyWikiData, projectAssetsPlugin, wikiDataPlugin } from '../../tools/wiki-data.mjs'

const docsRoot = fileURLToPath(new URL('../', import.meta.url))
const projectAssets = [{
  source: fileURLToPath(new URL('../../source-assets/branding/n-logo.svg', import.meta.url)),
  path: 'branding/n-logo.svg',
}, {
  source: fileURLToPath(new URL('../../source-assets/district-map/district.json', import.meta.url)),
  path: 'district-map/district.json',
}]

export default defineConfig({
  lang: 'zh-CN',
  title: 'N:SIDE',
  description: 'Null City 的世界、人物、日常与制作知识。',
  base: '/',
  head: [['link', { rel: 'icon', href: '/project-assets/branding/n-logo.svg', type: 'image/svg+xml' }]],
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
    logo: '/project-assets/branding/n-logo.svg',
    siteTitle: 'N:SIDE Wiki',
    nav: [
      { text: 'Overview', link: '/' },
      { text: '世界', link: '/world/null-city' },
      { text: '人物', link: '/characters/family' },
      { text: '地点', link: '/locations/n-district' },
      { text: '故事', link: '/story/' },
      { text: '玩法', link: '/gameplay/controls' },
      { text: '开发', link: '/conventions' },
    ],
    sidebar: buildSidebar(docsRoot),
    search: { provider: 'local' },
    outline: { level: [2, 3], label: '本页内容' },
    docFooter: { prev: '上一篇', next: '下一篇' },
    returnToTopLabel: '回到顶部',
    sidebarMenuLabel: '目录',
    darkModeSwitchLabel: '外观',
  },
  vite: {
    build: { emptyOutDir: true },
    plugins: [wikiDataPlugin(docsRoot), projectAssetsPlugin(projectAssets)],
  },
  buildEnd(config) {
    copyWikiData(docsRoot, config.outDir)
    copyProjectAssets(projectAssets, config.outDir)
  },
})
