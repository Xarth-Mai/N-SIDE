<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, ref, watch } from 'vue'
import { withBase } from 'vitepress'
import data from '@wiki-data/map.json'
import { heightColor, project } from '../../../tools/district-map.ts'

import type { Scene } from '../../../tools/district-types.ts'
const scene=shallowRef<Scene>()
const loading=ref(false),error=ref('')
let request: AbortController | undefined
async function loadScene(){
  request?.abort()
  const current=new AbortController();request=current
  loading.value=true;error.value=''
  try{
    const response=await fetch(withBase('/project-assets/district-map/map.json'),{signal:current.signal})
    if(!response.ok)throw new Error(`HTTP ${response.status}`)
    const map=await response.json()
    if(!map?.scene||!Array.isArray(map.scene.terrain)||!Array.isArray(map.scene.objects)||!Array.isArray(map.scene.surfaces)||typeof map.scene.water?.d!=='string')throw new Error('Invalid map')
    if(!current.signal.aborted)scene.value=map.scene
  }catch{if(!current.signal.aborted)error.value='地图加载失败，请重试'}
  finally{if(!current.signal.aborted)loading.value=false}
}
onMounted(loadScene)
const svg=ref<SVGSVGElement | null>(null),mode=ref('scape'),frame=ref('all'),selected=ref<string | null>(null),names=ref(true),expanded=ref(false)
const size=ref([700,700]),camera=ref({x:260,y:-140,width:1700})
const place=computed(()=>data.places.find(p=>p.id===selected.value))
const view=computed(()=>{const c=camera.value,h=c.width*size.value[1]/size.value[0];return `${c.x-c.width/2} ${c.y-h/2} ${c.width} ${h}`})
const scale=computed(()=>size.value[0]/camera.value.width)
let observer: ResizeObserver | undefined
watch(svg,element=>{
  observer?.disconnect()
  if(!element)return
  observer=new ResizeObserver(([entry])=>{size.value=[entry.contentRect.width,entry.contentRect.height];if(frame.value==='all')fit('all')})
  observer.observe(element)
},{flush:'post'})
function fit(which: string){
  frame.value=which
  camera.value=which==='home'?{x:130,y:-215,width:360}:{x:260,y:-140,width:Math.max(1750,1500*size.value[0]/size.value[1])}
}
function choose(id: string){
  selected.value=id
  if(camera.value.width>600){const target=data.places.find(p=>p.id===id);if(!target)return;const [x,y]=project(target.position);camera.value={x,y,width:360};frame.value='detail'}
}
function zoom(factor: number){frame.value='detail';camera.value={...camera.value,width:Math.max(220,Math.min(2400,camera.value.width*factor))}}
let pointer: {id: number; x: number; y: number} | null=null
function down(e: PointerEvent){if(e.button!==0||(e.target instanceof Element && e.target.closest('[data-place]')))return;pointer={id:e.pointerId,x:e.clientX,y:e.clientY};svg.value?.setPointerCapture(e.pointerId)}
function move(e: PointerEvent){if(pointer?.id!==e.pointerId)return;const k=scale.value;camera.value={...camera.value,x:camera.value.x-(e.clientX-pointer.x)/k,y:camera.value.y-(e.clientY-pointer.y)/k};frame.value='detail';pointer={id:e.pointerId,x:e.clientX,y:e.clientY}}
function up(e: PointerEvent){if(pointer?.id!==e.pointerId)return;pointer=null;if(svg.value?.hasPointerCapture(e.pointerId))svg.value?.releasePointerCapture(e.pointerId)}
function key(e: KeyboardEvent){
  if(e.key==='Escape'){expanded.value=false;return}
  const delta=({ArrowLeft:[-1,0],ArrowRight:[1,0],ArrowUp:[0,-1],ArrowDown:[0,1]} as Record<string,number[]>)[e.key]
  if(delta){e.preventDefault();frame.value='detail';camera.value={...camera.value,x:camera.value.x+delta[0]*40/scale.value,y:camera.value.y+delta[1]*40/scale.value}}
  else if(['+','=','-'].includes(e.key)){e.preventDefault();zoom(e.key==='-'?1.25:.8)}
}
let previousOverflow=''
watch(expanded,on=>{
  if(on){previousOverflow=document.body.style.overflow;document.body.style.overflow='hidden'}
  else document.body.style.overflow=previousOverflow
})
onBeforeUnmount(()=>{request?.abort();observer?.disconnect();if(expanded.value)document.body.style.overflow=previousOverflow})
const landmarks=['01','04','15','19','23','26','35','79']
const visiblePlaces=computed(()=>{
  const c=camera.value,h=c.width*size.value[1]/size.value[0],near=c.width<620
  const candidates=data.places.filter(p=>near||landmarks.includes(p.id)).map(p=>({...p,point:project(p.position)})).filter(p=>Math.abs(p.point[0]-c.x)<c.width/2-24/scale.value&&Math.abs(p.point[1]-c.y)<h/2-24/scale.value)
  const result: (typeof candidates[number] & {dx: number; dy: number})[]=[],boxes: {x: number; y: number; w: number; h: number}[]=[]
  for(const p of candidates.sort((a,b)=>Number(b.id===selected.value)-Number(a.id===selected.value)||Number(b.id==='04')-Number(a.id==='04'))){
    if(result.some(q=>Math.abs(q.point[0]-p.point[0])*scale.value<46&&Math.abs(q.point[1]-p.point[1])*scale.value<46))continue
    const px=(p.point[0]-c.x)*scale.value+size.value[0]/2,py=(p.point[1]-c.y)*scale.value+size.value[1]/2
    const w=(p.name.length+(structure.value?5:0))*12
    for(const [dx,dy] of [[18,-18],[-w-18,-18],[18,22],[-w-18,22]]){
      const box={x:px+dx-3,y:py+dy-13,w:w+6,h:18}
      if(box.x<5||box.x+box.w>size.value[0]-5||box.y<5||box.y+box.h>size.value[1]-5)continue
      if(boxes.some(b=>box.x<b.x+b.w&&box.x+box.w>b.x&&box.y<b.y+b.h&&box.y+box.h>b.y))continue
      boxes.push(box);result.push({...p,dx,dy});break
    }
  }
  return result
})
const structure=computed(()=>mode.value==='roads')
</script>

