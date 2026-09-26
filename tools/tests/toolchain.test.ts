import { test, expect } from 'bun:test'
import { mkdtempSync, writeFileSync, rmSync, openSync, closeSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import type { Plugin } from 'vite'
import type { TransformPluginContext } from 'rollup'
import { searchIndexAsset } from '../wiki-search.ts'

const transform = searchIndexAsset().transform as Extract<Plugin['transform'], Function>

test('search adapter emits the original index and preserves the lazy loader contract', async () => {
  const index = JSON.stringify({documentCount: 1, storedFields: {'0': {title: '仅玩家可见'}}})
  let emitted: {type: string; source: string; name: string} | undefined
  const context = {
    load: async ({id}: {id: string}) => { expect(id).toBe('/@localSearchIndexroot'); return {code: `export default ${JSON.stringify(index)}`} },
    emitFile: (asset: typeof emitted) => { emitted=asset; return 'TEST' },
  } as unknown as TransformPluginContext
  const input = `export default {"root": () => import('@localSearchIndexroot')}`
  const result = await transform.call(context, input, '/@localSearchIndex', {})
  expect(emitted?.source).toBe(index)
  expect(emitted?.name).toBe('search-index.json')
  if (!result || typeof result === 'string') throw new Error('Missing transformed loader')
  const factory = new Function('fetch', result.code!.replace('export default', 'return').replace('import.meta.ROLLUP_FILE_URL_TEST', "'/search.json'"))
  let calls=0
  const loaders=factory(async (url: string) => { calls++;expect(url).toBe('/search.json');return new Response(index) })
  expect(calls).toBe(0)
  expect(await loaders.root()).toEqual({default:index})
  await expect(factory(async () => new Response('',{status:503})).root()).rejects.toThrow('503')
  expect(await transform.call(context,input,'/@localSearchIndex',{ssr:true})).toBeUndefined()
  await expect(transform.call(context,'changed API','/@localSearchIndex',{})).rejects.toThrow('loader changed')
})

test('Vue checker reports errors inside a real SFC instead of silently skipping it', async () => {
  const directory=mkdtempSync(join(tmpdir(),'nside-vue-check-'))
  try {
    writeFileSync(join(directory,'probe.vue'), '<script setup lang="ts">const value: number = "wrong"</script><template>{{ value }}</template>')
    writeFileSync(join(directory,'tsconfig.json'), JSON.stringify({extends:resolve('tsconfig.vue.json'),include:['probe.vue'],exclude:[],compilerOptions:{types:[]}}))
    const log=join(directory,'check.log'), descriptor=openSync(log,'w')
    try {
      const result=Bun.spawn(['bun','tools/check-vue.ts','-p',join(directory,'tsconfig.json')],{stdout:descriptor,stderr:descriptor})
      expect(await result.exited).not.toBe(0)
    } finally { closeSync(descriptor) }
    expect(readFileSync(log,'utf8')).toContain("Type 'string' is not assignable to type 'number'")
  } finally { rmSync(directory,{recursive:true,force:true}) }
})
