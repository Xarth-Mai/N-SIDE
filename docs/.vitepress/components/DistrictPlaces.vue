<script setup lang="ts">
import source from '../../../source-assets/district-map/district.json'
import type { District } from '../../../tools/district-types.ts'
import { routeProfile } from '../../../tools/district-plan.ts'
const data: District = source
const blocks=data.blocks??[]
const places=(block: string)=>data.places.filter(p=>p.block===block && p.featured)
const parcel=(id?: string)=>data.parcels.find(p=>p.id===id)
const arrivalSummary=(arrival: {label: string; level: string; nodes: string[]})=>`${arrival.label} · ${arrival.level} · 高程 ${data.nodes[arrival.nodes.at(-1)!][2]} m · 接入后 ${Math.round(routeProfile(data.nodes,arrival.nodes).at(-1)!.distance)} m`
</script>

<template>
  <div class="place-program">
    <section v-for="block in blocks" :key="block.id" :aria-label="block.name">
      <h3 :id="block.id">{{ block.id }} · {{ block.name }}</h3>
      <p>{{ block.role }}</p>
      <div class="place-list">
        <article v-for="p in places(block.id)" :key="p.id" :id="`place-${p.id}`">
          <h4><span>{{ p.id }}</span> {{ p.name }} <small v-if="p.detail">重点</small></h4>
          <p>{{ p.use }} · {{ p.users }}</p>
          <dl>
            <dt>到达</dt><dd>{{ p.entry }}</dd>
            <template v-if="p.arrivals?.public"><dt>主入口</dt><dd>{{ arrivalSummary(p.arrivals.public) }}</dd></template>
            <template v-if="p.arrivals?.service"><dt>服务口</dt><dd>{{ arrivalSummary(p.arrivals.service) }}</dd></template>
            <dt>活动</dt><dd>{{ p.activity }}</dd>
            <dt>时段</dt><dd>{{ p.time }}</dd>
            <dt>运营</dt><dd>{{ p.operations }}</dd>
            <dt>空间</dt><dd>{{ p.parcel }} · {{ parcel(p.parcel)?.use }}<template v-if="p.parent"> · 与 {{ data.places.find(q=>q.id===p.parent)?.name }} 共址，独立使用</template></dd>
          </dl>
          <details v-if="p.brief"><summary>空间设计简报</summary>
            <dl><dt>布局</dt><dd>{{ p.brief.space }}</dd><dt>动线</dt><dd>{{ p.brief.flow }}</dd><dt>后勤</dt><dd>{{ p.brief.service }}</dd><dt>开放</dt><dd>{{ p.brief.public }}</dd><dt>依据</dt><dd>{{ p.brief.reference.join('、') }} · 见城市空间研究</dd></dl>
          </details>
        </article>
      </div>
    </section>
  </div>
</template>

<style scoped>
.place-program section{margin:28px 0}.place-list{display:grid;gap:12px;grid-template-columns:repeat(2,minmax(0,1fr))}.place-list article{padding:16px;border:1px solid var(--vp-c-divider);border-radius:8px;background:var(--vp-c-bg-soft)}.place-list h4{margin:0 0 8px;font-size:16px}.place-list h4 span{font-weight:400;color:var(--vp-c-text-2)}.place-list small{font-size:11px;font-weight:400;color:var(--vp-c-brand-1)}.place-list p{font-size:13px;line-height:1.6;margin:8px 0}.place-list dl{display:grid;grid-template-columns:3em 1fr;gap:6px 8px;font-size:12px;line-height:1.7}.place-list dt{color:var(--vp-c-text-2);white-space:nowrap}.place-list dd{margin:0;overflow-wrap:anywhere}.place-list summary{cursor:pointer;font-size:13px;padding:10px 0;min-height:44px}@media(max-width:700px){.place-list{grid-template-columns:1fr}}
</style>
