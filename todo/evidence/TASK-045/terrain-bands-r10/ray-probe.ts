import {readFileSync} from 'node:fs'
import {buildGround} from '../../../../tools/district-map.ts'
const map=JSON.parse(readFileSync('source-assets/district-map/district.json','utf8'))
const ground=buildGround(map)
const state=JSON.parse(readFileSync('todo/evidence/TASK-045/terrain-bands-r10/off-native/state.json','utf8')).samples.at(-1)
const sub=(a:number[],b:number[])=>a.map((x,i)=>x-b[i]),dot=(a:number[],b:number[])=>a.reduce((r,v,i)=>r+v*b[i],0)
const cross=(a:number[],b:number[])=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
const q=state.rotation,eye=state.position,tan=Math.tan(55*Math.PI/360)
const rows=[]
for(const x of [600,1000])for(const y of [125,127,129,208,210,212,214]) {
 const v=[((x+.5)/1280*2-1)*tan*1280/720,(1-(y+.5)/720*2)*tan,-1]
 const uv=cross(q,v),uuv=cross(q,uv),d=v.map((n,i)=>n+2*(q[3]*uv[i]+uuv[i]))
 let best=Infinity,tri:any
 for(const source of ground.triangles) {
  const p=source.map(([x,y,z])=>[x,z,-y]);const e=sub(p[1],p[0]),f=sub(p[2],p[0]),h=cross(d,f),a=dot(e,h)
  if(Math.abs(a)<1e-8)continue
  const s=sub(eye,p[0]),u=dot(s,h)/a;if(u<0||u>1)continue
  const k=cross(s,e),vv=dot(d,k)/a;if(vv<0||u+vv>1)continue
  const t=dot(f,k)/a;if(t>0&&t<best){best=t;tri=source}
 }
 const hit=eye.map((n,i)=>n+best*d[i]);rows.push({pixel:[x,y],hit:[hit[0],-hit[2],hit[1]],depth:best,triangle:tri})
}
console.log(JSON.stringify({method:'Delaunator source-mesh rays; locating features only, not native Spade validation',rows},null,2))
