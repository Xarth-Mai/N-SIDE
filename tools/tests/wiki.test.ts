import type { TestContext } from 'node:test'
import type { DefaultTheme } from 'vitepress'
import type { Connect, ViteDevServer } from 'vite'
import type { IncomingMessage, ServerResponse } from 'node:http'
import type { WikiProfile } from '../wiki-data.ts'
import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync, cpSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, relative } from 'node:path'
import { buildSidebar } from '../../docs/.vitepress/sidebar.ts'
import { copyWikiData, listWikiData, wikiDataPlugin } from '../wiki-data.ts'
import { prepareWiki, checkWikiBuild, playerMap } from '../wiki.ts'

function fixture(t: TestContext) {
  const root = mkdtempSync(join(tmpdir(), 'n-side-wiki-'))
  t.after(() => rmSync(root, { recursive: true, force: true }))
  function put(file: string, content: string | Uint8Array = '# Page\n') {
    const path = join(root, file); mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, content); return path
  }
  return { root, put }
}
const links = (items: DefaultTheme.SidebarItem[]): string[] => items.flatMap(item => [...(item.link ? [item.link] : []), ...links(item.items ?? [])])
function request(root: string, profile: WikiProfile, url: string, method = 'GET') {
  let handler!: Connect.NextHandleFunction; wikiDataPlugin(root, profile).configureServer({ middlewares: { use(fn: Connect.NextHandleFunction) { handler = fn } } } as unknown as ViteDevServer)
  const result: {next: boolean; headers: Record<string,string | number | readonly string[]>; body?: string; error?: unknown} = { next: false, headers: {} }
  handler({ url, method } as IncomingMessage, { setHeader(k: string, v: string | number | readonly string[]) { result.headers[k] = v }, end(body?: Buffer) { result.body = body?.toString() } } as ServerResponse, error => { result.next = true; result.error = error })
  return result
}
function wikiFixture(t: TestContext) {
  const f = fixture(t)
  f.put('docs/player/index.md', '# 玩家入口\n\n[故事](encyclopedia/story/main/last-toy.md)\n')
  f.put('docs/player/encyclopedia/story/main/last-toy.md', '---\nsubject_id: QST-002\ndesign_state: accepted\n---\n# 最后的玩具\n\n米娜自己决定打包\n')
  f.put('docs/dev/index.md', '# 开发入口\n\n[私有规格](design/spec.md)\n')
  f.put('docs/dev/design/spec.md', '# DEV_ONLY_SENTINEL\n\n内部规格 [数据](data.json)\n')
  f.put('docs/dev/design/data.json', '{"DEV_ONLY_SENTINEL":true}')
  f.put('docs/public/private.json', '{"DEV_ONLY_SENTINEL":true}')
  f.put('docs/public/private.webp', 'DEV_ONLY_SENTINEL')
  f.put('source-assets/branding/n-logo.svg', '<svg/>')
  for (const name of ['district-map.ts', 'district-plan.ts', 'district-architecture.ts', 'district-geometry.ts', 'district-types.ts', 'story-graph.ts']) f.put(`tools/${name}`, readFileSync(new URL(`../${name}`, import.meta.url)))
  for (const name of ['DistrictMap.vue', 'DistrictPlan.vue', 'DistrictArchitecture.vue', 'DistrictPlaces.vue', 'StoryGraph.vue', 'StoryCondition.vue']) f.put(`docs/.vitepress/components/${name}`, readFileSync(new URL(`../../docs/.vitepress/components/${name}`, import.meta.url)))
  const quest = JSON.parse(readFileSync(new URL('../../docs/dev/design/catalogs/quests.json', import.meta.url),'utf8')).quests.find((q: {id:string})=>q.id==='QST-002')
  quest.story.display_order=1; quest.story.recommended=[]; quest.story.related=[]; quest.story.required=[]
  f.put('docs/dev/design/catalogs/quests.json',JSON.stringify({story_schema_version:1,quests:[quest]}))
  f.put('docs/dev/design/quests/QST-002/narrative.json',readFileSync(new URL('../../docs/dev/design/quests/QST-002/narrative.json',import.meta.url)))
  f.put('source-assets/district-map/district.json', readFileSync(new URL('../../source-assets/district-map/district.json', import.meta.url), 'utf8'))
  f.put('todo/evidence/TASK-007/r1/migration-map.json', JSON.stringify({ files: [
    { source: 'docs/story/main/last-toy.md', targets: ['docs/player/encyclopedia/story/main/last-toy.md'] },
    { source: 'docs/production/private.md', targets: ['docs/dev/design/spec.md'] },
  ] }))
  return f
}

