// A real VitePress failure probe, isolated from repository documents
import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync, rmSync, readFileSync, symlinkSync, openSync, closeSync } from 'node:fs'
import { join, dirname, resolve } from 'node:path'
import { prepareWiki } from '../../../../tools/wiki.mjs'
const root = resolve(import.meta.dir, '../../../..')
const fixture = join(root, 'output/wiki-boundary-negative')
rmSync(fixture, { recursive: true, force: true })
mkdirSync(fixture, { recursive: true })
symlinkSync(join(root, 'node_modules'), join(fixture, 'node_modules'))
const put = (path, text) => { const file = join(fixture, path); mkdirSync(dirname(file), { recursive: true }); writeFileSync(file, text) }
for (const path of ['docs/.vitepress/config.mjs', 'docs/.vitepress/sidebar.mjs', 'tools/wiki-data.mjs', 'source-assets/branding/n-logo.svg', 'source-assets/district-map/district.json', ...['DistrictMap.vue', 'DistrictPlan.vue', 'DistrictArchitecture.vue', 'DistrictPlaces.vue'].map(x => 'docs/.vitepress/components/' + x), ...['district-map.mjs', 'district-plan.mjs', 'district-architecture.mjs', 'district-geometry.mjs'].map(x => 'tools/' + x)]) put(path, readFileSync(join(root, path)))
put('docs/dev/design/data.json', '{"DEV_ONLY_SENTINEL":true}')
put('docs/dev/design/spec.md', '# DEV_ONLY_SENTINEL\n')
put('docs/player/index.md', '# 玩家资料\n\n<script setup>\nimport secret from /* normal comment */ "@wiki-data/../../../../../docs/dev/design/data.json"\n</script>\n\n<pre>{{ secret }}</pre>\n')
const { source } = prepareWiki(fixture, 'player')
const logfile = join(import.meta.dir, 'wiki-negative-build.log')
const log = openSync(logfile, 'w')
const child = Bun.spawn(['bun', 'run', 'vitepress', 'build', source], { cwd: root, stdout: log, stderr: log })
const code = await child.exited
closeSync(log)
assert.notEqual(code, 0)
assert.match(readFileSync(logfile, 'utf8'), /Unpublished Wiki module/)
put('docs/player/index.md', '# 玩家资料\n\n<!-- @include: ../../../../../docs/dev/design/spec.md -->\n')
assert.throws(() => prepareWiki(fixture, 'player'), /external include/)
const result = { result: 'PASS', module_escape: { vitepress_exit: code, diagnostic: 'Unpublished Wiki module' }, external_include: { rejected_before_build: true } }
writeFileSync(join(import.meta.dir, 'wiki-negative.json'), JSON.stringify(result, null, 2) + '\n')
console.log(JSON.stringify(result))
