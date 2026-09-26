<script setup lang="ts">
import source from '../../../source-assets/district-map/district.json'
import type { District, Point, Building, Road, Surface } from '../../../tools/district-types.ts'
import { routeProfile } from '../../../tools/district-plan.ts'
import { roadWidth, segmentIntervals } from '../../../tools/district-geometry.ts'
import { sectionRoads } from '../../../tools/district-architecture.ts'
const data: District = source

const props=defineProps<{blockIds?: string[]; sectionIds?: string[]}>()
const blocks=data.blocks.filter(b=>!props.blockIds||props.blockIds.includes(b.id))
const parcels=data.parcels.filter(p=>blocks.some(b=>b.id===p.block))
const buildings=data.buildings.filter(b=>parcels.some(p=>p.id===b.parcel))
const contextBuildings=props.blockIds?data.buildings.filter(b=>b.bank==='district'&&!buildings.includes(b)):[]
const connections=props.blockIds?[]:data.connections
const positions=[...blocks.flatMap(b=>b.polygon),...connections.map(c=>c.to)]
const x0=Math.min(...positions.map(p=>p[0]))-40,y0=-Math.max(...positions.map(p=>p[1]))-40
const width=Math.max(...positions.map(p=>p[0]))-x0+40,height=-Math.min(...positions.map(p=>p[1]))-y0+60
const points=(polygon: Point[])=>polygon.map(([x,y])=>`${x},${-y}`).join(' ')
const modes=[{id:'plain',name:'城市总平面 · 无编号'},{id:'access',name:'道路与公共到达 · 入口和使用边界'}]
const landmarks=data.places.filter(p=>['01','04','15','19','23','26','79'].includes(p.id)&&blocks.some(b=>b.id===p.block))
const roads=data.roads.filter(r=>r.kind!=='interior')
const ground=data.surfaces.filter(s=>!s.elevated),elevated=data.surfaces.filter(s=>s.elevated)
const entrances=buildings.flatMap(b=>(b.design?.entries??[]).map(e=>({...e,id:`${b.id}-${e.role}-${e.node}`,point:data.nodes[e.node],label:`${b.id} · ${e.level}`})))
const roleColor: Record<string,string>={public:'#397b85',resident:'#638657',student:'#7777a0',service:'#a06e50'}
const roleName: Record<string,string>={public:'公共入口',resident:'住宅入口',student:'校园入口',service:'服务入口'}
const roadColor=(r: Road)=>r.access==='service'?'#b38a72':r.access==='resident'?'#97ac80':r.access==='controlled'?'#a89ab8':['bridge','deck'].includes(r.kind)?'#849d9d':['shore','trail'].includes(r.kind)?'#9bb8ae':'#c3aa80'
const surfaceColor=(s: Surface)=>(({park:'#d8e2c7',garden:'#d8e2c7',private:'#e7dfcb',service:'#ded3c9',court:'#e6e4d8',platform:'#dfd8c7'} as Record<string,string>)[s.kind]??'#e4e5d3')
const buildingColor=(b: Pick<Building, "place" | "design" | "kind">)=>b.place==='04'?'#c9a987':b.kind==='school'?'#b3c6cd':b.kind==='home'?'#c1ccc2':b.kind==='civic'?'#c0b8c9':'#d6c9b6'
const profiles=(data.sections??[]).filter(s=>!props.sectionIds||props.sectionIds.includes(s.id)).map(section=>{
  const samples=routeProfile(data.nodes,section.nodes).map(p=>({...p,height:p.point[2],label:section.labels?.[p.id]}))
  const distance=samples.at(-1)?.distance??0
  const volumes=data.buildings.filter(b=>b.bank==='district').flatMap(building=>samples.slice(1).flatMap((p,i)=>segmentIntervals(samples[i].point,p.point,building.polygon,true).map(([lo,hi])=>({building,start:samples[i].distance+p.length*lo,end:samples[i].distance+p.length*hi}))).filter(v=>v.end-v.start>.01))
  const crossingParts=samples.slice(1).flatMap((p,i)=>p.length?sectionRoads(data,{line:[samples[i].point,p.point]}).filter(r=>['bridge','deck'].includes(r.kind)).map(r=>({...r,start:samples[i].distance+r.span[0],end:samples[i].distance+r.span[1]})):[]).sort((a,b)=>a.start-b.start)
  const crossings: typeof crossingParts=[]
  for(const part of crossingParts){
    const previous=crossings.find(r=>r.kind===part.kind&&Math.abs(r.elevation-part.elevation)<.01&&r.end>=part.start-.01)
    if(previous)previous.end=Math.max(previous.end,part.end)
    else crossings.push(part)
  }
  const floor=Math.floor(Math.min(...samples.map(p=>p.height))/5)*5
  const ceiling=Math.max(floor+5,Math.ceil(Math.max(...samples.map(p=>p.height),...volumes.map(v=>v.building.elevation+v.building.height),...crossings.map(r=>r.elevation+2))/5)*5)
  const sx=870/Math.max(distance,1),sy=180/(ceiling-floor)
  return {...section,samples,volumes,crossings,distance,floor,ceiling,exaggeration:sy/sx,x:(p: {distance: number})=>65+p.distance*sx,y:(p: {height: number})=>220-(p.height-floor)*sy}
}).filter(section=>section.samples.length>1)
</script>

