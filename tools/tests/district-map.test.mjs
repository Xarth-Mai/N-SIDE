import { test } from 'node:test'
import assert from 'node:assert/strict'
import data from '../../source-assets/district-map/district.json' with { type: 'json' }
import { terrainHeight, heightColor } from '../district-map.mjs'
import { architectureStats, sectionRoads } from '../district-architecture.mjs'
import { frameworkCoverage, reachableNodes } from '../district-plan.mjs'
import {polygonArea,pointInside,onBoundary,segmentInside,polygonInside,intersectionArea,roadWidth,roadAllowed,roadHitsBuilding} from '../district-geometry.mjs'

const linked=(a,b,user='public')=>data.roads.some(r=>roadAllowed(r,user)&&r.nodes.some((id,i)=>i>0&&((id===a&&r.nodes[i-1]===b)||(id===b&&r.nodes[i-1]===a))))
const reachable=(user,start='station')=>{
  const seen=new Set([start]);let previous=-1
  while(previous!==seen.size){previous=seen.size;for(const r of data.roads.filter(r=>roadAllowed(r,user)))if(r.nodes.some(id=>seen.has(id)))r.nodes.forEach(id=>seen.add(id))}
  return seen
}

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

test('shopping stays local and public hillside routes remain connected', () => {
  const shopping=data.routes.find(r=>r.id==='shopping')
  assert.equal(shopping.nodes[0],'home');assert.equal(shopping.nodes.at(-1),'home')
  for(const id of ['court','market_turn','market_back','community'])assert.ok(shopping.nodes.includes(id))
  assert.ok(!shopping.nodes.includes('station'),'buying food does not require the station detour')
  for(let i=1;i<shopping.nodes.length;i++)assert.ok(linked(shopping.nodes[i-1],shopping.nodes[i]))
  const publicNodes=reachable('public','home')
  for(const id of ['steps_mid','upper_homes','north','forest','summit'])assert.ok(publicNodes.has(id),id)
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
  const {inside,planStats,parcelStats}=await import('../district-plan.mjs')
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
  assert.equal(stats.capacity-stats.buildings,stats.remaining)
  for(const p of parcelStats(data))assert.ok(Number.isInteger(p.capacity)&&p.remaining>=0,`${p.id}: ${p.drawn} volumes exceed capacity ${p.capacity}`)
  for(const b of data.blocks){
    assert.equal(data.places.filter(p=>p.block===b.id&&p.featured).length,b.targetPlaces)
    assert.equal(data.parcels.filter(p=>p.block===b.id).reduce((n,p)=>n+p.capacity,0),b.targetBuildings)
  }
  for(const p of data.parcels){
    assert.ok(data.nodes[p.access],p.id)
    assert.ok(polygonInside(p.polygon,blocks.get(p.block).polygon),p.id)
  }
  for(const p of data.places){
    assert.ok(blocks.has(p.block),p.id)
    if(p.featured)assert.ok(parcels.has(p.parcel),p.id)
    if(p.parcel){assert.equal(parcels.get(p.parcel).block,p.block);assert.ok(inside(p.position,parcels.get(p.parcel).polygon),p.id)}
    if(p.building)assert.ok(buildings.has(p.building),p.id)
    if(p.parent){assert.ok(places.has(p.parent));assert.equal(p.building,places.get(p.parent).building);assert.notEqual(p.id,p.parent)}
    if(p.detail)assert.ok(p.brief?.space&&p.brief?.flow&&p.brief?.service&&p.brief?.public&&p.brief?.reference.length,p.id)
  }
  for(const b of data.buildings.filter(b=>b.bank==='district'))assert.ok(polygonInside(b.polygon,parcels.get(b.parcel).polygon),b.id)
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
  while(previous!==seen.size){previous=seen.size;for(const r of data.roads.filter(r=>!['steps','lift','deck','interior'].includes(r.kind)&&roadAllowed(r,'public')))if(r.nodes.some(id=>seen.has(id)))r.nodes.forEach(id=>seen.add(id))}
  for(const id of ['home','library_gate','school_gate','interest_mid','music_west','river_w','east_clinic','upper'])assert.ok(seen.has(id),id)
})

