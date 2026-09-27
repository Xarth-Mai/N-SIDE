import { createHash } from 'node:crypto'
import { clip, polygonArea, roadOffsets } from './district-geometry.ts'
import type { District, Point } from './district-types.ts'

type RoadData=Pick<District,'nodes'|'roads'|'routes'>
type Segment={road:number;nodes:string[];kind:string;polygon:Point[];patches:Point[][]}
const cross=(a:Point,b:Point,c:Point)=>(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
const lerp=(a:Point,b:Point,t:number)=>a.map((v,i)=>v+(b[i]-v)*t)
const edgeKey=(ids:string[])=>[...ids].sort().join('|')
const overlaps=(a:Point[],b:Point[])=>[0,1].every(i=>Math.min(...a.map(p=>p[i]))<Math.max(...b.map(p=>p[i]))&&Math.min(...b.map(p=>p[i]))<Math.max(...a.map(p=>p[i])))
function height(polygon:Point[],p:Point) {
  if(polygon.every(v=>v[2]===polygon[0][2]))return polygon[0][2]
  const [a,b,c]=polygon,area=cross(a,b,c)
  return (cross(p,b,c)*a[2]+cross(a,p,c)*b[2]+cross(a,b,p)*c[2])/area
}

function segments(data:RoadData):Segment[] {
  return data.roads.flatMap((road,index)=>{
    if(road.building||['interior','lift','bridge','deck'].includes(road.kind))return []
    const points=road.nodes.map(id=>{
      const p=data.nodes[id]
      if(!p||p.length!==3||!p.every(Number.isFinite))throw new Error(`roads[${index}]: invalid node ${id}`)
      return p
    })
    if(points.length<2||!Number.isFinite(road.width)||road.width<=0)throw new Error(`roads[${index}]: requires two nodes and positive width`)
    const offsets=roadOffsets(points,road.width)
    return points.slice(1).map((b,i)=>{
      const a=points[i],u=offsets[i],v=offsets[i+1]
      // Same vertex order and triangle diagonal as geometry.rs::ribbon_height
      const polygon=[[a[0]-u[0],a[1]-u[1],a[2]],[b[0]-v[0],b[1]-v[1],b[2]],[b[0]+v[0],b[1]+v[1],b[2]],[a[0]+u[0],a[1]+u[1],a[2]]]
      let patches:Point[][]
      if(road.kind==='steps') {
        const count=Math.max(1,Math.ceil(Math.abs(b[2]-a[2])/.17))
        patches=Array.from({length:count},(_,j)=>{
          const z=a[2]+(b[2]-a[2])*(j+.5)/count
          return [lerp(polygon[0],polygon[1],j/count),lerp(polygon[0],polygon[1],(j+1)/count),lerp(polygon[3],polygon[2],(j+1)/count),lerp(polygon[3],polygon[2],j/count)].map(p=>[p[0],p[1],z])
        })
      }else patches=[[polygon[0],polygon[1],polygon[2]],[polygon[0],polygon[2],polygon[3]]]
      return {road:index,nodes:road.nodes.slice(i,i+2),kind:road.kind,polygon,patches:patches.filter(p=>polygonArea(p)>1e-8)}
    })
  })
}

export function roadWidthConflicts(data:RoadData,scope:string) {
  const route=scope==='all'?undefined:data.routes.find(r=>r.id===scope)
  if(scope!=='all'&&!route)throw new Error(`Unknown route ${JSON.stringify(scope)}`)
  const selected=route?new Set(route.nodes.slice(1).map((id,i)=>edgeKey([route.nodes[i],id]))):undefined
  const roads=segments(data),conflicts=[]
  if(selected)for(const edge of selected)if(!roads.some(r=>edgeKey(r.nodes)===edge))throw new Error(`Route ${scope}: missing ground road ${edge}`)
  // ponytail: pairwise bounds suit this district; add a spatial index only if measured map size requires it
  for(let i=0;i<roads.length;i++)for(let j=i+1;j<roads.length;j++) {
    const a=roads[i],b=roads[j]
    if(selected&&!selected.has(edgeKey(a.nodes))&&!selected.has(edgeKey(b.nodes)))continue
    if(!overlaps(a.polygon,b.polygon))continue
    const intersection=clip(a.polygon,b.polygon),area=polygonArea(intersection)
    if(area<=.5)continue
    let maxDelta=0,point:Point=[]
    for(const p of a.patches)for(const q of b.patches) {
      if(!overlaps(p,q))continue
      const overlap=clip(p,q)
      if(polygonArea(overlap)<1e-8)continue
      for(const v of overlap) {
        const delta=Math.abs(height(p,v)-height(q,v))
        if(delta>maxDelta){maxDelta=delta;point=v.slice(0,2)}
      }
    }
    if(maxDelta>.35)conflicts.push({a:{road:a.road,nodes:a.nodes,kind:a.kind},b:{road:b.road,nodes:b.nodes,kind:b.kind},area,maxDelta,point,sharedNodes:a.nodes.filter(id=>b.nodes.includes(id))})
  }
  return conflicts.sort((a,b)=>b.maxDelta-a.maxDelta)
}

if(import.meta.main) {
  try {
    const scope=process.argv[2]
    if(!scope||process.argv.length!==3)throw new Error('Usage: bun tools/check-road-width.ts <route-id|all>')
    const text=await Bun.file(new URL('../source-assets/district-map/district.json',import.meta.url)).text()
    const conflicts=roadWidthConflicts(JSON.parse(text),scope)
    console.log(JSON.stringify({scope,source_sha256:createHash('sha256').update(text).digest('hex'),result:conflicts.length?'FAIL':'PASS',thresholds:{overlapArea:0.5,heightDelta:0.35},scopeNote:'Ground-road top-surface candidates, including shared endpoints; bounded miters and horizontal stair treads match geometry.rs. Buildings, platform masks, retaining walls, bridge clearance and player collision require separate checks.',conflicts},null,2))
    process.exitCode=conflicts.length?1:0
  }catch(error) {
    console.error(`[road-width] ${error instanceof Error?error.message:String(error)}`)
    process.exitCode=1
  }
}
