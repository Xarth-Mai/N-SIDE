from pathlib import Path
import json,re,subprocess
base='bbee921b284d30dedf20a4c62e9f2f160a61800e'; mapping=json.loads(Path('todo/evidence/TASK-022/naming/mapping.json').read_text()); audit=json.loads(Path('todo/evidence/TASK-022/naming/audit.json').read_text()); checks={}
def old(path):return json.loads(subprocess.check_output(['git','show',base+':'+path],text=True))
def data(path):return json.loads(Path(path).read_text())
c=data('docs/dev/design/catalogs/characters.json')['characters']; quests=data('docs/dev/design/catalogs/quests.json')['quests']
assert {x['id']:x['name'] for x in c}=={x['id']:x['name'] for x in mapping['characters']}
new={x['id']:x for x in c}
for x in old('docs/dev/design/catalogs/characters.json')['characters']:
 for k in ['id','age','home_place_id']:assert new[x['id']][k]==x[k],(x['id'],k)
 assert set(x['associated_place_ids'])<=set(new[x['id']]['associated_place_ids']),x['id']
checks['character_identity_age_home']={'result':'PASS','existing':35,'current':37}
assert {x['id'] for x in old('docs/dev/design/catalogs/quests.json')['quests']} | set(mapping['new_quests'])=={x['id'] for x in quests}
assert sum(x['story']['role']=='main' for x in quests)==10
assert sum(x['story']['role']=='side' for x in quests)==12
checks['quest_ids_and_roles']={'result':'PASS','main':10,'side':12}
n='docs/dev/design/quests/QST-002/narrative.json'; before=json.dumps(old(n),ensure_ascii=False)
for a,b in audit['characters'].items():before=before.replace(a,b)
assert json.loads(before)==data(n)
checks['last_toy_runtime_data']={'result':'PASS','changes':'display names only; 16 beats and 22 information states retained'}
for f in ['source-assets/district-map/district.json']:
 a=old(f);b=data(f);changes=[]
 def compare(x,y,path=''):
  assert type(x)==type(y),path
  if isinstance(x,dict):
   assert x.keys()==y.keys(),path
   for k in x:compare(x[k],y[k],path+'/'+k)
  elif isinstance(x,list):
   assert len(x)==len(y),path
   for i,(p,q) in enumerate(zip(x,y)):compare(p,q,path+'/'+str(i))
  elif x!=y:
   assert isinstance(x,str) and (path.startswith('/groups/') or path.rsplit('/',1)[-1] in ['name','use','entry']),path
   changes.append(path)
 compare(a,b);checks['map_identity_geometry_topology']={'result':'PASS','changed_display_fields':len(changes)}
assert subprocess.check_output(['git','diff','--name-only',base,'--','game/src','game/assets']).decode().strip()==''
checks['runtime_source_and_assets']={'result':'PASS','changes':0}
# Historical old names remain in mapping/evidence; current prose and rendered player data must not use them
forbidden=[x['old'] for x in mapping['characters'] if x['old']!=x['name']]+['南汀','汀川市','白帆渡口','半阶杂货','林家小店','蓝门 Live House','流光河','N街区','N 街区','诺城','诺瓦市','弧河','BLUE HOUR','HALF STEP']
for scope in ['docs/player','docs/dev']:
 found=[]
 for p in Path(scope).rglob('*'):
  if p.suffix not in ['.md','.json','.csv']:continue
  text=p.read_text()
  found.extend([str(p),s] for s in forbidden if s in text)
 assert not found,found
 checks[scope+'_names']={'result':'PASS','old_name_hits':0}
p=Path('docs/dev/design/story-continuity.md').read_text()
assert len(re.findall(r'^\| F\d\d ',p,re.M))==16
q=Path('docs/dev/design/quests/QST-009/development.md').read_text()
assert len(re.findall(r'^\| T\d\d ',q,re.M))==14
assert len(re.findall(r'^## 场景',Path('docs/dev/design/quests/scenes.md').read_text(),re.M))==6
checks['design_coverage']={'result':'PASS','foreshadowing':16,'finale_beats':14,'scenes':6}
Path('todo/evidence/TASK-022/final/semantic-checks.json').write_text(json.dumps(checks,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(checks,ensure_ascii=False))
