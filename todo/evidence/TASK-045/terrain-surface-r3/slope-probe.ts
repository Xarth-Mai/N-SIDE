// Coarse source-area inventory; Bevy's clipped rendered triangle checks remain authoritative
import {buildGround} from '../../../../tools/district-map.ts'
const source = Bun.file(new URL('../../../../source-assets/district-map/district.json', import.meta.url))
const map = await source.json()
const ranges = [0, 20, 35, 45, 60, 90].slice(1).map((to,i)=>({from:[0,20,35,45,60][i],to,triangles:0,area_m2:0}))
for (const points of buildGround(map).triangles) {
  const a=points[1].map((v,i)=>v-points[0][i]), b=points[2].map((v,i)=>v-points[0][i])
  const n=[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
  const magnitude=Math.hypot(...n), slope=Math.acos(Math.abs(n[2])/magnitude)*180/Math.PI
  const bin=ranges.find(r=>slope<=r.to)!
  bin.triangles++; bin.area_m2+=magnitude/2
}
console.log(JSON.stringify({scope:'Delaunator source ground before road/plot clipping, including boundary skirt',source_sha256:new Bun.CryptoHasher('sha256').update(await source.arrayBuffer()).digest('hex'),ranges},null,2))
