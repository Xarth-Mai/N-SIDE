import type { Point, Building, Road } from './district-types.ts'
// Simple polygons in design metres; boundary contact has no occupied area
const EPS=1e-7
const cross=(a: Point,b: Point,c: Point)=>(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
const at=(a: Point,b: Point,t: number)=>a.map((v,i)=>v+(b[i]-v)*t)
const edges=(p: Point[])=>p.map((a,i)=>[a,p[(i+1)%p.length]])
const signedArea=(p: Point[])=>edges(p).reduce((n,[a,b])=>n+a[0]*b[1]-b[0]*a[1],0)/2
export const polygonArea=(p: Point[])=>Math.abs(signedArea(p))
export const onBoundary=(p: Point,polygon: Point[])=>edges(polygon).some(([a,b])=>Math.abs(cross(a,b,p))<EPS&&p[0]>=Math.min(a[0],b[0])-EPS&&p[0]<=Math.max(a[0],b[0])+EPS&&p[1]>=Math.min(a[1],b[1])-EPS&&p[1]<=Math.max(a[1],b[1])+EPS)
export function pointInside(p: Point,polygon: Point[],boundary=true){
  if(onBoundary(p,polygon))return boundary
  let inside=false
  for(const [a,b] of edges(polygon))if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])inside=!inside
  return inside
}

// Split at every edge crossing, including collinear boundary endpoints
export function segmentIntervals(a: Point,b: Point,polygon: Point[],boundary=false){
  const dx=b[0]-a[0],dy=b[1]-a[1],length2=dx*dx+dy*dy
  if(length2<EPS)return pointInside(a,polygon,boundary)?[[0,1]]:[]
  const cuts=[0,1]
  for(const [c,d] of edges(polygon)){
    const ex=d[0]-c[0],ey=d[1]-c[1],den=dx*ey-dy*ex
    if(Math.abs(den)>EPS){
      const t=((c[0]-a[0])*ey-(c[1]-a[1])*ex)/den,u=((c[0]-a[0])*dy-(c[1]-a[1])*dx)/den
      if(t>-EPS&&t<1+EPS&&u>-EPS&&u<1+EPS)cuts.push(Math.max(0,Math.min(1,t)))
    }else if(Math.abs(cross(a,b,c))<EPS)for(const p of [c,d]){
      const t=((p[0]-a[0])*dx+(p[1]-a[1])*dy)/length2
      if(t>0&&t<1)cuts.push(t)
    }
  }
  cuts.sort((a,b)=>a-b)
  return cuts.slice(1).flatMap((hi,i)=>hi-cuts[i]>EPS&&pointInside(at(a,b,(cuts[i]+hi)/2),polygon,boundary)?[[cuts[i],hi]]:[])
}
export const segmentInside=(a: Point,b: Point,p: Point[])=>pointInside(a,p)&&pointInside(b,p)&&Math.abs(segmentIntervals(a,b,p,true).reduce((n,[lo,hi])=>n+hi-lo,0)-1)<EPS
export const polygonInside=(p: Point[],container: Point[])=>edges(p).every(([a,b])=>segmentInside(a,b,container))

function triangles(polygon: Point[]){
  const p=signedArea(polygon)<0?[...polygon].reverse():[...polygon],out=[]
  while(p.length>3){
    const i=p.findIndex((b,i)=>{
      const a=p[(i+p.length-1)%p.length],c=p[(i+1)%p.length]
      return cross(a,b,c)>EPS&&!p.some(v=>v!==a&&v!==b&&v!==c&&cross(a,b,v)>=-EPS&&cross(b,c,v)>=-EPS&&cross(c,a,v)>=-EPS)
    })
    if(i<0){
      const collinear=p.findIndex((b,i)=>Math.abs(cross(p[(i+p.length-1)%p.length],b,p[(i+1)%p.length]))<EPS)
      if(collinear<0)throw new Error('Expected a non-self-intersecting polygon')
      p.splice(collinear,1)
    }else{out.push([p[(i+p.length-1)%p.length],p[i],p[(i+1)%p.length]]);p.splice(i,1)}
  }
  if(p.length===3)out.push(p)
  return out
}
function cut(subject: Point[],distance: (p: Point) => number){
  const out=[]
  for(const [p,q] of edges(subject)){
    const u=distance(p),v=distance(q)
    if(u>=-EPS)out.push(p)
    if((u>=-EPS)!==(v>=-EPS))out.push(at(p,q,u/(u-v)))
  }
  return out
}
function clip(subject: Point[],convex: Point[]){
  let out=subject
  for(const [a,b] of edges(signedArea(convex)<0?[...convex].reverse():convex)){
    out=cut(out,p=>cross(a,b,p))
    if(!out.length)break
  }
  return out
}
export function intersectionArea(a: Point[],b: Point[]){
  // ponytail: pairwise ear clipping suits the small map; index/cache if geometry grows
  return triangles(a).reduce((sum,t)=>sum+triangles(b).reduce((n,u)=>n+polygonArea(clip(t,u)),0),0)
}
export const roadWidth=(r: Pick<Road, "kind"> & Partial<Road>)=>r.width??(({main:10,avenue:15,shore:7,bridge:16,deck:5,steps:6,trail:4,service:5,landing:9} as Record<string,number>)[r.kind]??7)
export const roadAllowed=(road: Partial<Road>,user='public')=>road.access==='controlled'?(road.users??[]).includes(user):['service','resident'].includes(road.access ?? '')?road.access===user:!road.access||road.access==='public'

export function roadHitsBuilding(a: Point,b: Point,width: number,building: Pick<Building,"polygon" | "elevation" | "height">){
  const dz=b[2]-a[2],bottom=building.elevation,top=bottom+building.height
  let lo=0,hi=1
  if(Math.abs(dz)<EPS){if(a[2]<bottom-EPS||a[2]>=top-EPS)return false}
  else{const t=[(bottom-a[2])/dz,(top-a[2])/dz].sort((a,b)=>a-b);lo=Math.max(0,t[0]);hi=Math.min(1,t[1]);if(hi-lo<EPS)return false}
  const p=at(a,b,lo),q=at(a,b,hi),dx=q[0]-p[0],dy=q[1]-p[1],length=Math.hypot(dx,dy)
  if(segmentIntervals(a,b,building.polygon).some(([start,end])=>Math.min(end,hi)-Math.max(start,lo)>EPS))return true
  if(width===0)return false
  if(length<EPS)return pointInside(p,building.polygon,false)
  const x=-dy/length*width/2,y=dx/length*width/2
  // Flat ends meet facade doors without projecting a round cap through the wall
  let strip=[[p[0]+x,p[1]+y],[q[0]+x,q[1]+y],[q[0]-x,q[1]-y],[p[0]-x,p[1]-y]]
  const outline=signedArea(building.polygon)<0?[...building.polygon].reverse():building.polygon
  for(const [end,other] of [[a,b],[b,a]])if(onBoundary(end,outline))for(const [u,v] of edges(outline))if(onBoundary(end,[u,v])&&cross(u,v,other)<-EPS){
    // An oblique doorway terminates against its facade, not perpendicular to its centreline
    strip=cut(strip,p=>-cross(u,v,p))
  }
  return strip.length>2&&intersectionArea(strip,building.polygon)>EPS
}
