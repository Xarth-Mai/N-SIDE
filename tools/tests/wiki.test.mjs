import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, relative } from 'node:path'
import { buildSidebar } from '../../docs/.vitepress/sidebar.mjs'
import { copyWikiData, listWikiData, wikiDataPlugin } from '../wiki-data.mjs'

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'n-side-wiki-'))
  t.after(() => rmSync(root, { recursive: true, force: true }))
  function put(file, content = '# Page\n') {
    const path = join(root, file)
    mkdirSync(dirname(path), { recursive: true })
    writeFileSync(path, content)
    return path
  }
  return { root, put }
}

function links(items) {
  return items.flatMap(item => [...(item.link ? [item.link] : []), ...links(item.items ?? [])])
}

function request(root, url, method = 'GET') {
  let handler
  wikiDataPlugin(root).configureServer({ middlewares: { use(fn) { handler = fn } } })
  const result = { next: false, headers: {}, body: undefined }
  handler({ url, method }, {
    setHeader(key, value) { result.headers[key] = value },
    end(body) { result.body = body?.toString() },
  }, error => { result.next = true; result.error = error })
  return result
}

test('sidebar uses the four selected groups', t => {
  const { root, put } = fixture(t)
  put('vision.md', '# 项目愿景\n'); put('conventions.md', '# 项目约定\n')
  const sidebar = buildSidebar(root)
  assert.deepEqual(sidebar.map(item => item.text), ['Overview', 'Universe', 'Game', 'Development'])
  assert.deepEqual(links(sidebar), ['/vision', '/conventions'])
})

test('new content pages appear in the sidebar', t => {
  const { root, put } = fixture(t)
  put('characters/example.md', '# 新角色\n')
  assert.ok(links(buildSidebar(root)).includes('/characters/example'))
})

test('section index and object README retain distinct routes', t => {
  const { root, put } = fixture(t)
  put('quests/index.md', '# 游戏任务\n')
  put('quests/QST-001/README.md', '# 委托一\n')
  put('quests/QST-001/development.md', '# 开发稿\n')
  const routes = links(buildSidebar(root))
  assert.deepEqual(routes, ['/quests/', '/quests/QST-001/README', '/quests/QST-001/development'])
})

test('navigation scans knowledge categories', t => {
  const { root, put } = fixture(t)
  put('todo/work.md'); put('.vitepress/cache.md'); put('public/example.md')
  assert.deepEqual(links(buildSidebar(root)), [])
})

test('data export preserves source paths and bytes', t => {
  const { root, put } = fixture(t)
  const json = '{"label":"中文"}\n'
  const csv = 'line_id,text\n1,你好\n'
  put('templates/narrative.json', json); put('quests/QST-001/dialogue.csv', csv)
  const output = join(root, '.vitepress/dist')
  assert.equal(copyWikiData(root, output), 2)
  assert.equal(readFileSync(join(output, 'templates/narrative.json'), 'utf8'), json)
  assert.equal(readFileSync(join(output, 'quests/QST-001/dialogue.csv'), 'utf8'), csv)
})

test('data export scans original content rather than build output', t => {
  const { root, put } = fixture(t)
  put('templates/data.json', '{}')
  put('public/images/meta.json', '{}'); put('.vitepress/cache/data.json', '{}')
  put('node_modules/pkg/package.json', '{}')
  assert.deepEqual(listWikiData(root).map(file => relative(root, file)), ['templates/data.json'])
})

test('dev server provides raw JSON and CSV', t => {
  const { root, put } = fixture(t)
  put('templates/data.json', '{"a":1}')
  put('templates/dialogue.csv', 'id,text\n1,你好\n')
  const json = request(root, '/templates/data.json')
  assert.equal(json.body, '{"a":1}')
  assert.equal(json.headers['Content-Type'], 'application/json; charset=utf-8')
  assert.equal(json.next, false)
  const csv = request(root, '/templates/dialogue.csv')
  assert.equal(csv.body, 'id,text\n1,你好\n')
})

test('HEAD sends data headers with an empty body', t => {
  const { root, put } = fixture(t)
  put('templates/data.json', '{}')
  const result = request(root, '/templates/data.json', 'HEAD')
  assert.equal(result.next, false); assert.equal(result.body, undefined)
  assert.equal(result.headers['Content-Type'], 'application/json; charset=utf-8')
})

test('other requests continue through Vite', t => {
  const { root, put } = fixture(t)
  put('templates/data.json', '{}')
  for (const [url, method] of [['/vision', 'GET'], ['/missing.json', 'GET'], ['/templates/data.json?import', 'GET'], ['/templates/data.json', 'POST']]) {
    assert.equal(request(root, url, method).next, true)
  }
})

test('public and configuration paths use their own handlers', t => {
  const { root, put } = fixture(t)
  put('.vitepress/config.json', '{}'); put('public/images/data.json', '{}')
  for (const url of ['/.vitepress/config.json', '/public/images/data.json']) {
    assert.equal(request(root, url).next, true)
  }
})

test('encoded data paths are served', t => {
  const { root, put } = fixture(t)
  put('templates/a b.csv', 'x,y\n')
  assert.equal(request(root, '/templates/a%20b.csv').body, 'x,y\n')
})

test('data handler keeps file resolution inside the source root', t => {
  const { root, put } = fixture(t)
  const source = join(root, 'docs'); mkdirSync(source)
  const external = put('external.json', '{}')
  symlinkSync(external, join(source, 'linked.json'))
  assert.equal(request(source, '/linked.json').next, true)
  assert.deepEqual(listWikiData(source), [])
})


test('inline template placeholders remain literal Vue text', async () => {
  const { createMarkdownRenderer } = await import('vitepress')
  const { default: config } = await import('../../docs/.vitepress/config.mjs')
  const md = await createMarkdownRenderer(process.cwd(), config.markdown)
  assert.match(md.render('`{{交付目标、相关设计}}`'), /<code v-pre[^>]*>\{\{交付目标、相关设计\}\}<\/code>/)
})
