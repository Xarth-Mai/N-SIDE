import { test } from 'node:test'
import assert from 'node:assert/strict'
import data from '../../source-assets/district-map/district.json' with { type: 'json' }
import { terrainHeight, heightColor } from '../district-map.mjs'

test('map places and roads have valid shared references', () => {
  assert.deepEqual(data.places.map(p=>p.id).sort(), Array.from({length:29},(_,i)=>String(i+1).padStart(2,'0')))
  for(const point of Object.values(data.nodes)) assert.ok(point.length===3 && point.every(Number.isFinite))
  for(const road of data.roads) assert.ok(road.nodes.length>=2 && road.nodes.every(id=>data.nodes[id]))
  for(const place of data.places){
    assert.ok(data.groups[place.group])
    assert.ok(place.position.length===3 && place.position.every(Number.isFinite))
  }
})

test('terrain colors follow interpolated ground elevation', () => {
  const points=[[0,0,0],[10,0,80]]
  assert.equal(terrainHeight(points,0,0),0)
  assert.equal(terrainHeight(points,5,0),40)
  assert.equal(terrainHeight(points,10,0),80)
  assert.equal(heightColor(0),'rgb(223,213,175)')
  assert.equal(heightColor(80),'rgb(103,139,112)')
  assert.equal(heightColor(-10),heightColor(0))
  assert.equal(heightColor(100),heightColor(80))
})

test('simplified map retains the shopping short loop and public hillside connection', () => {
  const linked=(a,b)=>data.roads.some(r=>r.nodes.some((n,i)=>i>0 && ((n===a && r.nodes[i-1]===b)||(n===b && r.nodes[i-1]===a))))
  for(const nodes of [
    ['home','court','market_turn','alley_corner','market_back','community','home'],
    ['home','steps_mid','upper_homes','north','forest','summit'],
  ]) for(let i=1;i<nodes.length;i++)assert.ok(linked(nodes[i-1],nodes[i]))
})

test('orthographic projection preserves parallel edges and separates elevation from depth', async () => {
  const {project,depth}=await import('../district-map.mjs')
  const a=project([10,20,0]),b=project([10,20,10])
  assert.equal(a[0],b[0])
  assert.ok(b[1]<a[1])
  const delta=(a,b)=>a.map((v,i)=>v-b[i])
  const u=delta(project([30,40,5]),project([20,40,5]))
  const v=delta(project([130,140,50]),project([120,140,50]))
  u.forEach((n,i)=>assert.ok(Math.abs(n-v[i])<1e-10))
  assert.ok(depth([0,0])>depth([0,100]))
  for(const building of data.buildings){assert.ok(building.height>0);assert.ok(Number.isFinite(building.elevation))}
})

test('scene keeps platforms below volumes and geometry is finite', async () => {
  const {buildScene}=await import('../district-map.mjs'),scene=buildScene(data)
  assert.ok(scene.surfaces.some(s=>s.place==='04'))
  assert.ok(scene.objects.some(o=>o.place==='04'&&o.kind==='building'))
  assert.ok(scene.objects.every(o=>o.kind!=='surface'))
  assert.ok(scene.objects.every((o,i)=>!i||o.depth>=scene.objects[i-1].depth))
  for(const shape of [...scene.terrain,scene.water,...scene.surfaces.flatMap(s=>s.shapes),...scene.objects.flatMap(o=>o.shapes)]){
    assert.ok(!/NaN|Infinity|undefined/.test(shape.d))
  }
  assert.ok(data.places.every(p=>p.use&&p.entry))
})
