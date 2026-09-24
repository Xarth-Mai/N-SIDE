import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { basename, join, relative, sep } from 'node:path'

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
  const entries = readdirSync(directory, { withFileTypes: true })
    .filter(entry => !entry.name.startsWith('.'))
    .sort((a, b) => a.name.localeCompare(b.name, 'en'))
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

/** Build the four approved navigation groups from the current source files. */
export function buildSidebar(root) {
  const groups = [
    ['Universe', [['World', 'world'], ['Characters', 'characters'], ['Locations', 'locations'], ['Enemies', 'enemies']]],
    ['Game', [['Gameplay', 'gameplay'], ['Narrative', 'narrative'], ['Quests', 'quests']]],
    ['Development', [['Production', 'production'], ['Templates', 'templates']]],
  ]
  return [
    {
      text: 'Overview',
      items: [['Vision', 'vision.md'], ['Conventions', 'conventions.md']]
        .filter(([, file]) => existsSync(join(root, file)))
        .map(([text, file]) => ({ ...page(root, join(root, file)), text })),
    },
    ...groups.map(([text, sections]) => ({
      text,
      items: sections.map(([label, directory]) => section(root, join(root, directory), label)).filter(Boolean),
    })),
  ]
}
