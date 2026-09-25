import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { basename, join, relative, sep } from 'node:path'

const readingOrder = {
  world: ['null-city.md', 'nightmares.md', 'history.md'],
  characters: ['family.md', 'brother.md', 'sister.md', 'agent.md'],
  locations: ['n-district.md', 'shop.md', 'stargazing-terrace.md'],
  gameplay: ['controls.md', 'daily-life.md', 'levels.md', 'dream-diving.md', 'combat.md'],
  narrative: ['workflow.md', 'data.md', 'examples', 'playtest.md'],
  production: ['workflow.md', 'gameplay.md', 'district-space.md', 'art.md', 'audio.md', 'assets.md', 'engineering.md', 'wiki.md'],
  templates: ['character.md', 'location.md', 'enemy.md', 'daily-scene.md', 'quest.md', 'case-development.md', 'narrative-playtest.md', 'continuity-review.md', 'task.md'],
}

/** Read the visible heading used by the Wiki navigation. */
function title(file) {
  const text = readFileSync(file, 'utf8')
  return (text.match(/^#\s+(.+)$/m)?.[1] ?? basename(file, '.md'))
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/\s+\{#[^}]+\}\s*$/, '')
}

function page(root, file) {
  let route = relative(root, file).split(sep).join('/').replace(/\.md$/, '')
  route = route === 'index' ? '' : route.replace(/\/index$/, '/')
  return { text: title(file), link: `/${route}` }
}

function section(root, directory, label) {
  if (!existsSync(directory)) return null
  const order = readingOrder[relative(root, directory)] ?? []
  const rank = name => order.includes(name) ? order.indexOf(name) : order.length
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
      const nested = section(root, file, entry.name === 'examples' ? '示例' : entry.name)
      if (nested) items.push(nested)
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      items.push(page(root, file))
    }
  }
  if (!landing && !items.length) return null
  const result = { text: label ?? (landing ? title(landing) : basename(directory)) }
  if (landing) result.link = page(root, landing).link
  if (items.length) Object.assign(result, { collapsed: true, items })
  return result
}

/** Build the player-facing and developer-facing navigation groups. */
export function buildSidebar(root) {
  const groups = [
    ['游戏指南', [['世界', 'world'], ['人物', 'characters'], ['地点', 'locations'], ['敌人', 'enemies'], ['玩法', 'gameplay']]],
    ['开发', [['叙事', 'narrative'], ['任务', 'quests'], ['制作', 'production'], ['模板', 'templates']]],
  ]
  return [
    {
      text: 'Overview',
      link: '/',
    },
    ...groups.map(([text, sections]) => {
      const items = sections.map(([label, directory]) => section(root, join(root, directory), label)).filter(Boolean)
      if (text === '开发' && existsSync(join(root, 'conventions.md'))) {
        items.unshift({ ...page(root, join(root, 'conventions.md')), text: '项目约定' })
      }
      return { text, items }
    }),
  ]
}
