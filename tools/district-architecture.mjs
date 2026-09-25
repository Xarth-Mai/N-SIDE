import { polygonArea } from './district-plan.mjs'
import { roadWidth, segmentIntervals } from './district-geometry.mjs'

// Keep section distances in metres while sharing the map's polygon clipping
export function sectionIntervals([a,b], polygon) {
  const length=Math.hypot(b[0]-a[0],b[1]-a[1])
  return length?segmentIntervals(a,b,polygon,true).map(([start,end])=>[start*length,end*length]):[]
}

export function streetPosition(point, route) {
  let station=0,best={distance:Infinity,station:0}
  for(let i=1;i<route.length;i++) {
    const a=route[i-1],b=route[i],dx=b[0]-a[0],dy=b[1]-a[1],length=Math.hypot(dx,dy)
    if(!length)continue
    const t=Math.max(0,Math.min(1,((point[0]-a[0])*dx+(point[1]-a[1])*dy)/length**2)),distance=Math.hypot(point[0]-a[0]-dx*t,point[1]-a[1]-dy*t)
    if(distance<best.distance)best={distance,station:station+length*t}
    station+=length
  }
  return best
}

export function architectureStats(data, architecture) {
  const buildings=data.buildings.filter(b=>architecture.buildings.includes(b.id))
  return architecture.types.map(type=> {
    const items=buildings.filter(b=>b.design?.type===type.id)
    return {...type,count:items.length,footprint:items.reduce((sum,b)=>sum+polygonArea(b.polygon)-(b.design.lightwell?polygonArea(b.design.lightwell):0),0),floorArea:items.reduce((sum,b)=>sum+b.design.floors.reduce((area,floor)=>area+polygonArea(b.polygon)-(floor.rooms?.filter(room=>room.kind==='court').reduce((voids,room)=>voids+polygonArea(room.polygon),0)||(b.design.lightwell?polygonArea(b.design.lightwell):0)),0),0)}
  })
}

export function sectionRoads(data, section) {
  const hits=[],[p,q]=section.line,sx=q[0]-p[0],sy=q[1]-p[1],cutLength=Math.hypot(sx,sy)
  for(const road of data.roads) {
    if(['interior','lift'].includes(road.kind))continue
    const width=roadWidth(road)
    for(let i=1;i<road.nodes.length;i++) {
      const a=data.nodes[road.nodes[i-1]],b=data.nodes[road.nodes[i]],dx=b[0]-a[0],dy=b[1]-a[1],length=Math.hypot(dx,dy),cross=sx*dy-sy*dx
      // Nearly longitudinal streets belong to the ground profile, not a cross-width label
      if(!length||Math.abs(cross)<cutLength*length*.2)continue
      const t=((a[0]-p[0])*dy-(a[1]-p[1])*dx)/cross,u=((a[0]-p[0])*sy-(a[1]-p[1])*sx)/cross
      if(t<0||t>1||u<0||u>1)continue
      const ox=-dy/length*width/2,oy=dx/length*width/2,polygon=[[a[0]+ox,a[1]+oy],[b[0]+ox,b[1]+oy],[b[0]-ox,b[1]-oy],[a[0]-ox,a[1]-oy]]
      const spans=sectionIntervals(section.line,polygon),station=t*cutLength,elevation=a[2]+u*(b[2]-a[2])
      if(!spans.length)continue
      const span=[Math.min(...spans.map(s=>s[0])),Math.max(...spans.map(s=>s[1]))],previous=hits.find(hit=>Math.abs(hit.station-station)<.01&&Math.abs(hit.elevation-elevation)<.1)
      if(previous){previous.span=[Math.min(previous.span[0],span[0]),Math.max(previous.span[1],span[1])];if(width>previous.width)Object.assign(previous,{width,kind:road.kind,access:road.access})}
      else hits.push({kind:road.kind,width,span,station,elevation,access:road.access})
    }
  }
  return hits.sort((a,b)=>a.station-b.station)
}