test('geometry handles concave boundaries, shared walls, road width and local elevation', () => {
  const square=[[0,0],[2,0],[2,2],[0,2]],concave=[[0,0],[4,0],[4,1],[1,1],[1,4],[0,4]]
  assert.equal(intersectionArea(square,square),4)
  const turn=p=>p.map(([x,y])=>[(x-y)/Math.sqrt(2)+17,(x+y)/Math.sqrt(2)-9])
  assert.ok(Math.abs(intersectionArea(turn(square),turn([[1,0],[3,0],[3,2],[1,2]]).reverse())-2)<1e-7,'rotated polygons and reversed winding')
  assert.equal(intersectionArea(square,[[2,0],[4,0],[4,2],[2,2]]),0,'shared wall has no area')
  assert.equal(intersectionArea(concave,[[2,2],[3,2],[3,3],[2,3]]),0,'concave notch remains empty')
  assert.ok(!segmentInside([.5,3],[3,.5],concave),'inside endpoints can leave a concave support')
  assert.ok(segmentInside([0,0],[4,0],concave),'boundary path is supported')
  const building={polygon:square,elevation:0,height:3}
  assert.ok(roadHitsBuilding([-1,1,0],[3,1,0],0,building))
  assert.ok(!roadHitsBuilding([-1,0,0],[3,0,0],0,building),'tangent centerline')
  assert.ok(roadHitsBuilding([-1,-.25,0],[3,-.25,0],1,building),'width reaches wall while centerline misses')
  assert.ok(!roadHitsBuilding([-2,1,0],[0,1,0],1,building),'flat entrance end stops at facade')
  assert.ok(!roadHitsBuilding([-2,-1,0],[0,1,0],1,building),'oblique doorway ends on the facade')
  assert.ok(roadHitsBuilding([-2,-.2,0],[1,-.2,0],1,building),'side overlap near an entrance remains a collision')
  assert.ok(!roadHitsBuilding([-10,1,-10],[10,1,10],1,{...building,elevation:6,height:2}),'height only overlaps elsewhere along road')
  assert.ok(roadHitsBuilding([-10,1,-10],[10,1,10],1,{...building,elevation:1,height:2}))
  const courtyard=[[0,0],[4,0],[4,4],[3,4],[3,1],[1,1],[1,4],[0,4]]
  assert.ok(!roadHitsBuilding([-1,3,-1],[5,3,5],.5,{polygon:courtyard,elevation:1.5,height:1}),'altitude overlap in a concave gap is clear')
})

test('all streets clear building volumes at their actual widths and elevations', () => {
  const conflicts=[]
  for(const b of data.buildings)for(const r of data.roads)for(let i=1;i<r.nodes.length;i++){
    if(r.building===b.id&&['interior','lift'].includes(r.kind))continue
    const a=data.nodes[r.nodes[i-1]],c=data.nodes[r.nodes[i]]
    if(roadHitsBuilding(a,c,roadWidth(r),b))conflicts.push(`${b.id}: ${r.nodes[i-1]} → ${r.nodes[i]} (${roadWidth(r)} m)`)
  }
  assert.deepEqual(conflicts,[])
})