<template>
  <section class="district-plan" aria-label="Null Site城市框架与关键剖面">
    <figure v-for="mode in modes" :key="mode.id" class="framework-plan">
      <figcaption class="plan-caption">{{ mode.name }}<span>北向上 · 设计米 · 道路按实际宽度绘制</span></figcaption>
      <div class="plan-scroll" tabindex="0" role="region" :aria-label="`${mode.name}，窄屏可横向滚动`">
        <svg class="plan-map" :viewBox="`${x0} ${y0} ${width} ${height}`" role="img" :aria-label="mode.name">
          <title>{{ mode.name }}</title>
          <desc>连续建筑、院落、校园、公共空间和道路使用同一组设计坐标；无编号图用于阅读空间，到达图表示公共、住户、校园和后勤边界</desc>
          <rect :x="x0" :y="y0" :width="width" :height="height" fill="#f3f1e8"/>
          <polygon :points="points(data.terrain.water)" fill="#c8e0dd"/>
          <polygon v-for="block in blocks" :key="block.id" :points="points(block.polygon)" fill="#e7ebdc" stroke="#d2d7c7" stroke-width="1"/>
          <polygon v-for="parcel in parcels" :key="parcel.id" :points="points(parcel.polygon)" fill="#fffdf5" fill-opacity=".55" stroke="#c8c4b7" stroke-width=".6"><title>{{ parcel.use }}</title></polygon>
          <polygon v-for="(surface,i) in ground" :key="i" :points="points(surface.polygon)" :fill="surfaceColor(surface)" stroke="#b6baa8" stroke-width=".5"><title>{{ surface.name??surface.kind }} · {{ surface.elevation }} m</title></polygon>
          <g fill="none" stroke-linejoin="round" stroke-linecap="butt">
            <g v-for="(road,i) in roads" :key="i">
              <polyline :points="points(road.nodes.map(id=>data.nodes[id]))" stroke="#aaa994" :stroke-width="roadWidth(road)+.8"/>
              <polyline :points="points(road.nodes.map(id=>data.nodes[id]))" :stroke="mode.id==='access'?roadColor(road):'#fffdf6'" :stroke-width="roadWidth(road)" :stroke-dasharray="['steps','trail'].includes(road.kind)?'2 2':undefined"><title>{{ road.kind }} · {{ roadWidth(road) }} m · {{ road.access??'public' }}</title></polyline>
            </g>
          </g>
          <polygon v-for="building in contextBuildings" :key="`context-${building.id}`" :points="points(building.polygon)" fill="#d5d6cb" stroke="#b3b8ac" stroke-width=".6"/>
          <polygon v-for="building in buildings" :key="building.id" :points="points(building.polygon)" :fill="mode.id==='plain'?buildingColor(building):'#dad9d1'" stroke="#85908a" stroke-width=".8"><title>{{ building.id }} · {{ building.design?.floors.map(f=>f.use).join(' / ') }} · 基底 {{ building.elevation }} m · 高 {{ building.height }} m</title></polygon>
          <polygon v-for="(surface,i) in elevated" :key="`e-${i}`" :points="points(surface.polygon)" :fill="surface.building?'#acc698':'#d8cbb1'" stroke="#6e8a72" stroke-width="1"><title>{{ surface.name??'公共平台' }} · {{ surface.elevation }} m</title></polygon>
          <g v-if="mode.id==='access'" class="entrances">
            <g v-for="entrance in entrances" :key="entrance.id" :transform="`translate(${entrance.point[0]} ${-entrance.point[1]})`">
              <rect v-if="entrance.role==='service'" x="-2.2" y="-2.2" width="4.4" height="4.4" :fill="roleColor[entrance.role]"/>
              <circle v-else r="2.5" :fill="roleColor[entrance.role]"/>
              <title>{{ entrance.label }} · {{ roleName[entrance.role] }} · {{ entrance.point[2] }} m</title>
            </g>
          </g>
          <g v-if="mode.id==='access'" class="landmark-labels"><text v-for="place in landmarks" :key="place.id" :x="place.position[0]+10" :y="-place.position[1]-12">{{ place.name.replace('小型','').split('与')[0] }}</text></g>
          <g v-for="connection in connections" :key="connection.id" class="connections">
            <line :x1="data.nodes[connection.from][0]" :y1="-data.nodes[connection.from][1]" :x2="connection.to[0]" :y2="-connection.to[1]" stroke="#557d82" stroke-width="2" stroke-dasharray="6 4"/>
            <text v-if="mode.id==='access'" :x="connection.to[0]" :y="-connection.to[1]-8" class="connection-label" text-anchor="middle">{{ connection.name }}</text>
          </g>
          <g :transform="`translate(${x0+25} ${y0+height-25})`" class="scale-bar"><path d="M 0 -4 V 0 H 100 V -4" fill="none" stroke="#53645c" stroke-width="1.5"/><text x="50" y="17" text-anchor="middle">100 m</text></g>
        </svg>
      </div>
    </figure>
    <ul class="plan-legend" aria-label="到达图图例"><li><span style="color:#397b85">●</span>公共入口</li><li><span style="color:#638657">●</span>住户入口</li><li><span style="color:#7777a0">●</span>校园入口</li><li><span style="color:#a06e50">■</span>服务入口</li><li>浅棕：公共道路</li><li>灰绿：住户通路</li><li>紫灰：受管理通路</li><li>棕色：后勤通路</li></ul>
    <div v-for="profile in profiles" :key="profile.id" class="plan-section">
      <h4>{{ profile.id }} · {{ profile.name }}</h4>
      <p>{{ profile.note }}</p>
      <div class="plan-scroll" tabindex="0" role="region" :aria-label="`${profile.name}剖面，窄屏可横向滚动`">
        <svg class="profile-map" viewBox="0 0 1000 300" role="img" :aria-label="`${profile.name}剖面，水平距离${Math.round(profile.distance)}米，纵向放大${profile.exaggeration.toFixed(1)}倍`">
          <title>{{ profile.name }}关键剖面</title>
          <desc>沿指定节点累计平距绘制高程。纵向放大 {{ profile.exaggeration.toFixed(1) }} 倍，坡线倾角不表示实际坡度</desc>
          <g class="profile-grid" v-for="i in 5" :key="i"><line x1="65" :y1="40+(i-1)*45" x2="935" :y2="40+(i-1)*45"/><text x="55" :y="44+(i-1)*45" text-anchor="end">{{ (profile.ceiling-(profile.ceiling-profile.floor)*(i-1)/4).toFixed(1) }}</text></g>
          <text x="65" y="22" class="axis-label">高程 / m</text><text x="935" y="22" text-anchor="end" class="axis-label">纵向放大 {{ profile.exaggeration.toFixed(1) }} 倍</text>
          <polygon :points="`${profile.samples.map(p=>`${profile.x(p)},${profile.y(p)}`).join(' ')} 935,220 65,220`" fill="#e1e8d7"/>
          <rect v-for="(volume,i) in profile.volumes" :key="i" :x="profile.x({distance:volume.start})" :y="profile.y({height:volume.building.elevation+volume.building.height})" :width="profile.x({distance:volume.end})-profile.x({distance:volume.start})" :height="profile.y({height:volume.building.elevation})-profile.y({height:volume.building.elevation+volume.building.height})" fill="#d5d0bf" stroke="#8c9487" stroke-width="1"><title>{{ volume.building.id }} · 基底 {{ volume.building.elevation }} m · 建筑高 {{ volume.building.height }} m</title></rect>
          <g v-for="(crossing,i) in profile.crossings" :key="`cross-${i}`"><line :x1="profile.x({distance:crossing.start})" :x2="profile.x({distance:crossing.end})" :y1="profile.y({height:crossing.elevation})" :y2="profile.y({height:crossing.elevation})" stroke="#647e88" stroke-width="4"/><text :x="profile.x({distance:(crossing.start+crossing.end)/2})" :y="profile.y({height:crossing.elevation})-8" text-anchor="middle" class="axis-label">{{ crossing.kind==='bridge'?'桥面':'平台' }} {{ crossing.elevation }} m</text></g>
          <polyline :points="profile.samples.map(p=>`${profile.x(p)},${profile.y(p)}`).join(' ')" fill="none" stroke="#647e69" stroke-width="3" stroke-linejoin="round"/>
          <g v-for="(sample,i) in profile.samples" :key="`${sample.id}-${i}`">
            <circle :cx="profile.x(sample)" :cy="profile.y(sample)" r="3.5" fill="#b5794e"/>
            <template v-if="sample.label||i===0||i===profile.samples.length-1">
              <line :x1="profile.x(sample)" :y1="profile.y(sample)+7" :x2="profile.x(sample)" :y2="230" stroke="#aeb7a8" stroke-dasharray="3 3"/>
              <text :x="profile.x(sample)" :y="245+(i%2)*34" :text-anchor="i===0?'start':i===profile.samples.length-1?'end':'middle'" class="sample-label">{{ sample.label??(i===0?'起点':'终点') }}<tspan :x="profile.x(sample)" dy="15">{{ Math.round(sample.distance) }} m · {{ sample.height }} m</tspan></text>
            </template>
          </g>
        </svg>
      </div>
    </div>
  </section>
