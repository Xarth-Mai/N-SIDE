import { test } from 'node:test'
import assert from 'node:assert/strict'
import { roadWidthConflicts } from '../check-road-width.ts'
import type { District } from '../district-types.ts'
import source from '../../source-assets/district-map/district.json' with { type: 'json' }

test('shared endpoints do not hide width conflicts and truly level junctions pass',()=>{
  const data:Pick<District,'nodes'|'roads'|'routes'>={
    nodes:{center:[0,0,0],up:[10,0,10],side:[10,2,0]},
    roads:[{nodes:['center','up'],kind:'steps',width:4},{nodes:['center','side'],kind:'lane',width:4}],
    routes:[{id:'walk',name:'test walk',nodes:['center','up'],user:'public'}],
  }
  const bad=roadWidthConflicts(data,'walk')
  assert.equal(bad.length,1)
  assert.deepEqual(bad[0].sharedNodes,['center'])
  assert.ok(bad[0].maxDelta>5,'same graph node does not make overlapping slopes coplanar')
  data.nodes.up[2]=0
  assert.deepEqual(roadWidthConflicts(data,'walk'),[])
  assert.throws(()=>roadWidthConflicts(data,'typo'),/Unknown route/)
})

test('a selected route is checked against other roads without absorbing unrelated conflicts',()=>{
  const data:Pick<District,'nodes'|'roads'|'routes'>={
    nodes:{a:[0,0,0],b:[10,0,0],c:[100,0,0],d:[110,0,10],e:[110,2,0]},
    roads:[{nodes:['a','b'],kind:'lane',width:4},{nodes:['c','d'],kind:'steps',width:4},{nodes:['c','e'],kind:'lane',width:4}],
    routes:[{id:'walk',name:'test walk',nodes:['a','b'],user:'public'}],
  }
  assert.deepEqual(roadWidthConflicts(data,'walk'),[])
  assert.equal(roadWidthConflicts(data,'all').length,1)
  data.nodes.c=[5,-5,1];data.nodes.d=[5,5,1]
  assert.ok(roadWidthConflicts(data,'walk').some(c=>c.a.nodes.includes('a')&&c.b.nodes.includes('c')))
})

test('upper-street and foothill junctions join at road width with usable stair treads',()=>{
  const data:District=source
  const junctions=new Set(['old_home','home_north','west_upper','upper_homes','foothill_east','north'])
  const conflicts=roadWidthConflicts(data,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>id.startsWith('junction_')))
  assert.deepEqual(conflicts,[],'the repaired approach must clear every other road, including its shared node')
  for(const road of data.roads.filter(r=>r.kind==='steps'&&r.nodes.some(id=>id.startsWith('stair_old_home_level_west_upper_')||id.startsWith('stair_home_north_level_upper_homes_')||id.startsWith('stair_foothill_east_fw_e_forest_turn_')))) {
    const [a,b]=road.nodes.map(id=>data.nodes[id]),risers=Math.ceil(Math.abs(b[2]-a[2])/.17)
    assert.ok(risers<=48&&Math.hypot(b[0]-a[0],b[1]-a[1])/risers>=.28,`${road.nodes.join(' -> ')}: clearing the junction must retain a usable flight`)
  }
  assert.deepEqual(roadWidthConflicts(data,'hill-short'),[],'the previously checked short ascent remains clear')
})

test('campus stairs and market entrances meet the full road width at their own levels',()=>{
  const junctions=new Set(['market_entry','market_turn','school_steps_east_base','school_steps_east_mid','school_steps_east_rest','fw_e_school_edge_top'])
  const conflicts=roadWidthConflicts(source,'all').filter(c=>c.sharedNodes.some(id=>junctions.has(id))||[...c.a.nodes,...c.b.nodes].some(id=>/^junction_(school|market)_/.test(id)))
  assert.deepEqual(conflicts,[],'approach landings must clear both neighboring roads and the next stair flight')
  assert.ok(!source.roads.some(r=>r.nodes.some((id,i)=>id===r.nodes[i+2])),'an exact out-and-back road segment creates duplicate ribbons and an undefined turn')
})
