import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { buildGround } from '../../../../tools/district-map.ts'
import { roadSurfaceConflicts } from '../../../../tools/check-road-width.ts'
import { polygonInside, pointInside, polygonArea, intersectionArea, roadOffsets } from '../../../../tools/district-geometry.ts'
import type { District } from '../../../../tools/district-types.ts'
import candidate from './candidate.json'
import propsReport from './props-report.json'

const root=new URL('../../../../',import.meta.url)
const text=readFileSync(new URL('source-assets/district-map/district.json',root),'utf8')
const source:District=JSON.parse(text)
const sha=createHash('sha256').update(text).digest('hex')
assert.equal(sha,candidate.source_sha256,'frozen map changed; review source before reusing candidate')
const building=source.buildings.find(b=>b.id==='V-15')!
const parcel=source.parcels.find(p=>p.id==='P09-A')!
const surface=candidate.surface
assert.equal(surface.kind,'service');assert.equal(surface.access,'service')
assert.equal(polygonArea(surface.polygon),198)
assert.ok(polygonInside(surface.polygon,parcel.polygon),'candidate escapes existing parcel')
assert.equal(source.roads.filter(r=>JSON.stringify(r.nodes)===JSON.stringify(candidate.preserve.service_road_nodes)).length,1)
const service=source.roads.find(r=>JSON.stringify(r.nodes)===JSON.stringify(candidate.preserve.service_road_nodes))!
assert.equal(service.access,'service');assert.equal(service.width,3)
for(const b of source.buildings) assert.ok(intersectionArea(surface.polygon,b.polygon)<1e-7,`candidate overlaps building ${b.id}`)
for(const s of source.surfaces.filter(s=>!s.elevated&&!s.building))assert.ok(intersectionArea(surface.polygon,s.polygon)<1e-7,`candidate overlaps existing surface ${s.id}`)
for(const area of candidate.reserved_areas)assert.ok(polygonInside(area.polygon,surface.polygon),`reserved ${area.name} outside surface`)
const roadPolygons=source.roads.flatMap((r,index)=>{
  if(r.building||['lift','interior','bridge','deck'].includes(r.kind))return []
  const points=r.nodes.map(n=>source.nodes[n]);const offsets=roadOffsets(points,r.width)
  return points.slice(1).map((b,i)=>{const a=points[i],u=offsets[i],v=offsets[i+1];return {road:index,nodes:r.nodes.slice(i,i+2),kind:r.kind,access:r.access??'public',polygon:[[a[0]-u[0],a[1]-u[1]],[b[0]-v[0],b[1]-v[1]],[b[0]+v[0],b[1]+v[1]],[a[0]+u[0],a[1]+u[1]]],heights:[a[2],b[2]]}})
})
const overlapping=roadPolygons.filter(r=>intersectionArea(r.polygon,surface.polygon)>1e-7)
assert.ok(overlapping.length>0)
for(const road of overlapping) {
  assert.equal(road.access,'service','proposal reaches a public road')
  assert.ok(road.heights.every(h=>Math.abs(h-surface.elevation)<1e-8),`candidate reaches slope ${road.nodes}`)
}
const mutable=structuredClone(source);mutable.surfaces.push(surface)
assert.deepEqual(roadSurfaceConflicts(mutable),[],'candidate causes road/surface conflicts')
function propClear(bounds:number[][]) {
  const [[x0,x1],[y0,y1]]=bounds;const footprint=[[x0,y0],[x1,y0],[x1,y1],[x0,y1]]
  for(const area of candidate.reserved_areas) assert.ok(intersectionArea(footprint,area.polygon)<1e-8,`prop overlaps ${area.name}`)
  // Include a standing capsule margin around the entire projected object, not just its feet
  const margin=.32;const expanded=[[x0-margin,y0-margin],[x1+margin,y0-margin],[x1+margin,y1+margin],[x0-margin,y1+margin]]
  for(const road of roadPolygons) assert.ok(intersectionArea(expanded,road.polygon)<1e-8,`prop capsule margin touches road ${road.road}`)
}
const ground=buildGround(source)
for(const prop of propsReport.props) {
  propClear(prop.map_bounds_xyz)
  for(const foot of prop.foot_points) {
    assert.ok(pointInside(foot,surface.polygon),`${prop.model} foot outside new surface`)
    assert.ok(Math.abs(foot[2]-surface.elevation)<.001,`${prop.model} foot unsupported`)
  }
}
let lo=Infinity,hi=-Infinity,n=0
for(let x=314;x<=326;x+=.25)for(let y=250;y<=266.5;y+=.25){const h=ground.height(x,y);lo=Math.min(lo,h);hi=Math.max(hi,h);n++}
assert.ok(hi-25<.11&&25-lo<.02,'earthwork envelope exceeded')
const invalid=structuredClone(source);invalid.surfaces.push({...surface,elevation:24.5})
assert.ok(roadSurfaceConflicts(invalid).some(c=>c.surface===surface.id),'failure check must reject lowered forecourt')
assert.throws(()=>propClear([[319.8,320.2],[260,261],[25,29.5]]),/prop overlaps/,'failure check must reject a prop in the service corridor')
assert.equal(createHash('sha256').update(readFileSync(new URL('source-assets/district-map/district.json',root))).digest('hex'),sha,'probe modified frozen source')
const report={status:'PASS',source_sha256:sha,source_unchanged:true,surface,area_m2:198,overlapping_roads:overlapping,ground:{samples:n,spacing_m:.25,min:lo,max:hi,max_cut_m:hi-25,max_fill_m:25-lo},props:propsReport.props.map(p=>({model:p.model,map_bounds_xyz:p.map_bounds_xyz,foot_count:p.foot_points.length})),capsule_margin_m:.32,reserved_areas:candidate.reserved_areas,failure_checks:['lowered surface rejected','object inside service corridor rejected'],limits:['source broad geometry, not Rust Ground/capsule proof','no actual vehicle turning simulation','GPU and human walking NOT RUN']}
console.log(JSON.stringify(report,null,2))
