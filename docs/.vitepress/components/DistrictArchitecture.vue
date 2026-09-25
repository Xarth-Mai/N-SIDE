<script setup>
import data from '../../../source-assets/district-map/district.json'
import { inside, polygonArea, routeProfile } from '../../../tools/district-plan.mjs'
import { roadWidth } from '../../../tools/district-geometry.mjs'
import { architectureStats, sectionIntervals, sectionRoads, streetPosition } from '../../../tools/district-architecture.mjs'

const props=defineProps({section:{type:String,default:'P3-A1'}})
const architecture=data.architectures.find(a=>a.id===props.section)
const buildings=data.buildings.filter(b=>architecture?.buildings.includes(b.id)),byId=Object.fromEntries(data.buildings.map(b=>[b.id,b]))
const points=polygon=>polygon.map(([x,y])=>`${x},${-y}`).join(' ')
const path=polygon=>polygon.map(([x,y],i)=>`${i?'L':'M'} ${x} ${-y}`).join(' ')+' Z'
const footprint=b=>path(b.polygon)+(b.design?.lightwell?' '+path(b.design.lightwell):'')
const center=polygon=>polygon.reduce((sum,p)=>[sum[0]+p[0]/polygon.length,sum[1]+p[1]/polygon.length],[0,0])
const box=(polygon,pad=0)=> {
  const x=Math.min(...polygon.map(p=>p[0]))-pad,y=-Math.max(...polygon.map(p=>p[1]))-pad
  return {x,y,width:Math.max(...polygon.map(p=>p[0]))-x+pad,height:-Math.min(...polygon.map(p=>p[1]))-y+pad}
}
const viewBox=b=>`${b.x} ${b.y} ${b.width} ${b.height}`
const frame=architecture?box(architecture.boundary,18):{x:0,y:0,width:100,height:100}
const onMap=polygon=>polygon.some(p=>inside(p,architecture.boundary))||architecture.boundary.some(p=>inside(p,polygon))
const roads=architecture?data.roads.filter(r=>r.kind!=='interior'&&r.nodes.some(id=>inside(data.nodes[id],architecture.boundary))):[]
const fixtures=architecture?.fixtures??[]
const fixtureColor={bench:'#a88863',locker:'#738f96',screen:'#647c6d',drain:'#8d938a'}
const surfaces=architecture?data.surfaces.filter(s=>onMap(s.polygon)):[]
const parcels=architecture?data.parcels.filter(p=>onMap(p.polygon)):[]
const trees=architecture?data.trees.filter(p=>inside(p,architecture.boundary)):[]
const contextBuildings=architecture?data.buildings.filter(b=>!architecture.buildings.includes(b.id)&&onMap(b.polygon)):[]
const width=roadWidth
const doorLeaf=([a,b])=>{const dx=b[0]-a[0],dy=b[1]-a[1],length=Math.hypot(dx,dy);return `M ${a[0]} ${-a[1]} l ${-dy} ${-dx} M ${b[0]} ${-b[1]} A ${length} ${length} 0 0 0 ${a[0]-dy} ${-a[1]-dx}`}
const doorLine=(b,p)=>{
  const edges=b.polygon.map((a,i)=>{const c=b.polygon[(i+1)%b.polygon.length],dx=c[0]-a[0],dy=c[1]-a[1],length=Math.hypot(dx,dy),t=Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/length**2));return {dx,dy,length,distance:Math.hypot(p[0]-a[0]-dx*t,p[1]-a[1]-dy*t)}}).sort((a,b)=>a.distance-b.distance)
  const edge=edges[0];return [[p[0]-.6*edge.dx/edge.length,p[1]-.6*edge.dy/edge.length],[p[0]+.6*edge.dx/edge.length,p[1]+.6*edge.dy/edge.length]]
}
const colors={shop:'#e5cfad',home:'#c5d2ca',service:'#d5cbd7',core:'#bac9d2',court:'#e8efda'}
const buildingColor=b=>b.place==='04'?'#cba883':({slope:'#bacfc0',mixed:'#c3cdd0',interest:'#dfcaaa',maker:'#c0cbbc'}[b.design?.type]??'#dad5c4')
const roleName={public:'客',resident:'住',service:'货'}
const roleLabel={public:'顾客入口',resident:'住户入口',service:'服务入口'}
const roleColor={public:'#276f81',resident:'#627a4a',service:'#a3694e'}
const surfaceColor=s=>({park:'#d8e2c7',private:'#e7dfcb',service:'#ded3c9',court:'#e6e4d8',platform:'#e1dac5'}[s.kind]??'#e6e4d8')
const entrances=b=>(b.design?.entries??[]).map(e=>({...e,point:data.nodes[e.node]})).filter(e=>e.point)
function canopy(b) {
  const front=b.design?.front
  if(!front||!b.design.canopy)return []
  const [a,c]=front,dx=c[0]-a[0],dy=c[1]-a[1],length=Math.hypot(dx,dy),mid=center(front),step=[-dy/length*b.design.canopy,dx/length*b.design.canopy]
  const sign=inside([mid[0]+step[0],mid[1]+step[1]],b.polygon)?-1:1
  return [a,c,[c[0]+step[0]*sign,c[1]+step[1]*sign],[a[0]+step[0]*sign,a[1]+step[1]*sign]]
}
const modes=[{id:'before',name:'深化前 · 建筑体量'},{id:'after',name:'深化后 · 建筑与入口'},{id:'plain',name:'完整平面 · 无编号'},{id:'scenes',name:'生活场景与视线'}]
const samples=buildings.filter(b=>b.design.floors.some(f=>f.rooms)).sort((a,b)=>Number(b.id==='V-04')-Number(a.id==='V-04')).map(b=>{const type=architecture.types.find(t=>t.id===b.design.type);return {...type,id:b.id,name:b.id==='V-04'?'小店与楼上住家':type.name,building:b,frame:box(b.polygon,3)}})
const street=architecture?.street.nodes.map(id=>data.nodes[id])??[]
const profile=architecture?routeProfile(data.nodes,architecture.street.nodes):[]
const streetLength=profile.at(-1)?.distance??0
const stairRuns=profile.slice(1).flatMap((b,i)=>data.roads.some(r=>r.kind==='steps'&&r.nodes.some((id,j)=>j&&((id===b.id&&r.nodes[j-1]===profile[i].id)||(id===profile[i].id&&r.nodes[j-1]===b.id))))?[{a:profile[i],b}]:[])
const stairHatch=room=>{const b=box(room.polygon),w=Math.min(1.5,b.width-.6),h=Math.min(3,b.height-.6);return Array.from({length:7},(_,i)=>`M ${b.x+.3} ${b.y+.3+i*h/6} h ${w}`).join(' ')}
const roadName={main:'生活主街',avenue:'城市道路',lane:'公共支路',service:'服务通道',steps:'公共台阶',deck:'公共平台',trail:'步道',landing:'平台路',shore:'滨水路',crossing:'过街'}
const frontages=(architecture?.street.facades??[]).map(facade=>{
  const route=facade.line.map(p=>[...p,0]),length=Math.hypot(facade.line[1][0]-facade.line[0][0],facade.line[1][1]-facade.line[0][1])
  const items=facade.buildings.map(id=>{const b=byId[id],stations=b.polygon.map(p=>streetPosition(p,route).station);return {...b,start:Math.min(...stations),end:Math.max(...stations)}})
  const floor=Math.floor(Math.min(...items.map(b=>b.elevation))/5)*5,ceiling=Math.ceil(Math.max(...items.map(b=>b.elevation+b.height))/5)*5
  const ground=items.toSorted((a,b)=>a.start-b.start).flatMap(b=>[[b.start,facade.upper?b.design.floors[1].z:b.elevation],[b.end,facade.upper?b.design.floors[1].z:b.elevation]])
  return {...facade,route,length,buildings:items,ground,floor,frame:{x:-4,y:-ceiling-5,width:length+8,height:ceiling-floor+12}}
})
function facadeEntries(b,facade){
  return entrances(b).filter(e=>{
    const edge=doorLine(b,e.point),dx=edge[1][0]-edge[0][0],dy=edge[1][1]-edge[0][1],normal=[-dy,dx],mid=e.point
    const sign=inside([mid[0]+normal[0]*.1,mid[1]+normal[1]*.1],b.polygon)?-1:1
    const station=streetPosition(mid,facade.route).station,t=station/facade.length,view=facade.line[0].map((v,i)=>v+(facade.line[1][i]-v)*t)
    return ((view[0]-mid[0])*normal[0]+(view[1]-mid[1])*normal[1])*sign>.01
  })
}
const bottom=Math.floor(Math.min(...buildings.map(b=>b.elevation),...street.map(p=>p[2]),0)/5)*5
const top=Math.ceil(Math.max(...buildings.map(b=>b.elevation+b.height),40)/5)*5
const elevationFrame={x:-12,y:-top-12,width:streetLength+24,height:top-bottom+30}
const sections=architecture?.sections.map(section=> {
  const length=Math.hypot(section.line[1][0]-section.line[0][0],section.line[1][1]-section.line[0][1]),route=section.line.map(p=>[...p,0])
  const cuts=buildings.flatMap(b=>sectionIntervals(section.line,b.polygon).map(([start,end])=>({...b,start,end})))
  const floor=Math.min(...section.ground.map(p=>p[2]),...cuts.map(b=>b.elevation))-3,ceiling=Math.max(...section.ground.map(p=>p[2]),...cuts.map(b=>b.elevation+b.height))+5
  return {...section,length,cuts,roads:sectionRoads(data,section),ground:section.ground.map(p=>[streetPosition(p,route).station,p[2]]),frame:{x:-3,y:-ceiling,width:length+6,height:ceiling-floor+10}}
})??[]
const stats=architecture?architectureStats(data,architecture):[]
const roomLabel=room=>room.label??center(room.polygon)
const roomLines=room=>{const letters=[...room.name],width=box(room.polygon).width,count=Math.max(2,Math.floor((width-.5)/.8));return Array.from({length:Math.ceil(letters.length/count)},(_,i)=>letters.slice(i*count,(i+1)*count).join(''))}
const roomCuts=(section,b,floor)=>(floor.rooms??[]).flatMap(room=>sectionIntervals(section.line,room.polygon).map(([start,end])=>({...room,start,end})))
const floorHeight=(b,i)=>((b.design?.floors[i+1]?.z)??(b.elevation+b.height))-b.design.floors[i].z
</script>

