import { searchIndexAsset } from '../../tools/wiki-search.ts'
import type { MarkdownOptions } from 'vitepress'
import type { WikiProfile } from '../../tools/wiki-data.ts'
import { defineConfig } from 'vitepress'
import { isAbsolute, join, resolve } from 'node:path'
import { existsSync, realpathSync } from 'node:fs'
import { buildSidebar } from './sidebar.ts'
import { copyWikiData, wikiDataPlugin, within, wikiOutput } from '../../tools/wiki-data.ts'

export const markdown: MarkdownOptions = {
  config(md) {
    const renderCode = md.renderer.rules.code_inline
    md.renderer.rules.code_inline = (tokens, idx, options, env, self) => {
      tokens[idx].attrSet('v-pre', '')
      return renderCode!(tokens, idx, options, env, self)
    }
  },
}

// Both build and dev use a prepared audience tree; invoking VitePress on docs is unsupported
export function wikiConfig({ root, source, profile }: { root: string; source: string; profile: WikiProfile }) {
  const allowed = [source, realpathSync(join(root, 'node_modules'))]
  return defineConfig({
    lang: 'zh-CN', title: 'N:SIDE',
    description: profile === 'player' ? 'Null City 的世界、人物与故事' : 'N:SIDE 项目设计与开发手册',
    base: '/', cleanUrls: true, markdown,
    outDir: wikiOutput(root, profile), cacheDir: resolve(source, '../cache'),
    srcExclude: ['_components/**', '_tools/**', '_data/**'],
    head: [
      ['link', { rel: 'icon', href: '/favicon.ico', sizes: '16x16 32x32 48x48' }],
      ['link', { rel: 'icon', href: '/project-assets/branding/n-logo.svg', type: 'image/svg+xml' }],
      ['link', { rel: 'apple-touch-icon', href: '/apple-touch-icon.png', sizes: '180x180' }],
    ],
    themeConfig: {
      logo: '/project-assets/branding/n-logo.svg', siteTitle: `N:SIDE ${profile === 'player' ? '百科' : '开发'}`,
      nav: [
        { text: '游戏百科', link: '/player/' },
        { text: '故事', link: '/player/story/' },
        ...(profile === 'dev' ? [{ text: '开发手册', link: '/dev/handbook/' }, { text: '开发资料', link: '/dev/' }] : []),
      ],
      sidebar: buildSidebar(source, profile), search: { provider: 'local' },
      outline: { level: [2, 3], label: '本页内容' },
      docFooter: { prev: '上一篇', next: '下一篇' }, returnToTopLabel: '回到顶部', sidebarMenuLabel: '目录', darkModeSwitchLabel: '外观',
    },
    vite: {
      build: { emptyOutDir: true },
      resolve: { alias: { '@wiki-components': join(source, '_components'), '@wiki-tools': join(source, '_tools'), '@wiki-data': join(source, '_data') } },
      plugins: [searchIndexAsset(), wikiDataPlugin(source, profile), {
        name: 'n-side-wiki-boundary',
        enforce: 'pre',
        load(id) {
          // VitePress 1.6 resolves these virtual modules with absolute-looking IDs
          if (['/@siteData', '/@localSearchIndex', '/@localSearchIndexroot'].includes(id)) return
          const path = id.split(/[?#]/, 1)[0]
          if (!isAbsolute(path)) return
          // The dev server resolves route IDs before converting them to Markdown modules
          const page = join(source, path)
          if ([page, `${page}.md`, join(page, 'index.md')].some(file => existsSync(file) && within(source, realpathSync(file)))) return
          const published = join(source, 'public', path)
          if (existsSync(published) && within(join(source, 'public'), realpathSync(published))) return
          const actual = existsSync(path) ? realpathSync(path) : path
          if (!allowed.some(directory => within(directory, actual))) throw new Error(`Unpublished Wiki module: ${id}`)
        },
        // VitePress adds the workspace root; replace that default after config merging
        configResolved(config) { config.server.fs.strict = true; config.server.fs.allow = allowed },
        configureServer(server) {
          server.middlewares.use((req, res, next) => {
            let url
            try { url = decodeURIComponent(new URL(req.url ?? '/', 'http://localhost').pathname) }
            catch { res.statusCode = 400; res.end('Invalid URL'); return }
            if (profile === 'player' && (/^\/(?:dev|production|templates|narrative|quests)(?:\/|$)/.test(url) || url.includes('district-map/district.json'))) {
              res.statusCode = 404; res.end('Not published'); return
            }
            next()
          })
        },
      }],
    },
    buildEnd(config) { copyWikiData(source, config.outDir, profile) },
  })
}

export default () => { throw new Error('Choose a Wiki audience: bun tools/wiki.ts build|dev player|dev') }
