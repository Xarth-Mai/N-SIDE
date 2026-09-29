"""Compare delivered rig/atlas/unchanged clips against the actual f93af27 GLB."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'tools'))
from glb import read_glb,values
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--asset',type=Path,default=ROOT/'game/assets/characters/CHR-001/yao-grey-study.glb')
parser.add_argument('--output',type=Path,default=Path(__file__).with_name('preserved-contract.json'))
args=parser.parse_args()
relative='game/assets/characters/CHR-001/yao-grey-study.glb'
baseline=subprocess.run(['git','show','f93af27:'+relative],cwd=ROOT,check=True,capture_output=True).stdout
with tempfile.TemporaryDirectory(prefix='nside-animation-contract-') as temporary:
    path=Path(temporary)/'baseline.glb';path.write_bytes(baseline)
    before,bb=read_glb(path)
after,ab=read_glb(args.asset)
def animation(doc,binary,name):
    clip=next(a for a in doc['animations'] if a['name']==name)
    curves={}
    for channel in clip['channels']:
        sampler=clip['samplers'][channel['sampler']]
        curves[doc['nodes'][channel['target']['node']]['name']+'.'+channel['target']['path']]={'interpolation':sampler.get('interpolation','LINEAR'),'input':values(doc,binary,doc['accessors'][sampler['input']]),'output':values(doc,binary,doc['accessors'][sampler['output']])}
    return curves
old={name:animation(before,bb,name) for name in ('Idle','Walk','Run')}
new={name:animation(after,ab,name) for name in ('Idle','Walk','Run')}
support=('Root.','Hips.','Thigh.','Shin.')
def embedded(doc,binary):
    hashes=[]
    for image in doc['images']:
        view=doc['bufferViews'][image['bufferView']]
        offset=view.get('byteOffset',0)
        hashes.append(hashlib.sha256(binary[offset:offset+view['byteLength']]).hexdigest())
    return hashes
checks={'node_identity_and_bind_transforms':before['nodes']==after['nodes'],'inverse_binds':values(before,bb,before['accessors'][before['skins'][0]['inverseBindMatrices']])==values(after,ab,after['accessors'][after['skins'][0]['inverseBindMatrices']]),'materials':before['materials']==after['materials'],'embedded_atlas':embedded(before,bb)==embedded(after,ab),'Idle_unchanged':old['Idle']==new['Idle'],'Walk_unchanged':old['Walk']==new['Walk'],'Run_support_hierarchy_unchanged':{k:v for k,v in old['Run'].items() if k.startswith(support)}=={k:v for k,v in new['Run'].items() if k.startswith(support)},'Run_changed':old['Run']!=new['Run']}
report={'baseline_commit':'f93af27','baseline_sha256':hashlib.sha256(baseline).hexdigest(),'asset_sha256':hashlib.sha256(args.asset.read_bytes()).hexdigest(),'scope':'Rig/atlas and animation contracts; head-hair mesh changes belong to separate model-r6 review','status':'PASS' if all(checks.values()) else 'FAIL','checks':checks,'changed_Run_channels':sorted(key for key in old['Run'].keys()|new['Run'].keys() if old['Run'].get(key)!=new['Run'].get(key))}
args.output.write_text(json.dumps(report,indent=2)+'\n')
print(report['status'],checks)
assert all(checks.values()),'Run must change while Idle/Walk, the support chain, rig and atlas remain unchanged'