<template>
  <section v-if="architecture" class="district-architecture" :aria-label="`${architecture.name}建筑图纸`">
    <p class="drawing-note">功能分区与楼梯占位 · 同一套米坐标与相对高程 · 北向上 · 宽图可横向滚动 · 平面、立面与剖面按标尺读取</p>
    <figure v-for="mode in modes" :key="mode.id">
      <figcaption>{{ mode.name }}<span>{{ mode.id==='before'?'原体量 / 当前道路基准 · '+architecture.baseline.commit:'同范围、同尺度' }} · 20 m 标尺</span></figcaption>
      <div class="drawing-scroll" tabindex="0" role="region" :aria-label="`${mode.name}，可横向滚动`">
        <svg :viewBox="viewBox(frame)" :style="{width:`${frame.width*2.5}px`}" role="img" :aria-label="mode.name">
          <title>{{ mode.name }}</title><desc>街道两侧、建筑背面、院落、公共平台与住宅，采用北向上的正投影平面</desc>
          <rect :x="frame.x" :y="frame.y" :width="frame.width" :height="frame.height" fill="#f8f6ee"/>
          <polygon :points="points(architecture.boundary)" fill="#f0eee2" stroke="#98a58e" stroke-width=".6" stroke-dasharray="3 2"/>
          <polygon v-for="parcel in parcels" :key="parcel.id" :points="points(parcel.polygon)" fill="none" stroke="#d2cbbb" stroke-width=".4"/>
          <polygon v-for="surface in surfaces" :key="surface.id??surface.place" :points="points(surface.polygon)" :fill="surfaceColor(surface)" stroke="#b5b59f" stroke-width=".3"/>
          <g v-for="(road,i) in roads" :key="`r-${i}`" fill="none" stroke-linejoin="round" stroke-linecap="round">
            <polyline :points="points(road.nodes.map(id=>data.nodes[id]))" stroke="#acaa99" :stroke-width="width(road)+.8"/>
            <polyline :points="points(road.nodes.map(id=>data.nodes[id]))" :stroke="road.access==='service'?'#dfcdbc':road.access==='resident'?'#dce5ce':'#fffdf7'" :stroke-width="width(road)" :stroke-dasharray="road.kind==='steps'?'1 1':undefined"/>
          </g>
          <path v-for="building in contextBuildings" :key="building.id" :d="footprint(building)" fill="#e3e0d5" stroke="#aaa99a" stroke-width=".4"/>
          <g v-for="building in (mode.id==='before'?architecture.baseline.buildings:buildings)" :key="building.id">
            <path :d="footprint(building)" fill-rule="evenodd" :fill="mode.id==='before'?'#d6d3c6':buildingColor(building)" stroke="#655f51" stroke-width=".55"/>
            <template v-if="mode.id!=='before'">
              <polygon v-if="canopy(building).length" :points="points(canopy(building))" fill="#c08b62" fill-opacity=".65" stroke="#876b4f" stroke-width=".3"/>
              <polyline v-if="building.design?.front" :points="points(building.design.front)" fill="none" stroke="#5e9298" stroke-width="1"/>
              <g v-for="entry in entrances(building)" :key="entry.node" :transform="`translate(${entry.point[0]} ${-entry.point[1]})`">
                <circle r="1.8" :fill="roleColor[entry.role]" stroke="#fffdf7" stroke-width=".4"/>
              </g>
            </template>
            <text v-if="mode.id==='before'||mode.id==='after'" :x="center(building.polygon)[0]" :y="-center(building.polygon)[1]" text-anchor="middle" font-size="4" class="halo">{{ building.id }}<tspan v-if="building.design" :x="center(building.polygon)[0]" dy="5">{{ building.design.floors.length }} 层 · +{{ building.elevation }}</tspan></text>
          </g>
          <polygon v-for="(fixture,i) in fixtures" :key="`fixture-${i}`" :points="points(fixture.polygon)" :fill="fixtureColor[fixture.kind]" stroke="#555e4d" stroke-width=".2"><title>{{ fixture.name }} · +{{ fixture.elevation }} m</title></polygon>
          <g v-for="(tree,i) in trees" :key="`t-${i}`"><circle :cx="tree[0]" :cy="-tree[1]" r="4" fill="#b4c995" fill-opacity=".85" stroke="#73906e" stroke-width=".5"/><circle :cx="tree[0]" :cy="-tree[1]" r=".6" fill="#827054"/></g>
          <template v-if="mode.id==='after'">
            <g v-for="section in sections" :key="section.id"><polyline :points="points(section.line)" fill="none" stroke="#945f50" stroke-width=".5" stroke-dasharray="3 2"/><text :x="section.line[0][0]" :y="-section.line[0][1]-3" font-size="5" class="halo">{{ section.id }}</text></g>
            <text v-for="(surface,i) in surfaces.filter(s=>s.name)" :key="`n-${surface.id}`" :x="center(surface.polygon)[0]" :y="-center(surface.polygon)[1]" text-anchor="middle" font-size="4" class="halo">S{{ i+1 }}</text>
          </template>
          <g v-if="mode.id==='scenes'">
            <g v-for="scene in architecture.scenes" :key="scene.id">
              <template v-if="scene.towards"><line :x1="scene.position[0]" :y1="-scene.position[1]" :x2="scene.towards[0]" :y2="-scene.towards[1]" stroke="#ae714b" stroke-width="1" stroke-dasharray="3 2"/><circle :cx="scene.towards[0]" :cy="-scene.towards[1]" r="2" fill="none" stroke="#ae714b" stroke-width=".8"/></template>
              <circle :cx="scene.position[0]" :cy="-scene.position[1]" r="4" fill="#975f40" stroke="#fffdf7" stroke-width=".7"/><text :x="scene.position[0]" :y="-scene.position[1]+1.7" text-anchor="middle" font-size="4.5" fill="white">{{ scene.id }}</text>
            </g>
          </g>
          <g :transform="`translate(${frame.x+10} ${frame.y+15})`"><path d="M 0 10 V 0 L -2 4 M 0 0 L 2 4" fill="none" stroke="#506452" stroke-width=".7"/><text x="0" y="-3" font-size="5" text-anchor="middle">N</text></g>
          <g :transform="`translate(${frame.x+10} ${frame.y+frame.height-10})`"><path d="M 0 -3 V 0 H 20 V -3" fill="none" stroke="#506452" stroke-width=".7"/><text x="10" y="6" text-anchor="middle" font-size="5">20 m</text></g>
        </svg>
      </div>
    </figure>
    <p class="legend"><span>蓝绿边 · 临街门窗</span><span>赭色带 · 雨棚</span><span>蓝点 · 顾客入口</span><span>绿点 / 绿路 · 住户到达</span><span>棕点 / 棕路 · 服务到达</span><span>虚线 · 地块 / 剖切 / 视线方向</span></p>
    <details><summary>场地与高程索引</summary><table><thead><tr><th>图示</th><th>场地</th><th>高程 / m</th></tr></thead><tbody><tr v-for="(surface,i) in surfaces.filter(s=>s.name)" :key="surface.id"><td>S{{ i+1 }}</td><td>{{ surface.name }}</td><td>{{ surface.elevation }}</td></tr></tbody></table></details>

    <h3>建筑层平面</h3>
    <section v-for="sample in samples" :key="sample.id" class="sample">
      <h4>{{ sample.name }} · {{ sample.building.id }}</h4><p>{{ sample.notes }}</p>
      <div class="floor-strip">
        <figure v-for="floor in sample.building.design.floors.filter(f=>f.rooms)" :key="floor.name">
          <figcaption>{{ floor.name }} · +{{ floor.z }} m<span>{{ floor.use }}</span></figcaption>
          <div class="drawing-scroll" tabindex="0" role="region" :aria-label="`${sample.name}${floor.name}平面`">
            <svg :viewBox="viewBox(sample.frame)" :style="{width:`${Math.max(330,sample.frame.width*12)}px`}" role="img" :aria-label="`${sample.name}${floor.name}平面`">
              <rect :x="sample.frame.x" :y="sample.frame.y" :width="sample.frame.width" :height="sample.frame.height" fill="#faf8f0"/>
              <path :d="footprint(sample.building)" fill="#fffef9" fill-rule="evenodd" stroke="#514f46" stroke-width=".22"/>
              <g v-for="room in floor.rooms" :key="room.name"><polygon :points="points(room.polygon)" :fill="room.kind==='court'?'#faf8f0':colors[room.kind]" :stroke-dasharray="room.kind==='court'?'.3 .2':undefined" stroke="#726e61" stroke-width=".12"/><path v-if="room.kind==='core'&amp;&amp;room.name.includes('梯')" :d="stairHatch(room)" stroke="#526979" stroke-width=".1" fill="none" opacity=".45"/><text :x="roomLabel(room)[0]" :y="-roomLabel(room)[1]-.4*(roomLines(room).length-1)" text-anchor="middle" font-size=".8"><tspan v-for="(part,i) in roomLines(room)" :key="i" :x="roomLabel(room)[0]" :dy="i?1.1:0">{{ part }}</tspan><tspan v-if="room.kind==='court'" :x="roomLabel(room)[0]" dy="1.1" font-size=".65">透空</tspan></text></g>
              <g v-for="(opening,i) in floor.openings??[]" :key="`opening-${i}`"><polyline :points="points(opening.line)" fill="none" stroke="#faf8f0" stroke-width=".35"/><path v-if="opening.kind==='door'" :d="doorLeaf(opening.line)" fill="none" stroke="#8d8572" stroke-width=".07"/></g><polyline :points="points(sample.building.design.front)" fill="none" stroke="#598d95" stroke-width=".3"/>
              <polygon v-if="canopy(sample.building).length" :points="points(canopy(sample.building))" fill="#c89f74" fill-opacity=".5" stroke="#ab845e" stroke-width=".1"/>
              <polyline v-for="entry in entrances(sample.building).filter(e=>e.level===floor.name)" :key="`gap-${entry.node}`" :points="points(doorLine(sample.building,entry.point))" fill="none" stroke="#faf8f0" stroke-width=".5"/><g v-for="entry in entrances(sample.building).filter(e=>e.level===floor.name)" :key="entry.node" :transform="`translate(${entry.point[0]} ${-entry.point[1]})`"><circle r=".55" :fill="roleColor[entry.role]" stroke="#fff" stroke-width=".1"/><text y=".3" text-anchor="middle" fill="white" font-size=".75">{{ roleName[entry.role] }}</text></g>
              <g :transform="`translate(${sample.frame.x+1} ${sample.frame.y+sample.frame.height-1.2})`"><path d="M 0 -.3 V 0 H 5 V -.3" fill="none" stroke="#506452" stroke-width=".1"/><text x="2.5" y=".9" text-anchor="middle" font-size=".7">5 m · 北 ↑</text></g>
            </svg>
          </div>
        </figure>
      </div>
      <p class="drawing-note">{{ entrances(sample.building).map(e=>`${roleLabel[e.role]} ${e.level} / +${e.point[2]} m`).join(' · ') }}</p><p class="drawing-note">{{ sample.building.design.floors.map(f=>`${f.name} ${f.use}`).join('；') }}</p>
      <p class="drawing-note">占地 {{ Math.round(polygonArea(sample.building.polygon)-(sample.building.design.lightwell?polygonArea(sample.building.design.lightwell):0)) }} m² · {{ sample.reference.join('、') }}</p>
    </section>

    <h3>两侧立面与道路纵剖面</h3>
    <p class="drawing-note">按真实街面分段正投影 · 每段自 0 m 起算 · 水平与竖直等比例</p>
    <figure v-for="facade in frontages" :key="facade.name">
      <figcaption>{{ facade.name }}<span>m / 相对高程</span></figcaption>
      <div class="drawing-scroll" tabindex="0" role="region" :aria-label="facade.name">
        <svg :viewBox="viewBox(facade.frame)" :style="{width:`${facade.frame.width*6}px`}" role="img" :aria-label="facade.name">
          <rect :x="facade.frame.x" :y="facade.frame.y" :width="facade.frame.width" :height="facade.frame.height" fill="#faf8f0"/>
          <g v-for="building in facade.buildings" :key="building.id">
            <rect :x="building.start" :y="-building.elevation-building.height" :width="building.end-building.start" :height="building.height" :fill="buildingColor(building)" stroke="#605f53" stroke-width=".3"/>
            <g v-for="(floor,i) in building.design.floors" :key="floor.name">
              <line :x1="building.start" :y1="-floor.z" :x2="building.end" :y2="-floor.z" stroke="#746b58" stroke-width=".2"/>
              <rect v-for="j in Math.max(1,Math.floor((building.end-building.start)/4))" :key="j" :x="building.start+(j-.5)*(building.end-building.start)/Math.max(1,Math.floor((building.end-building.start)/4))-1" :y="-floor.z-Math.min(floorHeight(building,i)-.8,2.6)" width="2" :height="Math.min(1.8,floorHeight(building,i)-1.2)" fill="#7f9fa2" stroke="#eff2e8" stroke-width=".15"/>
            </g>
            <rect v-if="building.design.canopy" :x="building.start" :y="-building.elevation-3.2" :width="building.end-building.start" height=".45" fill="#b98960"/>
            <g v-for="entry in facadeEntries(building,facade)" :key="entry.node"><rect :x="streetPosition(entry.point,facade.route).station-.6" :y="-entry.point[2]-2.3" width="1.2" height="2.3" :fill="roleColor[entry.role]"/><text :x="streetPosition(entry.point,facade.route).station" :y="-entry.point[2]-3.1" text-anchor="middle" font-size="2" class="door-label">{{ roleName[entry.role] }}</text></g>
            <text :x="(building.start+building.end)/2" :y="-building.elevation-building.height-2" text-anchor="middle" font-size="2.5">{{ building.id }}</text>
          </g>
          <polygon :points="`${facade.ground.map(([x,z])=>`${x},${-z}`).join(' ')} ${facade.ground.at(-1)[0]},${-facade.floor+1} ${facade.ground[0][0]},${-facade.floor+1}`" fill="#e1e5d3"/><polyline :points="facade.ground.map(([x,z])=>`${x},${-z}`).join(' ')" fill="none" stroke="#6c775d" stroke-width=".25"/>
          <g v-for="mark in Math.floor(facade.length/10)+1" :key="mark"><line :x1="(mark-1)*10" :y1="-facade.floor+2" :x2="(mark-1)*10" :y2="-facade.floor+3" stroke="#737565" stroke-width=".2"/><text :x="(mark-1)*10" :y="-facade.floor+5.5" text-anchor="middle" font-size="2">{{ (mark-1)*10 }} m</text></g>
        </svg>
      </div>
    </figure>
    <figure>
      <figcaption>{{ architecture.name }}街道纵剖面<span>水平 / 竖直 1:1 · 高程沿节点插值</span></figcaption>
      <div class="drawing-scroll" tabindex="0" role="region" :aria-label="`${architecture.name}街道纵剖面`">
        <svg :viewBox="viewBox(elevationFrame)" :style="{width:`${elevationFrame.width*4}px`}" role="img" :aria-label="`${architecture.name}街道等比例纵剖面`">
          <rect :x="elevationFrame.x" :y="elevationFrame.y" :width="elevationFrame.width" :height="elevationFrame.height" fill="#faf8f0"/>
          <polygon :points="`${profile.map(p=>`${p.distance},${-p.point[2]}`).join(' ')} ${streetLength},${-bottom} 0,${-bottom}`" fill="#e1e5d3"/>
          <polyline :points="profile.map(p=>`${p.distance},${-p.point[2]}`).join(' ')" fill="none" stroke="#64775d" stroke-width=".7"/>
          <g v-for="(run,i) in stairRuns" :key="`stairs-${i}`"><line v-for="j in Math.max(2,Math.ceil(run.b.length/3))" :key="j" :x1="run.a.distance+(run.b.distance-run.a.distance)*j/Math.max(2,Math.ceil(run.b.length/3))" :x2="run.a.distance+(run.b.distance-run.a.distance)*j/Math.max(2,Math.ceil(run.b.length/3))" :y1="-run.a.point[2]-(run.b.point[2]-run.a.point[2])*j/Math.max(2,Math.ceil(run.b.length/3))-.6" :y2="-run.a.point[2]-(run.b.point[2]-run.a.point[2])*j/Math.max(2,Math.ceil(run.b.length/3))+.6" stroke="#927356" stroke-width=".4"/><text :x="(run.a.distance+run.b.distance)/2" :y="-(run.a.point[2]+run.b.point[2])/2+5" font-size="3" text-anchor="middle">台阶路段</text></g>
          <g v-for="(point,i) in profile" :key="point.id"><circle :cx="point.distance" :cy="-point.point[2]" r=".7" fill="#996c4e"/><text :x="point.distance" :y="-point.point[2]-4-(i%2)*6" text-anchor="middle" font-size="2.5">{{ point.point[2] }} m<tspan :x="point.distance" dy="3">{{ Math.round(point.distance) }} m</tspan></text></g>
          <g :transform="`translate(0 ${-bottom+5})`"><path d="M 0 -1 V 0 H 20 V -1" stroke="#61715d" fill="none" stroke-width=".3"/><text x="10" y="4" text-anchor="middle" font-size="3">20 m</text></g>
        </svg>
      </div>
    </figure>

    <h3>建筑与场地横剖面</h3>
    <figure v-for="section in sections" :key="section.id">
      <figcaption>{{ section.id }} · {{ section.name }}<span>水平 / 竖直 1:1</span></figcaption>
      <div class="drawing-scroll" tabindex="0" role="region" :aria-label="section.name">
        <svg :viewBox="viewBox(section.frame)" :style="{width:`${section.frame.width*10}px`}" role="img" :aria-label="section.name">
          <rect :x="section.frame.x" :y="section.frame.y" :width="section.frame.width" :height="section.frame.height" fill="#faf8f0"/>
          <polygon :points="`${section.ground.map(([x,z])=>`${x},${-z}`).join(' ')} ${section.length},${section.frame.y+section.frame.height} 0,${section.frame.y+section.frame.height}`" fill="#e4e4d3"/>
          <polyline :points="section.ground.map(([x,z])=>`${x},${-z}`).join(' ')" fill="none" stroke="#788064" stroke-width=".25"/>
          <g v-for="(building,j) in section.cuts" :key="`${building.id}-${j}`">
            <rect :x="building.start" :y="-building.elevation-building.height" :width="building.end-building.start" :height="building.height" fill="#f5eee0" stroke="#58594d" stroke-width=".18"/>
            <g v-for="(floor,i) in building.design.floors" :key="floor.name">
              <rect v-for="room in roomCuts(section,building,floor)" :key="room.name" :x="room.start" :y="-floor.z-floorHeight(building,i)" :width="room.end-room.start" :height="floorHeight(building,i)" :fill="colors[room.kind]" stroke="#a3a38f" stroke-width=".08"/>
              <line :x1="building.start" :y1="-floor.z" :x2="building.end" :y2="-floor.z" stroke="#5f6154" stroke-width=".22"/>
              <text :x="building.start+.4" :y="-floor.z-.5" font-size=".9">{{ floor.name }} +{{ floor.z }}</text>
            </g>
            <rect v-for="(voidCut,i) in building.design.lightwell?sectionIntervals(section.line,building.design.lightwell):[]" :key="`v-${i}`" :x="voidCut[0]" :y="-building.elevation-building.height-.2" :width="voidCut[1]-voidCut[0]" :height="building.height+.2" fill="#faf8f0"/>
            <text :x="(building.start+building.end)/2" :y="-building.elevation-building.height-1" text-anchor="middle" font-size="1.1">{{ building.id }}</text>
          </g>
          <g v-for="(point,i) in section.ground" :key="i"><circle :cx="point[0]" :cy="-point[1]" r=".2" fill="#8f674e"/><text :x="point[0]" :y="-point[1]+1.5+(i%2)*1.5" text-anchor="middle" font-size=".9">+{{ point[1] }}</text></g>
          <g v-for="(road,i) in section.roads" :key="`road-${i}`"><line :x1="road.span[0]" :y1="-road.elevation" :x2="road.span[1]" :y2="-road.elevation" :stroke="road.kind==='steps'?'#a67750':'#798b87'" stroke-width=".4"/><line :x1="road.station" :y1="-road.elevation+.4" :x2="road.station" :y2="section.frame.y+section.frame.height-5-(i%2)*3" stroke="#9f9985" stroke-width=".07" stroke-dasharray=".3 .3"/><text :x="road.station" :y="section.frame.y+section.frame.height-5-(i%2)*3" text-anchor="middle" font-size=".9">{{ road.access==='service'?'服务通道':roadName[road.kind]??road.kind }} {{ road.width }} m<tspan :x="road.station" dy="1.2">+{{ road.elevation.toFixed(2) }} m</tspan></text></g>
          <g :transform="`translate(0 ${section.frame.y+section.frame.height-1.5})`"><path d="M 0 -.3 V 0 H 5 V -.3" stroke="#61715d" fill="none" stroke-width=".1"/><text x="2.5" y="1" text-anchor="middle" font-size=".8">5 m</text></g>
        </svg>
      </div>
    </figure>
    <h3>场景位置与通行</h3>
    <div class="drawing-scroll" tabindex="0" role="region" aria-label="生活场景表"><table><thead><tr><th>场景</th><th>使用者与动作</th><th>可见变化</th><th>通行关系</th></tr></thead><tbody><tr v-for="scene in architecture.scenes" :key="scene.id"><th>{{ scene.id }} · {{ scene.label }}<small>{{ scene.theme }} / +{{ scene.position[2] }} m</small></th><td>{{ scene.who }} · {{ scene.action }}</td><td>{{ scene.change }}</td><td>{{ scene.passage }}</td></tr></tbody></table></div>
    <h3>类型与面积</h3>
    <div class="drawing-scroll" tabindex="0" role="region" aria-label="建筑类型与面积统计"><table><thead><tr><th>类型</th><th>数量</th><th>占地 / m²</th><th>楼面 / m²</th><th>图纸样本</th><th>依据</th></tr></thead><tbody><tr v-for="type in stats" :key="type.id"><th>{{ type.name }}</th><td>{{ type.count }}</td><td>{{ Math.round(type.footprint) }}</td><td>{{ Math.round(type.floorArea) }}</td><td>{{ type.sample }}</td><td>{{ type.reference.join('、') }}</td></tr></tbody></table></div>
    <p class="drawing-note">占地与楼面扣除天井，楼面按各层轮廓初算 · 具体墙厚、退台和结构进一步核对</p>
    <details><summary>逐栋建筑与楼层用途</summary><div class="drawing-scroll" tabindex="0" role="region" aria-label="样段建筑表"><table><thead><tr><th>建筑 / 地块</th><th>类型</th><th>各层用途</th><th>状态</th></tr></thead><tbody><tr v-for="building in buildings" :key="building.id"><th>{{ building.id }}<small>{{ building.parcel }}</small></th><td>{{ architecture.types.find(t=>t.id===building.design.type).name }}</td><td><div v-for="floor in building.design.floors" :key="floor.name">{{ floor.name }} · +{{ floor.z }} m · {{ floor.use }}</div></td><td>建筑方案</td></tr></tbody></table></div></details>
  </section>
