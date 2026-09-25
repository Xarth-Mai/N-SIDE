const tilt = 55 * Math.PI / 180, turn = 18 * Math.PI / 180
const c = Math.cos(turn), s = Math.sin(turn)
// A fixed river-side orthographic camera; coordinates and road lengths stay unchanged
export const project = ([x,y,z=0]) => [x*c+y*s, (x*s-y*c)*Math.sin(tilt)-z*Math.cos(tilt)]
export const depth = ([x,y]) => x*s-y*c
export const svgPath = points => points.map((p,i)=>`${i?'L':'M'}${project(p).map(v=>v.toFixed(2)).join(',')}`).join(' ')
export const heightColor = h => `rgb(${[223,213,175].map((v,i)=>Math.round(v+([103,139,112][i]-v)*Math.max(0,Math.min(1,h/80)))).join(',')})`

export function terrainHeight(points,x,y) {
  let sum=0, weight=0
  for(const [px,py,z] of points) {
    const d=(px-x)**2+(py-y)**2
    if(d===0)return z
    const w=1/d**1.5
    sum+=z*w;weight+=w
  }
  return sum/weight
}
const lerp = (a,b,t) => a.map((v,i)=>v+(b[i]-v)*t)
const tint = (rgb,factor) => `rgb(${rgb.map(v=>Math.round(Math.max(0,Math.min(255,v*factor)))).join(',')})`
const face = (points,fill,extra={}) => ({d:svgPath(points)+' Z',fill,...extra})
const line = (points,stroke,width=1,extra={}) => ({d:svgPath(points),fill:'none',stroke,width,...extra})
const rectangle = (x,y,w,h,z) => [[x,y,z],[x+w,y,z],[x+w,y+h,z],[x,y+h,z]]
const circle = (x,y,z,r) => Array.from({length:12},(_,i)=>[x+Math.cos(i*Math.PI/6)*r,y+Math.sin(i*Math.PI/6)*r,z])
const contains = (polygon,x,y) => {
  let inside=false
  for(let i=0,j=polygon.length-1;i<polygon.length;j=i++) {
    const [a,b]=polygon[i],[u,v]=polygon[j]
    if((b>y)!==(v>y) && x<(u-a)*(y-b)/(v-b)+a)inside=!inside
  }
  return inside
}

