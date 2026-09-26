import { test, expect } from 'bun:test'
import { resolve } from 'node:path'
import { readFileSync } from 'node:fs'
import { effects, remaining, satisfied } from '../story-graph.ts'
import type { Condition } from '../story-graph.ts'
import { loadQuests, projectStories, validateStories } from '../story-data.ts'
import type { Atom, Quest, Requirement } from '../story-data.ts'
const root=resolve(import.meta.dir,'../..')
const fresh=()=>structuredClone(loadQuests(root))
const q=(qs: Quest[],id: string)=>qs.find(q=>q.id===id)!
const complete=(quest_id: string): Atom=>({type:'quest',quest_id,target:quest_id,value:true,reason:'test dependency'})
const gate=(condition: Requirement)=>({required_for_completion:true,scope:'start',label:'test gate',condition})
const fact=(name:string): Condition=>({fact:name,quest_id:name,label:name,reason:'reason'})

test('nested ALL/ANY keeps alternatives after partial satisfaction, never unlocks from one parent',()=>{
 const tree={all:[fact('A'),{any:[fact('C'),{all:[fact('D'),fact('E')]}]}]}
 expect(satisfied(tree,new Set(['A']))).toBe(false)
 expect(remaining(tree,new Set(['A']))).toEqual({all:[{any:[fact('C'),{all:[fact('D'),fact('E')]}]}]})
 expect(satisfied(tree,new Set(['A','D']))).toBe(false)
 expect(satisfied(tree,new Set(['A','D','E']))).toBe(true)
 expect(remaining(tree,new Set(['A','C']))).toBeNull()
})

test('all 22 stories validate; projection excludes narrative internals and design metadata',()=>{
 expect(validateStories(root)).toEqual([])
 const nodes=projectStories(root)
 expect(nodes.filter(n=>n.role==='main')).toHaveLength(10)
 expect(nodes.filter(n=>n.role==='side')).toHaveLength(12)
 expect(JSON.stringify(nodes)).not.toMatch(/QST-002-B016|state_changes|resolution_committed|accepted|narrative.json|review|dev\/design/)
 const aftermath=effects(nodes,'QST-002')
 expect(aftermath).toHaveLength(1)
 expect(aftermath[0].gate.scope).toBe('米娜来信重访')
 expect(effects(nodes,'QST-108')).toEqual([])
})

test('current story separates early history, later proof and optional life outcomes',()=>{
 const qs=fresh(), nodes=projectStories(root)
 expect(q(qs,'QST-003').story.outcomes.some(o=>o.id==='source-ended')).toBe(true)
 expect(q(qs,'QST-006').story.outcomes.some(o=>o.id==='source-ended')).toBe(false)
 const proof=q(qs,'QST-006').story.outcomes.find(o=>o.id==='old-method-limits')!
 expect(q(qs,'QST-006').story.required.some(g=>g.scope===proof.gate)).toBe(true)
 expect(effects(nodes,'QST-005').some(e=>e.node.id==='QST-006')).toBe(true)
 const final=qs.find(q=>q.id==='QST-009') as Quest & {diver:string;reality_receiver:string}
 expect([final.diver,final.reality_receiver]).toEqual(['CHR-002','CHR-001'])
 const late=q(qs,'QST-107')
 expect(late.story.optional).toBe(true)
 expect(effects(nodes,'QST-003').some(e=>e.node.id===late.id&&e.gate.scope==='start')).toBe(true)
 for(const id of ['QST-019','QST-020','QST-021','QST-022']) {
  expect(q(qs,id).story.optional).toBe(true)
  expect(q(qs,id).story.required).toEqual([])
  expect(effects(nodes,id)).toEqual([])
 }
})

test('unknown quests, self locks and invalid typed narrative targets fail explicitly',()=>{
 for(const condition of [complete('QST-999'),complete('QST-001'),{...complete('QST-002'),type:'beat',target:'QST-002-B999'}, { ...complete('QST-002'),type:'information',target:'QST-002-I007',value:'disproved'}, {...complete('QST-002'),type:'state',target:'qst002.output_isolated',value:false}] as Requirement[]) {
  const qs=fresh();q(qs,'QST-001').story.required=[gate(condition)]
  expect(validateStories(root,qs).length).toBeGreaterThan(0)
 }
 const qs=fresh();q(qs,'QST-003').story.required=[gate({...complete('QST-002'),type:'state',target:'qst002.output_isolated'})]
 expect(validateStories(root,qs)).toEqual([])
})

