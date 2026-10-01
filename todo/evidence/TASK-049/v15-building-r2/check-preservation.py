"""Compare frozen r1 components and embedded resources against this asset revision"""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
ROOT=Path(__file__).resolve().parents[4]
HERE=Path(__file__).resolve().parent
old=json.loads((HERE/'r1-components.json').read_text())['bounds']
new=json.loads((HERE/'component-bounds.json').read_text())['bounds']
removed=sorted(set(old)-set(new))
changed=sorted(name for name in old.keys() & new.keys() if old[name] != new[name])
unchanged=sorted(name for name in old.keys() & new.keys() if old[name] == new[name])
assert removed==['Street programme 1 composition','Street programme 1 footer','Street programme 1 horizon','Street programme 1 light stripe']
assert changed==['Street programme 1 print'] and len(unchanged)==53

def resources(raw):
    size=struct.unpack_from('<I',raw,12)[0]
    gltf=json.loads(raw[20:20+size]);binary=raw[28+size:]
    images={}
    for image in gltf['images']:
        view=gltf['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
        images[image['name']]=hashlib.sha256(binary[offset:offset+view['byteLength']]).hexdigest()
    return {m['name']:m for m in gltf['materials']},images
path='game/assets/environment/buildings/v15-mirror-hall-facade.glb'
r1=subprocess.check_output(['git','show','0efb89c:'+path],cwd=ROOT)
assert hashlib.sha256(r1).hexdigest()=='07e23026c08d5967f85adc825ff518a3a4e34fac453565c71545a2d22dddd2b2'
old_mats,old_images=resources(r1);new_mats,new_images=resources((ROOT/path).read_bytes())
# Texture indices are stable because the six public images precede the new poster
assert all(new_mats[name]==value for name,value in old_mats.items())
assert all(new_images[name]==value for name,value in old_images.items())
result={'result':'PASS','baseline_commit':'0efb89c','baseline_glb_sha256':hashlib.sha256(r1).hexdigest(),
        'old_component_count':len(old),'current_component_count':len(new),'unchanged_count':len(unchanged),
        'unchanged_components':unchanged,'changed_components':changed,'removed_components':removed,
        'new_component_count':len(set(new)-set(old)),'preserved_materials':sorted(old_mats),
        'preserved_embedded_image_sha256':old_images,'exception':'Replace only Street programme 1 image and its four geometric overlay layers'}
(HERE/'preservation-check.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
