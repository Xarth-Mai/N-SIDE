"""Compare the r3 character contract against the committed r2 source"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(root / 'tools'))
from glb import read_glb, values

runtime = 'game/assets/characters/CHR-001/yao-grey-study.glb'
baseline = subprocess.run(['git', 'show', f'dbe5ad3e2b4eda5bebfd176e6d40bee9b075bdce:{runtime}'], cwd=root, check=True, capture_output=True).stdout
with tempfile.TemporaryDirectory(prefix='nside-character-r3-') as directory:
    path = Path(directory) / 'r2.glb'
    path.write_bytes(baseline)
    old, old_binary = read_glb(path)
new, binary = read_glb(root / runtime)

def clips(document, data):
    result = {}
    for clip in document['animations']:
        result[clip['name']] = []
        for channel in clip['channels']:
            sampler = clip['samplers'][channel['sampler']]
            result[clip['name']].append({
                'target': channel['target'],
                'interpolation': sampler.get('interpolation', 'LINEAR'),
                **{field: values(document, data, document['accessors'][sampler[field]]) for field in ('input', 'output')},
            })
    return result

texture = 'source-assets/characters/CHR-001/model/grey-study.png'
inputs = json.loads((Path(__file__).parent / 'input-hashes.json').read_text())
checks = {
    'animation_sample_data_identical_to_r2': clips(old, old_binary) == clips(new, binary),
    'node_transforms_identical_to_r2': old['nodes'] == new['nodes'],
    'material_identical_to_r2': old['materials'] == new['materials'],
    'atlas_bytes_identical_to_r2': inputs[texture] == hashlib.sha256((root / texture).read_bytes()).hexdigest(),
    'inverse_bind_values_identical_to_r2': values(old, old_binary, old['accessors'][old['skins'][0]['inverseBindMatrices']]) == values(new, binary, new['accessors'][new['skins'][0]['inverseBindMatrices']]),
}
assert all(checks.values()), checks
report = {
    'status': 'PASS',
    **checks,
    'mesh_count': len(new['meshes']),
    'primitive_count': sum(len(mesh['primitives']) for mesh in new['meshes']),
    'triangle_count': sum(new['accessors'][primitive['indices']]['count'] // 3 for mesh in new['meshes'] for primitive in mesh['primitives']),
}
(Path(__file__).parent / 'preserved-contract.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
