"""Material-only delivery check against b948cab, including actual GLB texture bytes."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'tools'))
from glb import read_glb, values
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--asset', type=Path, default=ROOT / 'game/assets/characters/CHR-001/yao-grey-study.glb')
parser.add_argument('--output', type=Path, default=Path(__file__).with_name('preserved-contract.json'))
args = parser.parse_args()
relative = 'game/assets/characters/CHR-001/yao-grey-study.glb'
revision = subprocess.run(['git', 'rev-parse', 'b948cab'], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
baseline = subprocess.run(['git', 'show', revision + ':' + relative], cwd=ROOT, check=True, capture_output=True).stdout
with tempfile.TemporaryDirectory(prefix='nside-material-contract-') as temporary:
    path = Path(temporary) / 'baseline.glb'
    path.write_bytes(baseline)
    before, bb = read_glb(path)
after, ab = read_glb(args.asset)
def accessor(doc, binary, index):
    return values(doc, binary, doc['accessors'][index])
def geometry(doc, binary):
    return [{'name': mesh['name'], 'primitives': [{'mode': p.get('mode', 4), 'material': p['material'], 'indices': accessor(doc, binary, p['indices']), 'attributes': {key: accessor(doc, binary, i) for key, i in p['attributes'].items()}} for p in mesh['primitives']]} for mesh in doc['meshes']]
def animation(doc, binary):
    return {clip['name']: {doc['nodes'][c['target']['node']]['name'] + '.' + c['target']['path']: {'interpolation': clip['samplers'][c['sampler']].get('interpolation', 'LINEAR'), 'input': accessor(doc, binary, clip['samplers'][c['sampler']]['input']), 'output': accessor(doc, binary, clip['samplers'][c['sampler']]['output'])} for c in clip['channels']} for clip in doc['animations']}
def texture(doc, binary, reference):
    image = doc['images'][doc['textures'][reference['index']]['source']]
    view = doc['bufferViews'][image['bufferView']]
    offset = view.get('byteOffset', 0)
    return binary[offset:offset + view['byteLength']]
material = after['materials'][0]
pbr = material['pbrMetallicRoughness']
assert 'metallicRoughnessTexture' in pbr and 'normalTexture' in material, 'GLB is missing authored surface or normal texture'
base = texture(after, ab, pbr['baseColorTexture'])
old_base = texture(before, bb, before['materials'][0]['pbrMetallicRoughness']['baseColorTexture'])
surface_bytes = texture(after, ab, pbr['metallicRoughnessTexture'])
normal_bytes = texture(after, ab, material['normalTexture'])
surface = Image.open(io.BytesIO(surface_bytes)).convert('RGB')
normal = Image.open(io.BytesIO(normal_bytes)).convert('RGB')
regions = [('skin', .64), ('hair', .52), ('jacket', .80), ('shirt', .88), ('pants', .76), ('shoe', .62), ('ink', .72), ('white', .86)]
centres = {name: surface.getpixel(((i % 2) * 512 + 256, 1023 - ((i // 2) * 256 + 128))) for i, (name, _) in enumerate(regions)}
checks = {
    'nodes_and_bind_transforms_unchanged': before['nodes'] == after['nodes'],
    'skin_joints_unchanged': before['skins'][0]['joints'] == after['skins'][0]['joints'],
    'inverse_bind_matrices_unchanged': accessor(before, bb, before['skins'][0]['inverseBindMatrices']) == accessor(after, ab, after['skins'][0]['inverseBindMatrices']),
    'geometry_normals_UV_weights_indices_unchanged': geometry(before, bb) == geometry(after, ab),
    'all_animation_samples_unchanged': animation(before, bb) == animation(after, ab),
    'base_texture_bytes_unchanged': base == old_base,
    'one_opaque_material': len(after['materials']) == 1 and material.get('alphaMode', 'OPAQUE') == 'OPAQUE' and 'KHR_materials_unlit' not in material.get('extensions', {}),
    'surface_1K': surface.size == (1024, 1024),
    'normal_1K': normal.size == (1024, 1024),
    'roughness_factor_one': pbr.get('roughnessFactor', 1) == 1,
    'roughness_G_matches_regions': all(abs(centres[name][1] / 255 - expected) <= 1 / 255 for name, expected in regions),
    'all_metallic_B_zero': surface.getchannel('B').getextrema() == (0, 0),
    'normal_scale_one': material['normalTexture'].get('scale', 1) == 1,
    'normal_small_and_outward': min(normal.getchannel('B').getextrema()) > 252 and all(123 <= min(normal.getchannel(channel).getextrema()) <= max(normal.getchannel(channel).getextrema()) <= 132 for channel in ('R', 'G')),
    'all_maps_uv0': all(ref.get('texCoord', 0) == 0 for ref in (pbr['baseColorTexture'], pbr['metallicRoughnessTexture'], material['normalTexture'])),
}
report = {'status': 'PASS' if all(checks.values()) else 'FAIL', 'baseline_commit': revision, 'baseline_glb_sha256': hashlib.sha256(baseline).hexdigest(), 'asset_sha256': hashlib.sha256(args.asset.read_bytes()).hexdigest(), 'checks': checks, 'material': material, 'surface_region_centres_RGB': centres, 'embedded_image_sha256': {'base': hashlib.sha256(base).hexdigest(), 'surface': hashlib.sha256(surface_bytes).hexdigest(), 'normal': hashlib.sha256(normal_bytes).hexdigest()}, 'scope': 'Actual GLB data and source contract; excludes artistic acceptance and GPU behavior'}
args.output.write_text(json.dumps(report, indent=2) + '\n')
print(report['status'], checks)
assert all(checks.values()), 'Material candidate changed preserved contracts or lost authored texture channels'
