import type { District } from './district-types.ts'
// Areas and section distances use design metres, independently of camera projection
import { polygonArea, pointInside, onBoundary, roadAllowed } from './district-geometry.ts'
export { polygonArea, pointInside as inside } from './district-geometry.ts'

export function routeProfile(nodes: District["nodes"], ids: string[]) {
  let distance=0
  return ids.map((id,i)=> {
    const point=nodes[id],previous=i?nodes[ids[i-1]]:point
    const length=Math.hypot(point[0]-previous[0],point[1]-previous[1])
    distance+=length
    return {id,point,distance,length,rise:point[2]-previous[2],grade:length?(point[2]-previous[2])/length*100:null}
  })
}

export function parcelStats(data: District) {
  return data.parcels.map(parcel=>{
    const drawn=data.buildings.filter(b=>b.bank==='district'&&b.parcel===parcel.id).length
    return {...parcel,drawn,remaining:parcel.capacity-drawn}
  })
}

export function planStats(data: District) {
  const parcels=parcelStats(data)
  return {
    blocks:data.blocks.length,
    parcelArea:data.parcels.reduce((sum,p)=>sum+polygonArea(p.polygon),0),
    urbanArea:data.blocks.reduce((sum,b)=>sum+polygonArea(b.urbanPolygon??b.polygon),0),
    blockArea:data.blocks.reduce((sum,b)=>sum+polygonArea(b.polygon),0),
    capacity:data.parcels.reduce((sum,p)=>sum+p.capacity,0),
    remaining:parcels.reduce((sum,p)=>sum+p.remaining,0),
    places:data.places.filter(p=>p.featured).length,
    detailed:data.places.filter(p=>p.detail).length,
    buildings:data.buildings.filter(b=>b.bank==='district').length,
    background:data.buildings.filter(b=>b.bank==='opposite').length,
  }
}

export function housingEstimate(data: District,drawn=false) {
  const gross=drawn?data.buildings.filter(b=>b.bank==='district').reduce((sum,b)=>{
    const share=data.parcels.find(p=>p.id===b.parcel)?.housing?.share??0
    const area=polygonArea(b.polygon)-(b.design?.lightwell?polygonArea(b.design.lightwell):0)
    const floors=b.design?.floors.filter(f=>f.z<b.elevation+b.height).length??0
    return sum+area*floors*share
  },0):data.parcels.reduce((sum,p)=>sum+(p.housing?polygonArea(p.polygon)*p.housing.coverage*p.housing.floors*p.housing.share:0),0)
  const model=data.housingAssumptions,units=gross*model.netRatio/model.unitArea
  return {gross,units,residents:model.occupancy.map((value,i)=>Math.round(units*value*model.household[i]))}
}

export function reachableNodes(data: {roads: (Pick<District["roads"][number], "nodes"> & Partial<District["roads"][number]>)[]},user='public',start='station') {
  const reached=new Set([start])
  let previous=-1
  while(previous!==reached.size){
    previous=reached.size
    for(const road of data.roads)if(roadAllowed(road,user)&&road.nodes.some(id=>reached.has(id)))road.nodes.forEach(id=>reached.add(id))
  }
  return reached
}

// Coverage measures recorded geometry and connectivity, not design approval
export function frameworkCoverage(data: District) {
  const reached=new Map<string, Set<string>>()
  const connected=(entry: {role?: string; node: string})=>{
    const role=entry.role??'public'
    if(!reached.has(role))reached.set(role,reachableNodes(data,role))
    return reached.get(role)!.has(entry.node)
  }
  return data.blocks.map(block=>{
    const parcels=data.parcels.filter(p=>p.block===block.id)
    const buildings=data.buildings.filter(b=>b.bank==='district'&&parcels.some(p=>p.id===b.parcel))
    const entries=buildings.flatMap(b=>b.design?.entries??[])
    const complete=buildings.filter(b=>b.design?.floors?.length&&b.design.floors.every(f=>f.use)&&b.design.entries?.length&&Number.isFinite(b.elevation)&&b.height>0)
    const arrived=complete.filter(b=>b.design!.entries.every(e=>{
      const p=data.nodes[e.node],floor=b.design!.floors.find(f=>f.name===e.level)
      return p&&floor&&Math.abs(p[2]-floor.z)<1e-6&&onBoundary(p,b.polygon)&&connected(e)
    }))
    const roads=data.roads.filter(r=>r.kind!=='interior'&&r.nodes.some(id=>pointInside(data.nodes[id],block.polygon)))
    const linked=roads.filter(r=>r.nodes.every(node=>connected({node,role:r.access==='controlled'?r.users?.[0]:r.access??'public'}))).length
    return {id:block.id,name:block.name,buildings:buildings.length,described:complete.length,arrived:arrived.length,entries:entries.length,connected:entries.filter(connected).length,roads:roads.length,widths:roads.filter(r=>r.width>0).length,linked}
  })
}
