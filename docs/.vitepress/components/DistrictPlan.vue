<script setup>
import data from '../../../source-assets/district-map/district.json'
import { routeProfile } from '../../../tools/district-plan.mjs'

const blocks=data.blocks??[],parcels=data.parcels??[],connections=data.connections??[]
const positions=[...blocks.flatMap(b=>b.polygon),...Object.values(data.nodes),...connections.map(c=>c.to)]
const x0=Math.min(...positions.map(p=>p[0]))-100,y0=-Math.max(...positions.map(p=>p[1]))-100
const width=Math.max(...positions.map(p=>p[0]))-x0+100,height=-Math.min(...positions.map(p=>p[1]))-y0+100
const points=polygon=>polygon.map(([x,y])=>`${x},${-y}`).join(' ')
const roadStyle={avenue:[18,'#c3aa80'],main:[13,'#c3aa80'],bridge:[18,'#8b9b9d'],lane:[8,'#c1b9a7'],service:[6,'#ad9a8b'],shore:[9,'#9bb8ae'],steps:[5,'#a58468'],trail:[5,'#a8b39a'],crossing:[9,'#c3aa80'],landing:[8,'#a8b39a']}
const landmarks=data.places.filter(p=>['01','04','15','19','23','26'].includes(p.id))
const profiles=(data.sections??[]).map(section=>{
  const samples=routeProfile(data.nodes,section.nodes).map(p=>({...p,height:p.point[2],label:section.labels?.[p.id]}))
  const distance=samples.at(-1)?.distance??0
  const floor=Math.floor(Math.min(...samples.map(p=>p.height))/5)*5
  const ceiling=Math.max(floor+5,Math.ceil(Math.max(...samples.map(p=>p.height))/5)*5)
  const sx=870/Math.max(distance,1),sy=180/(ceiling-floor)
  return {...section,samples,distance,floor,ceiling,exaggeration:sy/sx,x:p=>65+p.distance*sx,y:p=>220-(p.height-floor)*sy}
}).filter(section=>section.samples.length>1)
</script>

