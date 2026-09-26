import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, extname, join, relative, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import { buildScene } from './district-map.mjs'
import { assertProfile, filesIn, listWikiData, within } from './wiki-data.mjs'

const REPOSITORY = 'https://github.com/Xarth-Mai/N-SIDE/blob/main/'
const SUPPORT = ['district-map.mjs', 'district-plan.mjs', 'district-architecture.mjs', 'district-geometry.mjs']
const COMPONENTS = ['DistrictMap.vue', 'DistrictPlan.vue', 'DistrictArchitecture.vue', 'DistrictPlaces.vue']
const IMAGE = new Set(['.svg', '.webp', '.png', '.jpg', '.jpeg'])
const PUBLIC_ASSETS = {
  'images/hero.webp': ['player', 'dev'],
  'images/hillside-reference.webp': ['dev'],
  'images/hillside-layout-concept.webp': ['dev'],
}
const put = (path, text) => { mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, text) }
const copy = (source, target) => { mkdirSync(dirname(target), { recursive: true }); copyFileSync(source, target) }
const route = path => `/${path.replace(/\.md$/, '').replace(/(^|\/)index$/, '$1')}`

/** Allow only the map's already visible labels and generated drawing primitives */
export function playerMap(data) {
  const scene = buildScene(data)
  return {
    groups: Object.fromEntries(Object.entries(data.groups).map(([id, name]) => [id, String(name)])),
    places: data.places.map(p => ({ id: p.id, name: p.name, group: p.group, position: p.position,
      use: p.use, entry: p.entry, ...(p.page ? { page: p.page.replace('/locations/', '/player/encyclopedia/locations/') } : {}) })),
    scene,
  }
}

function redirect(to) {
  return `<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="robots" content="noindex"><meta http-equiv="refresh" content="0;url=${to}"><link rel="canonical" href="${to}"></head><body><a href="${to}">页面已迁移</a><script>location.replace(${JSON.stringify(to)} + location.hash)</script></body></html>\n`
}

