import { test } from 'node:test'
import assert from 'node:assert/strict'
import source from '../../source-assets/district-map/district.json' with { type: 'json' }
import { bakeTerrain, mountainBase, replaceTerrain } from '../terrain-shape.ts'
import type { District } from '../district-types.ts'
import { buildGround } from '../district-map.ts'
import { roadOffsets } from '../district-geometry.ts'

const data:District=source

test('second short-ascent leg meets terrain across stair and landing edges',{timeout:20_000},()=>{
  const result=bakeTerrain(data),ground=buildGround(result),route=result.routes.find(r=>r.id==='hill-short')!
  const nodes=new Set(route.nodes.slice(route.nodes.indexOf('hill_short_rest1_departure'),route.nodes.indexOf('hill_short_rest2_arrival')+1))
  const roads=result.roads.filter(r=>r.nodes.every(id=>nodes.has(id)))
  assert.equal(roads.length,23,'twelve flights and eleven landings remain covered')
  for(const road of roads) {
    const points=road.nodes.map(id=>result.nodes[id]),offsets=roadOffsets(points,road.width)
    for(let i=1;i<points.length;i++)for(const t of [0,.25,.5,.75,1])for(const side of [-1,1]) {
      const a=points[i-1],b=points[i],start=offsets[i-1],end=offsets[i]
      const x=a[0]+(b[0]-a[0])*t+(start[0]+(end[0]-start[0])*t)*side
      const y=a[1]+(b[1]-a[1])*t+(start[1]+(end[1]-start[1])*t)*side
      const delta=ground.height(x,y)-(a[2]+(b[2]-a[2])*t)
      assert.ok(Math.abs(delta)<.01,`${road.nodes.join(' -> ')} at ${t}/${side}: terrain differs from road grade by ${delta}m`)
    }
  }
})

test('descent eye-level slope uses local terrain detail instead of a 60m face',{timeout:20_000},()=>{
  const ground=buildGround(bakeTerrain(data)),point=[160,740]
  const triangle=ground.triangles.find(t=>t.every((a,i)=>{
    const b=t[(i+1)%3]
    return (b[0]-a[0])*(point[1]-a[1])-(b[1]-a[1])*(point[0]-a[0])>=-1e-9
  }))!
  assert.ok(triangle,'the actual descent camera slope remains inside terrain')
  const longest=Math.max(...triangle.map((a,i)=>Math.hypot(a[0]-triangle[(i+1)%3][0],a[1]-triangle[(i+1)%3][1])))
  assert.ok(longest<20,`the slope in eye-descent-cut still has a ${longest}m triangle edge`)
})

test('descent bank changes grade gradually and adjoining upper stairs retain support',()=>{
  const ground=buildGround(data)
  // Source positions hit by the actual r6 cut-view rays; the former first three faces exceeded 52 degrees
  const slopes=[[171.4934,739.7493],[156.4618,741.9763],[155.4546,738.915],[133.13,746.5454],[129.2551,746.5177]].map(([x,y])=>{
    const dx=(ground.height(x+.1,y)-ground.height(x-.1,y))/.2,dy=(ground.height(x,y+.1)-ground.height(x,y-.1))/.2
    return Math.atan(Math.hypot(dx,dy))*180/Math.PI
  })
  assert.ok(slopes.every(a=>a>25&&a<48),`the near bank must remain sloped without the former straight steep wedge: ${slopes}`)
  assert.ok(Math.max(...slopes)-Math.min(...slopes)>5,'the inspected bank must not become one uniform inclined plane')
  for(const from of ['stair_hill_short_rest2_departure_hill_short_rest3_arrival_05_out','stair_hill_short_rest2_departure_hill_short_rest3_arrival_06_in']) {
    const road=data.roads.find(r=>r.nodes[0]===from)!,points=road.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,road.width)
    for(const t of [0,.25,.5,.75,1])for(const side of [-1,0,1]) {
      const [a,b]=points,[start,end]=offsets
      const x=a[0]+(b[0]-a[0])*t+(start[0]+(end[0]-start[0])*t)*side,y=a[1]+(b[1]-a[1])*t+(start[1]+(end[1]-start[1])*t)*side
      assert.ok(Math.abs(ground.height(x,y)-(a[2]+(b[2]-a[2])*t))<.01,`${from}: local reshaping must not leave the next stair leg unsupported`)
    }
  }
})