test('district ground-road crossings share a junction and its elevation', () => {
  const issues=[]
  const cross=(a,b)=>a[0]*b[1]-a[1]*b[0],subtract=(a,b)=>a.map((n,i)=>n-b[i])
  const segments=data.roads.filter(r=>!['interior','lift','deck','bridge'].includes(r.kind)).flatMap(r=>r.nodes.slice(1).map((id,i)=>({ids:[r.nodes[i],id],a:data.nodes[r.nodes[i]],b:data.nodes[id]})))
  for(let i=0;i<segments.length;i++)for(const b of segments.slice(i+1)){
    const a=segments[i]
    if(a.ids.some(id=>b.ids.includes(id)))continue
    const u=subtract(a.b,a.a),v=subtract(b.b,b.a),den=cross(u,v)
    if(Math.abs(den)<1e-7)continue // Shared or parallel stretches are not transverse junctions
    const t=cross(subtract(b.a,a.a),v)/den,s=cross(subtract(b.a,a.a),u)/den
    if(t<-1e-7||t>1+1e-7||s<-1e-7||s>1+1e-7)continue
    const point=a.a.map((n,j)=>n+u[j]*t),otherZ=b.a[2]+v[2]*s
    const label=`${a.ids.join(' → ')} / ${b.ids.join(' → ')}`
    if(Math.abs(point[2]-otherZ)>1e-5)issues.push(`${label}: crossing elevations ${point[2].toFixed(3)} / ${otherZ.toFixed(3)}`)
    else issues.push(`${label}: same-level crossing lacks a shared node`)
  }
  assert.deepEqual(issues,[])
})

test('city framework records every building and reaches its same-level facade entrances', () => {
  const coverage=frameworkCoverage(data)
  assert.equal(coverage.length,12)
  for(const block of coverage){
    assert.ok(block.buildings>0,block.id)
    assert.equal(block.described,block.buildings,`${block.id}: missing building use or entrances`)
    assert.equal(block.arrived,block.buildings,`${block.id}: entrance is disconnected or not on its level/facade`)
    assert.equal(block.connected,block.entries,`${block.id}: disconnected entry`)
    assert.equal(block.widths,block.roads,`${block.id}: road width not recorded`)
    assert.equal(block.linked,block.roads,`${block.id}: disconnected road`)
  }
  for(const b of data.buildings.filter(b=>b.bank==='district')){
    let previous=b.elevation-1
    for(const f of b.design.floors){
      assert.ok(f.z>=b.elevation&&f.z<=b.elevation+b.height&&f.z>previous,`${b.id}: invalid level ${f.name}`)
      previous=f.z
    }
    if(b.design.status==='framework')assert.ok(b.design.floors.every(f=>!f.rooms&&!f.openings),`${b.id}: framework unexpectedly includes interiors`)
  }
  for(const r of data.roads)assert.ok(Number.isFinite(r.width)&&r.width>0,'all roads need design widths')
  // A student-only gateway cannot become a public shortcut in coverage counts
  const gate={nodes:{station:[0,0,0],gate:[1,0,0]},roads:[{nodes:['station','gate'],access:'controlled',users:['student']}]}
  assert.ok(!reachableNodes(gate).has('gate'))
  assert.ok(reachableNodes(gate,'student').has('gate'))
})

test('ordinary streets keep their design grade and deliveries avoid stair-only approaches', async () => {
  const {routeProfile}=await import('../district-plan.mjs')
  for(const r of data.roads.filter(r=>!['interior','lift','steps','trail','bridge','deck'].includes(r.kind)))for(const p of routeProfile(data.nodes,r.nodes).slice(1)){
    assert.ok(p.grade!==null?Math.abs(p.grade)<=10:!p.rise,`${p.id}: ordinary street exceeds 10 percent design grade`)
  }
  const service=reachableNodes({...data,roads:data.roads.filter(r=>!['steps','trail','deck'].includes(r.kind)&&(r.kind!=='lift'||r.access==='service'))},'service')
  // The small shrine is maintained on foot via its mountain path
  for(const b of data.buildings.filter(b=>b.kind!=='shrine'))for(const e of b.design?.entries??[])if(e.role==='service')assert.ok(service.has(e.node),`${b.id}: deliveries depend on stairs or a foot trail`)
  const breakfast=reachableNodes({...data,roads:data.roads.filter(r=>r.kind!=='steps')})
  assert.ok(breakfast.has('v_51_door'),'short entrance stairs have a level shopfront alternative')
})

