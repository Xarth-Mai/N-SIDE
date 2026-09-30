// Source-ground diagnostic, not a replacement for captured GPU mesh inspection
import {buildGround} from '../../../../tools/district-map.ts'
const root = new URL('../../../../', import.meta.url)
const sourceFile = Bun.file(new URL('source-assets/district-map/district.json', root))
const data = await sourceFile.json()
const sourceSha = new Bun.CryptoHasher('sha256').update(await sourceFile.arrayBuffer()).digest('hex')
const capture = await Bun.file(new URL('output/visual-r9/before-understory/state.json', root)).json()
const sample = capture.samples.find((s: any) => s.frame === 59)
const origin = sample.position, rotation = sample.rotation
const sub = (a: number[], b: number[]) => a.map((v, i) => v-b[i])
const cross = (a: number[], b: number[]) => [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
const dot = (a: number[],b: number[]) => a.reduce((s,v,i)=>s+v*b[i],0)
const unit = (v: number[])=>v.map(x=>x/Math.hypot(...v))
const triangles = buildGround(data).triangles.map(t=>t.map(p=>[p[0],p[2],-p[1]]))
function ray(x: number,y: number) {
  const t = Math.tan(55*Math.PI/360)
  const v = [(2*(x+.5)/1280-1)*t*1280/720,(1-2*(y+.5)/720)*t,-1]
  const u = rotation.slice(0,3), uv = cross(u,v), uuv = cross(u,uv)
  return unit(v.map((c,i)=>c+2*rotation[3]*uv[i]+2*uuv[i]))
}
function hit(x: number,y: number) {
  const d=ray(x,y); let result: any = null
  for(const points of triangles) {
    const e1=sub(points[1],points[0]), e2=sub(points[2],points[0])
    const h=cross(d,e2), determinant=dot(e1,h)
    if(Math.abs(determinant)<1e-9)continue
    const s=sub(origin,points[0]), u=dot(s,h)/determinant
    if(u<0||u>1)continue
    const q=cross(s,e1),v=dot(d,q)/determinant
    if(v<0||u+v>1)continue
    const distance=dot(e2,q)/determinant
    if(distance<0||result&&distance>=result.distance)continue
    const normal=unit(cross(e1,e2)), abs=normal.map(Math.abs)
    const chart=abs[1]>=abs[0]&&abs[1]>=abs[2]?'XZ':abs[0]>=abs[2]?'ZY':'XY'
    const p=origin.map((o: number,i: number)=>o+d[i]*distance)
    const uv=chart==='XZ'?[p[0],-p[2]]:chart==='ZY'?[p[2],p[1]]:[p[0],p[1]]
    result={distance,point:p,normal,view_normal_dot:Math.abs(dot(d,normal)),chart,uv,points}
  }
  return result
}
const results=[]
for(const pixel of [[780,320],[950,500],[1160,640],[500,580],[450,280]]) {
  const p=hit(...pixel as [number,number]),px=hit(pixel[0]+1,pixel[1]),py=hit(pixel[0],pixel[1]+1)
  results.push({pixel,...p,uv_dx_m:px&&sub(px.uv,p.uv),uv_dy_m:py&&sub(py.uv,p.uv)})
}
console.log(JSON.stringify({scope:'source ground; uncut Delaunator may use different coplanar diagonals from Bevy Spade',source_ground_sha256:sourceSha,frame:59,origin,rotation,fov_degrees:55,results},null,2))