export function buildScene(data) {
  const samples=[...Object.entries(data.nodes).filter(([id])=>!id.startsWith('bridge') && !data.elevatedNodes?.includes(id)).map(([,p])=>p),...data.terrain.samples]
  const ground=(x,y)=> {
    if(y < -140 && y > -390)return 0
    if(y <= -390)return 8
    const platform=data.surfaces.find(a=>!a.elevated && contains(a.polygon,x,y))
    return platform?.elevation ?? terrainHeight(samples,x,y)
  }
  const terrain=[],objects=[]
  let serial=0
  const add=(position,kind,shapes,place)=>objects.push({key:serial++,depth:depth(position),kind,shapes,place})
  const light=[-.45,-.35,.82]
  for(let y=-800;y<1600;y+=50) for(let x=-1200;x<1700;x+=50) {
    const vertices=[[x,y],[x+50,y],[x+50,y+50],[x,y+50]].map(([a,b])=>[a,b,ground(a,b)])
    for(const ids of [[0,1,2],[0,2,3]]) {
      const p=ids.map(i=>vertices[i]),u=p[1].map((v,i)=>v-p[0][i]),v=p[2].map((v,i)=>v-p[0][i])
      const normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
      normal[0]*=3;normal[1]*=3
      const brightness=.7+.36*normal.reduce((n,v,i)=>n+v*light[i],0)/Math.hypot(...normal)
      const h=p.reduce((sum,p)=>sum+p[2],0)/3
      const hill=Math.max(0,Math.min(1,(h-22)/45))
      const base=[228,225,207].map((v,i)=>v+([153,174,135][i]-v)*hill)
      terrain.push({...face(p,tint(base,brightness)),height:h})
    }
  }
  // Water and quay share one shoreline; only the engineered bank has a vertical face
  const shore=data.terrain.water.slice(2).reverse().map(([x,y])=>[x,y,6])
  const bank=[]
  for(let i=1;i<shore.length;i++)bank.push(face([shore[i-1],shore[i],...[shore[i],shore[i-1]].map(([x,y])=>[x,y,0])],'#a5ac9b'))
  add([300,-133],'bank',bank)
  for(const area of data.surfaces) {
    const top=area.polygon.map(([x,y])=>[x,y,area.elevation])
    const colors={private:'#d8d4be',school:'#dfdcc6',service:'#cecbb9',park:'#b4c796',platform:'#e2d9bd',court:'#ded9bf'}
    const shapes=[]
    for(let i=0;i<top.length;i++) {
      const a=top[i],b=top[(i+1)%top.length]
      if(i<2)shapes.push(face([a,b,[...b.slice(0,2),b[2]-2],[...a.slice(0,2),a[2]-2]],'#b0b09e'))
    }
    shapes.push(face(top,colors[area.kind]))
    add(top[0],'surface',shapes,area.place)
    if(area.boundary) for(let i=1;i<area.boundary.length;i++) {
      const a=[...area.boundary[i-1],area.elevation],b=[...area.boundary[i],area.elevation]
      add(lerp(a,b,.5),'wall',[face([a,b,[b[0],b[1],b[2]+2.5],[a[0],a[1],a[2]+2.5]],'#9aab99')],area.place)
    }
    if(area.kind==='school') {
      const [x,y]=area.polygon[0],z=area.elevation+.2
      const track=rectangle(x+25,y+19,80,40,z)
      add([x+65,y+39],'detail',[face(track,'#caab91'),line([...track,track[0]],'#f5eee0',1),line([[x+65,y+19,z],[x+65,y+59,z]],'#f5eee0',1)])
    }
  }
  for(const road of data.roads) for(let i=1;i<road.nodes.length;i++) {
    const a=data.nodes[road.nodes[i-1]],b=data.nodes[road.nodes[i]]
    const length=Math.hypot(b[0]-a[0],b[1]-a[1])
    if(road.kind==='lift'){add(a,'lift',[line([a,b],'#8a9e93',5)]);continue}
    if(length===0)continue
    const width=({main:10,avenue:15,shore:7,bridge:16,deck:5,steps:6,trail:4,service:5,landing:9}[road.kind]??7)
    const n=Math.ceil(length/(road.kind==='steps'?3:20))
    const offset=[-(b[1]-a[1])/length*width/2,(b[0]-a[0])/length*width/2]
    for(let j=0;j<n;j++) {
      let u=lerp(a,b,j/n),v=lerp(a,b,(j+1)/n)
      const sides=p=>[-1,1].map(k=>[p[0]+offset[0]*k,p[1]+offset[1]*k,p[2]+.2])
      const [ul,ur]=sides(u),[vl,vr]=sides(v),shapes=[]
      if(road.kind==='bridge') {
        shapes.push(face([[ul[0]+6,ul[1]-7,0],[ur[0]+6,ur[1]-7,0],[vr[0]+6,vr[1]-7,0],[vl[0]+6,vl[1]-7,0]],'#416e70',{opacity:.22}))
        if(j%4===0)shapes.push(face([ul,ur,[ur[0],ur[1],0],[ul[0],ul[1],0]],'#a5ada6'))
        shapes.push(face([ul,vl,[vl[0],vl[1],vl[2]-2],[ul[0],ul[1],ul[2]-2]],'#9aa69e'))
      }
      shapes.push(face([ul,ur,vr,vl],road.kind==='service'?'#c7c4af':road.kind==='trail'?'#d9d0ae':'#f0e9d5',{stroke:'#b9baa5',width:.4}))
      if(road.kind==='steps'){
        const level=Math.max(u[2],v[2])+.2
        const tread=[ul,ur,vr,vl].map(p=>[p[0],p[1],level])
        shapes.push(face(tread,'#ece4cd'),line([tread[2],tread[3]],'#899785',.9))
      }
      if(road.kind==='bridge')shapes.push(line([ul,vl],'#f3f0df',1.4),line([ur,vr],'#f3f0df',1.4))
      add(lerp(u,v,.5),'road',shapes)
    }
  }
  for(const b of data.buildings) {
    const [[x,y],,[right,back]]=b.polygon,z=b.elevation,h=b.height,w=right-x,l=back-y
    const south=[[x,y,z],[right,y,z],[right,y,z+h],[x,y,z+h]]
    const east=[[right,y,z],[right,back,z],[right,back,z+h],[right,y,z+h]]
    const roof=({home:'#b67752',shop:'#9daea8',school:'#7e999e',station:'#91a6ac',shrine:'#8a7568'}[b.kind]??'#94a2a1')
    const shapes=[face(b.polygon.map(([x,y])=>[x+5,y-7,z]),'#5b6b5d',{opacity:.16}),face(south,'#eee6d2'),face(east,'#c4cbbb')]
    const top=z+h,ridge=top+3
    shapes.push(face([[x-1,y-1,top],[right+1,y-1,top],[right+1,y+l/2,ridge],[x-1,y+l/2,ridge]],roof,{stroke:'#75837e',width:.6}))
    shapes.push(face([[x-1,y+l/2,ridge],[right+1,y+l/2,ridge],[right+1,back+1,top],[x-1,back+1,top]],'#bac6bd',{stroke:'#75837e',width:.6}))
    for(let row=4;row<h-2;row+=6)for(let col=5;col<w-4;col+=9) shapes.push(face([[x+col,y-.1,z+row],[x+col+4,y-.1,z+row],[x+col+4,y-.1,z+row+3],[x+col,y-.1,z+row+3]],'#779395'))
    if(b.kind==='shop')shapes.push(face([[x,y-4,z+5],[right,y-4,z+5],[right,y,z+6],[x,y,z+6]],'#dbb88a'))
    if(b.kind==='home') {
      for(const t of [.15,.65]) {
        const yy=y+l*t
        shapes.push(face([[right+.1,yy,z+2],[right+.1,yy+l*.15,z+2],[right+.1,yy+l*.15,z+5],[right+.1,yy,z+5]],'#83aaa7'))
      }
      shapes.push(face([[right,y,z+6],[right+3,y,z+5],[right+3,back,z+5],[right,back,z+6]],'#d89460'))
      shapes.push(face([[right+.2,y+l*.4,z],[right+.2,y+l*.55,z],[right+.2,y+l*.55,z+4],[right+.2,y+l*.4,z+4]],'#526e6a'))
    }
    if(b.kind==='civic') {
      // Civic roofs are flat; the shared cinema roof carries its public garden
      shapes.splice(3,2,face(rectangle(x,y,w,l,top),b.place==='15'?'#a9bf91':'#b3bdb7',{stroke:'#75837e',width:.6}))
    }
    add([right,y],'building',shapes,b.place)
  }
  for(const [x,y] of data.trees) {
    const z=ground(x,y),shapes=[face(circle(x+4,y-5,z,8),'#4e6959',{opacity:.18}),line([[x,y,z],[x,y,z+12]],'#8c8c6d',2)]
    for(const [dx,dy,dz,r,color] of [[2,0,12,8,'#6e9274'],[-2,1,15,7,'#8aaa7d'],[-3,2,18,4,'#b0c399']])shapes.push(face(circle(x+dx,y+dy,z+dz,r),color))
    add([x,y],'tree',shapes)
  }
  // Benches and delivery crates give the shop's public and service spaces different uses
  add([61,220],'detail',[face(rectangle(51,218,15,3,17),'#a88e6c'),line([[51,221,17],[66,221,17]],'#7c8066',1.5)])
  add([20,278],'detail',[face(rectangle(14,272,7,6,22),'#b9a786'),face([[14,272,20],[21,272,20],[21,272,22],[14,272,22]],'#958971')],'28')
  objects.sort((a,b)=>a.depth-b.depth)
  return {terrain,surfaces:objects.filter(o=>o.kind==='surface'),water:face(data.terrain.water.map(p=>[...p,0]),'#86b9b9'),objects:objects.filter(o=>o.kind!=='surface')}
}
