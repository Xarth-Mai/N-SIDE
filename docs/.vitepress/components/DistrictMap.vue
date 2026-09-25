<script setup>
import data from '../../../source-assets/district-map/district.json'
import { heightColor, terrainHeight, svgPath } from '../../../tools/district-map.mjs'

const polygons = points => points.map(p=>`${p[0]},${-p[1]}`).join(' ')
const ground = Object.entries(data.nodes).filter(([id])=>!id.startsWith('bridge')).map(([,p])=>p)
const terrain = []
for (let y=-560;y<1240;y+=30) for(let x=-290;x<1000;x+=30) {
  terrain.push({x,y:-y-30,color:heightColor(terrainHeight(ground,x+15,y+15))})
}
const roadWidth = kind => ({main:7,avenue:11,shore:5,bridge:11,steps:3,trail:3,service:3}[kind]??4)
</script>

<template>
  <figure class="district-map">
    <svg class="map-canvas" viewBox="-230 -1180 1170 1680" role="img" aria-label="N街区静态总览，底色由低处的浅沙色逐渐变为高处的深绿色，地点编号对应图下索引">
      <title>N街区 · 从河岸到摘星台</title>
      <defs><filter id="district-elevation"><feGaussianBlur stdDeviation="12"/></filter></defs>
      <g aria-label="地势" filter="url(#district-elevation)"><rect v-for="(cell,i) in terrain" :key="i" :x="cell.x" :y="cell.y" width="30.2" height="30.2" :fill="cell.color"/></g>
      <polygon v-for="(t,i) in data.terrain" :key="i" :points="polygons(t.polygon)" fill="#477e8c" stroke="#e1d4ac" stroke-width="5"/>
      <g aria-label="道路"><path v-for="(road,i) in data.roads" :key="i" :d="svgPath(road.nodes.map(id=>data.nodes[id]))" fill="none" stroke="#f1e3bc" :stroke-width="roadWidth(road.kind)" :stroke-dasharray="['steps','trail'].includes(road.kind) ? '5 5' : undefined" stroke-linecap="round" stroke-linejoin="round"/></g>
      <g aria-label="建筑"><polygon v-for="(b,i) in data.buildings" :key="i" :points="polygons(b.polygon)" fill="#e3d3ac" stroke="#697961" stroke-width="2"/></g>
      <g fill="#587b5c" opacity=".65" aria-label="树木"><circle v-for="(p,i) in data.trees" :key="i" :cx="p[0]" :cy="-p[1]" r="7"/></g>
      <g aria-label="地点"><g v-for="p in data.places" :key="p.id">
        <circle :cx="p.position[0]" :cy="-p.position[1]" r="14" fill="#f5edda" stroke="#3c585b" stroke-width="2"/>
        <text :x="p.position[0]" :y="-p.position[1]" text-anchor="middle" dominant-baseline="central" fill="#304a50" font-size="17" font-weight="700">{{ p.id }}</text>
      </g></g>
      <text v-for="p in data.places.filter(p=>['04','19','23'].includes(p.id))" :key="`label-${p.id}`" :x="p.position[0]+23" :y="-p.position[1]-21" class="place-label" font-size="22">{{ p.name }}</text>
      <text v-for="l in data.labels" :key="l.text" :x="l.position[0]" :y="-l.position[1]" class="place-label" font-size="28" letter-spacing="2">{{ l.text }}</text>
    </svg>
    <figcaption>
      <div class="height-legend"><span>低</span><i :style="{background:`linear-gradient(to right, ${heightColor(0)}, ${heightColor(80)})`}"/><span>高</span><span>虚线 · 台阶与步道</span></div>
      <ol class="map-index"><li v-for="p in data.places" :key="p.id"><span>{{ p.id }}</span>{{ p.name }}</li></ol>
    </figcaption>
  </figure>
</template>

<style scoped>
.district-map{margin:24px 0;border:1px solid #65796e;border-radius:8px;overflow:hidden;background:#263e45;color:#ede4cf}
.map-canvas{display:block;width:100%;height:auto}
.place-label{fill:#fff3d5;stroke:#446255;stroke-width:3;paint-order:stroke;stroke-linejoin:round;font-weight:600}
figcaption{padding:14px}.height-legend{display:flex;align-items:center;gap:10px;font-size:12px;flex-wrap:wrap}.height-legend i{width:110px;height:10px;border-radius:3px}
.map-index{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:5px 12px;list-style:none!important;margin:14px 0 0!important;padding:0!important;font-size:12px}.map-index li{margin:0;display:flex;gap:7px}.map-index span{color:#d6bb84;font-variant-numeric:tabular-nums}
@media(max-width:600px){.map-index{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>
