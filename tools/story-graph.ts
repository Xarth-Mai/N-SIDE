/** Public reading model: no save data or design status is exported */
export type Fact = { fact: string; quest_id: string; label: string; reason: string }
export type Condition = Fact | { all: Condition[] } | { any: Condition[] }
export type Gate = { required_for_completion: boolean; scope: string; label: string; condition: Condition }
export type StoryNode = {
  id: string; title: string; chapter: string; role: 'main' | 'side'; act: number; order: number
  placement: string; summary: string; url: string; required: Gate[]
  recommended: { quest_id: string; reason: string }[]
  related: { quest_id: string; type: string; description: string }[]
}
export const acts = ['熟悉这里', '重新理解过去', '处理今天']
export const relationLabels: Record<string, string> = {
  character_continuation: '人物延续', clue_setup: '伏笔铺垫', clue_payoff: '回收', aftermath: '事件后续',
  location_change: '场所变化', thematic_echo: '主题呼应', historical_context: '历史补证',
}
export function leaves(condition: Condition): Fact[] {
  return 'all' in condition ? condition.all.flatMap(leaves) : 'any' in condition ? condition.any.flatMap(leaves) : [condition]
}
export function satisfied(condition: Condition, facts: ReadonlySet<string>): boolean {
  return 'all' in condition ? condition.all.every(c => satisfied(c, facts)) : 'any' in condition ? condition.any.some(c => satisfied(c, facts)) : facts.has(condition.fact)
}
/** Preserve the ALL/ANY tree when showing what remains */
export function remaining(condition: Condition, facts: ReadonlySet<string>): Condition | null {
  if (satisfied(condition, facts)) return null
  if ('all' in condition) return { all: condition.all.map(c => remaining(c, facts)).filter((c): c is Condition => c !== null) }
  if ('any' in condition) return { any: condition.any.map(c => remaining(c, facts)).filter((c): c is Condition => c !== null) }
  return condition
}
export function effects(nodes: StoryNode[], id: string) {
  return nodes.flatMap(node => node.required.filter(gate => leaves(gate.condition).some(f => f.quest_id === id)).map(gate => ({ node, gate })))
}
export function connections(nodes: StoryNode[], id: string) {
  const selected = nodes.find(n => n.id === id)
  return nodes.flatMap(node => [
    ...(selected?.related.filter(r => r.quest_id === node.id).map(r => ({ node, ...r })) ?? []),
    ...node.related.filter(r => r.quest_id === id).map(r => ({ ...r, node })),
  ])
}
