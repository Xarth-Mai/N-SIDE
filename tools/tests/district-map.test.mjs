import { test } from 'node:test'
import assert from 'node:assert/strict'
import data from '../../source-assets/district-map/district.json' with { type: 'json' }
import { terrainHeight, heightColor } from '../district-map.mjs'

test('map places and roads have valid shared references', () => {
  const ids = new Set(data.places.map(p=>p.id))
  assert.ok(ids.size > 0)
  assert.equal(ids.size, data.places.length, 'place IDs must be unique')
  assert.ok(data.places.every(p=>typeof p.id==='string' && p.id.trim()))
  for(const object of [...data.buildings,...data.surfaces]) {
    if(object.place!==undefined) assert.ok(ids.has(object.place), `unknown place: ${object.place}`)
  }
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

test('expanded plan counts destinations separately and keeps parcel attachments valid', async () => {
  const {inside,planStats}=await import('../district-plan.mjs')
  const parcels=new Map(data.parcels.map(p=>[p.id,p])),blocks=new Map(data.blocks.map(b=>[b.id,b]))
  const buildings=new Map(data.buildings.map(b=>[b.id,b])),places=new Map(data.places.map(p=>[p.id,p]))
  assert.equal(parcels.size,data.parcels.length)
  assert.equal(buildings.size,data.buildings.length)
  assert.equal(blocks.size,data.blocks.length)
  const stats=planStats(data)
  assert.ok(stats.blocks>=10&&stats.blocks<=14)
  assert.ok(stats.capacity>=220&&stats.capacity<=350)
  assert.ok(stats.places>=70&&stats.places<=100)
  assert.equal(stats.detailed,24)
  for(const b of data.blocks){
    assert.equal(data.places.filter(p=>p.block===b.id&&p.featured).length,b.targetPlaces)
    assert.equal(data.parcels.filter(p=>p.block===b.id).reduce((n,p)=>n+p.capacity,0),b.targetBuildings)
  }
  for(const p of data.parcels){
    assert.ok(data.nodes[p.access],p.id)
    assert.ok(p.polygon.every(xy=>inside(xy,blocks.get(p.block).polygon)),p.id)
  }
  for(const p of data.places){
    assert.ok(blocks.has(p.block),p.id)
    if(p.featured)assert.ok(parcels.has(p.parcel),p.id)
    if(p.parcel){assert.equal(parcels.get(p.parcel).block,p.block);assert.ok(inside(p.position,parcels.get(p.parcel).polygon),p.id)}
    if(p.building)assert.ok(buildings.has(p.building),p.id)
    if(p.parent){assert.ok(places.has(p.parent));assert.equal(p.building,places.get(p.parent).building);assert.notEqual(p.id,p.parent)}
    if(p.detail)assert.ok(p.brief?.space&&p.brief?.flow&&p.brief?.service&&p.brief?.public&&p.brief?.reference.length,p.id)
  }
  for(const b of data.buildings.filter(b=>b.bank==='district'))assert.ok(b.polygon.every(xy=>inside(xy,parcels.get(b.parcel).polygon)),b.id)
})

test('sections follow connected routes and distinguish a lift from a slope', async () => {
  const {routeProfile}=await import('../district-plan.mjs')
  const linked=(a,b)=>data.roads.some(r=>r.nodes.some((n,i)=>i>0&&((n===a&&r.nodes[i-1]===b)||(n===b&&r.nodes[i-1]===a))))
  for(const route of [...data.routes,...data.sections])for(let i=1;i<route.nodes.length;i++)assert.ok(linked(route.nodes[i-1],route.nodes[i]),`${route.id}: ${route.nodes[i]}`)
  const home=routeProfile(data.nodes,data.routes.find(r=>r.id==='home').nodes)
  assert.ok(home.at(-1).distance>280&&home.at(-1).distance<320)
  assert.equal(home.at(-1).point[2]-home[0].point[2],12)
  const ramp=routeProfile(data.nodes,data.routes.find(r=>r.id==='home-ramp').nodes)
  assert.ok(ramp.slice(1).every(p=>Math.abs(p.grade)<=5))
  const lift=routeProfile(data.nodes,['cinema_lift_low','cinema_lift_high'])
  assert.equal(lift[1].length,0);assert.equal(lift[1].grade,null);assert.equal(lift[1].rise,12)
  // The ground network remains usable when decks, steps and lifts are unavailable
  const seen=new Set(['station']);let previous=-1
  while(previous!==seen.size){previous=seen.size;for(const r of data.roads.filter(r=>!['steps','lift','deck'].includes(r.kind)))if(r.nodes.some(id=>seen.has(id)))r.nodes.forEach(id=>seen.add(id))}
  for(const id of ['home','library_gate','school_gate','interest_mid','music_west','river_w','east_clinic','upper'])assert.ok(seen.has(id),id)
})

test('street centerlines avoid the interiors of building footprints', () => {
  const crosses=(a,b,polygon)=> {
    const [[x,y],,[u,v]]=polygon
    let lo=0,hi=1
    for(const [start,delta,min,max] of [[a[0],b[0]-a[0],x+.01,u-.01],[a[1],b[1]-a[1],y+.01,v-.01]]){
      if(delta===0){if(start<min||start>max)return false}
      else{const t=[(min-start)/delta,(max-start)/delta].sort((a,b)=>a-b);lo=Math.max(lo,t[0]);hi=Math.min(hi,t[1])}
    }
    return lo<=hi
  }
  assert.ok(crosses([-1,1],[3,1],[[0,0],[2,0],[2,2],[0,2]]))
  assert.ok(!crosses([-1,0],[3,0],[[0,0],[2,0],[2,2],[0,2]]))
  for(const b of data.buildings)for(const r of data.roads.filter(r=>!['lift','deck'].includes(r.kind)))for(let i=1;i<r.nodes.length;i++)assert.ok(!crosses(data.nodes[r.nodes[i-1]],data.nodes[r.nodes[i]],b.polygon),`${b.id}: ${r.nodes.join(' → ')}`)
})

test('housing estimates use residential floor area rather than the volume count', async () => {
  const {housingEstimate}=await import('../district-plan.mjs')
  const estimate=housingEstimate({parcels:[{polygon:[[0,0],[10,0],[10,20],[0,20]],housing:{coverage:.5,floors:4,share:.75}},{polygon:[[0,0],[100,0],[100,100],[0,100]]}],housingAssumptions:{netRatio:.8,unitArea:60,occupancy:[.5,1],household:[3,3]}})
  assert.equal(estimate.gross,300)
  assert.equal(estimate.units,4)
  assert.deepEqual(estimate.residents,[6,12])
})
