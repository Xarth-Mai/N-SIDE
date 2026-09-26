import { Glob } from 'bun'
import assert from 'node:assert/strict'
import MiniSearch from 'minisearch'

const results=[]
for(const profile of ['player','dev']){
  const directory=`output/wiki/${profile}/dist/assets`
  const files=[...new Glob('search-index.*.json').scanSync(directory)]
  assert.equal(files.length,1)
  const text=await Bun.file(`${directory}/${files[0]}`).text()
  const index=MiniSearch.loadJSON(text,{fields:['title','titles','text'],storeFields:['title','titles']})
  const hits=index.search('最后的玩具',{prefix:true})
  assert(hits.some(hit=>hit.id.startsWith('/player/encyclopedia/story/main/last-toy#')))
  if(profile==='player')assert(Object.values(JSON.parse(text).documentIds).every(id=>typeof id==='string'&&(id.startsWith('/player/')||id.startsWith('/#'))))
  results.push({profile,documents:index.documentCount,hitCount:hits.length,hits:hits.slice(0,3).map(({id,title})=>({id,title}))})
}
await Bun.write(new URL('./search.json',import.meta.url),JSON.stringify(results,null,2)+'\n')
console.log('PASS: full-text indices load and preserve audience-specific result anchors')