test('terrain bake preserves authored geometry and repeats without drift',{timeout:20_000},()=>{
  const result=bakeTerrain(data),count=data.terrain.bake!.authored
  assert.deepEqual(bakeTerrain(result),result)
  assert.deepEqual(result.terrain.samples.slice(0,count),data.terrain.samples.slice(0,count))
  assert.deepEqual({...result,terrain:data.terrain},data,'roads, entrances, buildings and platforms stay unchanged')
  assert.ok(result.terrain.samples.length>count,'natural terrain is actually refined beyond authored controls')
  assert.ok(result.terrain.samples.every(p=>p.every(Number.isFinite)&&p[2]<=450))
  assert.ok(result.terrain.samples.slice(count).every(p=>p[2]<450),'only authored summit controls can reach the highest level')
  const grid=new Map(result.terrain.samples.slice(count).map(([x,y,z])=>[`${x},${y}`,z])),spacing=data.terrain.bake!.spacing
  for(const [x,y,z] of result.terrain.samples.slice(count).filter(p=>p[1]>=850))for(const key of [`${x+spacing},${y}`,`${x},${y+spacing}`]) {
    const neighbor=grid.get(key)
    if(neighbor!==undefined)assert.ok(Math.abs(neighbor-z)/spacing<2,'natural mountain cells must not reintroduce the former 2:1 folds')
  }
  assert.ok(Math.abs(mountainBase(data,-70,930)-mountainBase(data,530,930))>20,'east and west shoulders have distinct large forms')
})

test('terrain noise protects complete road segments and footprints',{timeout:20_000},()=>{
  const sample=structuredClone(data)
  // Avoid the authored [-700,570,44] control: the fixture must itself be a valid Ground
  sample.nodes.terrain_test_a=[-680,570,125];sample.nodes.terrain_test_b=[-400,570,125]
  sample.roads.push({nodes:['terrain_test_a','terrain_test_b'],kind:'lane',width:4})
  sample.surfaces.push({polygon:[[-700,630],[-500,630],[-500,700],[-700,700]],elevation:132,kind:'platform'})
  const samples=bakeTerrain(sample).terrain.samples
  assert.equal(samples.find(p=>p[0]===-580&&p[1]===570)?.[2],125,'long road midpoint remains at its true grade')
  assert.equal(samples.find(p=>p[0]===-580&&p[1]===690)?.[2],132,'platform interior is protected')
  const result=bakeTerrain(data),ground=buildGround(result),rest=result.surfaces.find(s=>s.id==='fw-e-existing-20')!
  for(const id of ['fw-f-plateau-west-court','fw-f-plateau-east-court','hill-short-rest1','hill-short-rest2','hill-short-rest3']) {
    const area=result.surfaces.find(s=>s.id===id)!
    for(let i=0;i<area.polygon.length;i++)for(const t of [.25,.5,.75]) {
      const a=area.polygon[i],b=area.polygon[(i+1)%area.polygon.length]
      assert.ok(Math.abs(ground.height(a[0]+t*(b[0]-a[0]),a[1]+t*(b[1]-a[1]))-area.elevation)<.01,`${id}: courtyard edge must not become a tall unsupported cut`)
    }
  }
  // Preserve the former bent-road failure independently of later street/landing revisions
  const bent=structuredClone(data)
  bent.nodes={summit:data.nodes.summit,home:data.nodes.home,hillgate:data.nodes.hillgate,a:[280,365,72.315789],b:[292,400,79.368421],c:[280,440,87.428571]}
  bent.roads=[{nodes:['a','b','c'],kind:'steps',width:6}];bent.buildings=[];bent.surfaces=[];bent.elevatedNodes=[]
  assert.ok(Math.abs(buildGround(bakeTerrain(bent)).height(292.081206495,410.169618192)-81.3834585)<.01,'bent stair edges must meet terrain without an artificial cut bank')
  const streets=result.roads.filter(r=>r.nodes.some(id=>id.includes('upper')&&id.includes('fw_e_school_up_w'))||(r.nodes.includes('cinema_upper')&&r.nodes.includes('school_n')))
  assert.ok(streets.length>1,'current flights and landings remain covered')
  for(const road of streets)for(let i=1;i<road.nodes.length;i++) {
    const from=road.nodes[i-1],to=road.nodes[i],a=result.nodes[from],b=result.nodes[to]
    const dx=b[0]-a[0],dy=b[1]-a[1],length=Math.hypot(dx,dy)
    for(const t of [.25,.5,.75])for(const side of [-1,0,1]) {
      const x=a[0]+dx*t-dy/length*road.width/2*side,y=a[1]+dy*t+dx/length*road.width/2*side
      assert.ok(Math.abs(ground.height(x,y)-(a[2]+(b[2]-a[2])*t))<.01,`${from} to ${to}: actual road ribbon must meet ground at ${t}/${side}`)
    }
  }

  for(let i=0;i<rest.polygon.length;i++)for(const t of [0,.25,.5,.75]) {
    const a=rest.polygon[i],b=rest.polygon[(i+1)%rest.polygon.length],x=a[0]+t*(b[0]-a[0]),y=a[1]+t*(b[1]-a[1])
    assert.ok(Math.abs(ground.height(x,y)-rest.elevation)<1e-6,'interpolated platform edges must not become isolated triangular cut banks')
  }
})