<template>
  <section class="district-map" :class="{expanded}" aria-label="N街区导览图" @keydown.esc="expanded=false">
    <header class="map-header"><div><span class="map-kicker">NULL CITY · N DISTRICT</span><h3>沿着坡道，回到家</h3></div><button type="button" @click="expanded=!expanded">{{ expanded?'退出全屏':'全屏查看' }}</button></header>
    <div class="map-toolbar">
      <div class="map-tabs" aria-label="地图样式"><button type="button" :aria-pressed="mode==='scape'" @click="mode='scape'">街区风貌</button><button type="button" :aria-pressed="structure" @click="mode='roads'">道路结构</button></div>
      <div class="map-tabs" aria-label="地图取景"><button type="button" :aria-pressed="frame==='all'" @click="fit('all')">全图</button><button type="button" :aria-pressed="frame==='home'" @click="fit('home')">小店附近</button></div>
      <label class="map-label-toggle"><input v-model="names" type="checkbox">地名</label>
    </div>
    <div class="map-stage" :aria-busy="loading">
      <p v-if="error" class="map-placeholder" role="alert">{{ error }} <button type="button" @click="loadScene">重试</button></p>
      <p v-else-if="!scene" class="map-placeholder" role="status">正在加载地图…</p>
      <svg v-if="scene" ref="svg" class="map-canvas" :viewBox="view" tabindex="0" role="group" aria-label="街区斜俯视导览，方向键移动，加减键缩放；地点也可从下方列表选择" @keydown="key" @pointerdown="down" @pointermove="move" @pointerup="up" @pointercancel="up" @lostpointercapture="pointer=null">
        <title>N街区 · 河岸、街坊与山坡</title>
        <rect x="-3000" y="-3000" width="6000" height="6000" fill="#e5e4d5"/>
        <g class="terrain"><path v-for="(face,i) in scene.terrain" :key="i" :d="face.d" :fill="structure?heightColor(face.height):face.fill" :stroke="structure?heightColor(face.height):face.fill" stroke-width=".8"/></g>
        <path :d="scene.water.d" :fill="scene.water.fill"/>
        <g stroke="#c0d9d3" stroke-width="1" opacity=".65"><path v-for="i in 12" :key="i" :d="`M ${project([-280+i*90,-220-i%3*40,0]).join(',')} l 35 9 m 10 3 l 12 3`"/></g>
        <g v-for="surface in scene.surfaces" :key="surface.key" :data-surface-place="surface.place"><path v-for="(shape,j) in surface.shapes" :key="j" :d="shape.d" :fill="shape.fill" :stroke="selected===surface.place?'#c2763b':shape.stroke" :stroke-width="selected===surface.place?1.7:shape.width"/></g>
        <g v-for="object in scene.objects" :key="object.key" :data-kind="object.kind" :data-object-place="object.place" :opacity="structure&&['building','tree'].includes(object.kind)?.18:1">
          <path v-for="(shape,j) in object.shapes" :key="j" :d="shape.d" :fill="shape.fill" :fill-rule="shape.fillRule" :stroke="selected===object.place&&object.kind==='building'?'#c2763b':shape.stroke" :stroke-width="selected===object.place&&object.kind==='building'?1.7:shape.width" :opacity="shape.opacity" stroke-linejoin="round"/>
        </g>
        <g v-if="names" class="map-labels">
          <g v-for="p in visiblePlaces" :key="p.id" :transform="`translate(${p.point}) scale(${1/scale})`" :data-place="p.id" role="button" tabindex="0" :aria-label="`${p.id} ${p.name}`" :aria-pressed="selected===p.id" @click.stop="choose(p.id)" @keydown.enter.stop.prevent="choose(p.id)" @keydown.space.stop.prevent="choose(p.id)">
            <rect x="-22" y="-22" width="44" height="44" rx="22" fill="transparent"/>
            <circle r="5" :fill="selected===p.id?'#be794b':'#f8f4e7'" stroke="#637c73" stroke-width="1.5"/>
            <path :d="`M 0 0 L ${p.dx>0?14:-14} ${p.dy-4} L ${p.dx>0?p.dx-2:p.dx+p.name.length*12+2} ${p.dy-4}`" fill="none" stroke="#637c73" stroke-width="1"/>
            <text :x="p.dx" :y="p.dy" font-size="12" class="place-label">{{ structure?p.id+' · ':'' }}{{ p.name }}</text>
          </g>
        </g>
      </svg>
      <div class="map-zoom"><button type="button" aria-label="放大地图" @click="zoom(.75)">＋</button><button type="button" aria-label="缩小地图" @click="zoom(1.333)">−</button></div>
      <div class="map-credit">N街区 <span>／</span> {{ structure?'道路与公共边界':'街区风貌' }}</div>
    </div>
    <div v-if="place" class="map-detail" aria-live="polite"><button class="detail-close" type="button" aria-label="关闭地点详情" @click="selected=null">×</button><span class="map-kicker">{{ data.groups[place.group] }} · {{ place.id }}</span><h4>{{ place.name }}</h4><p>{{ place.use }}</p><p class="entry">{{ place.entry }}</p><a v-if="place.page" :href="withBase(place.page)">地点介绍 →</a></div>
    <footer><span v-if="structure" class="height-legend">低 <i :style="{background:`linear-gradient(to right,${heightColor(0)},${heightColor(80)})`}"/> 高 <span>细墙线 · 地块边界</span></span><span v-else>从河岸到山林 · 沿路认识街坊</span><span>细看地点可放大，或使用地点索引</span></footer>
    <details class="map-index"><summary>地点索引 · {{ data.places.length }}</summary><div><button v-for="p in data.places" :key="p.id" type="button" @click="choose(p.id);frame='detail';camera={...camera,x:project(p.position)[0],y:project(p.position)[1],width:360}"><span>{{ p.id }}</span>{{ p.name }}</button></div></details>
  </section>