test('public streets stay outside same-level resident and service courts', () => {
  const issues=[]
  for(const s of data.surfaces.filter(s=>['resident','service'].includes(s.access)))for(const r of data.roads.filter(r=>roadAllowed(r,'public')&&!['bridge','deck','lift','interior'].includes(r.kind)))for(let i=1;i<r.nodes.length;i++){
    const a=data.nodes[r.nodes[i-1]],b=data.nodes[r.nodes[i]]
    if(roadHitsBuilding(a,b,roadWidth(r),{polygon:s.polygon,elevation:s.elevation-.1,height:.2}))issues.push(`${s.id}: ${r.nodes[i-1]} → ${r.nodes[i]}`)
  }
  assert.deepEqual(issues,[])
})

test('building volumes share boundaries without occupying the same space', () => {
  const conflicts=[]
  for(let i=0;i<data.buildings.length;i++)for(const b of data.buildings.slice(i+1)){
    const a=data.buildings[i]
    if(Math.min(a.elevation+a.height,b.elevation+b.height)-Math.max(a.elevation,b.elevation)<=1e-7)continue
    const area=intersectionArea(a.polygon,b.polygon)
    if(area>1e-7)conflicts.push(`${a.id} / ${b.id}: ${area.toFixed(3)} m²`)
  }
  assert.deepEqual(conflicts,[])
})

test('detailed places connect street access to separate public and service entrances', async () => {
  const {inside}=await import('../district-plan.mjs')
  const reached=reachable('public')
  for(const p of data.places) {
    assert.ok(data.nodes[p.access],`${p.id}: unknown street access`)
    if(p.featured)assert.ok(reached.has(p.access),`${p.id}: street access disconnected from public network`)
    if(p.detail)assert.ok(p.arrivals?.public&&p.arrivals?.service,`${p.id}: missing entrance paths`)
    for(const [role,arrival] of Object.entries(p.arrivals??{})) {
      assert.ok(arrival.label&&arrival.level&&arrival.nodes.length>=2,p.id)
      assert.equal(arrival.nodes[0],p.access,`${p.id}: entrance path must start at street access`)
      for(let i=1;i<arrival.nodes.length;i++)assert.ok(linked(arrival.nodes[i-1],arrival.nodes[i],arrival.user??role),`${p.id} ${role}: broken or restricted approach`)
      const end=data.nodes[arrival.nodes.at(-1)]
      const building=data.buildings.find(b=>b.id===p.building)
      const parcel=data.parcels.find(q=>q.id===p.parcel)
      if(role==='service'&&building)for(let i=1;i<arrival.nodes.length;i++)assert.ok(data.roads.some(r=>roadAllowed(r,'service')&&!['steps','trail','deck'].includes(r.kind)&&(r.kind!=='lift'||r.access==='service')&&r.nodes.some((id,j)=>j>0&&((id===arrival.nodes[i]&&r.nodes[j-1]===arrival.nodes[i-1])||(id===arrival.nodes[i-1]&&r.nodes[j-1]===arrival.nodes[i])))),`${p.id}: deliveries depend on steps or a public lift`)
      assert.ok(inside(end,building?.polygon??parcel.polygon),`${p.id} ${role}: entrance outside its destination`)
      if(!building){
        const surfaces=data.surfaces.filter(s=>s.place===p.id&&s.elevation===end[2])
        assert.ok(surfaces.some(s=>inside(end,s.polygon)),`${p.id} ${role}: entrance outside its same-level public space`)
      }
      if(role==='public')assert.equal(end[2],p.position[2],`${p.id}: arrival at wrong level`)
      if(building&&!p.parent){
        assert.ok(onBoundary(end,building.polygon),`${p.id} ${role}: missing facade entrance`)
      }
    }
  }
})

