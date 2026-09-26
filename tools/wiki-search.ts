import type { Plugin } from 'vite'

/** VitePress 1.6.4 exposes one lazy loader per locale; keep its string-index contract */
export function searchIndexAsset(): Plugin {
  return {
    name: 'n-side-search-asset',
    apply: 'build',
    async transform(code, id, options) {
      if (id !== '/@localSearchIndex' || options?.ssr) return
      if (code !== `export default {"root": () => import('@localSearchIndexroot')}`) {
        throw new Error('VitePress local search loader changed; review the pinned search asset adapter')
      }
      const module = await this.load({ id: '/@localSearchIndexroot' })
      if (!module.code?.startsWith('export default ')) throw new Error('Missing VitePress search index')
      const index: unknown = JSON.parse(module.code.slice('export default '.length))
      if (typeof index !== 'string') throw new Error('Expected a serialized VitePress search index')
      const parsed: unknown = JSON.parse(index)
      if (!parsed || typeof parsed !== 'object' || !('documentCount' in parsed)) throw new Error('Invalid MiniSearch index')
      const reference = this.emitFile({ type: 'asset', name: 'search-index.json', source: index })
      return {
        code: `export default {root: async () => {
          const response = await fetch(import.meta.ROLLUP_FILE_URL_${reference})
          if (!response.ok) throw new Error('Search index request failed: ' + response.status)
          return {default: await response.text()}
        }}`,
        map: null,
      }
    },
  }
}
