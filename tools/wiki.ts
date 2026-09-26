import { loadQuests, projectStories, validateStories, auditMarkdown } from './story-data.ts'
import type { District, PlayerMap } from './district-types.ts'
import type { WikiProfile } from './wiki-data.ts'
export type WikiManifest = { profile: WikiProfile; pages: string[]; data: string[]; public: string[]; aliases: {old: string; to: string}[] }
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, extname, join, relative, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import { buildScene } from './district-map.ts'
import { assertProfile, filesIn, listWikiData, within, wikiOutput } from './wiki-data.ts'

// Superseded page URLs remain redirects; names in current content have one canonical form
const RENAMED_PAGES: Record<string, string> = {
  'player/locations/n-district.md': 'player/locations/null-site.md',
  'player/characters/family-members/chen-jing-he.md': 'player/characters/family-members/chen-mei-hui.md',
  'player/characters/family-members/lin-qi-ming.md': 'player/characters/family-members/tsukishiro-shu.md',
  'player/characters/neighbors/chu-qing.md': 'player/characters/neighbors/robin.md',
  'player/characters/neighbors/gu-zhu.md': 'player/characters/neighbors/gu-wen-zhen.md',
  'player/characters/neighbors/he-qiu.md': 'player/characters/neighbors/he-jia.md',
  'player/characters/neighbors/he-xin.md': 'player/characters/neighbors/kevin.md',
  'player/characters/neighbors/jiang-tang.md': 'player/characters/neighbors/xiao-man.md',
  'player/characters/neighbors/lu-heng.md': 'player/characters/neighbors/lu-cheng.md',
  'player/characters/neighbors/ning-lang.md': 'player/characters/neighbors/yuma.md',
  'player/characters/neighbors/qiao-yin.md': 'player/characters/neighbors/joey.md',
  'player/characters/neighbors/qiu-ning.md': 'player/characters/neighbors/qiu-li.md',
  'player/characters/neighbors/shen-ling.md': 'player/characters/neighbors/nora.md',
  'player/characters/neighbors/song-lan.md': 'player/characters/neighbors/song-mei-xiang.md',
  'player/characters/neighbors/su-he.md': 'player/characters/neighbors/su-mi.md',
  'player/characters/neighbors/tang-yu.md': 'player/characters/neighbors/tang-hui.md',
  'player/characters/neighbors/wen-ran.md': 'player/characters/neighbors/emma.md',
  'player/characters/neighbors/yao-an.md': 'player/characters/neighbors/a-dong.md',
  'player/characters/neighbors/ye-chu.md': 'player/characters/neighbors/ye-zhen.md',
  'player/characters/neighbors/zhao-yan.md': 'player/characters/neighbors/a-jie.md',
  'player/characters/neighbors/zheng-bo.md': 'player/characters/neighbors/zheng-wen-liang.md',
  'player/characters/neighbors/zheng-che.md': 'player/characters/neighbors/zheng-tuo.md',
  'player/characters/neighbors/zhong-fan.md': 'player/characters/neighbors/lao-zhong.md',
  'player/characters/shared-dreams/ji-wen.md': 'player/characters/shared-dreams/wien.md',
  'player/characters/shared-dreams/xu-yao.md': 'player/characters/shared-dreams/haruka.md',
}
const REPOSITORY = 'https://github.com/Xarth-Mai/N-SIDE/blob/main/'
const SUPPORT = ['district-map.ts', 'district-plan.ts', 'district-architecture.ts', 'district-geometry.ts', 'district-types.ts', 'story-graph.ts']
const COMPONENTS = ['DistrictMap.vue', 'DistrictPlan.vue', 'DistrictArchitecture.vue', 'DistrictPlaces.vue', 'StoryGraph.vue', 'StoryCondition.vue']
const IMAGE = new Set(['.svg', '.webp', '.png', '.jpg', '.jpeg'])
const PUBLIC_ASSETS: Record<string, WikiProfile[]> = {
  'images/shop-street.webp': ['dev'],
  'images/station-street.webp': ['dev'],
  'images/cinema-music-street.webp': ['dev'],
  'images/riverside.webp': ['dev'],
}
const put = (path: string, text: string) => { mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, text) }
const copy = (source: string, target: string) => { mkdirSync(dirname(target), { recursive: true }); copyFileSync(source, target); chmodSync(target, 0o644) }
const route = (path: string) => `/${path.replace(/\.md$/, '').replace(/(^|\/)index$/, '$1')}`