test('sidebar follows audience and preserves story reading order', t => {
  const { root, put } = fixture(t)
  put('player/index.md'); put('dev/index.md'); put('dev/design/spec.md')
  put('_data/stories.json',JSON.stringify([{role:'main',order:1,url:'/player/encyclopedia/story/main/prologue'},{role:'main',order:2,url:'/player/encyclopedia/story/main/last-toy'}]))
  for (const name of ['last-toy', 'prologue']) put(`player/encyclopedia/story/main/${name}.md`)
  assert.deepEqual(links(buildSidebar(root, 'player')), ['/', '/player/', '/player/encyclopedia/story/main/prologue', '/player/encyclopedia/story/main/last-toy'])
  assert.ok(links(buildSidebar(root, 'dev')).includes('/dev/design/spec'))
})

test('data export is fail-closed and dev-only, ignoring hidden files and symlinks', t => {
  const { root, put } = fixture(t)
  put('dev/design/data.json', '{"a":1}\n'); put('player/leak.json', '{}'); put('dev/.cache/cache.json', '{}')
  symlinkSync(put('external.json', '{}'), join(root, 'dev/linked.json'))
  assert.throws(() => listWikiData(root), /profile/)
  assert.deepEqual(listWikiData(root, 'player'), [])
  assert.deepEqual(listWikiData(root, 'dev').map(p => relative(root, p)), ['dev/design/data.json'])
  assert.equal(copyWikiData(root, join(root, 'out'), 'dev'), 1)
  assert.equal(readFileSync(join(root, 'out/dev/design/data.json'), 'utf8'), '{"a":1}\n')
  assert.equal(request(root, 'player', '/dev/design/data.json').next, true)
  assert.equal(request(root, 'dev', '/dev/design/data.json').body, '{"a":1}\n')
  assert.equal(request(root, 'dev', '/dev/design/data.json', 'HEAD').body, undefined)
})

test('publication requires a known profile and an actual entry page', t => {
  const { root } = fixture(t)
  assert.throws(() => prepareWiki(root, 'unknown'), /profile/)
  assert.throws(() => prepareWiki(root, 'player'), /missing entry/)
})

test('isolated player tree excludes developer pages/data/public and uses redirects', t => {
  const { root, put } = wikiFixture(t)
  const player = prepareWiki(root, 'player'), dev = prepareWiki(root, 'dev')
  assert.deepEqual(player.manifest.data, [])
  assert.doesNotMatch(readFileSync(join(player.source, 'player/encyclopedia/story/main/last-toy.md'), 'utf8'), /design_state|subject_id/)
  assert.ok(!player.manifest.pages.some(p => p.startsWith('dev/')))
  assert.ok(!player.manifest.public.some(p => p.includes('private')))
  assert.deepEqual(player.manifest.aliases.map(a => a.old), ['story/main/last-toy.html'])
  const redirect = readFileSync(join(player.source, 'public/story/main/last-toy.html'), 'utf8')
  assert.match(redirect, /location\.hash/); assert.doesNotMatch(redirect, /米娜/)
  assert.ok(dev.manifest.data.includes('dev/design/data.json'))
  assert.ok(dev.manifest.aliases.some(a => a.old === 'production/private.html'))
  put('docs/player/leak.md', '# 泄漏\n\n[内部](../dev/design/data.json)\n')
  assert.throws(() => prepareWiki(root, 'player'), /unpublished local reference/)
})

test('player imports and public data cannot bypass the publication boundary', t => {
  const { root, put } = wikiFixture(t)
  put('docs/player/leak.md', '# 泄漏\n\n<script setup>\nimport x from "../../dev/design/data.json"\n</script>\n')
  assert.throws(() => prepareWiki(root, 'player'), /unpublished import/)
  for (const source of ['import x from "@wiki-data/../../dev/data.json"', 'import "../../dev/design/data.json"', 'const pages=import.meta.glob("../../dev/**")']) {
    put('docs/player/leak.md', `# 泄漏\n\n<script setup>\n${source}\n</script>\n`)
    assert.throws(() => prepareWiki(root, 'player'), /unpublished import|glob imports/)
  }
  for (const directive of ['<!-- @include: ../../../../../docs/dev/design/spec.md -->', '<<< ../../../../../docs/dev/design/spec.md']) {
    put('docs/player/leak.md', `# 泄漏\n\n${directive}\n`)
    assert.throws(() => prepareWiki(root, 'player'), /external include/)
  }
  put('docs/player/leak.md', '# 泄漏\n\n[附件](../public/private.json)\n')
  assert.throws(() => prepareWiki(root, 'player'), /Unsupported public/)
  put('docs/player/leak.md', '# 泄漏\n\n![内部图片](../public/private.webp)\n')
  assert.throws(() => prepareWiki(root, 'player'), /Unsupported public/)
})

