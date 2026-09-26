import { afterEach, expect, test } from 'bun:test'
import { createHash } from 'node:crypto'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { validate } from '../validate-skills.ts'

const roots: string[] = []
afterEach(() => { for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true }) })
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'nside-skills-')); roots.push(root)
  const write = (file: string, body: string) => { mkdirSync(dirname(join(root, file)), { recursive: true }); writeFileSync(join(root, file), body) }
  const base = '.agents/skills/example'
  write(`${base}/SKILL.md`, '---\nname: example\ndescription: >-\n  Read a source.\n  Check its evidence.\nmetadata:\n  version: "1"\n---\n# Example\n[More](references/more.md#detail)\n')
  write(`${base}/references/more.md`, '# More\n## Detail\nOriginal text\n')
  write(`${base}/UPSTREAM.md`, '# Upstream\nOriginal author\n')
  write('third_party/skills/source/LICENSE', 'Original author and license\n')
  write('.agents/skills/project/SKILL.md', '---\nname: project\ndescription: Project work\n---\n# Project\n')
  const hash = (file: string) => createHash('sha256').update(readFileSync(join(root, file))).digest('hex')
  const manifest: Record<string, any> = {
    schema_version: 2, policy: { required_skills: ['project', 'example'] },
    local_skills: [{ name: 'project', local_path: '.agents/skills/project', purpose: 'Project entrypoint', files: ['SKILL.md'] }],
    sources: [{ id: 'source', repo: 'https://github.com/author/repo', commit: 'a'.repeat(40), author: 'Original Author', license: 'MIT', license_files: [{ upstream_path: 'LICENSE', local_path: 'third_party/skills/source/LICENSE', sha256: hash('third_party/skills/source/LICENSE') }] }],
    skills: [{ name: 'example', source: 'source', upstream_path: 'skills/example', local_path: base, mode: 'verbatim', files: ['SKILL.md', 'references/more.md'].map(path => ({ path, sha256: hash(`${base}/${path}`) })) }], references: [],
  }
  const save = () => write('third_party/skills/manifest.json', JSON.stringify(manifest))
  const add = (path: string, body: string) => { write(`${base}/${path}`, body); manifest.skills[0].files.push({ path, sha256: hash(`${base}/${path}`) }); save() }
  const alter = (path: string, body: string) => { write(`${base}/${path}`, body); manifest.skills[0].files.find((f: {path: string; sha256: string}) => f.path === path).sha256 = hash(`${base}/${path}`); save() }
  save(); return { root, write, manifest, save, add, alter, base }
}
const failure = (f: ReturnType<typeof fixture>, message: string) => { const result = validate(f.root); expect(result.ok).toBe(false); expect(result.issues.some(x => x.message.includes(message))).toBe(true) }
test('real Bun YAML, complete coverage and anchors pass', () => expect(validate(fixture().root).ok).toBe(true))
const cases: [string, (f: ReturnType<typeof fixture>) => unknown, string][] = [
  ['missing manifest', f => rmSync(join(f.root, 'third_party/skills/manifest.json')), 'cannot be read'],
  ['empty object', f => f.write('third_party/skills/manifest.json', '{}'), 'schema_version'],
  ['null manifest', f => f.write('third_party/skills/manifest.json', 'null'), 'schema_version'],
  ['bad JSON', f => f.write('third_party/skills/manifest.json', '{'), 'cannot be read'],
  ['unknown version', f => { f.manifest.schema_version = 3; f.save() }, 'schema_version'],
  ['bad adoption schema', f => { f.manifest.adoptions = {}; f.save() }, 'adoptions must be an array'],
  ['missing adoption target', f => { f.manifest.adoptions = [{source:'source',local_paths:['docs/missing.md']}]; f.save() }, 'missing adoption destination'],
  ['empty sources', f => { f.manifest.sources = []; f.save() }, 'sources'],
  ['empty imports', f => { f.manifest.skills = []; f.save() }, 'skills'],
  ['empty policy', f => { f.manifest.policy.required_skills = []; f.save() }, 'required_skills'],
  ['null skill', f => { f.manifest.skills.push(null); f.save() }, 'invalid skill'],
  ['bad files type', f => { f.manifest.skills[0].files = {}; f.save() }, 'files must be'],
  ['null file record', f => { f.manifest.skills[0].files.push(null); f.save() }, 'invalid file record'],
  ['bad source', f => { f.manifest.sources.push(null); f.save() }, 'invalid source'],
  ['bad fields', f => { f.manifest.skills = {}; f.save() }, 'skills'],
  ['missing project entrypoint', f => rmSync(join(f.root, '.agents/skills/project'), { recursive: true }), 'missing required skill project'],
  ['empty skills directory', f => rmSync(join(f.root, '.agents/skills'), { recursive: true }), 'missing required skill'],
  ['duplicate source', f => { f.manifest.sources.push(f.manifest.sources[0]); f.save() }, 'duplicate source'],
  ['duplicate path', f => { f.manifest.skills.push(f.manifest.skills[0]); f.save() }, 'duplicate local path'],
  ['missing reference', f => rmSync(join(f.root, f.base, 'references/more.md')), 'missing local reference'],
  ['altered source', f => f.write(`${f.base}/references/more.md`, 'Changed'), 'source hash'],
  ['unregistered script', f => f.write(`${f.base}/scripts/untracked.py`, 'pass\n'), 'unregistered file'],
  ['missing command path', f => f.alter('SKILL.md', readFileSync(join(f.root, f.base, 'SKILL.md'), 'utf8') + '\n```sh\npython scripts/missing.py\n```\n'), 'missing command path'],
  ['bad anchor', f => f.alter('references/more.md', '# More\n## Renamed\n'), 'missing anchor'],
  ['bad YAML metadata', f => f.add('agents/openai.yaml', 'interface: ['), 'host metadata'],
  ['bad metadata type', f => f.add('agents/openai.yaml', 'policy:\n  allow_implicit_invocation: "yes"\n'), 'host metadata'],
  ['missing icon', f => f.add('agents/openai.yaml', 'interface:\n  icon_small: "./assets/missing.png"\n'), 'missing icon'],
  ['missing license', f => rmSync(join(f.root, 'third_party/skills/source/LICENSE')), 'missing required file'],
  ['optional links cannot exempt missing files', f => { rmSync(join(f.root, f.base, 'references/more.md')); f.manifest.skills[0].optional_upstream_links = [{ from: 'SKILL.md', target: 'references/more.md#detail', url: `https://github.com/author/repo/blob/${'a'.repeat(40)}/references/more.md` }]; f.save() }, 'no longer exempts'],
  ['invalid nested fields', f => { f.manifest.skills[0].commands = {}; f.manifest.skills[0].mode = 1; f.save() }, 'invalid mode'],
  ['unavailable command', f => { f.add('scripts/run.py', 'pass\n'); f.manifest.skills[0].commands = [{ from: 'SKILL.md', path: 'scripts/run.py', executable: 'nside-no-such-executable' }]; f.save() }, 'required executable unavailable'],
]
for (const [name, mutate, message] of cases) test(name, () => { const f = fixture(); mutate(f); failure(f, message) })