</template>

<style scoped>
.district-plan{margin:24px 0;color:#3e544b}.framework-plan{margin:24px 0}.plan-caption{display:flex;justify-content:space-between;gap:8px;flex-wrap:wrap;font-size:14px;font-weight:600;margin-bottom:8px}.plan-caption span{font-weight:400;font-size:12px;color:var(--vp-c-text-2)}
.plan-scroll{overflow-x:auto;border:1px solid #cbd0c1;border-radius:8px;background:#f8f7f0}.plan-scroll:focus-visible{outline:2px solid #8a684b;outline-offset:2px}.plan-map,.profile-map{display:block;width:100%;min-width:0}.landmark-labels text{font-size:16px;font-weight:600;fill:#815a3d}.connection-label,.scale-bar text{font-size:15px;fill:#42686b}.landmark-labels text,.connection-label{paint-order:stroke;stroke:#f8f6ed;stroke-width:3px;stroke-linejoin:round}
.plan-legend{display:flex;flex-wrap:wrap;gap:8px 18px;padding:0!important;list-style:none!important;font-size:12px}.plan-legend li{display:flex;gap:6px;align-items:center;margin:0!important}
.plan-section{margin-top:28px}.plan-section h4{margin:0!important;font-size:15px}.plan-section p{margin:6px 0 12px;font-size:13px;color:var(--vp-c-text-2)}.profile-grid line{stroke:#d6ddce;stroke-width:1}.profile-grid text,.axis-label{font-size:16px;fill:#697a6b}.sample-label{font-size:16px;fill:#4d6253}.sample-label tspan{font-size:14px;fill:#758170}
@media(max-width:600px){.plan-map{min-width:950px}.profile-map{min-width:750px}}
</style>
