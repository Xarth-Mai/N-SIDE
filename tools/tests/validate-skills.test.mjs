import { afterEach, expect, test } from 'bun:test'
import { createHash } from 'node:crypto'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { validate } from '../validate-skills.mjs'

const roots = []
afterEach(() => { for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true }) })

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'nside-skills-'))
  roots.push(root)
  const write = (file, text) => { mkdirSync(dirname(join(root, file)), { recursive: true }); writeFileSync(join(root, file), text) }
  const path = '.agents/skills/example/SKILL.md'
  write(path, '---\nname: example\ndescription: >-\n  Read a source.\n  Check its evidence.\nmetadata:\n  version: "1"\n---\n# Example\n[More](references/more.md)\n')
  write('.agents/skills/example/references/more.md', 'Original text without a required project title\n')
  write('third_party/skills/source/LICENSE', 'Original author and license\n')
  const sha = file => createHash('sha256').update(readFileSync(join(root, file))).digest('hex')
  const manifest = {
    sources: [{ id: 'source', repo: 'https://github.com/author/repo', commit: 'a'.repeat(40), author: 'Original Author', license: 'MIT', license_files: [{ local_path: 'third_party/skills/source/LICENSE', sha256: sha('third_party/skills/source/LICENSE') }] }],
    skills: [{ name: 'example', source: 'source', local_path: '.agents/skills/example', mode: 'verbatim', files: [{ path: 'SKILL.md', sha256: sha(path) }] }], references: [],
  }
  const save = () => write('third_party/skills/manifest.json', JSON.stringify(manifest))
  save()
  return { root, write, manifest, save }
}

test('portable YAML and unmodified upstream references pass', () => {
  expect(validate(fixture().root).ok).toBe(true)
})

test('missing required reference and altered upstream file fail', () => {
  const f = fixture()
  rmSync(join(f.root, '.agents/skills/example/references/more.md'))
  f.write('.agents/skills/example/SKILL.md', readFileSync(join(f.root, '.agents/skills/example/SKILL.md'), 'utf8') + '\nChanged\n')
  const result = validate(f.root)
  expect(result.ok).toBe(false)
  expect(result.issues.some(x => x.message.includes('missing local reference'))).toBe(true)
  expect(result.issues.some(x => x.message.includes('source hash'))).toBe(true)
})

test('optional upstream cross-reference requires exact pinned source', () => {
  const f = fixture()
  rmSync(join(f.root, '.agents/skills/example/references/more.md'))
  f.manifest.skills[0].optional_upstream_links = [{ from: 'SKILL.md', target: 'references/more.md', url: `https://github.com/author/repo/blob/${'a'.repeat(40)}/references/more.md` }]
  f.save()
  expect(validate(f.root).ok).toBe(true)
  f.manifest.skills[0].optional_upstream_links[0].url = 'https://github.com/author/repo/blob/main/references/more.md'
  f.save()
  expect(validate(f.root).ok).toBe(false)
})

test('lost license attribution fails', () => {
  const f = fixture()
  rmSync(join(f.root, 'third_party/skills/source/LICENSE'))
  expect(validate(f.root).ok).toBe(false)
})