</template>

<style scoped>
.district-map{margin:28px 0;border:1px solid #c7cbbb;border-radius:12px;overflow:hidden;background:#f8f6ee;color:#354e47;font-family:var(--vp-font-family-base)}
.map-header{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:18px 20px 12px}.map-kicker{font-size:10px;letter-spacing:1.5px;color:#667e73}.map-header h3{margin:4px 0!important;padding:0!important;font-size:20px!important;letter-spacing:1px}
button{font:inherit;cursor:pointer;min-height:44px;border-radius:6px;font-size:12px;padding:8px 12px;color:inherit}button:focus-visible,[role=button]:focus-visible{outline:2px solid #b57643;outline-offset:-2px}.map-header button{border:1px solid #c9cfbf;white-space:nowrap}
.map-toolbar{display:flex;flex-wrap:wrap;gap:8px;padding:0 16px 12px;align-items:center}.map-tabs{display:flex;background:#eaece1;border-radius:7px;padding:3px}.map-tabs button[aria-pressed=true]{background:#fdfcf4;box-shadow:0 1px 4px #495b4420}.map-label-toggle{margin-left:auto;font-size:12px;display:flex;gap:5px;align-items:center;min-height:44px}
.map-stage{position:relative;isolation:isolate}.map-placeholder{height:730px;margin:0;display:grid;place-content:center;text-align:center}.map-canvas{display:block;width:100%;height:730px;touch-action:pan-y;cursor:grab}.map-canvas:active{cursor:grabbing}.map-canvas [role=button]{cursor:pointer}.place-label{fill:#354f47;stroke:#f8f5e7;stroke-width:3px;paint-order:stroke;stroke-linejoin:round;font-weight:600}
.map-zoom{position:absolute;right:12px;bottom:40px;display:flex;flex-direction:column;gap:4px}.map-zoom button{width:44px;background:#fffdf0ec;border:1px solid #c8ceba;font-size:20px}.map-credit{position:absolute;bottom:14px;left:18px;font-size:11px;letter-spacing:2px;pointer-events:none}.map-credit span{padding:0 8px;color:#81917d}
.map-detail{position:relative;padding:16px 45px 16px 20px;background:#fffdf5;border-top:1px solid #d1d5c8}.map-detail h4{margin:5px 0 8px;font-size:19px}.map-detail p{margin:4px 0;font-size:13px;line-height:1.6}.map-detail .entry{color:#718073}.map-detail a{font-size:13px;color:#8e633e}.detail-close{position:absolute;right:6px;top:6px;font-size:22px}
footer{padding:12px 18px;font-size:11px;color:#718073;display:flex;gap:10px;flex-wrap:wrap;justify-content:space-between}.height-legend{display:flex;gap:8px;align-items:center}.height-legend i{display:block;width:70px;height:8px}.height-legend span{margin-left:8px}
.map-index{padding:0 18px 16px;font-size:12px}.map-index summary{cursor:pointer;min-height:32px}.map-index>div{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:4px}.map-index button{text-align:left;background:#eaece1}.map-index span{font-size:10px;color:#8b775c;margin-right:8px}
.expanded{position:fixed;inset:0;z-index:1000;margin:0;border:0;border-radius:0;overflow:auto;height:100dvh}.expanded .map-placeholder,.expanded .map-canvas{height:calc(100dvh - 160px);min-height:320px}.expanded .map-detail{position:sticky;bottom:0;z-index:2;box-shadow:0 -4px 18px #485b4320}
@media(max-width:600px){.map-header{padding:14px 12px 8px}.map-header h3{font-size:18px!important}.map-toolbar{padding:0 10px 10px;gap:4px}.map-tabs button{padding:6px 8px}.map-label-toggle{margin-left:4px}.map-placeholder,.map-canvas{height:560px}.map-index>div{grid-template-columns:repeat(2,minmax(0,1fr))}.map-header button{padding:6px 9px}.expanded .map-canvas{touch-action:none}}
</style>
