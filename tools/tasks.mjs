import { existsSync, readFileSync, readdirSync, realpathSync, statSync, writeFileSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'
import { anchors } from './validate-skills.mjs'

const statuses = ['backlog', 'ready', 'active', 'review', 'blocked', 'done', 'cancelled']
const types = ['design', 'experiment', 'feature', 'asset', 'fix', 'document']
const fields = ['id', 'type', 'status', 'milestone', 'depends_on', 'specs', 'migrated_from', 'blocked_reason', 'resolution', 'acceptance']
const text = value => typeof value === 'string' && value.trim().length > 0
const object = value => value && typeof value === 'object' && !Array.isArray(value)
const taskId = value => typeof value === 'string' && /^TASK-\d{3,}$/.test(value)
const strings = value => Array.isArray(value) && value.every(text) && new Set(value).size === value.length
const withoutCode = body => body.replace(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\s*$/gm, '')

export function readTasks(root) {
  root = resolve(root)
  const issues = [], tasks = [], milestones = []
  const issue = (path, message) => issues.push({ path, message })
  const roadmap = 'todo/roadmap.md'
  if (!existsSync(join(root, roadmap))) issue(roadmap, 'missing roadmap')
  else {
    for (const match of readFileSync(join(root, roadmap), 'utf8').matchAll(/^## ([A-Z][A-Z0-9-]*)\s+(.+)$/gm)) {
      if (milestones.some(m => m.id === match[1])) issue(roadmap, `duplicate milestone ${match[1]}`)
      milestones.push({ id: match[1], title: match[2] })
    }
    if (!milestones.length) issue(roadmap, 'no milestone headings')
  }
  function reference(path, source, evidenceId) {
    if (!text(path)) { issue(source, 'reference must be a nonempty repository path'); return }
    const [file, fragment, extra] = path.split('#')
    const destination = resolve(root, file)
    if (!file || file.startsWith('/') || file.split('/').includes('..') || extra !== undefined || relative(root, destination).startsWith('..')) {
      issue(source, `invalid repository reference ${path}`); return
    }
    if (evidenceId && !new RegExp(`^todo/evidence/${evidenceId}/[^/]+/.+`).test(file)) issue(source, `acceptance record must be inside todo/evidence/${evidenceId}/run-id/`)
    if (!existsSync(destination) || !statSync(destination).isFile()) { issue(source, `missing reference ${path}`); return }
    if (evidenceId && !readFileSync(destination, 'utf8').trim()) issue(source, `empty acceptance record ${path}`)
    if (relative(root, realpathSync(destination)).startsWith('..')) { issue(source, `reference leaves repository ${path}`); return }
    if (fragment) {
      try {
        if (!file.endsWith('.md') || !anchors(readFileSync(destination, 'utf8'), file.startsWith('docs/')).has(decodeURIComponent(fragment))) issue(source, `missing anchor ${path}`)
      } catch { issue(source, `invalid anchor ${path}`) }
    }
  }
  const directory = join(root, 'todo/tasks')
  const files = existsSync(directory) ? readdirSync(directory).filter(name => name.endsWith('.md')).sort() : []
  if (!files.length) issue('todo/tasks', 'no task cards')
  for (const file of files) {
    const path = `todo/tasks/${file}`
    const source = readFileSync(join(root, path), 'utf8')
    const frontmatter = source.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/)
    let task
    try {
      if (!frontmatter) throw new Error('missing YAML frontmatter')
      task = Bun.YAML.parse(frontmatter[1])
      if (!object(task)) throw new Error('frontmatter must be an object')
    } catch (error) { issue(path, error.message); continue }
    for (const key of Object.keys(task)) if (!fields.includes(key)) issue(path, `unknown field ${key}`)
    if (!taskId(task.id)) issue(path, 'id must match TASK-000')
    else if (!file.startsWith(`${task.id}-`)) issue(path, 'filename must start with its task ID and remain stable')
    if (!types.includes(task.type)) issue(path, `invalid type ${task.type}`)
    if (!statuses.includes(task.status)) issue(path, `invalid status ${task.status}`)
    if (!milestones.some(m => m.id === task.milestone)) issue(path, `unknown milestone ${task.milestone}`)
    for (const key of ['depends_on', 'specs']) {
      if (!strings(task[key])) issue(path, `${key} must be a unique string array`)
    }
    if (Array.isArray(task.depends_on) && !task.depends_on.every(taskId)) issue(path, 'depends_on must contain task IDs')
    if ('migrated_from' in task && (!strings(task.migrated_from) || !task.migrated_from.length)) issue(path, 'migrated_from must be a nonempty unique string array')
    for (const [status, field] of [['blocked', 'blocked_reason'], ['cancelled', 'resolution']]) {
      if (task.status === status && !text(task[field])) issue(path, `${status} requires ${field}`)
      if (field in task && task.status !== status) issue(path, `${field} applies only to ${status}; move historical details into the body`)
    }
    if (task.status === 'done') {
      const acceptance = task.acceptance
      if (!object(acceptance)) issue(path, 'done requires acceptance')
      else {
        if (!['codex', 'author', 'playtester'].includes(acceptance.role)) issue(path, 'invalid acceptance.role')
        if (typeof acceptance.revision !== 'string' || !/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(acceptance.revision)) issue(path, 'acceptance.revision must be a full commit or content hash')
        reference(acceptance.record, path, task.id)
        for (const key of Object.keys(acceptance)) if (!['role', 'revision', 'record'].includes(key)) issue(path, `unknown acceptance field ${key}`)
      }
    } else if ('acceptance' in task) issue(path, 'only done retains current acceptance; preserve old evidence in the body when reopening')
    if (strings(task.specs)) for (const spec of task.specs) reference(spec, path)
    const body = withoutCode(frontmatter[2])
    const titles = [...body.matchAll(/^# (.+)$/gm)]
    if (titles.length !== 1) issue(path, 'task requires one level-one title')
    for (const section of ['目标与范围', '验收条件', '当前工作与下一步', '结果与证据']) {
      if (!body.includes(`\n## ${section}\n`)) issue(path, `missing section ${section}`)
    }
    const next = body.match(/^## 当前工作与下一步\n+([^\n#][^\n]*)/m)?.[1] ?? '见任务卡'
    tasks.push({ ...task, title: titles[0]?.[1] ?? file, path, next })
  }
  const byId = new Map()
  for (const task of tasks) {
    if (byId.has(task.id)) issue(task.path, `duplicate task ID ${task.id}`)
    byId.set(task.id, task)
  }
  for (const task of tasks) for (const id of Array.isArray(task.depends_on) ? task.depends_on : []) {
    const dependency = byId.get(id)
    if (!dependency) issue(task.path, `missing dependency ${id}`)
    else if (task.status !== 'cancelled' && dependency.status === 'cancelled') issue(task.path, `cancelled dependency ${id} must be replaced or removed`)
    else if (['active', 'review', 'done'].includes(task.status) && dependency.status !== 'done') issue(task.path, `dependency ${id} must be done before ${task.status}`)
  }
  const visited = new Set(), visiting = new Set()
  function visit(task) {
    if (visiting.has(task.id)) { issue(task.path, `dependency cycle at ${task.id}`); return }
    if (visited.has(task.id)) return
    visiting.add(task.id)
    for (const id of Array.isArray(task.depends_on) ? task.depends_on : []) if (byId.has(id)) visit(byId.get(id))
    visiting.delete(task.id); visited.add(task.id)
  }
  tasks.forEach(visit)
  return { ok: !issues.length, issues, tasks, milestones }
}

const boardText = value => value.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replace(/\s+/g, ' ').replace(/[\[\]|]/g, '\\$&')

export function renderBoard({ tasks, milestones }) {
  const byId = new Map(tasks.map(task => [task.id, task]))
  const rows = ['# 当前任务', '', '<!-- Generated by bun run tasks:sync; edit todo/tasks/*.md -->', '',
    '任务卡是唯一编辑源；[路线图](roadmap.md)记录成果门槛，[任务规则](../docs/dev/handbook/tasks.md)说明状态与证据。以下计数不表示工时或游戏完成比例', '',
    '| 里程碑 | 已完成 | 当前工作 | 待评审 | 受阻 | 可开始 | 待前置 | 待办 | 已取消 |', '| --- | --- | --- | --- | --- | --- | --- | --- | --- |']
  for (const milestone of milestones) {
    const group = tasks.filter(task => task.milestone === milestone.id)
    const count = status => group.filter(task => task.status === status).length
    rows.push(`| ${milestone.id} ${boardText(milestone.title)} | ${count('done')} | ${count('active')} | ${count('review')} | ${count('blocked')} | ${group.filter(task => task.status === 'ready' && task.depends_on.every(id => byId.get(id)?.status === 'done')).length} | ${group.filter(task => task.status === 'ready' && !task.depends_on.every(id => byId.get(id)?.status === 'done')).length} | ${count('backlog')} | ${count('cancelled')} |`)
  }
  for (const [status, title] of [['active', '进行中'], ['review', '等待评审'], ['blocked', '受阻'], ['ready', '近期可开始']]) {
    const group = tasks.filter(task => task.status === status && (status !== 'ready' || task.depends_on.every(id => byId.get(id)?.status === 'done')))
    rows.push('', `## ${title}`, '')
    if (!group.length) rows.push('当前无此类任务')
    for (const task of group) rows.push(`- [ ] [${task.id} ${boardText(task.title)}](${task.path.replace(/^todo\//, '')}) · \`${task.status}\` · ${task.milestone}：${boardText(task.status === 'blocked' ? task.blocked_reason : task.next)}`)
  }
  rows.push('', '全部任务与已结束记录保留在 [tasks/](tasks/)，证据保留在 [evidence/](evidence/)，历史输入见 [archive/legacy/](archive/legacy/)', '')
  return rows.join('\n')
}

export function run(root, command, log = console.log) {
  if (!['list', 'sync', 'check'].includes(command)) { log('Use list, sync or check'); return 1 }
  const result = readTasks(root)
  if (!result.ok) { for (const issue of result.issues) log(`${issue.path}: ${issue.message}`); return 1 }
  const board = renderBoard(result)
  const path = join(root, 'todo/README.md')
  if (command === 'sync') writeFileSync(path, board)
  else if (command === 'list') log(board)
  else if (!existsSync(path) || readFileSync(path, 'utf8') !== board) { log('todo/README.md is stale; run bun run tasks:sync'); return 1 }
  if (command !== 'list') log(`PASS: ${result.tasks.length} task cards; ${command}`)
  return 0
}

if (import.meta.main) {
  const args = process.argv.slice(2)
  const command = args.shift() ?? 'list'
  const rootIndex = args.indexOf('--root')
  const root = rootIndex >= 0 ? args[rootIndex + 1] : process.cwd()
  try { process.exitCode = run(root, command) }
  catch (error) { console.error(`FAIL: ${error.message}`); process.exitCode = 1 }
}
