import {buildGround} from '../../../../tools/district-map.ts'
import map from '../../../../source-assets/district-map/district.json'
const ground=buildGround(map as any)
const log=await Bun.file(new URL('../forest-r1/native-mature-tests.log',import.meta.url)).text()
const groups=[['hill_short_rest2',24],['hill_short_rest3',26],['hill_east_curve_08',24]] as const
for(const [group,[id,extent]] of groups.entries()){
 const trees=[...log.matchAll(/forest placement nodes\[([^\]]+)\].* x=([\d.]+) north=([\d.]+) root=([\d.]+) scale=([\d.]+)/g)].filter(m=>m[1]===id).map(m=>({p:[+m[2],+m[3]],r:+m[5]*2.3}))
 const center=[0,1].map(i=>trees.reduce((s,t)=>s+t.p[i],0)/trees.length),anchor=(map.nodes as any)[id],len=Math.hypot(center[0]-anchor[0],center[1]-anchor[1]),d=[(center[0]-anchor[0])/len,(center[1]-anchor[1])/len]
 for(const [patch,side] of [-1,1].entries()){
  const lobe=[center[0]-d[1]*extent*.65*side+d[0]*2,center[1]+d[0]*extent*.65*side+d[1]*2]
  const slots=[]
  for(let slot=0;slot<80;slot+=5){
   const scale=1.4+((slot*7+group*3+patch*5)%9)/8*.4,a=slot*2.399963+group*.6+patch,r=6*Math.sqrt((slot+.5)/80),p=[lobe[0]+r*Math.cos(a),lobe[1]+r*Math.sin(a)]
   const heights=Array.from({length:16},(_,i)=>ground.height(p[0]+.1*scale*Math.cos(i*Math.PI/8),p[1]+.1*scale*Math.sin(i*Math.PI/8)))
   const treeGap=Math.min(...trees.map(t=>Math.hypot(p[0]-t.p[0],p[1]-t.p[1])-t.r-.7*scale-.25))
   slots.push({slot,p,treeGap,rootSpan:Math.max(...heights)-Math.min(...heights)})
  }
  console.log(JSON.stringify({group,patch,lobe,shrubCandidates:slots.length,clearOfTrees:slots.filter(s=>s.treeGap>=0),supported:slots.filter(s=>s.rootSpan<=.28).length,evidence:'TS source-only probe; not native or full obstacle validation'}))
 }
}
