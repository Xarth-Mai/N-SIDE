<script setup lang="ts">
import { computed, ref } from 'vue'
import { withBase } from 'vitepress'
import stories from '@wiki-data/stories.json'
import { acts, connections, effects, leaves, relationLabels, remaining, satisfied } from '../../../tools/story-graph.ts'
import StoryCondition from './StoryCondition.vue'
const props = defineProps<{ questId?: string; track?: 'main' | 'side' }>()
const selectedId = ref(props.questId ?? '')
const selected = computed(() => stories.find(n => n.id === selectedId.value))
const facts = ref(new Set<string>())
const simulate = ref(false)
const downstream = computed(() => effects(stories, selectedId.value))
const related = computed(() => connections(stories, selectedId.value))
const find = (id: string) => stories.find(n => n.id === id)!
function toggle(fact: string) { const next = new Set(facts.value); next.has(fact) ? next.delete(fact) : next.add(fact); facts.value = next }
const main = stories.filter(n => n.role === 'main')
const sides = (order: number) => stories.filter(n => n.role === 'side' && n.order >= order && n.order < order + 1)
function emphasis(id: string) {
  if(id === selectedId.value) return '正在查看'
  if(!selected.value) return ''
  if(selected.value.required.some(g => leaves(g.condition).some(f => f.quest_id === id))) return '强制条件来源'
  if(downstream.value.some(e => e.node.id === id)) return '后续条件受影响'
  if(selected.value.recommended.some(r => r.quest_id === id)) return '推荐先读'
  if(related.value.some(r => r.node.id === id)) return '剧情关联'
  return ''
}
</script>
<template>
  <section class="story-graph" aria-label="故事关系">
    <template v-if="!questId">
      <div class="journey-intro"><span>街坊的日子 · 城市的回声</span><strong>{{ track === 'side' ? '八条可选生活故事' : '从门还开着，到明天照常营业' }}</strong></div>
      <p class="caption">位置表示推荐阅读阶段，不代表任务锁定。选择故事查看关系；下方详情可进行条件推演，不读取或保存玩家进度</p>
      <div v-if="track" class="story-list">
        <article v-for="node in stories.filter(n => n.role === track)" :key="node.id">
          <button :aria-pressed="selectedId === node.id" @click="selectedId = node.id">{{ node.chapter }} · {{ node.title }}</button>
          <p>{{ node.summary }}</p><a :href="withBase(node.url)">阅读正文 ↗</a>
        </article>
      </div>
      <template v-else>
        <nav class="act-nav" aria-label="跳转故事幕"><a v-for="(act, i) in acts" :key="act" :href="`#story-act-${i+1}`">{{ ['Ⅰ','Ⅱ','Ⅲ'][i] }} {{ act }}</a></nav>
        <div class="timeline" tabindex="0" aria-label="横向故事时间轴，可左右滚动">
          <section v-for="(act, i) in acts" :id="`story-act-${i+1}`" :key="act" class="act">
            <h2><span>{{ ['Ⅰ','Ⅱ','Ⅲ'][i] }}</span> {{ act }}</h2>
            <div class="columns">
              <div v-for="node in main.filter(n => n.act === i+1)" :key="node.id" class="column">
                <article class="card main-card" :class="{ selected: selectedId === node.id, linked: emphasis(node.id) }">
                  <div class="card-art" aria-hidden="true"><span>{{ node.chapter }}</span><i></i></div>
                  <button :aria-pressed="selectedId === node.id" aria-controls="story-detail" @click="selectedId = node.id"><small>主线 · {{ node.chapter }}</small><strong>{{ node.title }}</strong></button>
                  <p>{{ node.summary }}</p><a :href="withBase(node.url)">阅读正文 ↗</a><span v-if="emphasis(node.id)" class="relation-badge">{{ emphasis(node.id) }}</span>
                </article>
                <div class="axis" aria-hidden="true"><span>◆</span></div>
                <div class="side-stories">
                  <article v-for="side in sides(node.order)" :key="side.id" class="card side-card" :class="{selected:selectedId===side.id,linked:emphasis(side.id)}">
                    <button :aria-pressed="selectedId === side.id" aria-controls="story-detail" @click="selectedId = side.id"><small>可选支线</small><strong>{{ side.title }}</strong></button>
                    <p>{{ side.summary }}</p><small>{{ side.placement }}</small><a :href="withBase(side.url)">阅读正文 ↗</a><span v-if="emphasis(side.id)" class="relation-badge">{{ emphasis(side.id) }}</span>
                  </article>
                </div>
              </div>
            </div>
          </section>
        </div>
        <p class="caption">横向滚动浏览三幕 · 支线可跨阶段体验，卡片附近的位置不是唯一归属</p>
      </template>
    </template>
    <a v-if="selected && !questId" href="#story-detail">查看「{{ selected.title }}」的故事关系 ↓</a>
    <div v-if="selected" id="story-detail" class="detail">
      <header><small>{{ selected.role === 'main' ? '主线' : '可选支线' }} · {{ selected.placement }}</small><h2>{{ selected.title }} · 故事关系</h2><p>{{ selected.summary }}</p><a :href="withBase(selected.url)">阅读完整故事 ↗</a></header>
      <p class="caption">以下展示已登记的故事条件；没有硬锁不等于所有会面与场所时机已经确定</p>
      <div class="simulation"><label><input v-model="simulate" type="checkbox"> 条件推演（假设，不是存档进度）</label><button v-if="simulate" @click="facts = new Set()">清空假设</button><p v-if="simulate">勾选具体成果，查看仍缺的条件。完成来源任务不自动满足所有阶段、分支或实时状态</p></div>
      <div class="detail-grid">
        <section><h3>强制前置</h3><p v-if="!selected.required.length">正文未规定跨任务硬锁</p>
          <article v-for="gate in selected.required" :key="gate.scope"><h4>{{ gate.scope === 'start' ? '开始条件' : (gate.required_for_completion ? '必需阶段：' : '可选片段：')+gate.scope }} · {{ gate.label }}</h4><StoryCondition :condition="gate.condition" :facts="facts" :simulate="simulate" @toggle="toggle" /></article>
        </section>
        <section><h3>推荐先体验</h3><p v-if="!selected.recommended.length">没有额外推荐</p><ul><li v-for="r in selected.recommended" :key="r.quest_id"><a :href="withBase(find(r.quest_id).url)">{{ find(r.quest_id).title }}</a><p>{{ r.reason }} · 不阻塞</p></li></ul></section>
        <section><h3>剧情关联</h3><p v-if="!related.length">没有额外登记</p><ul><li v-for="(r,i) in related" :key="i"><a :href="withBase(r.node.url)">{{ r.node.title }}</a> · {{ relationLabels[r.type] }}<p>{{ r.description }}</p></li></ul></section>
        <section><h3>后续条件影响</h3><p v-if="!downstream.length">没有已登记的后续硬条件；剧情关联和推荐关系仍可延续</p>
          <article v-for="({node, gate},i) in downstream" :key="i"><h4><a :href="withBase(node.url)">{{ node.title }}</a> · {{ gate.scope === 'start' ? '开始' : gate.scope }}</h4>
            <p>本故事提供：{{ leaves(gate.condition).filter(f=>f.quest_id === selectedId).map(f=>f.label).join('；') }}</p>
            <p v-if="simulate" class="result">{{ satisfied(gate.condition,facts) ? '假设下此门槛满足；不代表整个任务已解锁' : '仍缺下列条件' }}</p>
            <StoryCondition v-if="!simulate || remaining(gate.condition,facts)" :condition="simulate ? remaining(gate.condition,facts)! : gate.condition" :facts="facts" :simulate="false" />
            <details v-if="simulate"><summary>选择此门槛的假设成果</summary><StoryCondition :condition="gate.condition" :facts="facts" :simulate="true" @toggle="toggle" /></details>
          </article>
        </section>
      </div>
    </div>
    <p v-else class="empty">选择一张卡片，展开它的前置、关联与后续条件</p>
  </section>
