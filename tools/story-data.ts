import { existsSync, readFileSync } from 'node:fs'
import { filesIn } from './wiki-data.ts'
import { join, resolve } from 'node:path'
import { effects, relationLabels } from './story-graph.ts'
import type { Condition, StoryNode } from './story-graph.ts'
export type Atom = { type: 'quest' | 'beat' | 'information' | 'state' | 'location' | 'relationship'; quest_id: string; target: string; value: unknown; reason: string }
export type Requirement = Atom | { all: Requirement[] } | { any: Requirement[] }
type Outcome = { id: string; type: Atom['type']; value: unknown; label: string; source: string; gate?: string }
export type Quest = { id: string; title: string; chapter_label: string; source_file: string; story: {
  role: 'main' | 'side'; optional: boolean; act: number; display_order: number; placement: string; summary: string
  narrative?: string; outcomes: Outcome[]; required: { required_for_completion: boolean; scope: string; label: string; condition: Requirement }[]
  recommended: { quest_id: string; reason: string; source: string }[]
  related: { quest_id: string; type: string; description: string; source: string }[]
  review: { basis: string; finding: string; pending: string[] }
} }
type Narrative = { quest_id: string; information: {id: string; proposition: string; initial_state: string}[]; beats: {id: string; player_goal: string; delivers: Record<string,string>; state_changes: Record<string,unknown>}[] }
const atomKey = (a: Atom) => JSON.stringify([a.quest_id, a.type, a.target, a.value])
const atoms = (c: Requirement): Atom[] => 'all' in c ? c.all.flatMap(atoms) : 'any' in c ? c.any.flatMap(atoms) : [c]
const read = (path: string) => JSON.parse(readFileSync(path, 'utf8'))
export function loadQuests(root: string): Quest[] {
  const catalog=read(join(root,'docs/dev/design/catalogs/quests.json'))
  if(catalog.story_schema_version!==1) throw new Error('quests.json: unsupported story_schema_version')
  return catalog.quests
}
function narratives(root: string, quests: Quest[]) {
  return new Map(quests.filter(q => q.story.narrative).map(q => [q.id, read(join(root, 'docs', q.story.narrative!)) as Narrative]))
}
function label(a: Atom, quests: Quest[], ns: Map<string, Narrative>) {
  const q = quests.find(q => q.id === a.quest_id)!
  if (a.type === 'quest') return `完成「${q.title}」`
  const n = ns.get(q.id)
  if (a.type === 'beat') return `完成节点「${n?.beats.find(b => b.id === a.target)?.player_goal ?? a.target}」`
  if (n && a.type === 'information') return `${n.information.find(i => i.id === a.target)?.proposition}（${a.value}）`
  if (n && a.type === 'state') return `已达到状态：${n.beats.find(b => JSON.stringify(b.state_changes[a.target]) === JSON.stringify(a.value))?.player_goal ?? '对应阶段成果'}`
  return q.story.outcomes.find(o => o.id === a.target)?.label ?? a.target
}
export function projectStories(root: string, quests = loadQuests(root)): StoryNode[] {
  const ns = narratives(root, quests), keys = new Map<string, string>()
  const convert = (c: Requirement): Condition => {
    if ('all' in c) return { all: c.all.map(convert) }
    if ('any' in c) return { any: c.any.map(convert) }
    const key = atomKey(c)
    if (!keys.has(key)) keys.set(key, `condition-${keys.size + 1}`)
    return { fact: keys.get(key)!, quest_id: c.quest_id, label: label(c, quests, ns), reason: c.reason }
  }
  return [...quests].sort((a,b) => a.story.display_order-b.story.display_order).map(q => ({
    id: q.id, title: q.title, chapter: q.chapter_label, role: q.story.role, act: q.story.act, order: q.story.display_order,
    placement: q.story.placement, summary: q.story.summary, url: '/' + q.source_file.replace(/\.md$/, ''),
    required: q.story.required.map(g => ({required_for_completion:g.required_for_completion,scope:g.scope,label:g.label,condition:convert(g.condition)})),
    recommended: q.story.recommended.map(r => ({quest_id:r.quest_id,reason:r.reason})),
    related: q.story.related.map(r => ({quest_id:r.quest_id,type:r.type,description:r.description})),
  }))
}
/** Check untrusted JSON before evaluating any condition */
export function validateStories(root: string, input: unknown = loadQuests(root)): string[] {
  const errors: string[] = [], add = (where: string, why: string) => errors.push(`${where}: ${why}`)
  const obj = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v)
  const text = (v: unknown) => typeof v === 'string' && v.trim().length > 0
  if (!Array.isArray(input)) return ['quests: expected array']
  const ids = new Set<string>(), orders = new Set<number>(), pages = new Set<string>()
  function source(v: unknown, where: string) {
    if (!text(v)) return add(where, 'missing source')
    const [file, anchor] = (v as string).split('#'), path = resolve(root,'docs',file)
    if (!path.startsWith(resolve(root,'docs/player/encyclopedia/story')+'/') || !existsSync(path)) return add(where,'invalid story source')
    if (anchor && !readFileSync(path,'utf8').includes(`## ${anchor}`)) add(where,'missing source heading')
  }
  function condition(v: unknown, where: string, owner: string, depth = 0) {
    if (depth > 30 || !obj(v)) return add(where,'invalid condition tree')
    if ('all' in v || 'any' in v) {
      const key = 'all' in v ? 'all' : 'any'
      if (Object.keys(v).length !== 1 || !Array.isArray(v[key]) || !v[key].length) return add(where,'ALL/ANY must have nonempty children and no mixed semantics')
      v[key].forEach((c: unknown,i: number) => condition(c,`${where}.${key}[${i}]`,owner,depth+1)); return
    }
    if (Object.keys(v).some(k => !['type','quest_id','target','value','reason'].includes(k)) || !['quest','beat','information','state','location','relationship'].includes(String(v.type)) || !text(v.quest_id) || !text(v.target) || !text(v.reason) || !('value' in v)) add(where,'invalid typed condition or mixed recommendation')
    if (v.quest_id === owner) add(where,'self dependency: use narrative.requires for internal beats')
  }
  input.forEach((q: unknown,i) => {
    const w = `quests[${i}]`
    if (!obj(q) || !text(q.id) || !obj(q.story)) { add(w,'missing identity/story'); return }
    const s=q.story, id=String(q.id)
    if(Object.keys(s).some(k=>!['role','optional','act','display_order','placement','summary','narrative','outcomes','required','recommended','related','review'].includes(k))) add(id,'unknown story field; use required/recommended/related instead of prerequisites or unlocks')
    if(!text(q.title)||!text(q.chapter_label)||!/^QST-\d{3,}$/.test(id)) add(id,'invalid quest identity/title')
    if(s.narrative !== undefined && s.narrative !== `dev/design/quests/${id}/narrative.json`) add(id,'invalid narrative path')
    if(ids.has(id)) add(id,'duplicate quest'); ids.add(id)
    if(!['main','side'].includes(String(s.role)) || typeof s.optional !== 'boolean' || (s.role==='side' && s.optional!==true)) add(id,'missing role / side optional policy')
    if(!Number.isInteger(s.act) || Number(s.act)<1 || Number(s.act)>3) add(id,'invalid act')
    if(typeof s.display_order!=='number' || !Number.isFinite(s.display_order) || s.display_order<=0 || orders.has(s.display_order)) add(id,'invalid/duplicate display order')
    orders.add(Number(s.display_order)); source(q.source_file,id)
    if(pages.has(String(q.source_file))) add(id,'duplicate story page'); pages.add(String(q.source_file))
    for(const f of ['placement','summary']) if(!text(s[f])) add(id,`missing ${f}`)
    if(!obj(s.review) || !text(s.review.finding) || !Array.isArray(s.review.pending) || s.review.pending.some(p=>!text(p))) add(id,'missing audit')
    else source(s.review.basis,id)
    const scopes=new Set<string>()
    for(const field of ['required','recommended','related','outcomes']) {
      if(!Array.isArray(s[field])) {add(id,`missing ${field}`); continue}
      for(const r of s[field]) {
        if(!obj(r)) {add(id,`invalid ${field}`); continue}
        if(field==='required') {
          if(Object.keys(r).some(k=>!['scope','label','condition','required_for_completion'].includes(k))) add(id,'unknown gate field')
          if(typeof r.required_for_completion !== 'boolean' || (r.scope==='start' && r.required_for_completion!==true) || !text(r.scope)||!text(r.label)||scopes.has(String(r.scope))) add(id,'invalid/duplicate gate')
          scopes.add(String(r.scope)); condition(r.condition,id,id)
        } else if(field==='outcomes') {
          if(!text(r.id)||!text(r.label)||!['information','state','location','relationship'].includes(String(r.type))||!('value' in r)) add(id,'invalid outcome')
          source(r.source,id)
        } else {
          const allowed=field==='recommended'?['quest_id','reason','source']:['quest_id','type','description','source']
          if(Object.keys(r).some(k=>!allowed.includes(k))||!text(r.quest_id)||!text(r[field==='recommended'?'reason':'description'])) add(id,`invalid ${field}; cannot carry blocking conditions`)
          if(field==='related' && !Object.hasOwn(relationLabels,String(r.type))) add(id,'unknown relation type')
          source(r.source,id)
        }
      }
    }
  })
  if(errors.length) return errors
  const quests=input as Quest[]
  for(const file of filesIn(join(root,'docs/player/encyclopedia/story')).filter(f=>f.endsWith('.md'))) {
    const id=readFileSync(file,'utf8').match(/^(?:quest_id|subject_id):\s*(QST-\d+)\s*$/m)?.[1]
    if(id && !quests.some(q=>q.id===id && resolve(root,'docs',q.source_file)===file)) add(id,'story page missing from authoritative catalog or mismatched identity')
  }
  let ns: Map<string,Narrative>
  try {ns=narratives(root,quests)} catch(e) {return [`narrative: ${String(e)}`]}
  for(const q of quests) {
    const s=q.story, seen=new Set<string>()
    if(s.narrative && (s.narrative!==`dev/design/quests/${q.id}/narrative.json` || ns.get(q.id)?.quest_id!==q.id)) add(q.id,'narrative ownership mismatch')
    for(const o of s.outcomes) {
      if(seen.has(o.id)) add(q.id,'duplicate outcome'); seen.add(o.id)
      if(o.gate && !s.required.some(g=>g.scope===o.gate)) add(q.id,'unknown outcome production gate')
      if(ns.has(q.id)) add(q.id,'use existing narrative delivers/state_changes instead of parallel outcomes')
    }
    for(const r of [...s.recommended,...s.related]) if(!ids.has(r.quest_id)||r.quest_id===q.id) add(q.id,'invalid relation/recommendation quest')
    for(const g of s.required) for(const a of atoms(g.condition)) {
      const target=quests.find(t=>t.id===a.quest_id)
      if(!target) {add(q.id,`unknown quest ${a.quest_id}`);continue}
      const n=ns.get(a.quest_id)
      const valid=a.type==='quest' ? a.target===a.quest_id && a.value===true
        : a.type==='beat' ? !!n?.beats.some(b=>b.id===a.target) && a.value===true
        : n && a.type==='information' ? n.information.some(i=>i.id===a.target) && (n.information.some(i=>i.id===a.target&&i.initial_state===a.value)||n.beats.some(b=>b.delivers[a.target]===a.value))
        : n && a.type==='state' ? n.beats.some(b=>Object.hasOwn(b.state_changes,a.target)&&JSON.stringify(b.state_changes[a.target])===JSON.stringify(a.value))
        : target.story.outcomes.some(o=>o.type===a.type&&o.id===a.target&&JSON.stringify(o.value)===JSON.stringify(a.value))
      if(!valid) add(q.id,`unknown/unproduced ${a.type} ${a.target} = ${JSON.stringify(a.value)}`)
    }
  }
  const mains=quests.filter(q=>q.story.role==='main').sort((a,b)=>a.story.display_order-b.story.display_order)
  if(mains.some((q,i)=>q.story.display_order!==i+1||(i>0&&q.story.act<mains[i-1].story.act))) add('main','abnormal chapter order/act')
  for(const q of quests.filter(q=>q.story.role==='side')) {
    const previous=mains.filter(m=>m.story.display_order<=q.story.display_order).at(-1)
    if(!previous||q.story.display_order>=mains.at(-1)!.story.display_order+1||q.story.act!==previous.story.act) add(q.id,'side display position/act outside main timeline')
  }
  if(errors.length) return errors
  // Least fixed point handles ANY escape paths; a stage result is unavailable until its gate is reachable
  function reachable(skipSide: boolean) {
    const started=new Set<string>(), ready=new Set<string>()
    function available(a: Atom): boolean {
      const q=quests.find(q=>q.id===a.quest_id)!
      if(!started.has(q.id)) return false
      if(a.type==='quest') return q.story.required.filter(g=>g.required_for_completion).every(g=>ready.has(`${q.id}/${g.scope}`))
      const gate=q.story.outcomes.find(o=>o.id===a.target)?.gate
      return !gate||ready.has(`${q.id}/${gate}`)
    }
    function evaluate(c: Requirement): boolean {return 'all' in c?c.all.every(evaluate):'any' in c?c.any.some(evaluate):available(c)}
    let changed=true
    while(changed) {
      changed=false
      for(const q of quests) {
        if(skipSide&&q.story.optional) continue
        const start=q.story.required.find(g=>g.scope==='start')
        if(!started.has(q.id)&&(!start||evaluate(start.condition))) {started.add(q.id);changed=true}
        if(started.has(q.id)) for(const g of q.story.required) if(!ready.has(`${q.id}/${g.scope}`)&&evaluate(g.condition)) {ready.add(`${q.id}/${g.scope}`);changed=true}
      }
    }
    return {started,ready}
  }
  const all=reachable(false), main=reachable(true)
  for(const q of quests) {
    if(!all.started.has(q.id)) add(q.id,'unreachable mandatory start cycle')
    for(const g of q.story.required) if(!all.ready.has(`${q.id}/${g.scope}`)) add(q.id,`unreachable mandatory stage ${g.scope}`)
    if(q.story.role==='main' && (!main.started.has(q.id)||q.story.required.some(g=>g.required_for_completion&&!main.ready.has(`${q.id}/${g.scope}`)))) add(q.id,'main unavoidably depends on optional side content')
  }
  return errors
}

