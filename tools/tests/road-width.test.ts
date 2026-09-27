import { test } from 'node:test'
import assert from 'node:assert/strict'
import { roadWidthConflicts } from '../check-road-width.ts'
import type { District } from '../district-types.ts'

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