test('cinema roof and exterior platform have distinct footprints and supported paths', async () => {
  const {inside}=await import('../district-plan.mjs')
  const roof=data.surfaces.find(s=>s.building==='V-15'),building=data.buildings.find(b=>b.id==='V-15')
  assert.ok(roof?.elevated)
  assert.equal(roof.elevation,building.elevation+building.height)
  assert.ok(polygonInside(roof.polygon,building.polygon))
  assert.ok(inside(data.nodes.cinema_roof,roof.polygon))
  assert.equal(data.nodes.cinema_roof[2],roof.elevation)
  for(const r of data.roads.filter(r=>r.surface)) {
    const surface=data.surfaces.find(s=>s.id===r.surface)
    assert.ok(surface,r.surface)
    for(const id of r.nodes){assert.ok(inside(data.nodes[id],surface.polygon),`${id}: outside ${r.surface}`);assert.equal(data.nodes[id][2],surface.elevation)}
    for(let i=1;i<r.nodes.length;i++)assert.ok(segmentInside(data.nodes[r.nodes[i-1]],data.nodes[r.nodes[i]],surface.polygon),`${r.nodes[i-1]} → ${r.nodes[i]} leaves ${surface.id}`)
  }
  assert.ok(data.roads.some(r=>r.surface&&data.surfaces.some(s=>s.id===r.surface&&s.elevated&&!s.building)))
  const reached=new Set(['upper']);let previous=-1
  while(previous!==reached.size){previous=reached.size;for(const r of data.roads.filter(r=>r.kind!=='lift'&&roadAllowed(r,'public')))if(r.nodes.some(n=>reached.has(n)))r.nodes.forEach(n=>reached.add(n))}
  assert.ok(reached.has('slope_platform_entry'),'upper street must connect to the platform without its lift')
})

test('lifestyle routes visit actual arrival endpoints in the stated order', () => {
  for(const route of data.routes.filter(r=>r.places)) {
    let cursor=0
    for(const id of route.places) {
      const place=data.places.find(p=>p.id===id),destination=place.arrivals?.public.nodes.at(-1)??place.access
      const index=route.nodes.indexOf(destination,cursor)
      assert.ok(index>=cursor,`${route.id}: missing or out-of-order destination ${id} ${destination}`)
      cursor=index+1
    }
    for(let i=1;i<route.nodes.length;i++)assert.ok(linked(route.nodes[i-1],route.nodes[i],route.user??'public'),`${route.id}: ${route.user??'public'} cannot use ${route.nodes[i-1]} → ${route.nodes[i]}`)
  }
})

test('housing estimates use residential floor area rather than the volume count', async () => {
  const {housingEstimate}=await import('../district-plan.mjs')
  const estimate=housingEstimate({parcels:[{polygon:[[0,0],[10,0],[10,20],[0,20]],housing:{coverage:.5,floors:4,share:.75}},{polygon:[[0,0],[100,0],[100,100],[0,100]]}],housingAssumptions:{netRatio:.8,unitArea:60,occupancy:[.5,1],household:[3,3]}})
  assert.equal(estimate.gross,300)
  assert.equal(estimate.units,4)
  assert.deepEqual(estimate.residents,[6,12])
  const drawn=housingEstimate({parcels:[{id:'P',housing:{share:.5}}],buildings:[{bank:'district',parcel:'P',polygon:[[0,0],[10,0],[10,10],[0,10]],elevation:0,height:6,design:{floors:[{z:0},{z:3},{z:6}]}}],housingAssumptions:{netRatio:.8,unitArea:40,occupancy:[1,1],household:[2,2]}},true)
  assert.equal(drawn.gross,100,'use drawn footprint and two occupied levels, exclude the roof')
  assert.equal(drawn.units,2)
})


test('architectural segments reference complete buildings and derive type counts from the same data', () => {
  assert.deepEqual(data.architectures.map(a=>a.id).sort(),['P3-A1','P3-A2'])
  for(const segment of data.architectures){
    assert.equal(segment.buildings.length,24,segment.id)
    assert.equal(new Set(segment.buildings).size,segment.buildings.length,`${segment.id}: repeated building`)
    assert.equal(new Set(segment.types.map(t=>t.id)).size,segment.types.length,`${segment.id}: repeated type`)
    for(const id of segment.buildings){
      const building=data.buildings.find(b=>b.id===id)
      assert.ok(building?.bank==='district',`${segment.id}: unknown district building ${id}`)
      assert.ok(polygonInside(building.polygon,segment.boundary),`${segment.id}: ${id} outside sample boundary`)
      assert.ok(segment.types.some(t=>t.id===building.design?.type),`${id}: unknown architectural type`)
    }
    for(const type of segment.types){
      assert.ok(segment.buildings.includes(type.sample),`${segment.id}: ${type.id} sample outside segment`)
      assert.equal(data.buildings.find(b=>b.id===type.sample).design.type,type.id)
    }
    const stats=architectureStats(data,segment)
    assert.equal(stats.reduce((sum,t)=>sum+t.count,0),segment.buildings.length)
    assert.ok(stats.every(t=>t.count>0&&t.footprint>0&&t.floorArea>=t.footprint),segment.id)
  }
})

