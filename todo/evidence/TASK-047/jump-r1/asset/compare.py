"""Compare exported character data with the retained pre-Jump local GLB snapshots"""
from pathlib import Path
import sys,json,hashlib
ROOT=Path(__file__).resolve().parents[5]
sys.path.insert(0,str(ROOT/'tools'))
from glb import read_glb,values

def normalized(path):
 d,b=read_glb(path)
 def accessor(i):return values(d,b,d['accessors'][i])
 def view(i):
  v=d['bufferViews'][i];start=v.get('byteOffset',0);return hashlib.sha256(b[start:start+v['byteLength']]).hexdigest()
 meshes=[{'name':m.get('name'),'primitives':[{'attributes':{k:accessor(v) for k,v in p['attributes'].items()},'indices':accessor(p['indices']),'material':p.get('material')} for p in m['primitives']]} for m in d['meshes']]
 skins=[dict(joints=s['joints'],inverse_bind=accessor(s['inverseBindMatrices'])) for s in d['skins']]
 animations={a['name']:[{'target':c['target'],'input':accessor(a['samplers'][c['sampler']]['input']),'output':accessor(a['samplers'][c['sampler']]['output']),'interpolation':a['samplers'][c['sampler']].get('interpolation','LINEAR')} for c in a['channels']] for a in d['animations']}
 return dict(nodes=d['nodes'],meshes=meshes,skins=skins,materials=d['materials'],textures=d['textures'],images=[view(i['bufferView']) for i in d['images']],animations=animations)
report={}
for cid,stem in [('CHR-001','yao'),('CHR-002','ling')]:
 before=ROOT/'output/jump-r1/before'/cid/(stem+'-grey-study.glb');after=ROOT/'game/assets/characters'/cid/(stem+'-grey-study.glb')
 a,b=normalized(before),normalized(after)
 checks={key:a[key]==b[key] for key in ['nodes','meshes','skins','materials','textures','images']}
 checks.update({name:a['animations'][name]==b['animations'][name] for name in ['Idle','Walk','Run']})
 checks['only_Jump_added']=set(b['animations'])-set(a['animations'])=={'Jump'}
 report[cid]={'checks':checks,'after_sha256':hashlib.sha256(after.read_bytes()).hexdigest()}
 assert all(checks.values()),(cid,checks)
(ROOT/'todo/evidence/TASK-047/jump-r1/asset/preserved-data.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
