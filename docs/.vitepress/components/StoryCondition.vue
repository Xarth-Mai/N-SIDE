<script setup lang="ts">
import { computed } from 'vue'
import { withBase } from 'vitepress'
import stories from '@wiki-data/stories.json'
import type { Condition } from '../../../tools/story-graph.ts'
import { satisfied } from '../../../tools/story-graph.ts'
const props = defineProps<{ condition: Condition; facts: Set<string>; simulate: boolean }>()
const source = computed(() => {
  const condition = props.condition
  return 'quest_id' in condition ? stories.find(n => n.id === condition.quest_id) : undefined
})
defineEmits<{ toggle: [fact: string] }>()
</script>
<template>
  <div class="condition">
    <template v-if="'all' in condition || 'any' in condition">
      <strong>{{ 'all' in condition ? '全部满足' : '任意一项满足' }}</strong>
      <span v-if="simulate"> · {{ satisfied(condition, facts) ? '假设下已满足' : '仍缺条件' }}</span>
      <ul>
        <li v-for="(child, i) in ('all' in condition ? condition.all : condition.any)" :key="i">
          <StoryCondition :condition="child" :facts="facts" :simulate="simulate" @toggle="$emit('toggle', $event)" />
        </li>
      </ul>
    </template>
    <template v-else>
      <label v-if="simulate"><input type="checkbox" :checked="facts.has(condition.fact)" @change="$emit('toggle', condition.fact)"> {{ condition.label }}</label>
      <span v-else>{{ condition.label }}</span>
      <small>{{ condition.reason }}</small>
      <a v-if="source" :href="withBase(source.url)">来源：{{ source.title }}</a>
    </template>
  </div>
</template>
<style scoped>
.condition { font-size: .92rem; line-height: 1.7 }
.condition ul { padding-left: 1.1rem; margin: .4rem 0 }
.condition small { display: block; color: var(--vp-c-text-2); margin-top: .15rem }
.condition a { font-size: .8rem }
.condition label { cursor: pointer }
.condition input { accent-color: var(--vp-c-brand-1); margin-right: .25rem }
</style>
