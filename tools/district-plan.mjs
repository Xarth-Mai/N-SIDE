// Areas and section distances use the plan's metre coordinates, independently of camera projection
export const polygonArea = points => Math.abs(points.reduce((sum, [x,y], i) => {
  const [u,v] = points[(i+1)%points.length]
  return sum+x*v-u*y
}, 0))/2

export function inside(point, polygon) {
  const [x,y]=point
  let result=false
  for(let i=0,j=polygon.length-1;i<polygon.length;j=i++) {
    const [a,b]=polygon[i],[u,v]=polygon[j]
    if(Math.abs((x-a)*(v-b)-(y-b)*(u-a))<1e-7 && x>=Math.min(a,u) && x<=Math.max(a,u) && y>=Math.min(b,v) && y<=Math.max(b,v))return true
    if((b>y)!==(v>y) && x<(u-a)*(y-b)/(v-b)+a)result=!result
  }
  return result
}

export function routeProfile(nodes, ids) {
  let distance=0
  return ids.map((id,i)=> {
    const point=nodes[id],previous=i?nodes[ids[i-1]]:point
    const length=Math.hypot(point[0]-previous[0],point[1]-previous[1])
    distance+=length
    return {id,point,distance,length,rise:point[2]-previous[2],grade:length?(point[2]-previous[2])/length*100:null}
  })
}

export function planStats(data) {
  return {
    blocks:data.blocks.length,
    parcelArea:data.parcels.reduce((sum,p)=>sum+polygonArea(p.polygon),0),
    urbanArea:data.blocks.reduce((sum,b)=>sum+polygonArea(b.urbanPolygon??b.polygon),0),
    blockArea:data.blocks.reduce((sum,b)=>sum+polygonArea(b.polygon),0),
    capacity:data.parcels.reduce((sum,p)=>sum+p.capacity,0),
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
