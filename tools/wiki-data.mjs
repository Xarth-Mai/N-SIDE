import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, realpathSync } from 'node:fs'
import { dirname, extname, isAbsolute, join, relative, resolve, sep } from 'node:path'

const TYPES = { '.json': 'application/json; charset=utf-8', '.csv': 'text/csv; charset=utf-8' }
export const PROFILES = ['player', 'dev']
export function assertProfile(profile) {
  if (!PROFILES.includes(profile)) throw new Error(`Unknown Wiki profile: ${profile}`)
}
export function within(root, path) {
  const name = relative(root, path)
  return !isAbsolute(name) && name !== '..' && !name.startsWith(`..${sep}`)
}
export function filesIn(root) {
  if (!existsSync(root)) return []
  return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, 'en')).flatMap(entry => {
    if (entry.name.startsWith('.') || entry.isSymbolicLink()) return []
    const file = join(root, entry.name)
    return entry.isDirectory() ? filesIn(file) : entry.isFile() ? [file] : []
  })
}

// Raw design data is dev-only; player data is an explicit generated map projection
export function listWikiData(root, profile) {
  assertProfile(profile)
  return profile === 'dev' ? filesIn(join(root, 'dev')).filter(file => TYPES[extname(file)]) : []
}
export function copyWikiData(root, output, profile) {
  const files = listWikiData(root, profile)
  for (const file of files) {
    const target = join(output, relative(root, file))
    mkdirSync(dirname(target), { recursive: true })
    copyFileSync(file, target)
  }
  return files.length
}
export function wikiDataPlugin(sourceRoot, profile) {
  const root = realpathSync(sourceRoot)
  const allowed = new Set(listWikiData(root, profile).map(file => resolve(file)))
  return {
    name: 'n-side-wiki-data',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (!['GET', 'HEAD'].includes(req.method)) return next()
        try {
          const url = new URL(req.url ?? '/', 'http://localhost')
          if (url.searchParams.has('import')) return next()
          const file = resolve(root, decodeURIComponent(url.pathname).replace(/^\/+/, ''))
          if (!allowed.has(file) || !within(root, realpathSync(file))) return next()
          res.setHeader('Content-Type', TYPES[extname(file)])
          res.end(req.method === 'HEAD' ? undefined : readFileSync(file))
        } catch (error) { next(error) }
      })
    },
  }
}
