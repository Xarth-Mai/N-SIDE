import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, realpathSync, statSync } from 'node:fs'
import { dirname, extname, isAbsolute, join, relative, resolve, sep } from 'node:path'

const TYPES = { '.json': 'application/json; charset=utf-8', '.csv': 'text/csv; charset=utf-8' }
const EXCLUDED = new Set(['public', 'node_modules'])

function within(root, path) {
  const name = relative(root, path)
  return !isAbsolute(name) && name !== '..' && !name.startsWith(`..${sep}`) && !name.startsWith(sep)
}

/** List source data that accompanies the Wiki's Markdown pages. */
export function listWikiData(root) {
  const files = []
  function walk(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.name.startsWith('.') || EXCLUDED.has(entry.name)) continue
      const file = join(directory, entry.name)
      if (entry.isDirectory()) walk(file)
      else if (entry.isFile() && TYPES[extname(file)]) files.push(file)
    }
  }
  walk(root)
  return files.sort()
}

/** Export data with its existing relative URL after the VitePress build. */
export function copyWikiData(root, output) {
  const files = listWikiData(root)
  for (const file of files) {
    const target = join(output, relative(root, file))
    mkdirSync(dirname(target), { recursive: true })
    copyFileSync(file, target)
  }
  return files.length
}

/** Serve the same source data when browsing the local VitePress dev server. */
export function wikiDataPlugin(sourceRoot) {
  const root = realpathSync(sourceRoot)
  return {
    name: 'n-side-wiki-data',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (!['GET', 'HEAD'].includes(req.method)) return next()
        try {
          const url = new URL(req.url ?? '/', 'http://localhost')
          if (url.searchParams.has('import')) return next()
          const name = decodeURIComponent(url.pathname).replace(/^\/+/, '')
          if (name.split('/').some(part => part.startsWith('.') || EXCLUDED.has(part))) return next()
          const type = TYPES[extname(name)]
          const file = resolve(root, name)
          if (!type || !within(root, file) || !existsSync(file) || !statSync(file).isFile()) return next()
          if (!within(root, realpathSync(file))) return next()
          res.setHeader('Content-Type', type)
          res.end(req.method === 'HEAD' ? undefined : readFileSync(file))
        } catch (error) {
          next(error)
        }
      })
    },
  }
}

/** Export project-level assets under a stable Wiki URL. */
export function copyProjectAssets(sourceRoot, output) {
  const publicRoot = join(output, 'project-assets')
  mkdirSync(publicRoot, { recursive: true })
  chmodSync(publicRoot, 0o755)
  for (const asset of sourceRoot) {
    const target = join(publicRoot, asset.path)
    mkdirSync(dirname(target), { recursive: true })
    chmodSync(dirname(target), 0o755)
    copyFileSync(asset.source, target)
    chmodSync(target, 0o644)
  }
  return sourceRoot.length
}

/** Serve project-level assets without keeping a second copy under docs/. */
export function projectAssetsPlugin(sourceRoot) {
  const assets = new Map(sourceRoot.map(asset => [`/${asset.path}`, realpathSync(asset.source)]))
  return {
    name: 'n-side-project-assets',
    configureServer(server) {
      server.middlewares.use('/project-assets', (req, res, next) => {
        if (!['GET', 'HEAD'].includes(req.method)) return next()
        try {
          const name = decodeURIComponent(new URL(req.url ?? '/', 'http://localhost').pathname)
          const file = assets.get(name)
          if (!file || !statSync(file).isFile()) return next()
          if (extname(file) === '.svg') res.setHeader('Content-Type', 'image/svg+xml')
          res.end(req.method === 'HEAD' ? undefined : readFileSync(file))
        } catch (error) {
          next(error)
        }
      })
    },
  }
}
