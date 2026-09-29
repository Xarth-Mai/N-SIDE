// Read-only source diagnostics; native Spade and real rendered evidence remain separate
import {buildGround} from '../../../../tools/district-map.ts'
import type {District, Point} from '../../../../tools/district-types.ts'
import assert from 'node:assert/strict'
const root=new URL('../../../../',import.meta.url)
const baseline='f93af27'
const oldText=Bun.spawnSync(['git','show',`${baseline}:source-assets/district-map/district.json`],{cwd:root.pathname}).stdout.toString()
const newText=await Bun.file(new URL('source-assets/district-map/district.json',root)).text()
const before:District=JSON.parse(oldText),after:District=JSON.parse(newText)
assert.deepEqual({...after,terrain:before.terrain},before)
assert.deepEqual(after.terrain.samples.slice(0,42),before.terrain.samples.slice(0,42))
const grounds=[buildGround(before),buildGround(after)]
const slope=(ground:ReturnType<typeof buildGround>,x:number,y:number)=>Math.atan(Math.hypot((ground.height(x+.1,y)-ground.height(x-.1,y))/.2,(ground.height(x,y+.1)-ground.height(x,y-.1))/.2))*180/Math.PI
const angles=grounds.map(g=>{
  const values=[]
  for(let y=704;y<=776;y+=4)for(let x=132;x<=216;x+=4)values.push(slope(g,x,y))
  values.sort((a,b)=>a-b)
  return {samples:values.length,p50:values[Math.floor(values.length*.5)],p95:values[Math.floor(values.length*.95)],maximum:values.at(-1),over55:values.filter(x=>x>55).length}
})
const triangleAt=(g:ReturnType<typeof buildGround>,p:number[])=>g.triangles.find(t=>t.every((a,i)=>{const b=t[(i+1)%3];return(b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0])>=-1e-8}))
const points=[[171.4934,739.7493],[156.4618,741.9763],[155.4546,738.915],[133.13,746.5454],[129.2551,746.5177]]
const rays=points.map(p=>({point:p,before_after:grounds.map(g=>({height:g.height(p[0],p[1]),slope_degrees:slope(g,p[0],p[1]),triangle:triangleAt(g,p)}))}))
const map=new Map(after.terrain.samples.map(p=>[p.slice(0,2).join(','),p[2]]))
const changes=before.terrain.samples.flatMap(p=>{const z=map.get(p.slice(0,2).join(','));return z!==undefined&&Math.abs(z-p[2])>.0001?[{point:p,after:z,delta:z-p[2]}]:[]})
const comparisons=[[-151.43885889,801.78806574],[-150,802.803],[-148.56114111,803.81793426],[-41.01511846,754.41628110]].map(p=>({point:p,before_after:grounds.map(g=>({height:g.height(p[0],p[1]),triangle:triangleAt(g,p)}))}))
const clearance=(ground:ReturnType<typeof buildGround>,eye:Point,target:Point)=>{
  const length=Math.hypot(target[0]-eye[0],target[1]-eye[1]);let min=Infinity
  for(let s=1;s<length-5;s+=.5){const p=eye.map((v,i)=>v+(target[i]-v)*s/length);min=Math.min(min,p[2]-ground.height(p[0],p[1]))}
  return min
}
const home=after.nodes.home,rest=after.nodes.hill_short_rest3,d=Math.hypot(home[0]-rest[0],home[1]-rest[1]),eye=[rest[0]+3*(home[0]-rest[0])/d,rest[1]+3*(home[1]-rest[1])/d,rest[2]+1.7]
const sightlines=['home','station','upper'].map(id=>({target:id,minimum_before_after:grounds.map(g=>clearance(g,eye,after.nodes[id]))}))
console.log(JSON.stringify({scope:'Delaunator uncut source Ground; not native placement, masked rendering or visual acceptance',baseline,source_sha256:[oldText,newText].map(x=>new Bun.CryptoHasher('sha256').update(x).digest('hex')),sample_counts:[before.terrain.samples.length,after.terrain.samples.length],non_terrain_unchanged:true,authored42_unchanged:true,old_coordinate_changes:changes,changed_control_bounds:[0,1].map(i=>[Math.min(...changes.map(p=>p.point[i])),Math.max(...changes.map(p=>p.point[i]))]),cut_window_slope_before_after:angles,original_capture_ray_positions:rays,native_comparison_requests:comparisons,third_rest_terrain_sightlines:sightlines},null,2))