test('representative buildings have consistent floors, room areas and facade entrances', () => {
  const issues=[]
  const buildings=[...new Set(data.architectures.flatMap(a=>a.buildings))].map(id=>data.buildings.find(b=>b.id===id))
  for(const b of buildings){
    assert.ok(b.design?.status&&b.design.type&&b.design.floors.length&&b.design.entries.length,b.id)
    let previous=b.elevation-1
    for(const f of b.design.floors){
      assert.ok(f.z>=b.elevation&&f.z<b.elevation+b.height&&f.z>previous,`${b.id} ${f.name}: floor elevation`);previous=f.z
      assert.ok(f.use)
      const rooms=f.rooms??[],footprint=polygonArea(b.polygon)
      if(rooms.reduce((n,r)=>n+polygonArea(r.polygon),0)>footprint+1e-7)issues.push(`${b.id} ${f.name}: room area exceeds footprint`)
      for(let i=0;i<rooms.length;i++){
        const room=rooms[i]
        if(!(polygonArea(room.polygon)>0&&polygonInside(room.polygon,b.polygon)))issues.push(`${b.id} ${f.name} ${room.name}: room outside footprint`)
        for(const other of rooms.slice(i+1))if(intersectionArea(room.polygon,other.polygon)>1e-7)issues.push(`${b.id} ${f.name}: ${room.name} overlaps ${other.name}`)
      }
    }
    for(const entry of b.design.entries){
      const p=data.nodes[entry.node],floor=b.design.floors.find(f=>f.name===entry.level)
      if(!(p&&floor&&p[2]===floor.z&&onBoundary(p,b.polygon)))issues.push(`${b.id}: ${entry.node} floor/facade mismatch`)
      if(!reachable(entry.role).has(entry.node))issues.push(`${b.id}: ${entry.role} cannot reach ${entry.node}`)
    }
  }
  assert.deepEqual(issues,[])
})

test('public, residents, students and service users have separate access rights', () => {
  const controlled={access:'controlled',users:['student','visitor','staff']}
  for(const role of ['public','resident','service'])assert.ok(!roadAllowed(controlled,role))
  assert.ok(roadAllowed(controlled,'student'))
  assert.ok(!roadAllowed({access:'unknown'},'public'))
  for(const road of data.roads)assert.ok([undefined,'public','controlled','resident','service'].includes(road.access))
  for(const route of data.routes)for(let i=1;i<route.nodes.length;i++)assert.ok(linked(route.nodes[i-1],route.nodes[i],route.user??'public'),`${route.id}: restricted route segment`)
  assert.ok(!roadAllowed({access:'controlled'},'student'),'controlled access needs an explicit user list')
  for(const access of ['resident','service'])for(const user of ['public','resident','student','service'])assert.equal(roadAllowed({access},user),access===user)
  const publicNodes=reachable('public'),students=reachable('student'),service=reachable('service')
  assert.ok(!publicNodes.has('school_building_entry'))
  assert.ok(students.has('school_building_entry'))
  assert.ok(!service.has('school_building_entry'),'service uses the campus service gate')
  assert.ok(service.has('school_service_entry'))
  assert.ok(!publicNodes.has('shop_rear_door'))
  assert.ok(!publicNodes.has('v_13_upper'))
  assert.ok(reachable('resident').has('v_13_upper'))
})

