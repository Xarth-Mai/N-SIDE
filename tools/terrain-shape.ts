import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import type { District, Point } from './district-types.ts'
import { pointInside, roadOffsets } from './district-geometry.ts'

const clamp=(x:number)=>Math.max(0,Math.min(1,x))
const smooth=(x:number)=>{const t=clamp(x);return t*t*(3-2*t)}
function noise(x:number,y:number,seed:number) {
  const ix=Math.floor(x),iy=Math.floor(y),u=smooth(x-ix),v=smooth(y-iy)
  const dot=(a:number,b:number)=>{
    let h=Math.imul(a^seed,374761393)^Math.imul(b,668265263)
    h=Math.imul(h^(h>>>13),1274126177)
    const angle=((h^(h>>>16))>>>0)/4294967296*Math.PI*2
    return Math.cos(angle)*(x-a)+Math.sin(angle)*(y-b)
  }
  const a=dot(ix,iy)*(1-u)+dot(ix+1,iy)*u,b=dot(ix,iy+1)*(1-u)+dot(ix+1,iy+1)*u
  return a*(1-v)+b*v
}

// Offline shape only: the Viewer and Wiki continue to consume baked district samples.
export function mountainBase(data:District,x:number,y:number) {
  const [px,py,top]=data.nodes.summit
  const angle=Math.atan2(x-px,py-y)
  const shoulder=1+.22*Math.sin(3*angle+.8)+.13*Math.cos(2*angle-.5)
  const r=Math.hypot((x-px)/650,(y-py)/(y<py?py-data.nodes.home[1]:1100))/shoulder
  const natural=28+(top-28)*Math.cos(clamp(r)*Math.PI/2)**2
  const urban=28+Math.max(0,y-data.nodes.home[1])*.42
  return Math.min(natural,urban)+(natural-Math.min(natural,urban))*smooth((y-600)/180)
}
const segmentPoint=(x:number,y:number,a:Point,b:Point)=>{
  const dx=b[0]-a[0],dy=b[1]-a[1],t=clamp(((x-a[0])*dx+(y-a[1])*dy)/(dx*dx+dy*dy||1))
  return a.map((n,i)=>n+(b[i]-n)*t)
}
export function bakeTerrain(data:District) {
  const result=structuredClone(data),recipe=result.terrain.bake
  if(!recipe||recipe.version!==1||!Number.isInteger(recipe.authored)||recipe.authored<3||recipe.authored>data.terrain.samples.length||!Number.isInteger(recipe.seed)||!Number.isFinite(recipe.spacing)||recipe.spacing<40||recipe.spacing>120)throw new Error('Invalid terrain.bake: version 1, authored sample prefix, integer seed and spacing 40..120 are required')
  const authored=result.terrain.samples.slice(0,recipe.authored)
  const ascent=data.routes.find(r=>r.id==='hill-short')?.nodes??[]
  const first=ascent.indexOf('hill_short_rest1_departure'),last=ascent.indexOf('hill_short_rest2_arrival')
  const refined=new Set(first>=0&&last>first?ascent.slice(first,last+1):[])
  const elevated=new Set(data.elevatedNodes),ids=new Set<string>(),segments:[Point,Point,Point,Point,boolean][]=[]
  for(const road of data.roads)if(!road.building&&!['bridge','deck','lift','interior'].includes(road.kind)&&!data.surfaces.some(s=>s.id===road.surface&&s.elevated)) {
    for(const id of road.nodes)if(!elevated.has(id))ids.add(id)
    const points=road.nodes.map(id=>data.nodes[id]),offsets=roadOffsets(points,road.width)
    for(let i=1;i<points.length;i++)segments.push([points[i-1],points[i],offsets[i-1],offsets[i],refined.has(road.nodes[i-1])&&refined.has(road.nodes[i])])
  }
  const nearby=segments.filter(s=>s[4])
  const nearAscent=(x:number,y:number)=>nearby.some(([a,b])=>{const p=segmentPoint(x,y,a,b);return Math.hypot(x-p[0],y-p[1])<=30})
  const controls=[...authored,...[...ids].map(id=>data.nodes[id])]
  const occupied=new Set(controls.map(p=>p.slice(0,2).join(',')))
  const generated:Point[]=[]
  const peak=data.nodes.summit
  const areas=[...data.buildings,...data.surfaces.filter(s=>!s.elevated)]
  // Sample full edges: corner-only controls let long Delaunay triangles cut through a terrace or street
  const addControl=(x:number,y:number,z:number)=>{
    x=Math.round(x*1e6)/1e6;y=Math.round(y*1e6)/1e6
    const key=`${x},${y}`
    if(y<270||y>1920||x< -700||x>1100||z>=peak[2]||occupied.has(key))return
    const point=[x,y,z];controls.push(point);generated.push(point);occupied.add(key)
  }
  for(const area of areas)for(let i=0;i<area.polygon.length;i++) {
    const a=area.polygon[i],b=area.polygon[(i+1)%area.polygon.length]
    const spacing=Math.min(a[1],b[1])<data.nodes.hillgate[1]||nearAscent(a[0],a[1])||nearAscent(b[0],b[1])?('height' in area?12:2):Infinity
    const count=Math.max(1,Math.ceil(Math.hypot(b[0]-a[0],b[1]-a[1])/spacing))
    for(let j=0;j<count;j++)addControl(a[0]+(b[0]-a[0])*j/count,a[1]+(b[1]-a[1])*j/count,area.elevation)
  }
  for(const [a,b,start,end,refine] of segments) {
    const dx=b[0]-a[0],dy=b[1]-a[1],length=Math.hypot(dx,dy)
    if(!length)continue
    const count=Math.max(1,Math.ceil(length/6))
    for(let j=0;j<=count;j++)for(const side of [-1,0,1]) {
      const t=j/count,x=a[0]+dx*t+(start[0]+(end[0]-start[0])*t)*side,y=a[1]+dy*t+(start[1]+(end[1]-start[1])*t)*side
      // The second short-ascent leg is also walked at eye level; center-only mountain controls left metre-high edge cuts
      if(y>data.nodes.hillgate[1]&&!refine)continue
      // Footprints own their ground; road overlaps are clipped to these same surfaces at rendering time
      const area=areas.find(area=>pointInside([x,y],area.polygon))
      addControl(x,y,area?.elevation??a[2]+(b[2]-a[2])*t)
    }
  }
  const residuals=controls.map(p=>({p,delta:p[2]-mountainBase(data,p[0],p[1])}))
  const sampleNatural=(x:number,y:number)=>{
    if(occupied.has(`${x},${y}`))return
    let nearest=Infinity,total=0,weight=0
    for(const {p,delta} of residuals) {
      const distance=Math.hypot(x-p[0],y-p[1]);nearest=Math.min(nearest,distance)
      if(distance<180){const w=(1-distance/180)**4;total+=w*delta;weight+=w}
    }
    let protectedHeight:number|undefined,protectedDistance=Infinity
    for(const [a,b] of segments) {
      const q=segmentPoint(x,y,a,b),distance=Math.hypot(x-q[0],y-q[1])
      if(distance<protectedDistance){protectedDistance=distance;protectedHeight=q[2]}
    }
    for(const area of areas) {
      const distance=pointInside([x,y],area.polygon)?0:Math.min(...area.polygon.map((a,i)=>{
        const q=segmentPoint(x,y,a,area.polygon[(i+1)%area.polygon.length]);return Math.hypot(x-q[0],y-q[1])
      }))
      if(distance<protectedDistance){protectedDistance=distance;protectedHeight=area.elevation}
    }
    nearest=Math.min(nearest,protectedDistance)
    const fade=smooth((y-255)/130)*smooth(nearest/55)*smooth(Math.hypot(x-peak[0],y-peak[1])/100)
    const warpX=70*noise(x/450,y/450,recipe.seed)*fade,warpY=55*noise(x/450+17,y/450-9,recipe.seed)*fade
    const base=mountainBase(data,x+warpX,y+warpY)
    const detail=9*noise(x/180,y/180,recipe.seed)+3*noise(x/75,y/75,recipe.seed+1)
    let height=base+total/(weight+.15)+detail*fade
    if(protectedHeight!==undefined)height=protectedHeight+(height-protectedHeight)*smooth(protectedDistance/45)
    height=Math.max(1,Math.min(peak[2]-.1,height))
    generated.push([x,y,Math.round(height*1000)/1000])
    occupied.add(`${x},${y}`)
  }
  for(let y=270;y<=1920;y+=recipe.spacing)for(let x=-700;x<=1100;x+=recipe.spacing)sampleNatural(x,y)
  // Refine only the visible cut beside this leg; share the same height function and preserve the distant mountain grid
  const ends=nearby.flatMap(([a,b])=>[a,b])
  if(ends.length) {
    const xs=ends.map(p=>p[0]),ys=ends.map(p=>p[1])
    for(let y=Math.floor((Math.min(...ys)-30)/12)*12;y<=Math.max(...ys)+30;y+=12)
      for(let x=Math.floor((Math.min(...xs)-30)/12)*12;x<=Math.max(...xs)+30;x+=12)
        if(nearAscent(x,y))sampleNatural(x,y)
  }
  result.terrain.samples=[...authored,...generated]
  return result
}
export function replaceTerrain(text:string,terrain:District['terrain']) {
  if((text.match(/^  "terrain": .*,$/gm)??[]).length!==1)throw new Error('Expected exactly one compact terrain line; source format was not rewritten')
  return text.replace(/^  "terrain": .*,$/m,'  "terrain": '+JSON.stringify(terrain)+',')
}

if(import.meta.main) {
  const args=process.argv.slice(2)
  if(args.some(a=>a!=='--check'))throw new Error('Usage: bun tools/terrain-shape.ts [--check]')
  const path=fileURLToPath(new URL('../source-assets/district-map/district.json',import.meta.url)),text=readFileSync(path,'utf8'),source=JSON.parse(text) as District
  const baked=bakeTerrain(source)
  const updated=replaceTerrain(text,baked.terrain)
  if(updated===text&&!args.includes('--check'))console.log('Terrain is current')
  else if(args.includes('--check')) {
    if(updated!==text)throw new Error('Baked terrain is stale; run bun tools/terrain-shape.ts')
    console.log(`PASS terrain bake: ${baked.terrain.samples.length} samples`)
  } else {writeFileSync(path,updated);console.log(`Baked ${baked.terrain.samples.length} terrain samples`)}
}