test('mandatory cycles with no entry fail, ANY external entry succeeds',()=>{
 const qs=fresh();q(qs,'QST-001').story.required=[gate(complete('QST-003'))];q(qs,'QST-003').story.required=[gate(complete('QST-001'))]
 expect(validateStories(root,qs).join()).toMatch(/cycle/)
 q(qs,'QST-001').story.required=[gate({any:[complete('QST-003'),complete('QST-002')]})]
 expect(validateStories(root,qs)).toEqual([])
})

test('optional side cannot be unavoidable, including a mandatory main stage; ANY main alternative is valid',()=>{
 const qs=fresh();q(qs,'QST-003').story.required=[gate(complete('QST-108'))]
 expect(validateStories(root,qs).join()).toMatch(/optional side/)
 q(qs,'QST-003').story.required=[gate({all:[complete('QST-002'),{any:[complete('QST-108'),complete('QST-001')]}]})]
 expect(validateStories(root,qs)).toEqual([])
 q(qs,'QST-003').story.required=[{...gate(complete('QST-108')),scope:'必需阶段'}]
 expect(validateStories(root,qs).join()).toMatch(/optional side/)
})

test('outcomes cannot escape their required production stage in a cycle',()=>{
 const qs=fresh();q(qs,'QST-006').story.required.push(gate({type:'information',quest_id:'QST-007',target:'current-risk',value:'confirmed',reason:'cycle'}))
 expect(validateStories(root,qs).join()).toMatch(/unreachable/)
})

test('recommendations and associations never block; mixed condition semantics are rejected',()=>{
 const qs=fresh();q(qs,'QST-001').story.recommended.push({quest_id:'QST-108',reason:'可选铺垫',source:q(qs,'QST-108').source_file})
 expect(validateStories(root,qs)).toEqual([])
 const raw=qs as unknown as {story:{recommended:Record<string,unknown>[]}}[]
 raw[0].story.recommended[0].condition={all:[complete('QST-108')]}
 expect(validateStories(root,qs).join()).toMatch(/cannot carry blocking/)
})

test('broken role, order, relation, schema and source references do not silently repair',()=>{
 const mutations: ((qs: Quest[])=>void)[]=[
  qs=>{q(qs,'QST-001').story.role='daily' as 'main'},
  qs=>{q(qs,'QST-002').story.display_order=1},
  qs=>{q(qs,'QST-101').story.act=3},
  qs=>{q(qs,'QST-001').story.related[0].quest_id='QST-999'},
  qs=>{q(qs,'QST-001').story.required=[gate({all:[]})]},
  qs=>{q(qs,'QST-003').story.outcomes[0].source+='-missing'},
  qs=>{q(qs,'QST-002').story.narrative='../outside.json'},
  qs=>{q(qs,'QST-007').story.outcomes[0].gate='missing'},
 ]
 for(const mutate of mutations){const qs=fresh();mutate(qs);expect(validateStories(root,qs).length).toBeGreaterThan(0)}
})

test('single source owns all indexed story pages',()=>{
 const qs=fresh()
 for(const quest of qs) expect(readFileSync(resolve(root,'docs',quest.source_file),'utf8')).toContain(quest.id)
})


test('removing an otherwise unreferenced story cannot silently drop a page',()=>{
 const qs=fresh().filter(q=>q.id!=='QST-102')
 expect(validateStories(root,qs).join()).toMatch(/missing from authoritative catalog/)
})


test('optional vignettes may depend on optional stories without blocking main completion',()=>{
 const qs=fresh();q(qs,'QST-003').story.required=[{...gate(complete('QST-108')),scope:'可选重访',required_for_completion:false}]
 q(qs,'QST-004').story.required=[gate(complete('QST-003'))]
 expect(validateStories(root,qs)).toEqual([])
})