/** Prepare an isolated document tree shared by production builds and the dev server */
export function prepareWiki(root, profile) {
  assertProfile(profile)
  root = resolve(root)
  const docs = join(root, 'docs'), base = join(root, 'output/wiki', profile), source = join(base, 'source')
  const selected = ['player', ...(profile === 'dev' ? ['dev'] : [])]
  const pages = selected.flatMap(name => filesIn(join(docs, name)).filter(file => extname(file) === '.md'))
  if (!pages.length || !existsSync(join(docs, profile, 'index.md'))) throw new Error(`${profile}: missing entry or pages`)
  rmSync(source, { recursive: true, force: true })
  mkdirSync(source, { recursive: true })
  const publicFiles = new Set(), pageSet = new Set(pages.map(file => resolve(file)))
  function link(url, file) {
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
      if (!IMAGE.has(extname(target)) || !existsSync(target) || !PUBLIC_ASSETS[relative(join(docs, 'public'), target)]?.includes(profile)) throw new Error(`Unsupported public attachment: ${url}`)
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
        if (!COMPONENTS.includes(name) || (profile === 'player' && name !== 'DistrictMap.vue')) throw new Error(`Unpublished component ${name}`)
        return `${quote}@wiki-components/${name}${quote}`
      })
      .replace(/(['"])[^'"\n]*source-assets\/district-map\/district.json\1/g, () => {
        if (profile === 'player') throw new Error('Player Markdown cannot import the full district source')
        return "'@wiki-data/district.json'"
      })
      .replace(/(['"])[^'"\n]*tools\/(district-[^/'"]+\.mjs)\1/g, (_all, quote, name) => {
        if (!SUPPORT.includes(name) || profile === 'player') throw new Error(`Unpublished tool import ${name}`)
        return `${quote}@wiki-tools/${name}${quote}`
      })
    // Source imports are a separate publication channel from Markdown hyperlinks
    const imports = new Set(['vue', 'vitepress', '@wiki-data/map.json', '@wiki-components/DistrictMap.vue',
      ...(profile === 'dev' ? ['@wiki-data/district.json', ...SUPPORT.map(n => `@wiki-tools/${n}`), ...COMPONENTS.map(n => `@wiki-components/${n}`)] : [])])
    if (/import\.meta\.glob/.test(text)) throw new Error(`${relative(root, file)}: glob imports are not publication inputs`)
    for (const match of text.matchAll(/\b(?:from\s*|import\s*(?:\(\s*)?)(['"])([^'"\n]+)\1/g)) {
      if (!imports.has(match[2])) throw new Error(`${relative(root, file)}: unpublished import ${match[2]}`)
    }
    put(join(source, relative(docs, file)), text)
  }
  for (const file of listWikiData(docs, profile)) copy(file, join(source, relative(docs, file)))
  for (const file of publicFiles) copy(file, join(source, 'public', relative(join(docs, 'public'), file)))
  copy(join(root, 'source-assets/branding/n-logo.svg'), join(source, 'public/project-assets/branding/n-logo.svg'))
  const district = JSON.parse(readFileSync(join(root, 'source-assets/district-map/district.json'), 'utf8'))
  const projected = JSON.stringify(playerMap(district)) + '\n'
  put(join(source, '_data/map.json'), projected)
  put(join(source, 'public/project-assets/district-map/map.json'), projected)
  if (profile === 'dev') {
    copy(join(root, 'source-assets/district-map/district.json'), join(source, '_data/district.json'))
    copy(join(root, 'source-assets/district-map/district.json'), join(source, 'public/project-assets/district-map/district.json'))
  }
  for (const name of SUPPORT.filter(name => profile === 'dev' || name === 'district-map.mjs')) copy(join(root, 'tools', name), join(source, '_tools', name))
  for (const name of COMPONENTS.filter(name => profile === 'dev' || name === 'DistrictMap.vue')) {
    const text = readFileSync(join(docs, '.vitepress/components', name), 'utf8')
      .replaceAll('../../../source-assets/district-map/district.json', '@wiki-data/district.json')
      .replaceAll('../../../tools/', '@wiki-tools/')
    put(join(source, '_components', name), text)
  }
  const mapping = join(root, 'todo/evidence/TASK-007/r1/migration-map.json')
  const aliases = []
  if (existsSync(mapping)) for (const entry of JSON.parse(readFileSync(mapping, 'utf8')).files) {
    if (entry.source === 'docs/index.md' || !entry.source.startsWith('docs/') || !entry.source.endsWith('.md')) continue
    const target = entry.targets.find(t => t.startsWith('docs/player/') || (profile === 'dev' && t.startsWith('docs/dev/')))
    if (!target || target === entry.source || !pageSet.has(resolve(root, target))) continue
    const old = entry.source.slice(5).replace(/\.md$/, '.html'), to = route(target.slice(5))
    put(join(source, 'public', old), redirect(to)); aliases.push({ old, to })
  }
  put(join(source, 'index.md'), `# N:SIDE ${profile === 'player' ? '游戏百科' : '开发资料'}\n\n[进入${profile === 'player' ? '游戏百科' : '开发资料'}](${profile}/index.md)\n`)
  put(join(source, '.vitepress/config.mjs'), `import { wikiConfig } from ${JSON.stringify(pathToFileURL(join(docs, '.vitepress/config.mjs')).href)}\nexport default wikiConfig(${JSON.stringify({ root, source, profile })})\n`)
  const manifest = { profile, pages: pages.map(p => relative(docs, p)), data: listWikiData(docs, profile).map(p => relative(docs, p)), public: filesIn(join(source, 'public')).map(p => relative(join(source, 'public'), p)), aliases }
  put(join(base, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n')
  return { source, output: join(base, 'dist'), manifest }
}

/** Check actual emitted pages and all raw data, not just sidebar configuration */
export function checkWikiBuild(root, profile) {
  assertProfile(profile)
  const base = join(resolve(root), 'output/wiki', profile), output = join(base, 'dist')
  const manifest = JSON.parse(readFileSync(join(base, 'manifest.json'), 'utf8'))
  if (manifest.profile !== profile) throw new Error('Build manifest profile mismatch')
  const files = filesIn(output), names = files.map(p => relative(output, p))
  for (const name of manifest.pages.map(p => p.replace(/\.md$/, '.html'))) if (!names.includes(name)) throw new Error(`Missing built page ${name}`)
  const raw = new Set([...manifest.data, ...manifest.public, 'hashmap.json'])
  const html = new Set(['index.html', '404.html', ...manifest.pages.map(p => p.replace(/\.md$/, '.html')), ...manifest.public.filter(p => p.endsWith('.html'))])
  for (const name of names) {
    if (name.endsWith('.html') && !html.has(name)) throw new Error(`Unexpected published page: ${name}`)
    if (['.json', '.csv'].includes(extname(name)) && !raw.has(name)) throw new Error(`Unexpected published data: ${name}`)
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
      const prepared = command === 'preview' ? { source: join(root, 'output/wiki', profile, 'source') } : prepareWiki(root, profile)
      if (command === 'prepare') console.log(JSON.stringify(prepared.manifest))
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
    } else throw new Error('Usage: bun tools/wiki.mjs build|dev|prepare|preview|check player|dev [port]')
  } catch (error) { console.error(error.message); process.exitCode = 1 }
}