test('shop supply starts on the city road and changes to a level trolley route at unloading', () => {
  const supply=data.logistics.find(l=>l.id==='shop-supply'),[vehicle,trolley]=supply.legs
  assert.equal(vehicle.nodes[0],'city_w');assert.equal(vehicle.nodes.at(-1),'loading')
  assert.equal(trolley.nodes[0],'loading');assert.equal(trolley.nodes.at(-1),'shop_rear_door')
  const bay=data.surfaces.find(s=>s.id===supply.unloading)
  assert.ok(pointInside(data.nodes.loading,bay.polygon));assert.equal(bay.elevation,data.nodes.loading[2])
  for(const leg of supply.legs){
    assert.ok(!leg.nodes.includes('shop_entry'))
    for(let i=1;i<leg.nodes.length;i++)assert.ok(data.roads.some(r=>roadAllowed(r,'service')&&!['steps','trail','deck','lift'].includes(r.kind)&&r.nodes.some((id,j)=>j>0&&((id===leg.nodes[i]&&r.nodes[j-1]===leg.nodes[i-1])||(id===leg.nodes[i-1]&&r.nodes[j-1]===leg.nodes[i])))),`${leg.mode}: ${leg.nodes[i-1]} → ${leg.nodes[i]}`)
  }
  assert.ok(trolley.nodes.every(id=>data.nodes[id][2]===data.nodes.loading[2]))
})

test('interest equipment deliveries reach a level backstage route from a gentle city-road approach', async () => {
  const {routeProfile}=await import('../district-plan.mjs')
  const supply=data.logistics.find(l=>l.id==='interest-equipment')
  assert.ok(supply,'interest equipment supply route')
  const [vehicle,trolley]=supply.legs
  assert.equal(vehicle.nodes[0],'city_west')
  assert.equal(vehicle.nodes.at(-1),'interest_unload')
  let cursor=0
  for(const id of ['station_west','station_west_turn','station_interest_turn','interest_s','interest_loading','interest_unload']){
    cursor=vehicle.nodes.indexOf(id,cursor)
    assert.ok(cursor>=0,`vehicle approach misses ${id}`);cursor++
  }
  assert.equal(trolley.nodes[0],'interest_unload')
  assert.equal(trolley.nodes.at(-1),'game_service_entry')
  const bay=data.surfaces.find(s=>s.id===supply.unloading)
  assert.ok(bay&&pointInside(data.nodes.interest_unload,bay.polygon))
  assert.equal(bay.elevation,12)
  for(const leg of supply.legs)for(let i=1;i<leg.nodes.length;i++)assert.ok(data.roads.some(r=>roadAllowed(r,'service')&&!['steps','trail','deck','lift'].includes(r.kind)&&r.nodes.some((id,j)=>j>0&&((id===leg.nodes[i]&&r.nodes[j-1]===leg.nodes[i-1])||(id===leg.nodes[i-1]&&r.nodes[j-1]===leg.nodes[i])))),`${leg.mode}: ${leg.nodes[i-1]} → ${leg.nodes[i]}`)
  assert.ok(routeProfile(data.nodes,vehicle.nodes).slice(1).every(p=>p.grade!==null&&Math.abs(p.grade)<=5),'vehicle route exceeds its 5% design target')
  assert.ok(trolley.nodes.every(id=>data.nodes[id][2]===12),'equipment trolley route must stay level')
  const segment=data.architectures.find(a=>a.id==='P3-A2')
  for(const b of data.buildings.filter(b=>segment.buildings.includes(b.id)&&['interest','maker'].includes(b.design.type))){
    const entries=b.design.entries.filter(e=>e.role==='service')
    assert.ok(entries.length,`${b.id}: equipment receiving entry`)
    for(const e of entries)assert.equal(data.nodes[e.node][2],12,`${b.id}: backstage receiving level`)
  }
})