test('inactive upstream reference links are checked', () => {
  const f = fixture(); const base = 'third_party/skills/source/references/review'
  const body = '---\nname: review\ndescription: Reference\n---\n# Review\n[Missing](missing.md)\n'
  f.write(`${base}/SKILL.md`, body)
  f.manifest.references.push({ name: 'review', source: 'source', upstream_path: 'review', local_path: base, mode: 'verbatim-reference', files: [{ path: 'SKILL.md', sha256: createHash('sha256').update(body).digest('hex') }] })
  f.save(); failure(f, 'missing local reference')
})
test('local hashes alone explicitly do not prove upstream equality', () => {
  const f = fixture(); f.alter('references/more.md', '# More\n## Detail\nLocally rewritten text\n')
  expect(validate(f.root).ok).toBe(true) // Independent source comparison is a separate command
})

test('Wiki anchors match the installed VitePress renderer while upstream keeps GitHub slugs', async () => {
  const { createMarkdownRenderer } = await import('vitepress')
  const { anchors } = await import('../validate-skills.ts')
  const body = '# 1. Échelle / 尺度（m）\n## 动作 `start_stop`\n## 标题\n## 标题\n## 指定 {#specific}\n'
  const renderer = await createMarkdownRenderer(process.cwd())
  const actual = new Set([...renderer.render(body).matchAll(/<h[1-6] id="([^"]+)"/g)].map(match => match[1]))
  expect(anchors(body, true)).toEqual(actual)
  expect(anchors('# 1. title')).toEqual(new Set(['1-title']))
})
