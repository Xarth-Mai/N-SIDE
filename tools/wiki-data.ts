import type { ViteDevServer } from 'vite'
export type WikiProfile = 'player' | 'dev'
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, realpathSync } from 'node:fs'
import { dirname, extname, isAbsolute, join, relative, resolve, sep } from 'node:path'

const TYPES: Record<string, string> = { '.json': 'application/json; charset=utf-8', '.csv': 'text/csv; charset=utf-8' }
export function wikiOutput(root: string, profile: WikiProfile) {
  return resolve(root, 'docs/.vitepress', profile === 'dev' ? 'dist' : 'dist-player')
}

export const PROFILES = ['player', 'dev']
export function assertProfile(profile: unknown): asserts profile is WikiProfile {
  if (typeof profile !== 'string' || !PROFILES.includes(profile)) throw new Error(`Unknown Wiki profile: ${profile}`)
}
export function within(root: string, path: string) {
  const name = relative(root, path)
  return !isAbsolute(name) && name !== '..' && !name.startsWith(`..${sep}`)
}
export function filesIn(root: string): string[] {
  if (!existsSync(root)) return []
  return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, 'en')).flatMap(entry => {
    if (entry.name.startsWith('.') || entry.isSymbolicLink()) return []
    const file = join(root, entry.name)
    return entry.isDirectory() ? filesIn(file) : entry.isFile() ? [file] : []
  })
}

// Raw design data is dev-only; player data is an explicit generated map projection
export function listWikiData(root: string, profile?: unknown) {
  assertProfile(profile)
  return profile === 'dev' ? filesIn(join(root, 'dev')).filter(file => TYPES[extname(file)]) : []
}
export function copyWikiData(root: string, output: string, profile: WikiProfile) {
  const files = listWikiData(root, profile)
  for (const file of files) {
    const target = join(output, relative(root, file))
    mkdirSync(dirname(target), { recursive: true })
    copyFileSync(file, target)
  }
  return files.length
}
export function wikiDataPlugin(sourceRoot: string, profile: WikiProfile) {
  const root = realpathSync(sourceRoot)
  const allowed = new Set(listWikiData(root, profile).map(file => resolve(file)))
  return {
    name: 'n-side-wiki-data',
    configureServer(server: ViteDevServer) {
      server.middlewares.use((req, res, next) => {
        if (!['GET', 'HEAD'].includes(req.method ?? '')) return next()
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