/** Allow only the map's already visible labels and generated drawing primitives */
export function playerMap(data: District): PlayerMap {
  const scene = buildScene(data)
  return {
    groups: Object.fromEntries(Object.entries(data.groups).map(([id, name]) => [id, String(name)])),
    places: data.places.map(p => ({ id: p.id, name: p.name, group: p.group, position: p.position,
      use: p.use, entry: p.entry, ...(p.page ? { page: p.page.replace('/locations/', '/player/locations/') } : {}) })),
    scene,
  }
}

function redirect(to: string) {
  return `<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="robots" content="noindex"><meta http-equiv="refresh" content="0;url=${to}"><link rel="canonical" href="${to}"></head><body><a href="${to}">页面已迁移</a><script>location.replace(${JSON.stringify(to)} + location.hash)</script></body></html>\n`
}

/** Prepare an isolated document tree shared by production builds and the dev server */
export function prepareWiki(root: string, profile: string) {
  assertProfile(profile)
  const audience = profile
  root = resolve(root)
  const docs = join(root, 'docs'), base = join(root, 'docs/.vitepress/cache/wiki', profile), source = join(base, 'source')
  const selected = ['player', ...(profile === 'dev' ? ['dev'] : [])]
  const pages = selected.flatMap(name => filesIn(join(docs, name)).filter(file => extname(file) === '.md'))
  if (!pages.length || !existsSync(join(docs, profile, 'index.md'))) throw new Error(`${profile}: missing entry or pages`)
  const quests = loadQuests(root)
  const storyErrors = validateStories(root, quests)
  if (storyErrors.length) throw new Error(storyErrors.join('\n'))
  const stories = projectStories(root, quests)
  rmSync(source, { recursive: true, force: true })
  mkdirSync(source, { recursive: true })
  const publicFiles = new Set<string>(), pageSet = new Set(pages.map(file => resolve(file)))
  function link(url: string, file: string) {
    if (/^(?:https?:|mailto:|#|data:)/.test(url)) return url
    const [name, fragment = ''] = url.split('#', 2)
    if (!name) return url
    const suffix = fragment ? `#${fragment}` : ''
    if (name.startsWith('/project-assets/')) {
      if (name !== '/project-assets/branding/n-logo.svg' && name !== '/project-assets/district-map/map.json' && !(profile === 'dev' && name === '/project-assets/district-map/district.json')) throw new Error(`${relative(root, file)}: unpublished project asset ${name}`)
      return url
    }
    const target = resolve(name.startsWith('/') ? docs : dirname(file), name.replace(/^\//, ''))
    const rel = relative(docs, target)
    if (within(join(docs, 'public'), target)) {
      if (!IMAGE.has(extname(target)) || !existsSync(target) || !PUBLIC_ASSETS[relative(join(docs, 'public'), target)]?.includes(audience)) throw new Error(`Unsupported public attachment: ${url}`)
      publicFiles.add(target)
      return `/${relative(join(docs, 'public'), target)}${suffix}`
    }
    if (!within(docs, target)) {
      if (profile === 'player' || !within(root, target)) throw new Error(`${relative(root, file)}: reference outside Wiki audience: ${url}`)
      return REPOSITORY + relative(root, target) + suffix
    }
    const markdown = extname(target) === '.md' ? target : `${target}.md`
    const landing = join(target, 'index.md')
    if (target === docs || markdown === join(docs, 'index.md')) return `/${suffix}`
    if (pageSet.has(target) || pageSet.has(markdown) || pageSet.has(landing)) return url
    if (profile === 'dev' && listWikiData(docs, profile).includes(target)) return url
    throw new Error(`${relative(root, file)}: unpublished local reference: ${url} (${rel})`)
  }
  for (const file of pages) {
    let text = readFileSync(file, 'utf8')
    if (/<!--\s*@include\s*:|^\s*<<<(?:\s|$)/m.test(text)) throw new Error(`${relative(root, file)}: external include/snippet inputs are not published`)
    // Source identity and design metadata remain in the repository, not player page payloads
    if (relative(docs, file).startsWith('player/')) text = text.replace(/^---\r?\n[\s\S]*?\r?\n---(?:\r?\n|$)/, '')
    text = text.replace(/(!?\[[^\]\n]*\]\()([^\s)]+)(\))/g, (_all, prefix, url, end) => `${prefix}${link(url, file)}${end}`)
      .replace(/((?:src|href)=["'])([^"']+)(["'])/g, (_all, prefix, url, end) => `${prefix}${link(url, file)}${end}`)
      .replace(/(['"])([^'"\n]*\.vitepress\/components\/([^/'"]+))\1/g, (_all, quote, _old, name) => {
        if (!COMPONENTS.includes(name) || (profile === 'player' && !['DistrictMap.vue', 'StoryGraph.vue', 'StoryCondition.vue'].includes(name))) throw new Error(`Unpublished component ${name}`)
        return `${quote}@wiki-components/${name}${quote}`
      })
      .replace(/(['"])[^'"\n]*source-assets\/district-map\/district.json\1/g, () => {
        if (profile === 'player') throw new Error('Player Markdown cannot import the full district source')
        return "'@wiki-data/district.json'"
      })
      .replace(/(['"])[^'"\n]*tools\/(district-[^/'"]+\.ts)\1/g, (_all, quote, name) => {
        if (!SUPPORT.includes(name) || profile === 'player') throw new Error(`Unpublished tool import ${name}`)
        return `${quote}@wiki-tools/${name}${quote}`
      })
    // Source imports are a separate publication channel from Markdown hyperlinks
    const imports = new Set(['vue', 'vitepress', '@wiki-data/map.json', '@wiki-components/DistrictMap.vue', '@wiki-components/StoryGraph.vue',
      ...(profile === 'dev' ? ['@wiki-data/district.json', ...SUPPORT.map(n => `@wiki-tools/${n}`), ...COMPONENTS.map(n => `@wiki-components/${n}`)] : [])])
    if (/import\.meta\.glob/.test(text)) throw new Error(`${relative(root, file)}: glob imports are not publication inputs`)
    for (const match of text.matchAll(/\b(?:from\s*|import\s*(?:\(\s*)?)(['"])([^'"\n]+)\1/g)) {
      if (!imports.has(match[2])) throw new Error(`${relative(root, file)}: unpublished import ${match[2]}`)
    }
    const story = stories.find(n => n.url === '/' + relative(docs, file).replace(/\.md$/, ''))
    const directory = 'player/story/'
    const rel = relative(docs, file)
    if (story || [directory+'index.md', directory+'main/index.md', directory+'daily/index.md'].includes(rel)) {
      const attr = story ? `quest-id="${story.id}"` : rel === directory+'main/index.md' ? 'track="main"' : rel === directory+'daily/index.md' ? 'track="side"' : ''
      text += `\n<script setup>\nimport StoryGraph from '@wiki-components/StoryGraph.vue'\n</script>\n\n<StoryGraph ${attr} />\n`
    }
    if (rel === 'dev/design/story-graph.md') text += '\n## 逐篇关系审查（由 catalog 生成）\n\n' + auditMarkdown(root)
    put(join(source, relative(docs, file)), text)
  }
  for (const file of listWikiData(docs, profile)) copy(file, join(source, relative(docs, file)))
  for (const file of publicFiles) copy(file, join(source, 'public', relative(join(docs, 'public'), file)))
  for (const name of ['favicon.ico', 'apple-touch-icon.png']) copy(join(docs, 'public', name), join(source, 'public', name))
  copy(join(root, 'source-assets/branding/n-logo.svg'), join(source, 'public/project-assets/branding/n-logo.svg'))
  const district = JSON.parse(readFileSync(join(root, 'source-assets/district-map/district.json'), 'utf8'))
  const map = playerMap(district)
  const projected = JSON.stringify(map) + '\n'
  const { scene: _scene, ...labels } = map
  put(join(source, '_data/stories.json'), JSON.stringify(stories) + '\n')
  put(join(source, '_data/map.json'), JSON.stringify(labels) + '\n')
  put(join(source, 'public/project-assets/district-map/map.json'), projected)
  if (profile === 'dev') {
    copy(join(root, 'source-assets/district-map/district.json'), join(source, '_data/district.json'))
    copy(join(root, 'source-assets/district-map/district.json'), join(source, 'public/project-assets/district-map/district.json'))
  }
  for (const name of SUPPORT.filter(name => profile === 'dev' || ['district-map.ts', 'district-types.ts', 'story-graph.ts'].includes(name))) copy(join(root, 'tools', name), join(source, '_tools', name))
  for (const name of COMPONENTS.filter(name => profile === 'dev' || ['DistrictMap.vue', 'StoryGraph.vue', 'StoryCondition.vue'].includes(name))) {
    const text = readFileSync(join(docs, '.vitepress/components', name), 'utf8')
      .replaceAll('../../../source-assets/district-map/district.json', '@wiki-data/district.json')
      .replaceAll('../../../tools/', '@wiki-tools/')
    put(join(source, '_components', name), text)
  }
  const mapping = join(root, 'todo/evidence/TASK-007/r1/migration-map.json')
  const aliases = []
  if (existsSync(mapping)) for (const entry of JSON.parse(readFileSync(mapping, 'utf8')).files) {
    if (entry.source === 'docs/index.md' || !entry.source.startsWith('docs/') || !entry.source.endsWith('.md')) continue
    const target = entry.targets.map((t: string) => {
      const page = t.replace(/^docs\//, '').replace('player/encyclopedia/', 'player/')
      return 'docs/' + (RENAMED_PAGES[page] ?? page)
    }).find((t: string) => t.startsWith('docs/player/') || (profile === 'dev' && t.startsWith('docs/dev/')))
    if (!target || target === entry.source || !pageSet.has(resolve(root, target))) continue
    const old = entry.source.slice(5).replace(/\.md$/, '.html'), to = route(target.slice(5))
    put(join(source, 'public', old), redirect(to)); aliases.push({ old, to })
  }
  for (const [previous, current] of Object.entries(RENAMED_PAGES)) {
    if (!pageSet.has(resolve(docs, current))) continue
    for (const path of [previous, previous.replace('player/', 'player/encyclopedia/')]) {
      const old = path.replace(/\.md$/, '.html'), to = route(current)
      put(join(source, 'public', old), redirect(to)); aliases.push({ old, to })
    }
  }
  for (const page of pages) {
    const name = relative(docs, page)
    if (!/^player\/(world|characters|locations|story|enemies|gameplay)\//.test(name)) continue
    const old = name.replace('player/', 'player/encyclopedia/').replace(/\.md$/, '.html'), to = route(name)
    put(join(source, 'public', old), redirect(to)); aliases.push({ old, to })
  }
  // Reuse audience landing content so the real homepage retains the overview and navigation
  const homeSections = selected.map(name => {
    const file = join(docs, name, 'index.md')
    return readFileSync(join(source, name, 'index.md'), 'utf8')
      .replace(/^(#{1,5}) /gm, '$1# ')
      .replace(/(!?\[[^\]\n]*\]\()([^\s)]+)(\))/g, (_all, prefix, url, end) => {
        const checked = link(url, file)
        if (/^(?:https?:|mailto:|#|data:|\/)/.test(checked)) return `${prefix}${checked}${end}`
        const [path, fragment] = checked.split('#', 2)
        return `${prefix}/${relative(docs, resolve(dirname(file), path))}${fragment ? '#'+fragment : ''}${end}`
      })
  })
  put(join(source, 'index.md'), '# N:SIDE Wiki\n\n<img src="/project-assets/branding/n-logo.svg" alt="N:SIDE Logo" width="160">\n\n' + homeSections.join('\n\n'))
  put(join(source, '.vitepress/config.ts'), `import { wikiConfig } from ${JSON.stringify(pathToFileURL(join(docs, '.vitepress/config.ts')).href)}\nexport default wikiConfig(${JSON.stringify({ root, source, profile })})\n`)
  const manifest = { profile, pages: pages.map(p => relative(docs, p)), data: listWikiData(docs, profile).map(p => relative(docs, p)), public: filesIn(join(source, 'public')).map(p => relative(join(source, 'public'), p)), aliases }
  put(join(base, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n')
  return { source, output: wikiOutput(root, profile), manifest }
}

/** Check actual emitted pages and all raw data, not just sidebar configuration */
export function checkWikiBuild(root: string, profile: string) {
  assertProfile(profile)
  const base = join(resolve(root), 'docs/.vitepress/cache/wiki', profile), output = wikiOutput(root, profile)
  const manifest: WikiManifest = JSON.parse(readFileSync(join(base, 'manifest.json'), 'utf8'))
  if (manifest.profile !== profile) throw new Error('Build manifest profile mismatch')
  const files = filesIn(output), names = files.map(p => relative(output, p))
  for (const name of manifest.pages.map(p => p.replace(/\.md$/, '.html'))) if (!names.includes(name)) throw new Error(`Missing built page ${name}`)
  const raw = new Set([...manifest.data, ...manifest.public, 'hashmap.json'])
  const html = new Set(['index.html', '404.html', ...manifest.pages.map(p => p.replace(/\.md$/, '.html')), ...manifest.public.filter(p => p.endsWith('.html'))])
  for (const name of names) {
    if (/^assets\/search-index\.[\w-]+\.json$/.test(name)) {
      const index: {documentCount: number; documentIds: Record<string, unknown>} = JSON.parse(readFileSync(join(output, name), 'utf8'))
      const routes = new Set(['/', ...manifest.pages.map(route)])
      if (!index.documentIds || Object.keys(index.documentIds).length !== index.documentCount || Object.values(index.documentIds).some(id => typeof id !== 'string' || !routes.has(id.split('#')[0]))) throw new Error(`Unpublished search document: ${name}`)
    }
    if (name.endsWith('.html') && !html.has(name)) throw new Error(`Unexpected published page: ${name}`)
    if (['.json', '.csv'].includes(extname(name)) && !raw.has(name) && !/^assets\/search-index\.[\w-]+\.json$/.test(name)) throw new Error(`Unexpected published data: ${name}`)
    if (profile === 'player' && name.startsWith('dev/')) throw new Error(`Developer file in player output: ${name}`)
  }
  if (profile === 'player') {
    const map = JSON.parse(readFileSync(join(output, 'project-assets/district-map/map.json'), 'utf8'))
    if (JSON.stringify(Object.keys(map).sort()) !== JSON.stringify(['groups', 'places', 'scene'])) throw new Error('Player map contains unexpected top-level fields')
    const fields = new Set(['id', 'name', 'group', 'position', 'use', 'entry', 'page'])
    for (const place of map.places) if (Object.keys(place).some(k => !fields.has(k))) throw new Error('Player map contains a design field')
    for (const file of files.filter(p => ['.html', '.js', '.json'].includes(extname(p)))) {
      const text = readFileSync(file, 'utf8')
      if (/QST-002-B001|resolution_committed|DOCS-PIPELINE|docs\/dev\/|dev\/design\/catalogs|district-map\/district\.json/.test(text)) throw new Error(`Developer content in player artifact: ${relative(output, file)}`)
    }
  }
  return { profile, pages: manifest.pages.length, files: files.length, result: 'PASS' }
}

if (import.meta.main) {
  const [command, profile, ...extra] = process.argv.slice(2)
  try {
    assertProfile(profile)
    const root = resolve(import.meta.dir, '..')
    if (command === 'check') console.log(JSON.stringify(checkWikiBuild(root, profile)))
    else if (['build', 'dev', 'prepare', 'preview'].includes(command)) {
      if (command === 'preview') checkWikiBuild(root, profile)
      const prepared = command === 'preview' ? { source: join(root, 'docs/.vitepress/cache/wiki', profile, 'source') } : prepareWiki(root, profile)
      if (command === 'prepare') console.log(JSON.stringify('manifest' in prepared ? prepared.manifest : undefined))
      else {
        // Use the installed VitePress CLI: its esbuild service fails under this Bun runtime
        const args = ['bun', 'run', 'vitepress', command, prepared.source]
        if (command !== 'build') {
          args.push('--host', '127.0.0.1', '--port', extra[0] ?? (command === 'dev' ? '5173' : '4173'), '--strictPort')
          console.log('Preview uses an isolated snapshot; restart this command after editing source documents')
        }
        const child = Bun.spawn(args, { cwd: root, stdin: 'inherit', stdout: 'inherit', stderr: 'inherit' })
        const code = await child.exited
        if (code !== 0) throw new Error(`VitePress ${command} failed (${code})`)
        if (command === 'build') console.log(JSON.stringify(checkWikiBuild(root, profile)))
      }
    } else throw new Error('Usage: bun tools/wiki.ts build|dev|prepare|preview|check player|dev [port]')
  } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1 }
}