<template>
  <section class="district-plan" aria-label="N街区扩容总图与关键剖面">
    <p class="plan-caption">总平面 · 街坊、地块与公共道路 <span>B 编号对应下表 · 米 / 设计高程</span></p>
    <div class="plan-scroll" tabindex="0" role="region" aria-label="街区总平面，窄屏可横向滚动">
      <svg class="plan-map" :viewBox="`${x0} ${y0} ${width} ${height}`"  role="img" aria-label="N街区双中心与十二街坊总平面">
        <title>N街区双中心与十二街坊总平面</title>
        <desc>底色划分街坊，细线围合地块，灰色体量为初设建筑。主要地点和通往图外城区的联系单独标注；上方为坡地，下方为河岸</desc>
        <defs><marker id="district-plan-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#557d82"/></marker></defs>
        <rect :x="x0" :y="y0" :width="width" :height="height" fill="#f3f1e8"/>
        <polygon :points="points(data.terrain.water)" fill="#c8e0dd"/>
        <g class="blocks"><polygon v-for="(block,i) in blocks" :key="block.id" :points="points(block.polygon)" :fill="['#e7e4d1','#e1e8d7','#e8ddd0'][i%3]" stroke="#adb69e" stroke-width="2"><title>{{ block.id }} {{ block.name }}：{{ block.role }}</title></polygon></g>
        <g class="parcels"><polygon v-for="parcel in parcels" :key="parcel.id" :points="points(parcel.polygon)" fill="#fffdf5" fill-opacity=".55" stroke="#bcb7a8" stroke-width="1.5"><title>{{ parcel.id }} · {{ parcel.use }}</title></polygon></g>
        <g class="buildings"><polygon v-for="(building,i) in data.buildings" :key="i" :points="points(building.polygon)" fill="#c9c8c0" stroke="#8c938c" stroke-width="1"/></g>
        <g class="roads" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <g v-for="(road,i) in data.roads" :key="i">
            <polyline :points="points(road.nodes.map(id=>data.nodes[id]))" stroke="#f8f6ed" :stroke-width="(roadStyle[road.kind]?.[0]??6)+4"/>
            <polyline :points="points(road.nodes.map(id=>data.nodes[id]))" :stroke="roadStyle[road.kind]?.[1]??'#c1b9a7'" :stroke-width="roadStyle[road.kind]?.[0]??6" :stroke-dasharray="['steps','trail'].includes(road.kind)?'3 5':undefined"/>
          </g>
        </g>
        <g class="connections">
          <g v-for="connection in connections" :key="connection.id">
            <line :x1="data.nodes[connection.from][0]" :y1="-data.nodes[connection.from][1]" :x2="connection.to[0]" :y2="-connection.to[1]" stroke="#557d82" stroke-width="3" stroke-dasharray="8 5" marker-end="url(#district-plan-arrow)"/>
            <text :x="(data.nodes[connection.from][0]+connection.to[0])/2" :y="-(data.nodes[connection.from][1]+connection.to[1])/2-15" class="connection-label" text-anchor="middle">{{ connection.name }}<title>{{ connection.use }}</title></text>
          </g>
        </g>
        <g class="block-labels"><text v-for="block in blocks" :key="block.id" :x="block.label[0]" :y="-block.label[1]" text-anchor="middle"><tspan :x="block.label[0]">{{ block.id }}</tspan><title>{{ block.name }}</title></text></g>
        <g class="landmark-labels">
          <g v-for="place in landmarks" :key="place.id">
            <circle :cx="place.position[0]" :cy="-place.position[1]" r="8" fill="#b5794e" stroke="#fffdf5" stroke-width="3"/>
            <text :x="place.position[0]+15" :y="-place.position[1]-12">{{ place.id }} {{ place.name.replace("小型", "").split("与")[0] }}</text>
          </g>
        </g>
        <g :transform="`translate(${x0+35} ${y0+height-45})`" class="scale-bar"><path d="M 0 -8 V 0 H 100 V -8" fill="none" stroke="#53645c" stroke-width="2"/><text x="50" y="27" text-anchor="middle">100 m</text></g>
      </svg>
    </div>
    <ul class="plan-legend" aria-label="总平面图例"><li><i class="legend-block"/>街坊范围</li><li><i class="legend-parcel"/>规划地块</li><li><i class="legend-building"/>初设建筑体量</li><li><i class="legend-road"/>城市道路与生活街</li><li><i class="legend-shore"/>滨水通道</li><li><i class="legend-trail"/>台阶与步道</li><li><i class="legend-link"/>图外城市联系</li></ul>
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
.district-plan{margin:24px 0;color:#3e544b}.plan-caption{display:flex;justify-content:space-between;gap:8px;flex-wrap:wrap;font-size:14px;font-weight:600}.plan-caption span{font-weight:400;font-size:12px;color:var(--vp-c-text-2)}
.plan-scroll{overflow-x:auto;border:1px solid #cbd0c1;border-radius:8px;background:#f8f7f0}.plan-scroll:focus-visible{outline:2px solid #8a684b;outline-offset:2px}.plan-map{display:block;width:100%;min-width:0}.block-labels text{font-size:34px;font-weight:600;fill:#566949}.block-labels tspan:first-child{font-size:32px;font-weight:400}.landmark-labels text{font-size:28px;font-weight:600;fill:#815a3d}.connection-label,.scale-bar text{font-size:27px;fill:#42686b}.block-labels text,.landmark-labels text,.connection-label{paint-order:stroke;stroke:#f8f6ed;stroke-width:5px;stroke-linejoin:round}
.plan-legend{display:flex;flex-wrap:wrap;gap:8px 18px;padding:0!important;list-style:none!important;font-size:12px}.plan-legend li{display:flex;gap:6px;align-items:center;margin:0!important}.plan-legend i{display:inline-block;width:20px;height:10px;border:1px solid #adb69e}.legend-block{background:#e1e8d7}.legend-parcel{background:#fffdf5}.legend-building{background:#c9c8c0}.plan-legend .legend-road{height:5px;background:#c3aa80;border:0}.plan-legend .legend-shore{height:5px;background:#9bb8ae;border:0}.plan-legend .legend-trail{height:0;border:0;border-top:3px dotted #a8b39a}.plan-legend .legend-link{height:0;border:0;border-top:2px dashed #557d82}
.plan-section{margin-top:28px}.plan-section h4{margin:0!important;font-size:15px}.plan-section p{margin:6px 0 12px;font-size:13px;color:var(--vp-c-text-2)}.profile-map{display:block;width:100%;min-width:0}.profile-grid line{stroke:#d6ddce;stroke-width:1}.profile-grid text,.axis-label{font-size:16px;fill:#697a6b}.sample-label{font-size:16px;fill:#4d6253}.sample-label tspan{font-size:14px;fill:#758170}
@media(max-width:600px){.plan-map{min-width:950px}.profile-map{min-width:750px}}
</style>
