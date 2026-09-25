// Areas and section distances use design metres, independently of camera projection
import { polygonArea } from './district-geometry.mjs'
export { polygonArea, pointInside as inside } from './district-geometry.mjs'

export function routeProfile(nodes, ids) {
  let distance=0
  return ids.map((id,i)=> {
    const point=nodes[id],previous=i?nodes[ids[i-1]]:point
    const length=Math.hypot(point[0]-previous[0],point[1]-previous[1])
    distance+=length
    return {id,point,distance,length,rise:point[2]-previous[2],grade:length?(point[2]-previous[2])/length*100:null}
  })
}

export function parcelStats(data) {
  return data.parcels.map(parcel=>{
    const drawn=data.buildings.filter(b=>b.bank==='district'&&b.parcel===parcel.id).length
    return {...parcel,drawn,remaining:parcel.capacity-drawn}
  })
}

export function planStats(data) {
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

export function housingEstimate(data) {
  const gross=data.parcels.reduce((sum,p)=>sum+(p.housing?polygonArea(p.polygon)*p.housing.coverage*p.housing.floors*p.housing.share:0),0)
  const model=data.housingAssumptions,units=gross*model.netRatio/model.unitArea
  return {gross,units,residents:model.occupancy.map((value,i)=>Math.round(units*value*model.household[i]))}
}
