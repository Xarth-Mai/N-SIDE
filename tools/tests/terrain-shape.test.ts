import { test } from 'node:test'
import assert from 'node:assert/strict'
import source from '../../source-assets/district-map/district.json' with { type: 'json' }
import { bakeTerrain, mountainBase, replaceTerrain } from '../terrain-shape.ts'
import type { District } from '../district-types.ts'
import { buildGround } from '../district-map.ts'

const data:District=source

test('terrain bake preserves authored geometry and repeats without drift',()=>{
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

test('terrain noise protects complete road segments and footprints',()=>{
  const sample=structuredClone(data)
  sample.nodes.terrain_test_a=[-700,570,125];sample.nodes.terrain_test_b=[-400,570,125]
  sample.roads.push({nodes:['terrain_test_a','terrain_test_b'],kind:'lane',width:4})
  sample.surfaces.push({polygon:[[-700,630],[-500,630],[-500,700],[-700,700]],elevation:132,kind:'platform'})
  const samples=bakeTerrain(sample).terrain.samples
  assert.equal(samples.find(p=>p[0]===-580&&p[1]===570)?.[2],125,'long road midpoint remains at its true grade')
  assert.equal(samples.find(p=>p[0]===-580&&p[1]===690)?.[2],132,'platform interior is protected')
  const result=bakeTerrain(data),ground=buildGround(result),rest=result.surfaces.find(s=>s.id==='fw-e-existing-20')!
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