</template>

<style scoped>
.district-architecture{color:#405348;margin:24px 0}.drawing-note{font-size:13px;color:var(--vp-c-text-2);line-height:1.7}figure{margin:24px 0}figcaption{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;font-size:14px;font-weight:600;margin-bottom:8px}figcaption span{font-weight:400;font-size:12px;color:var(--vp-c-text-2)}.drawing-scroll{overflow:auto;max-width:100%;background:#faf8f0;border:1px solid #cdcdbd;border-radius:6px}.drawing-scroll:focus-visible{outline:2px solid #8a684b;outline-offset:2px}svg{display:block;max-width:none;height:auto;color:#4b5347}svg text:not([fill]){fill:currentColor}.door-label{paint-order:stroke;stroke:#faf8f0;stroke-width:.5}.halo{paint-order:stroke;stroke:#faf8f0;stroke-width:1.8;stroke-linejoin:round}.legend{display:flex;flex-wrap:wrap;gap:6px 18px;font-size:12px}.floor-strip{display:flex;flex-wrap:wrap;gap:20px}.floor-strip figure{margin:8px 0;max-width:100%}.sample{margin:28px 0}.sample h4{font-size:16px;margin:0 0 6px}.sample>p{font-size:13px}table{margin:0;display:table;width:100%;font-size:13px;white-space:normal}th,td{min-width:70px;vertical-align:top}th{font-weight:600;text-align:left}th small{display:block;font-size:11px;font-weight:400;color:#778174;margin-top:4px}.district-architecture h3{font-size:20px;margin-top:36px}
</style>