test('map projection retains actual drawing/interaction and drops author fields', () => {
  const original = JSON.parse(readFileSync(new URL('../../source-assets/district-map/district.json', import.meta.url), 'utf8'))
  original.author_notes = 'DEV_ONLY_SENTINEL'; original.places[0].brief.secret = 'DEV_ONLY_SENTINEL'
  const projection = playerMap(original)
  assert.equal(projection.places.length, 91)
  assert.ok(projection.scene.objects.length > 0 && projection.scene.terrain.length > 0)
  assert.doesNotMatch(JSON.stringify(projection), /DEV_ONLY_SENTINEL|"architectures"|"operations"|"brief"/)
  assert.equal(projection.places.find(p => p.id === '04')!.page, '/player/encyclopedia/locations/shop')
})

test('artifact check detects injected developer raw data and search content', t => {
  const { root, put } = wikiFixture(t), { source, output, manifest } = prepareWiki(root, 'player')
  cpSync(join(source, 'public'), output, { recursive: true })
  for (const page of manifest.pages) put(relative(root, join(output, page.replace(/\.md$/, '.html'))), '<html>玩家</html>')
  assert.equal(checkWikiBuild(root, 'player').result, 'PASS')
  put(relative(root, join(output, 'dev/secret.json')), '{}')
  assert.throws(() => checkWikiBuild(root, 'player'), /Unexpected published data/)
  rmSync(join(output, 'dev'), { recursive: true })
  put(relative(root, join(output, 'secret.html')), 'DEV_ONLY_SENTINEL')
  assert.throws(() => checkWikiBuild(root, 'player'), /Unexpected published page/)
  rmSync(join(output, 'secret.html'))
  put(relative(root, join(output, 'assets/search.js')), 'DOCS-PIPELINE')
  assert.throws(() => checkWikiBuild(root, 'player'), /Developer content/)
})

test('inline template placeholders remain literal Vue text', async () => {
  const { createMarkdownRenderer } = await import('vitepress')
  const { markdown, default: directConfig } = await import('../../docs/.vitepress/config.ts')
  assert.throws(directConfig, /Choose a Wiki audience/)
  const md = await createMarkdownRenderer(process.cwd(), markdown)
  assert.match(md.render('`{{交付目标}}`'), /<code v-pre[^>]*>\{\{交付目标\}\}<\/code>/)
})

test('Vite module loading rejects source files outside the isolated tree', async t => {
  const { root } = wikiFixture(t)
  const { source } = prepareWiki(root, 'player')
  const { wikiConfig } = await import('../../docs/.vitepress/config.ts')
  const config = wikiConfig({ root: process.cwd(), source, profile: 'player' })
  const boundary = (config.vite!.plugins! as {name: string; load: (id: string) => undefined}[]).find(p => p.name === 'n-side-wiki-boundary')!
  assert.throws(() => boundary.load(join(root, 'docs/dev/design/data.json')), /Unpublished Wiki module/)
  assert.equal(boundary.load(join(source, '_data/map.json')), undefined)
  for (const id of ['/@siteData', '/@localSearchIndex', '/@localSearchIndexroot', '/player/', '/player/index', '/project-assets/branding/n-logo.svg']) assert.equal(boundary.load(id), undefined)
  assert.throws(() => boundary.load('/@localSearchIndex/../../docs/dev/design/data.json'), /Unpublished Wiki module/)
})


test('external search data remains inside the selected audience', t => {
  const { root, put } = wikiFixture(t), { source, output, manifest } = prepareWiki(root, 'player')
  cpSync(join(source, 'public'), output, { recursive: true })
  for (const page of manifest.pages) put(relative(root, join(output, page.replace(/\.md$/, '.html'))), '<html>玩家</html>')
  const search=relative(root, join(output,'assets/search-index.test.json'))
  put(search,JSON.stringify({documentCount:1,documentIds:{0:'/player/#入口'}}))
  assert.equal(checkWikiBuild(root,'player').result,'PASS')
  put(search,JSON.stringify({documentCount:1,documentIds:{0:'/dev/design/spec#内部'}}))
  assert.throws(()=>checkWikiBuild(root,'player'),/Unpublished search document/)
  const labels=JSON.parse(readFileSync(join(source,'_data/map.json'),'utf8'))
  assert.equal(labels.scene,undefined)
  assert.equal(labels.places.length,91)
})