</template>
<style scoped>
.story-graph { margin: 2rem 0; --story-gold: #a47d3b; }
.journey-intro { padding: 1.6rem; background: linear-gradient(115deg,var(--vp-c-bg-soft),var(--vp-c-brand-soft)); border-top: 2px solid var(--story-gold); }
.journey-intro span,.journey-intro strong { display: block }.journey-intro span { font-size: .8rem; letter-spacing: .2em; color: var(--vp-c-text-2) }.journey-intro strong { font-size: 1.45rem; margin-top: .6rem; line-height: 1.5 }
.caption,.card small { color: var(--vp-c-text-2); font-size: .82rem; line-height: 1.6 }
.act-nav { display: flex; flex-wrap: wrap; gap: .65rem; margin: 1rem 0 }.act-nav a { border: 1px solid var(--vp-c-divider); padding: .5rem .8rem; border-radius: .4rem; font-size: .85rem }
.timeline { display: flex; overflow-x: auto; gap: 1.5rem; padding: .5rem 0 1rem; scroll-behavior: smooth; scroll-padding: 1rem }
.act { flex-shrink: 0; scroll-margin: 1rem }.act h2 { font-size: 1.05rem; margin: .5rem 0 1rem; border: 0 }.act h2 span { color: var(--story-gold); margin-right: .5rem }.columns { display: flex; gap: 1rem }.column { width: 238px }.card { background: var(--vp-c-bg-soft); border: 1px solid var(--vp-c-divider); border-radius: .5rem; overflow: hidden; padding: 0 1rem 1rem; position: relative }.main-card { min-height: 326px }.card-art { height: 90px; margin: 0 -1rem .8rem; overflow: hidden; position: relative; background: linear-gradient(145deg,var(--vp-c-brand-soft),var(--vp-c-bg-alt)); border-bottom: 1px solid var(--vp-c-divider) }.card-art span { font-size: 2.9rem; opacity: .16; position: absolute; right: .65rem; bottom: 1rem; font-family: serif }.card-art i { position: absolute; width: 60px; height: 85px; left: 22px; top: 32px; border: 1px solid var(--story-gold); border-radius: 30px 30px 0 0; transform: rotate(-12deg) }.card button { display: block; width: 100%; text-align: left; cursor: pointer; padding: .3rem 0 }.card button small,.card button strong { display:block }.card button strong { font-size: 1.1rem; margin-top:.3rem }.card p { font-size:.85rem; line-height:1.75; margin:.55rem 0 }.card a { display:inline-block; font-size:.82rem; margin-top:.45rem }.axis { height:40px; display:flex; align-items:center; color:var(--story-gold); position:relative }.axis:before { content:''; position:absolute; height:1px; background:var(--story-gold); width:calc(100% + 1rem) }.axis span { z-index:1; background:var(--vp-c-bg); padding:0 .4rem; margin-left:1rem }.side-stories { display:grid; gap:.85rem }.side-card { padding-top:.65rem }.side-card>small { display:block }.selected { outline: 2px solid var(--vp-c-brand-1); outline-offset:-2px }.linked { border-color:var(--story-gold) }.relation-badge { display:block; font-size:.75rem; margin-top:.7rem; color:var(--vp-c-brand-1) }.detail { border:1px solid var(--vp-c-divider); border-radius:.7rem; padding:1.3rem; margin-top:1rem }.detail header h2 { margin:.5rem 0; border:0; padding:0; font-size:1.4rem }.detail header small { color:var(--vp-c-text-2) }.detail-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:1.4rem }.detail-grid h3 { font-size:1rem; border-bottom:1px solid var(--vp-c-divider); padding-bottom:.5rem }.detail-grid h4 { font-size:.9rem }.detail-grid p,.detail-grid li { font-size:.9rem; line-height:1.75 }.detail-grid article+article { margin-top:1rem }.detail-grid li p { margin:.2rem 0 .7rem }.simulation { background:var(--vp-c-bg-soft); padding:.8rem; margin:1rem 0; font-size:.85rem }.simulation button { padding:.2rem .5rem; border:1px solid var(--vp-c-divider); margin-left:.6rem; border-radius:.3rem }.simulation p { margin:.4rem 0 0 }.result { color:var(--vp-c-brand-1) }.empty { text-align:center; padding:1.5rem; background:var(--vp-c-bg-soft); color:var(--vp-c-text-2) }.story-list { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:1rem }.story-list article { border:1px solid var(--vp-c-divider); padding:1rem; border-radius:.4rem }.story-list button { font-weight:600; text-align:left; cursor:pointer }.story-list p,.story-list a { font-size:.85rem } button:focus-visible,a:focus-visible,input:focus-visible,.timeline:focus-visible { outline:3px solid var(--vp-c-brand-1); outline-offset:3px }
@media(max-width:640px) { .detail-grid,.story-list { grid-template-columns:1fr }.detail { padding:1rem }.column { width: min(76vw,260px) }.journey-intro strong { font-size:1.15rem }.act-nav a { font-size:.78rem }.timeline { scroll-snap-type:x proximity }.column { scroll-snap-align:start } }
@media(prefers-reduced-motion:reduce) { .timeline { scroll-behavior:auto } }
</style>
