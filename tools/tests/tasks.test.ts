import { afterEach, expect, test } from 'bun:test'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { tmpdir } from 'node:os'
import { readTasks, renderBoard, run } from '../tasks.ts'

const roots: string[] = []
afterEach(() => { for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true }) })
function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'nside-tasks-')); roots.push(root)
  const write = (path: string, body: string) => { mkdirSync(dirname(join(root, path)), { recursive: true }); writeFileSync(join(root, path), body) }
  const base = { id: 'TASK-020', type: 'experiment', status: 'done', milestone: 'G1', depends_on: [], specs: ['docs/dev/spec.md#规则'], acceptance: { role: 'codex', revision: 'a'.repeat(40), record: 'todo/evidence/TASK-020/run-1/result.md#结果' } }
  const body = '# 实验\n\n## 目标与范围\n\n验证假设\n\n## 验收条件\n\n保留真实否定结果\n\n## 当前工作与下一步\n\n按结果调整规格\n\n## 结果与证据\n\n假设已被否定，实验完成\n'
  const card = (value: Record<string, unknown> = base, path = `todo/tasks/${value.id}-experiment.md`) => write(path, `---\n${Bun.YAML.stringify(value)}\n---\n${body}`)
  write('todo/roadmap.md', '# Roadmap\n\n## G1 可玩的闭环\n\n## DOCS-PIPELINE 文档与任务\n')
  write('docs/dev/spec.md', '# 规格\n\n## 规则\n\n真实规则\n')
  write('todo/evidence/TASK-020/run-1/result.md', '# 实验\n\n## 结果\n\nFAIL: 假设不成立；实验按事先规则完成\n')
  card()
  return { root, write, card, base }
}
const failure = (f: ReturnType<typeof fixture>, part: string) => { const result = readTasks(f.root); expect(result.ok).toBe(false); expect(result.issues.some(issue => issue.message.includes(part))).toBe(true) }

test('real YAML supports a completed negative experiment and exact evidence anchors', () => {
  const f = fixture()
  expect(readTasks(f.root).ok).toBe(true)
  expect(renderBoard(readTasks(f.root))).not.toContain('61')
})

test('schema, paths and state guardrails reject corrupt cards', () => {
  const changes: [((value: ReturnType<typeof fixture>['base']) => Record<string, unknown>), string][] = [
    [value => ({ ...value, id: 'OTHER-020' }), 'id must'],
    [value => ({ ...value, type: 'system' }), 'invalid type'],
    [value => ({ ...value, status: 'in_progress' }), 'invalid status'],
    [value => ({ ...value, milestone: 'M0' }), 'unknown milestone'],
    [value => ({ ...value, title: 'duplicate title' }), 'unknown field'],
    [value => ({ ...value, depends_on: ['TASK-001', 'TASK-001'] }), 'unique string array'],
    [value => ({ ...value, specs: ['docs/dev/missing.md'] }), 'missing reference'],
    [value => ({ ...value, specs: ['docs/dev/spec.md#missing'] }), 'missing anchor'],
    [value => ({ ...value, specs: ['../outside.md'] }), 'invalid repository reference'],
    [value => ({ ...value, acceptance: undefined }), 'requires acceptance'],
    [value => ({ ...value, acceptance: { ...value.acceptance, revision: 'main' } }), 'full commit'],
    [value => ({ ...value, acceptance: { ...value.acceptance, record: 'docs/dev/spec.md' } }), 'inside todo/evidence'],
    [value => ({ ...value, acceptance: { ...value.acceptance, role: 'robot' } }), 'acceptance.role'],
    [value => ({ ...value, status: 'active' }), 'only done'],
    [value => ({ ...value, status: 'blocked', acceptance: undefined }), 'requires blocked_reason'],
    [value => ({ ...value, status: 'cancelled', acceptance: undefined }), 'requires resolution'],
  ]
  for (const [change, message] of changes) {
    const f = fixture(); f.card(change(f.base), 'todo/tasks/TASK-020-experiment.md'); failure(f, message)
  }
})

test('dependencies are unique, real, acyclic, and cancellation does not satisfy a result', () => {
  const f = fixture()
  const next = { id: 'TASK-021', type: 'feature', status: 'ready', milestone: 'G1', depends_on: ['TASK-020'], specs: [] }
  f.card(next)
  expect(readTasks(f.root).ok).toBe(true)
  expect(renderBoard(readTasks(f.root))).toContain('[TASK-021')
  f.card({ ...next, depends_on: ['TASK-999'] }); failure(f, 'missing dependency')
  f.card(next)
  f.card({ ...f.base, status: 'cancelled', acceptance: undefined, resolution: '移除假设对应范围' }); failure(f, 'cancelled dependency')
  f.card({ ...f.base, status: 'ready', acceptance: undefined, depends_on: ['TASK-021'] }); failure(f, 'cycle')
  f.card({ ...f.base, status: 'review', acceptance: undefined, depends_on: ['TASK-021'] }); failure(f, 'must be done')
  f.card(f.base); f.card(f.base, 'todo/tasks/TASK-020-duplicate.md'); failure(f, 'duplicate task ID')
})

test('reopening removes current acceptance while old evidence stays in the body', () => {
  const f = fixture()
  f.card({ ...f.base, status: 'active' }); failure(f, 'only done')
  const { acceptance, ...reopened } = f.base
  f.card({ ...reopened, status: 'active' })
  expect(readTasks(f.root).ok).toBe(true)
  expect(readFileSync(join(f.root, acceptance.record.split('#')[0]), 'utf8')).toContain('假设不成立')
})

test('check is read-only, sync deterministic, list read-only and stale output fails', () => {
  const f = fixture(), log = () => {}
  expect(run(f.root, 'sync', log)).toBe(0)
  const path = join(f.root, 'todo/README.md'), original = readFileSync(path, 'utf8'), before = statSync(path, { bigint: true }).mtimeNs
  expect(run(f.root, 'check', log)).toBe(0)
  expect(run(f.root, 'list', log)).toBe(0)
  expect(statSync(path, { bigint: true }).mtimeNs).toBe(before)
  expect(run(f.root, 'sync', log)).toBe(0)
  expect(readFileSync(path, 'utf8')).toBe(original)
  writeFileSync(path, 'stale\n')
  const staleTime = statSync(path, { bigint: true }).mtimeNs
  expect(run(f.root, 'check', log)).toBe(1)
  expect(readFileSync(path, 'utf8')).toBe('stale\n')
  expect(statSync(path, { bigint: true }).mtimeNs).toBe(staleTime)
})

test('empty task source or missing roadmap cannot pass', () => {
  const f = fixture()
  f.write('todo/evidence/TASK-020/run-1/result.md', ''); failure(f, 'empty acceptance record')
  rmSync(join(f.root, 'todo/tasks'), { recursive: true }); failure(f, 'no task cards')
  rmSync(join(f.root, 'todo/roadmap.md')); failure(f, 'missing roadmap')
})
