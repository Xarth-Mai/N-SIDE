import { createHash } from 'node:crypto'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { basename, dirname, join, relative, resolve } from 'node:path'

const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex')
const markdownLinks = /!?\[[^\]\n]*\]\(\s*(?:<([^>\n]+)>|([^\s)]+))[^\n)]*\)/g

function walk(directory) {
  if (!existsSync(directory)) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    return entry.isDirectory() ? walk(path) : entry.isFile() ? [path] : []
  })
}

export function validate(root) {
  root = resolve(root)
  const issues = []
  const issue = (path, message) => issues.push({ path: relative(root, path), message })
  const manifestPath = join(root, 'third_party/skills/manifest.json')
  let manifest = { sources: [], skills: [], references: [] }
  if (existsSync(manifestPath)) {
    try { manifest = JSON.parse(readFileSync(manifestPath, 'utf8')) }
    catch (error) { return { ok: false, issues: [{ path: relative(root, manifestPath), message: error.message }] } }
  }
  const sources = new Map()
  for (const source of manifest.sources) {
    if (sources.has(source.id)) issue(manifestPath, `duplicate source ${source.id}`)
    sources.set(source.id, source)
    if (!/^[a-f0-9]{40}$/.test(source.commit)) issue(manifestPath, `invalid commit for ${source.id}`)
    if (!source.author || !source.license || !source.license_files?.length) issue(manifestPath, `missing attribution for ${source.id}`)
    for (const file of source.license_files ?? []) checkHash(file.local_path, file.sha256)
  }

  function checkHash(path, expected) {
    const file = resolve(root, path)
    if (relative(root, file).startsWith('..') || !existsSync(file)) issue(file, 'missing or out-of-repository source file')
    else if (hash(file) !== expected) issue(file, 'content differs from recorded source hash')
  }

  const imported = new Map()
  for (const skill of [...manifest.skills, ...(manifest.references ?? [])]) {
    const source = sources.get(skill.source)
    if (!source) issue(manifestPath, `unknown source for ${skill.name}`)
    if (imported.has(skill.local_path)) issue(manifestPath, `duplicate local path ${skill.local_path}`)
    imported.set(skill.local_path, skill)
    for (const file of skill.files) checkHash(join(skill.local_path, file.path), file.sha256)
    for (const link of skill.optional_upstream_links ?? []) {
      if (!source || !link.url.startsWith(`${source.repo}/blob/${source.commit}/`)) issue(manifestPath, `optional reference is not pinned: ${link.url}`)
    }
  }

  const names = new Set()
  let skills = 0
  for (const file of walk(join(root, '.agents/skills'))) {
    if (!file.endsWith('.md')) continue
    let text = readFileSync(file, 'utf8')
    const local = relative(root, file).replaceAll('\\', '/')
    const skillDir = local.split('/').slice(0, 3).join('/')
    const source = imported.get(skillDir)
    if (basename(file) === 'SKILL.md') {
      skills++
      const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/)
      try {
        if (!match) throw new Error('missing YAML frontmatter')
        const metadata = Bun.YAML.parse(match[1])
        const name = metadata?.name
        if (typeof name !== 'string' || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(name) || name !== basename(dirname(file))) throw new Error('name must match the skill directory')
        if (typeof metadata.description !== 'string' || !metadata.description.trim()) throw new Error('description must be nonempty text')
        if (names.has(name)) throw new Error(`duplicate skill name ${name}`)
        names.add(name)
        text = text.slice(match[0].length)
      } catch (error) { issue(file, error.message) }
    }
    if (/\$\{(?:ENGINE_NAME|ASSET_SKILL_COMMAND|ENGINE_GUIDE_FILE|AGENT_NAME|RUNTIME_ASSET_DIR|ASSET_GEN_SKILL_DIR)\}/.test(text)) issue(file, 'unrendered Godogen runtime token')
    const prose = text.replace(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\s*$/gm, '').replace(/(`+).*?\1/g, '')
    for (const match of prose.matchAll(markdownLinks)) {
      const target = match[1] ?? match[2]
      if (/^(?:[a-z][a-z0-9+.-]*:|\/\/|#)/i.test(target)) continue
      const linkPath = decodeURIComponent(target.split('#')[0].split('?')[0])
      const destination = resolve(dirname(file), linkPath)
      if (existsSync(destination)) continue
      const optional = source?.optional_upstream_links?.find(link => link.from === relative(join(root, skillDir), file).replaceAll('\\', '/') && link.target === target)
      if (!optional) issue(file, `missing local reference ${target}`)
    }
  }
  return { ok: !issues.length, skills, imported: manifest.skills.length, issues }
}

if (import.meta.main) {
  const result = validate(process.argv[2] ?? '.')
  console.log(`${result.ok ? 'PASS' : 'FAIL'}: ${result.skills ?? 0} Skills; ${result.imported ?? 0} recorded imports`)
  for (const item of result.issues) console.error(`${item.path}: ${item.message}`)
  process.exitCode = result.ok ? 0 : 1
}