export function auditMarkdown(root: string) {
  const qs=loadQuests(root), nodes=projectStories(root,qs)
  const describe=(c: Condition): string=>'all' in c?'全部（'+c.all.map(describe).join('；')+'）':'any' in c?'任一（'+c.any.map(describe).join('；')+'）':`${c.label}：${c.reason}`
  return qs.map(q=>{
    const n=nodes.find(n=>n.id===q.id)!,s=q.story
    return `### ${q.id} · ${q.title}\n\n[正文](/${q.source_file}) · ${s.role==='main'?'主线':'可选支线'} · 第 ${s.act} 幕 · ${s.placement}\n\n${s.review.finding}\n\n- 强制条件：${n.required.map(g=>`${g.scope==='start'?'开始':`${g.required_for_completion?'必需阶段':'可选片段'}：${g.scope}`}／${g.label}：${describe(g.condition)}`).join('；')||'未规定跨任务硬锁'}\n- 推荐先读：${s.recommended.map(r=>`${qs.find(q=>q.id===r.quest_id)!.title}：${r.reason}`).join('；')||'无'}\n- 剧情关联：${s.related.map(r=>`${qs.find(q=>q.id===r.quest_id)!.title}（${relationLabels[r.type]}）：${r.description}`).join('；')||'无主动登记；反向关联由界面生成'}\n- 后续条件影响：${effects(nodes,q.id).map(e=>`${e.node.title}／${e.gate.label}`).join('；')||'无已登记硬条件；推荐和剧情关联仍独立显示'}\n- 待作者决定：${s.review.pending.join('；')}\n`
  }).join('\n')
}
if(import.meta.main) {
  const root=resolve(import.meta.dir,'..'), errors=validateStories(root)
  console.log(errors.length?errors.join('\n'):`PASS: ${loadQuests(root).length} stories; mandatory conditions, references, reachability and optional-side boundary`)
  process.exitCode=errors.length?1:0
}