test('terrain bake rejects invalid metadata and unsupported source formatting',()=>{
  for(const authored of [0,-1,1.5,data.terrain.samples.length+1]){
    const bad=structuredClone(data);bad.terrain.bake!.authored=authored
    assert.throws(()=>bakeTerrain(bad),/Invalid terrain.bake/)
  }
  assert.throws(()=>replaceTerrain(JSON.stringify(data,null,2),data.terrain),/exactly one compact terrain line/)
  assert.throws(()=>replaceTerrain('  "terrain": {},\n  "terrain": {},',data.terrain),/exactly one compact terrain line/)
  const text='  "terrain": {},'
  assert.equal(replaceTerrain(text,data.terrain),'  "terrain": '+JSON.stringify(data.terrain)+',')
})

test('third short-ascent terrace clears terrain toward city landmarks',()=>{
  const ground=buildGround(data),rest=data.nodes.hill_short_rest3,home=data.nodes.home
  // Same city-facing position as Viewer eye-ascent-3: 3m from center, 1.7m above the platform
  const distance=Math.hypot(home[0]-rest[0],home[1]-rest[1])
  const eye=[rest[0]+3*(home[0]-rest[0])/distance,rest[1]+3*(home[1]-rest[1])/distance,rest[2]+1.7]
  const cinema=data.buildings.find(b=>b.id==='V-15')!
  const roof=[0,1].map(axis=>cinema.polygon.reduce((sum,p)=>sum+p[axis],0)/cinema.polygon.length)
  const targets=[...['home','station','upper'].map(id=>({id,point:data.nodes[id]})),{id:'V-15 roof',point:[...roof,cinema.elevation+cinema.height]}]
  // This checks terrain only; buildings, trees and the rendered view remain separate evidence
  for(const {id,point} of targets) {
    const length=Math.hypot(point[0]-eye[0],point[1]-eye[1])
    for(let s=1;s<length-5;s+=.5) {
      const t=s/length,p=eye.map((v,i)=>v+(point[i]-v)*t)
      assert.ok(p[2]-ground.height(p[0],p[1])>.5,`${id}: terrain blocks the 1.7m eye at ${s}m`)
    }
  }
})

test('forest-edge hollow has broad concavity while the adjacent short route keeps its grade',()=>{
  const ground=buildGround(data)
  // Cross-section through the foreground seen by inspect-understory-r1, not a color/noise assertion
  const shoulders=(ground.height(132,848)+ground.height(156,848))/2
  assert.ok(shoulders-ground.height(144,848)>1.5,'the forest hollow must have visible volume rather than one inclined plane')
  for(let y=836;y<=860;y+=2)for(let x=136;x<=152;x+=2){
    const dx=(ground.height(x+.1,y)-ground.height(x-.1,y))/.2,dy=(ground.height(x,y+.1)-ground.height(x,y-.1))/.2
    assert.ok(Math.hypot(dx,dy)<1.5,`forest hollow forms an abrupt spike at ${x},${y}`)
  }
  for(const from of ['stair_hill_short_rest2_departure_hill_short_rest3_arrival_03_in','stair_hill_short_rest2_departure_hill_short_rest3_arrival_03_out']) {
    const road=data.roads.find(r=>r.nodes[0]===from)!
    const [a,b]=road.nodes.map(id=>data.nodes[id]),[u,v]=roadOffsets([a,b],road.width)
    for(const t of [0,.25,.5,.75,1])for(const side of [-1,0,1]){
      const x=a[0]+(b[0]-a[0])*t+(u[0]+(v[0]-u[0])*t)*side,y=a[1]+(b[1]-a[1])*t+(u[1]+(v[1]-u[1])*t)*side
      assert.ok(Math.abs(ground.height(x,y)-(a[2]+(b[2]-a[2])*t))<.01,'the hollow must not cut into the adjacent stair landing')
    }
  }
})

test('local forest road support leaves the existing terrain envelope outside the hollow unchanged',()=>{
  const samples=new Map(bakeTerrain(data).terrain.samples.map(([x,y,z])=>[`${x},${y}`,z]))
  // These existing controls drifted by up to 0.266m when the 1m road support entered the shaping envelope
  for(const [x,y,z] of [[160,752,293.766],[136,776,311.348],[168,784,322.258],[152,800,331.859]])
    assert.equal(samples.get(`${x},${y}`),z,`forest refinement changed the existing slope at ${x},${y}`)
})
