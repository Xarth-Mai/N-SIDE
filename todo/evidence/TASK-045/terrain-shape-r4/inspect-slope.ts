import data from '../../../../source-assets/district-map/district.json' with { type:'json' }
import {buildGround} from '../../../../tools/district-map.ts'
import {pointInside} from '../../../../tools/district-geometry.ts'
const ground=buildGround(data)
const rows=[]
for(let y=826;y<=878;y+=4)for(let x=124;x<=168;x+=4){
 let distance=Infinity,roadName=''
 for(const road of data.roads)if(!road.building&&!['bridge','deck','lift','interior'].includes(road.kind))for(let i=1;i<road.nodes.length;i++){
  const a=data.nodes[road.nodes[i-1]],b=data.nodes[road.nodes[i]],dx=b[0]-a[0],dy=b[1]-a[1],t=Math.max(0,Math.min(1,((x-a[0])*dx+(y-a[1])*dy)/(dx*dx+dy*dy)))
  const d=Math.hypot(x-a[0]-dx*t,y-a[1]-dy*t)-road.width/2
  if(d<distance){distance=d;roadName=road.nodes.join(' -> ')}
 }
 rows.push({p:[x,y,ground.height(x,y)],distance,roadName})
}
console.log(JSON.stringify(rows.filter(r=>r.p[0]===144||r.p[0]===152||r.p[0]===160),null,2))
