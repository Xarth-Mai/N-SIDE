"""Check delivered GLB geometry against the recorded r5 shape-freeze values."""
from pathlib import Path
import hashlib
import json
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'tools'))
from glb import read_glb,values

path=ROOT/'game/assets/characters/CHR-001/yao-grey-study.glb'
doc,binary=read_glb(path)
def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()
report={'scope':'Final geometry preservation against r5 shape freeze','glb_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'nodes_sha256':digest(doc['nodes']),'materials_sha256':digest(doc['materials']),'inverse_bind_sha256':digest(values(doc,binary,doc['accessors'][doc['skins'][0]['inverseBindMatrices']])),'geometry':[],'images':[]}
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        report['geometry'].append({key:digest(values(doc,binary,doc['accessors'][index])) for key,index in dict(primitive['attributes'],indices=primitive['indices']).items()})
for image in doc['images']:
    view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
    report['images'].append(hashlib.sha256(binary[offset:offset+view['byteLength']]).hexdigest())
previous=json.loads(Path(__file__).with_name('shape-input.json').read_text())
report['checks']={key:report[key]==previous[key] for key in ('nodes_sha256','materials_sha256','inverse_bind_sha256','geometry','images')}
report['status']='PASS' if all(report['checks'].values()) else 'FAIL'
Path(__file__).with_name('preserved-geometry.json').write_text(json.dumps(report,indent=2)+'\n')
print(report['status'],report['checks'])
assert all(report['checks'].values()),'Animation revision changed the frozen shape, bind or atlas'
