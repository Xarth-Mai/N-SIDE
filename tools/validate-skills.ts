// These JSON/YAML records are deliberately untrusted; each consumed field is checked below
type RawRecord = Record<string, any>
import { createHash } from 'node:crypto'
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { basename, dirname, join, relative, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'

const digest = (file: string) => createHash('sha256').update(readFileSync(file)).digest('hex')
const text = (value: unknown): value is string => typeof value === 'string' && value.trim().length > 0
const object = (value: unknown): value is RawRecord => value !== null && typeof value === 'object' && !Array.isArray(value)
const safePath = (value: unknown): value is string => text(value) && !value.startsWith('/') && !value.split('/').some(p => p === '..' || p === '')
const sha = (value: unknown) => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value)
const links = /!?\[[^\]\n]*\]\(\s*(?:<([^>\n]+)>|([^\s)]+))[^\n)]*\)/g
const clean = (value: string) => value.replace(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\s*$/gm, '').replace(/(`+).*?\1/g, '')
export function walk(directory: string): string[] {
  if (!existsSync(directory)) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    if (['__pycache__', '.DS_Store'].includes(entry.name)) return []
    const path = join(directory, entry.name)
    return entry.isDirectory() ? walk(path) : [path]
  })
}
export function anchors(body: string, vitepress = false) {
  const result = new Set()
  const counts = new Map()
  for (const match of body.replace(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\s*$/gm, '').matchAll(/^#{1,6}\s+(.+?)\s*#*$/gm)) {
    let id = match[1].match(/\{#([^}]+)\}/)?.[1]
    if (!id) {
      const title = match[1].replace(/<[^>]*>/g, '').replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replace(/`([^`]+)`/g, '$1').replace(/(\*\*|__|[~*])(.+?)\1/g, '$2')
      // VitePress 1.6 uses NFKD and separates punctuation; upstream Markdown uses GitHub slugs
      id = vitepress
        ? title.normalize('NFKD').replace(/[\u0300-\u036f\u0000-\u001f]/g, '').replace(/[\s~`!@#$%^&*()\-_+=[\]{}|\\;:"'“”‘’<>,.?/]+/g, '-').replace(/^-+|-+$/g, '').replace(/^(\d)/, '_$1').toLowerCase()
        : title.toLowerCase().replace(/[^\p{L}\p{N}_\-\s]/gu, '').trim().replace(/\s/g, '-')
      const count = counts.get(id) ?? 0
      counts.set(id, count + 1)
      if (count) id += `-${count}`
    }
    result.add(id)
  }
  for (const match of body.matchAll(/<(?:a|[^>\s]+)\s+[^>]*(?:id|name)=["']([^"']+)["']/g)) result.add(match[1])
  return result
}

export function validate(root: string) {
  root = resolve(root)
  const issues: {path: string; message: string}[] = []
  const issue = (path: string, message: string) => issues.push({ path: relative(root, path), message })
  const manifestPath = join(root, 'third_party/skills/manifest.json')
  const result = () => ({ ok: !issues.length, skills: names.size, imported: manifest?.skills?.length ?? 0, issues })
  const names = new Set()
  let manifest: RawRecord
  try { manifest = JSON.parse(readFileSync(manifestPath, 'utf8')) }
  catch (error) { issue(manifestPath, `manifest cannot be read: ${(error instanceof Error ? error.message : String(error))}`); return result() }
  if (!object(manifest) || manifest.schema_version !== 2) issue(manifestPath, 'schema_version must be 2')
  for (const key of ['sources', 'skills', 'local_skills', 'references']) {
    if (!Array.isArray(manifest?.[key]) || (key !== 'references' && !manifest[key].length)) issue(manifestPath, `${key} must be a nonempty array${key === 'references' ? ' (references may be empty)' : ''}`)
  }
  if (!object(manifest?.policy) || !Array.isArray(manifest.policy.required_skills) || !manifest.policy.required_skills.length || !manifest.policy.required_skills.every(text)) issue(manifestPath, 'policy.required_skills must be a nonempty name array')
  if (manifest?.adoptions !== undefined && !Array.isArray(manifest.adoptions)) issue(manifestPath, 'adoptions must be an array')
  if (issues.length) return result()

  const registered = new Set(['third_party/skills/manifest.json'])
  const sources = new Map<string, RawRecord>()
  const imports = new Map<string, RawRecord>()
  const units = [...manifest.skills, ...manifest.references, ...manifest.local_skills]
  function register(path: unknown, expected?: unknown) {
    if (!safePath(path)) { issue(manifestPath, `invalid file path ${path}`); return }
    if (registered.has(path)) issue(manifestPath, `duplicate file ${path}`)
    registered.add(path)
    const file = join(root, path)
    if (!existsSync(file) || !statSync(file).isFile()) issue(file, 'missing required file')
    else if (expected && (!sha(expected) || digest(file) !== expected)) issue(file, 'content differs from recorded source hash')
  }
  function paths(files: unknown, owner: string, prefix = '') {
    if (!Array.isArray(files) || !files.length) { issue(manifestPath, `files must be nonempty for ${owner}`); return }
    for (const file of files) {
      if (!object(file) || !safePath(file.path) || !sha(file.sha256)) { issue(manifestPath, `invalid file record for ${owner}`); continue }
      register(join(prefix, file.path), file.sha256)
      if (file.patch) {
        if (!object(file.patch) || !sha(file.patch.upstream_sha256) || !safePath(file.patch.file) || !text(file.patch.reason)) issue(manifestPath, `invalid patch for ${owner}/${file.path}`)
        else register(file.patch.file)
      }
    }
  }
  for (const source of manifest.sources) {
    if (!object(source) || !text(source.id)) { issue(manifestPath, 'invalid source'); continue }
    if (sources.has(source.id)) issue(manifestPath, `duplicate source ${source.id}`)
    sources.set(source.id, source)
    if (!/^https:\/\/github\.com\/[^/]+\/[^/]+$/.test(source.repo ?? '') || !/^[a-f0-9]{40}$/.test(source.commit ?? '')) issue(manifestPath, `invalid pinned source ${source.id}`)
    if (!text(source.author) || !text(source.license) || !Array.isArray(source.license_files) || !source.license_files.length) issue(manifestPath, `missing attribution for ${source.id}`)
    for (const file of Array.isArray(source.license_files) ? source.license_files : []) {
      if (!object(file) || !safePath(file.local_path) || !safePath(file.upstream_path) || !sha(file.sha256)) issue(manifestPath, `invalid license record for ${source.id}`)
      else register(file.local_path, file.sha256)
    }
  }
  for (const unit of units) {
    if (!object(unit) || !text(unit.name) || !safePath(unit.local_path)) { issue(manifestPath, 'invalid skill/reference entry'); continue }
    if ((manifest.references.includes(unit) && !unit.local_path.startsWith('third_party/skills/')) || (!manifest.references.includes(unit) && unit.local_path !== `.agents/skills/${unit.name}`)) issue(manifestPath, `invalid skill directory ${unit.local_path}`)
    if (imports.has(unit.local_path)) issue(manifestPath, `duplicate local path ${unit.local_path}`)
    imports.set(unit.local_path, unit)
    if (manifest.local_skills.includes(unit)) {
      if (!text(unit.purpose)) issue(manifestPath, `missing project purpose for ${unit.name}`)
      if (!Array.isArray(unit.files) || !unit.files.length || !unit.files.every(safePath)) issue(manifestPath, `invalid project files for ${unit.name}`)
      else for (const file of unit.files) register(join(unit.local_path, file))
    } else {
      if (!sources.has(unit.source) || !safePath(unit.upstream_path)) issue(manifestPath, `invalid source for ${unit.name}`)
      if (!['verbatim', 'verbatim-reference', 'patched', 'patched-reference', 'adapted'].includes(unit.mode)) issue(manifestPath, `invalid mode for ${unit.name}`)
      paths(unit.files, unit.name, unit.local_path)
      if (unit.mode !== 'adapted' && unit.local_path.startsWith('.agents/')) register(join(unit.local_path, 'UPSTREAM.md'))
      if (unit.optional_upstream_links?.length) issue(manifestPath, `optional_upstream_links no longer exempts references: ${unit.name}; patch links to pinned URLs`)
      const patches = unit.files?.filter?.((file: unknown) => object(file) && file.patch) ?? []
      if ((String(unit.mode).startsWith('patched')) !== (patches.length > 0)) issue(manifestPath, `patch mode does not match file patches: ${unit.name}`)
      if (unit.pinned_links !== undefined && !Array.isArray(unit.pinned_links)) issue(manifestPath, `pinned_links must be an array for ${unit.name}`)
      for (const link of Array.isArray(unit.pinned_links) ? unit.pinned_links : []) {
        const source = sources.get(unit.source)
        if (!object(link) || !safePath(link.from) || !text(link.url) || !link.url.startsWith(`${source?.repo}/blob/${source?.commit}/`)) issue(manifestPath, `invalid pinned link for ${unit.name}`)
        else {
          const from = join(root, unit.local_path, link.from)
          if (!existsSync(from) || !readFileSync(from, 'utf8').includes(`](${link.url})`)) issue(from, `pinned link is not used: ${link.url}`)
        }
      }
    }
    if (unit.commands !== undefined && !Array.isArray(unit.commands)) issue(manifestPath, `commands must be an array for ${unit.name}`)
    for (const command of Array.isArray(unit.commands) ? unit.commands : []) {
      if (!object(command) || !safePath(command.path) || !text(command.executable) || !safePath(command.from)) { issue(manifestPath, `invalid command for ${unit.name}`); continue }
      const script = join(root, unit.local_path, command.path)
      if (!existsSync(script)) issue(script, 'missing command script')
      if (!Bun.which(command.executable)) issue(script, `required executable unavailable: ${command.executable}`)
      const from = join(root, unit.local_path, command.from)
      if (!existsSync(from) || !readFileSync(from, 'utf8').includes(command.path)) issue(from, `command declaration does not match instruction: ${command.path}`)
      if (command.python_modules !== undefined && (!Array.isArray(command.python_modules) || !command.python_modules.every(text))) { issue(script, 'python_modules must be text array'); continue }
      for (const module of command.python_modules ?? []) {
        if (!/^[A-Za-z_]\w*(?:\.\w+)*$/.test(module)) issue(script, `invalid Python module: ${module}`)
        else if (spawnSync(command.executable, ['-c', `import ${module}`], { encoding: 'utf8' }).status !== 0) issue(script, `required Python module unavailable: ${module}`)
      }
    }
  }
  for (const file of [...walk(join(root, '.agents/skills')), ...walk(join(root, 'third_party/skills'))]) {
    const local = relative(root, file)
    if (!registered.has(local)) issue(file, 'unregistered file; declare source, patch or project ownership')
    if (!statSync(file).isFile()) { issue(file, 'expected regular file'); continue }
    const unit = [...imports.values()].find(value => local.startsWith(`${value.local_path}/`))
    if (basename(file) === 'openai.yaml' && basename(dirname(file)) === 'agents') {
      try {
        const metadata = Bun.YAML.parse(readFileSync(file, 'utf8'))
        if (!object(metadata)) throw new Error('host metadata must be an object')
        if (Object.keys(metadata).some(key => !['interface', 'dependencies', 'policy'].includes(key))) throw new Error('unsupported host metadata field')
        if (metadata.interface !== undefined) {
          if (!object(metadata.interface)) throw new Error('interface must be an object')
          for (const [key, value] of Object.entries(metadata.interface)) {
            if (!['display_name', 'short_description', 'icon_small', 'icon_large', 'brand_color', 'default_prompt'].includes(key)) throw new Error(`unsupported interface.${key}`)
            if (!text(value)) throw new Error(`interface.${key} must be text`)
            if (key.startsWith('icon_') && !existsSync(resolve(root, unit?.local_path ?? '', value))) throw new Error(`missing icon ${value}`)
          }
        }
        if (metadata.policy !== undefined && (!object(metadata.policy) || typeof metadata.policy.allow_implicit_invocation !== 'boolean')) throw new Error('policy.allow_implicit_invocation must be boolean')
        if (metadata.dependencies !== undefined && (!object(metadata.dependencies) || !Array.isArray(metadata.dependencies.tools) || metadata.dependencies.tools.some((tool: unknown) => !object(tool) || tool.type !== 'mcp' || !text(tool.value)))) throw new Error('invalid host tool dependencies')
      } catch (error) { issue(file, `host metadata: ${(error instanceof Error ? error.message : String(error))}`) }
    }
    if (!file.endsWith('.md')) continue
    let body = readFileSync(file, 'utf8')
    if (basename(file) === 'SKILL.md') {
      const match = body.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/)
      try {
        if (!match) throw new Error('missing YAML frontmatter')
        const metadata = Bun.YAML.parse(match[1])
        if (!object(metadata) || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(metadata.name ?? '') || metadata.name !== basename(dirname(file))) throw new Error('name must match the skill directory')
        if (!text(metadata.description)) throw new Error('description must be nonempty text')
        if (metadata.metadata !== undefined && !object(metadata.metadata)) throw new Error('metadata must be an object')
        if (local.startsWith('.agents/skills/')) {
          if (names.has(metadata.name)) throw new Error(`duplicate skill name ${metadata.name}`)
          names.add(metadata.name)
        }
        body = body.slice(match[0].length)
      } catch (error) { issue(file, (error instanceof Error ? error.message : String(error))) }
    }
    if (/\$\{(?:ENGINE_NAME|ASSET_SKILL_COMMAND|ENGINE_GUIDE_FILE|AGENT_NAME|RUNTIME_ASSET_DIR|ASSET_GEN_SKILL_DIR)\}/.test(body)) issue(file, 'unrendered Godogen runtime token')
    for (const match of clean(body).matchAll(links)) {
      const target = match[1] ?? match[2]
      if (/^(?:[a-z][a-z0-9+.-]*:|\/\/)/i.test(target)) continue
      try {
        const [link, anchor] = target.split('#')
        const destination = link ? resolve(dirname(file), decodeURIComponent(link.split('?')[0])) : file
        if (!existsSync(destination)) issue(file, `missing local reference ${target}`)
        else if (anchor && destination.endsWith('.md') && !anchors(readFileSync(destination, 'utf8'), relative(root, destination).startsWith('docs/')).has(decodeURIComponent(anchor))) issue(file, `missing anchor ${target}`)
      } catch (error) { issue(file, `invalid local reference ${target}: ${(error instanceof Error ? error.message : String(error))}`) }
    }
    // Only concrete interpreter script invocations, not arbitrary prose or asset examples
    for (const match of body.matchAll(/\b(?:python3?|bun|node|bash|sh)\s+([.\w/-]+\.(?:py|ts|mjs|js|sh))\b/g)) {
      const path = match[1]
      const base = /^(?:tools|game|\.agents)\//.test(path) ? root : join(root, unit?.local_path ?? '')
      if (!existsSync(resolve(base, path))) issue(file, `missing command path ${path}`)
    }
  }
  for (const adoption of manifest.adoptions ?? []) {
    if (!object(adoption) || !sources.has(adoption.source) || !Array.isArray(adoption.local_paths) || !adoption.local_paths.length) {
      issue(manifestPath, 'invalid adoption source or local_paths'); continue
    }
    for (const path of adoption.local_paths) if (!text(path) || path.startsWith('/') || path.split('/').includes('..') || !existsSync(join(root, path))) issue(manifestPath, `missing adoption destination ${path}`)
  }
  const expected = [...manifest.local_skills, ...manifest.skills].filter(object).map(unit => unit.name)
  for (const name of new Set([...expected, ...manifest.policy.required_skills])) if (!names.has(name)) issue(manifestPath, `missing required skill ${name}`)
  if (new Set(manifest.policy.required_skills).size !== manifest.policy.required_skills.length) issue(manifestPath, 'duplicate policy.required_skills')
  for (const name of names) if (!expected.includes(name)) issue(manifestPath, `unregistered skill ${name}`)
  return result()
}

if (import.meta.main) {
  const result = validate(process.argv[2] ?? '.')
  console.log(`${result.ok ? 'PASS' : 'FAIL'}: ${result.skills} Skills; ${result.imported} recorded imports`)
  for (const item of result.issues) console.error(`${item.path}: ${item.message}`)
  process.exitCode = result.ok ? 0 : 1
}