test('drawn room openings join actual walls and each occupied room has an access', () => {
  const issues=[]
  for(const b of data.buildings)for(const f of b.design?.floors??[])if(f.rooms){
    const rooms=f.rooms.filter(r=>r.kind!=='court'),touched=new Set()
    for(const opening of f.openings??[]){
      const [a,c]=opening.line
      assert.ok(['door','open'].includes(opening.kind)&&Math.hypot(c[0]-a[0],c[1]-a[1])>0)
      const adjacent=rooms.filter(r=>[a,c,a.map((v,i)=>(v+c[i])/2)].every(p=>onBoundary(p,r.polygon)))
      const exterior=[a,c].every(p=>onBoundary(p,b.polygon))
      if(!(adjacent.length===2||exterior&&adjacent.length===1))issues.push(`${b.id} ${f.name}: floating or ambiguous opening ${JSON.stringify(opening.line)}`)
      adjacent.forEach(r=>touched.add(r))
    }
    for(const r of rooms)if(!touched.has(r))issues.push(`${b.id} ${f.name}: ${r.name} has no drawn access`)
  }
  assert.deepEqual(issues,[])
})

test('station mixed-use and interest samples connect customer and backstage rooms through their own entrances', () => {
  const segment=data.architectures.find(a=>a.id==='P3-A2')
  for(const building of data.buildings.filter(b=>segment.buildings.includes(b.id)&&['mixed','interest','maker'].includes(b.design.type)&&b.design.floors[0].rooms)){
    const floor=building.design.floors[0],rooms=floor.rooms
    assert.ok(rooms?.length,`${building.id}: entrance-floor plan`)
    const pairs=(floor.openings??[]).map(o=>rooms.filter(r=>o.line.every(p=>onBoundary(p,r.polygon)))).filter(pair=>pair.length===2)
    for(const [role,kind] of [['public','shop'],['service','service']]){
      const entries=building.design.entries.filter(e=>e.role===role&&e.level===floor.name)
      assert.ok(entries.length,`${building.id}: ${role} entrance on ${floor.name}`)
      const allowed=rooms.filter(r=>r.kind===kind||r.kind==='core')
      const reached=new Set(allowed.filter(r=>entries.some(e=>onBoundary(data.nodes[e.node],r.polygon))))
      assert.ok(reached.size,`${building.id}: ${role} entrance does not enter its room network`)
      let size=-1
      while(size!==reached.size){size=reached.size;for(const pair of pairs)if(pair.some(r=>reached.has(r)))pair.filter(r=>allowed.includes(r)).forEach(r=>reached.add(r))}
      const destinations=rooms.filter(r=>r.kind===kind)
      assert.ok(destinations.length,`${building.id}: ${role} function rooms`)
      for(const room of destinations)assert.ok(reached.has(room),`${building.id}: ${role} route to ${room.name} crosses the other user area or is disconnected`)
    }
  }
})

test('shared stairs and separate apartments connect through shared circulation', () => {
  for(const [id,index] of [['V-13',0],['V-57',1]]) {
    const floor=data.buildings.find(b=>b.id===id).design.floors[index],rooms=floor.rooms
    const core=rooms.filter(r=>r.kind==='core'),seen=new Set(core.filter(r=>r.name.includes('梯')))
    const pairs=floor.openings.map(o=>rooms.filter(r=>o.line.every(p=>onBoundary(p,r.polygon))))
    let size=-1
    while(size!==seen.size){size=seen.size;for(const pair of pairs)if(pair.some(r=>seen.has(r)))pair.filter(r=>r.kind==='core').forEach(r=>seen.add(r))}
    assert.equal(seen.size,core.length,`${id}: shared circulation must not pass through a dwelling`)
    if(id==='V-57')for(const home of rooms.filter(r=>r.kind==='home'))assert.ok(pairs.some(pair=>pair.includes(home)&&pair.some(r=>seen.has(r))),`${id} ${home.name}: independent access`)
  }
  const crossing=sectionRoads(data,data.architectures.find(a=>a.id==='P3-A1').sections.find(s=>s.id==='B-B')).find(r=>r.kind==='steps')
  assert.equal(crossing.width,3)
  assert.equal(crossing.elevation,22.69)
})
