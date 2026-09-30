import {buildGround} from '../../../../tools/district-map.ts'
import type {District} from '../../../../tools/district-types.ts'
import assert from 'node:assert/strict'
const root=new URL('../../../../',import.meta.url)
const before:District=await Bun.file(new URL('output/assets/terrain-shape-r4/before-district.json',root)).json()
const after:District=await Bun.file(new URL('source-assets/district-map/district.json',root)).json()
assert.deepEqual({...after,terrain:before.terrain},before)
assert.deepEqual(after.terrain.samples.slice(0,42),before.terrain.samples.slice(0,42))
assert.deepEqual({...after.terrain,samples:before.terrain.samples},before.terrain)
assert.equal(after.nodes.summit[2],450)
assert.equal(Math.max(...after.terrain.samples.map(p=>p[2])),450)
const grounds=[buildGround(before),buildGround(after)]
const slope=(ground:ReturnType<typeof buildGround>,x:number,y:number)=>Math.atan(Math.hypot((ground.height(x+.1,y)-ground.height(x-.1,y))/.2,(ground.height(x,y+.1)-ground.height(x,y-.1))/.2))*180/Math.PI
const controls=new Map(after.terrain.samples.map(p=>[p.slice(0,2).join(','),p[2]]))
assert.ok(before.terrain.samples.every(p=>controls.has(p.slice(0,2).join(','))),'all old terrain coordinates must remain')
const changes=before.terrain.samples.flatMap(p=>{const z=controls.get(p.slice(0,2).join(','));return z!==undefined&&Math.abs(z-p[2])>.0001?[{point:p,after:z,delta:z-p[2]}]:[]})
const outsideChanges=changes.filter(({point:[x,y]})=>((-0.9682458366*(x-144)+.25*(y-850))/12)**2+((.25*(x-144)+.9682458366*(y-850))/30)**2>=1)
assert.deepEqual(outsideChanges,[],'road support must not change old terrain controls outside the hollow')
assert.ok(changes.every(({delta})=>delta<0&&delta>=-2.4),'the hollow lowers old controls by at most 2.4m')
const window=grounds.map(g=>{
 const values=[]
 for(let y=824;y<=876;y+=2)for(let x=130;x<=158;x+=2)values.push(slope(g,x,y))
 values.sort((a,b)=>a-b)
 return {samples:values.length,p50:values[Math.floor(values.length*.5)],p95:values[Math.floor(values.length*.95)],maximum:values.at(-1)}
})
const sections=[838,848,858,868].map(y=>({north:y,points:Array.from({length:9},(_,i)=>{const x=128+4*i;return {x,height:grounds.map(g=>g.height(x,y)),slope:grounds.map(g=>slope(g,x,y))}})}))
console.log(JSON.stringify({scope:'CPU Delaunator source ground, native Spade/runtime and visuals separate',baseline:'13397c5626c8952511ceda52d3c4a8877b901acc',non_terrain_unchanged:true,terrain_metadata_unchanged:true,authored42_unchanged:true,summit450_unchanged:true,all_old_coordinates_retained:true,samples:[before.terrain.samples.length,after.terrain.samples.length],old_coordinates_changed_outside_hollow:outsideChanges.length,old_coordinate_changes:changes,window_slopes:window,sections},null,2))
