import type { StoryNode } from '../../tools/story-graph.ts'
import type { DefaultTheme } from 'vitepress'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { basename, join, relative, sep } from 'node:path'

const readingOrder: Record<string, string[]> = {
  story: ['index.md', 'main', 'daily'],
  world: ['null-city.md', 'nightmares.md', 'great-shared-dream.md', 'history.md'],
  characters: ['family.md', 'brother.md', 'sister.md', 'agent.md'],
  locations: ['n-district.md', 'shop.md', 'stargazing-terrace.md', 'place-network.md', 'places'],
  gameplay: ['controls.md', 'daily-life.md', 'levels.md', 'dream-diving.md', 'combat.md'],
  narrative: ['workflow.md', 'data.md', 'examples', 'playtest.md'],
  production: ['workflow.md', 'gameplay.md', 'district-space.md', 'district-plan.md', 'district-architecture.md', 'district-station.md', 'district-waterfront.md', 'district-places.md', 'world-research.md', 'art.md', 'audio.md', 'assets.md', 'engineering.md', 'wiki.md'],
  templates: ['character.md', 'location.md', 'enemy.md', 'daily-scene.md', 'quest.md', 'case-development.md', 'narrative-playtest.md', 'continuity-review.md', 'task.md'],
}

/** Read the visible heading used by the Wiki navigation. */
function title(file: string) {
  const text = readFileSync(file, 'utf8')
  return (text.match(/^#\s+(.+)$/m)?.[1] ?? basename(file, '.md'))
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/\s+\{#[^}]+\}\s*$/, '')
}

function page(root: string, file: string) {
  let route = relative(root, file).split(sep).join('/').replace(/\.md$/, '')
  route = route === 'index' ? '' : route.replace(/\/index$/, '/')
  return { text: title(file), link: `/${route}` }
}

function section(root: string, directory: string, label?: string): DefaultTheme.SidebarItem | null {
  if (!existsSync(directory)) return null
  const key = relative(root, directory).replace(/^player\//, '')
  const storyData = join(root, '_data/stories.json')
  const stories: StoryNode[] = ['story/main', 'story/daily'].includes(key) && existsSync(storyData) ? JSON.parse(readFileSync(storyData, 'utf8')) : []
  const order = key === 'story/main' || key === 'story/daily'
    ? ['index.md', ...stories.filter(n => n.role === (key === 'story/main' ? 'main' : 'side')).sort((a,b)=>a.order-b.order).map(n=>basename(n.url)+'.md')]
    : readingOrder[key] ?? []
  const rank = (name: string) => order.includes(name) ? order.indexOf(name) : order.length
  const entries = readdirSync(directory, { withFileTypes: true })
    .filter(entry => !entry.name.startsWith('.'))
    .sort((a, b) => rank(a.name) - rank(b.name) || a.name.localeCompare(b.name, 'en'))
  const landing = ['index.md', 'README.md']
    .map(name => join(directory, name)).find(file => existsSync(file))
  const items = []
  for (const entry of entries) {
    const file = join(directory, entry.name)
    if (file === landing) continue
    if (entry.isDirectory()) {
      const nested = section(root, file, entry.name === 'examples' ? '示例' : undefined)
      if (nested) items.push(nested)
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      items.push(page(root, file))
    }
  }
  if (!landing && !items.length) return null
  const result: DefaultTheme.SidebarItem = { text: label ?? (landing ? title(landing) : basename(directory)) }
  if (landing) result.link = page(root, landing).link
  if (items.length) Object.assign(result, { collapsed: true, items })
  return result
}

/** Build encyclopedia and production navigation groups. */
export function buildSidebar(root: string, profile = 'dev') {
  const encyclopedia = ['world', 'characters', 'locations', 'story', 'enemies', 'gameplay']
  const labels = ['世界', '人物', '地点', '故事', '敌人', '玩法']
  const groups = [
    { text: '玩家资料', items: [
      page(root, join(root, 'player/index.md')),
      section(root, join(root, 'player/guide'), '操作指南'),
      ...encyclopedia.map((name, index) => section(root, join(root, 'player', name), labels[index])),
    ].filter(item => item !== null) },
  ]
  if (profile === 'dev') groups.push({ text: '开发资料', items: [
    page(root, join(root, 'dev/index.md')),
    ...['direction', 'design', 'engineering', 'production', 'validation', 'handbook', 'decisions'].map((name, index) =>
      section(root, join(root, 'dev', name), ['方向', '设计', '工程', '制作', '验收', '开发手册', '决策'][index])),
  ].filter(item => item !== null) })
  return [{ text: 'N:SIDE', link: '/' }, ...groups]
}
