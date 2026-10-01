"""Inspect exported binary accessors and the static Scene0 import contract"""
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[3]
MATERIALS = ROOT / 'source-assets/environment-kit/materials'
sys.path.insert(0, str(MATERIALS))
PATH = ROOT / 'game/assets/environment/buildings/v15-mirror-hall-facade.glb'
raw = PATH.read_bytes()
magic, version, length = struct.unpack_from('<4sII', raw)
assert (magic, version, length) == (b'glTF', 2, len(raw))
size, kind = struct.unpack_from('<I4s', raw, 12)
assert kind == b'JSON'
gltf = json.loads(raw[20:20+size])
offset = 20+size
size, kind = struct.unpack_from('<I4s', raw, offset)
assert kind == b'BIN\0'
binary = raw[offset+8:offset+8+size]


def accessor(index):
    a = gltf['accessors'][index]
    v = gltf['bufferViews'][a['bufferView']]
    code, width = {5121: ('B', 1), 5123: ('H', 2), 5125: ('I', 4), 5126: ('f', 4)}[a['componentType']]
    n = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}[a['type']]
    start = v.get('byteOffset', 0) + a.get('byteOffset', 0)
    stride = v.get('byteStride', width*n)
    return [struct.unpack_from('<'+code*n, binary, start+i*stride) for i in range(a['count'])]


assert gltf.get('scene', 0) == 0 and len(gltf['scenes']) == 1
assert len(gltf['nodes']) == len(gltf['meshes']) == 1
assert set(gltf.get('extensionsUsed', [])) <= {'KHR_materials_unlit', 'KHR_texture_transform'}
assert set(gltf['nodes'][0]) <= {'name', 'mesh'}
assert not any(gltf.get(k) for k in ['animations', 'skins', 'cameras'])
assert all('bufferView' in image and 'uri' not in image for image in gltf['images'])
records = json.loads((MATERIALS.parent/'asset-manifest.json').read_text())['files']
texture_sources, derived_normals = {}, {}
for stem in ('Plaster001', 'Concrete034', 'WoodSiding009'):
    material = next(m for m in gltf['materials'] if m['name'] == f'V15_ambientCG_{stem}')
    for suffix, slot in (('Color', material['pbrMetallicRoughness']['baseColorTexture']), ('NormalGL', material['normalTexture'])):
        image = gltf['images'][gltf['textures'][slot['index']]['source']]
        view = gltf['bufferViews'][image['bufferView']]
        start = view.get('byteOffset', 0)
        embedded = binary[start:start+view['byteLength']]
        source = MATERIALS/f'{stem}_1K-JPG_{suffix}.jpg'
        source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
        record = next((r for r in records if r['source'] == f'materials/{source.name}'), None)
        assert record and record['license'] == 'CC0-1.0' and record['sha256'] == source_hash, 'PBR input differs from AST-003 license/hash'
        texture_sources[str(source.relative_to(ROOT))] = source_hash
        if suffix == 'Color':
            assert hashlib.sha256(embedded).hexdigest() == source_hash, 'GLB changed original color bytes'
        else:
            derived = MATERIALS/f'{stem}-NormalGL-scale035.png'
            derived_hash = hashlib.sha256(derived.read_bytes()).hexdigest()
            assert image['mimeType'] == 'image/png' and hashlib.sha256(embedded).hexdigest() == derived_hash, 'GLB normal differs from shared PNG'
            assert slot.get('scale', 1) == 1, 'baked normal must use glTF scale 1'
            if '--source' not in sys.argv:
                from PIL import Image
                from bake_normals import baked_normal
                expected = baked_normal(source, .35)
                with Image.open(derived) as actual:
                    assert actual.mode == expected.mode and actual.size == expected.size and actual.tobytes() == expected.tobytes(), 'normal bake differs from xy*.35, z unchanged, normalize'
            derived_normals[str(derived.relative_to(ROOT))] = {'sha256': derived_hash, 'source': str(source.relative_to(ROOT)), 'source_sha256': source_hash, 'baked_scale': .35, 'gltf_scale': 1}
counts = {'vertices': 0, 'triangles': 0, 'degenerate_triangles': 0}
all_positions = []
for primitive in gltf['meshes'][0]['primitives']:
    assert primitive.get('mode', 4) == 4 and 'material' in primitive
    attributes = {name: accessor(index) for name, index in primitive['attributes'].items()}
    assert {'POSITION', 'NORMAL', 'TEXCOORD_0'} <= attributes.keys()
    assert all(math.isfinite(n) for values in attributes.values() for row in values for n in row)
    p = attributes['POSITION']
    assert len(p) == len(attributes['NORMAL']) == len(attributes['TEXCOORD_0'])
    assert all(.98 < sum(n*n for n in normal) < 1.02 for normal in attributes['NORMAL'])
    declared = gltf['accessors'][primitive['attributes']['POSITION']]
    for i in range(3):
        assert abs(min(row[i] for row in p)-declared['min'][i]) < 1e-4
        assert abs(max(row[i] for row in p)-declared['max'][i]) < 1e-4
    indices = [row[0] for row in accessor(primitive['indices'])]
    assert len(indices) % 3 == 0 and max(indices) < len(p)
    for i in range(0, len(indices), 3):
        a, b, c = (p[j] for j in indices[i:i+3])
        u, v = ([b[j]-a[j] for j in range(3)], [c[j]-a[j] for j in range(3)])
        cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
        if sum(n*n for n in cross) < 1e-16:
            counts['degenerate_triangles'] += 1
    counts['vertices'] += len(p)
    counts['triangles'] += len(indices)//3
    all_positions.extend(p)
poster_index = next(i for i,m in enumerate(gltf['materials']) if m['name'] == 'V15_Anke_programme')
poster = gltf['materials'][poster_index]
texture = gltf['textures'][poster['pbrMetallicRoughness']['baseColorTexture']['index']]
poster_image = gltf['images'][texture['source']]
v = gltf['bufferViews'][poster_image['bufferView']]
packed_poster = binary[v.get('byteOffset',0):v.get('byteOffset',0)+v['byteLength']]
source_poster = (ROOT/'game/assets/environment/posters/anke-cinema.png').read_bytes()
assert packed_poster == source_poster, 'The exported poster must preserve the reviewed PNG bytes'
assert struct.unpack_from('>II',packed_poster,16) == (1024,1620)
primitive = next(p for p in gltf['meshes'][0]['primitives'] if p['material'] == poster_index)
positions = accessor(primitive['attributes']['POSITION'])
uvs = accessor(primitive['attributes']['TEXCOORD_0'])
front = [(p,uv) for p,uv in zip(positions,uvs) if p[2] > .613]
assert len(front) >= 4
for p,uv in front:
    assert abs(uv[0]-(p[0]-10.54)/1.42) < 1e-5
    assert abs(uv[1]-(1-(p[1]-.45)/2.25)) < 1e-5
assert all(m.get('alphaMode','OPAQUE') == 'OPAQUE' for m in gltf['materials'])
assert len(gltf['materials']) <= 10 and len(gltf['images']) <= 7
assert counts['degenerate_triangles'] == 0, counts
assert counts['vertices'] < 30000 and counts['triangles'] < 16000
assert min(row[1] for row in all_positions) >= -1e-5
assert max(row[1] for row in all_positions) < 12
report = {'result': 'PASS', 'file': str(PATH.relative_to(ROOT)), 'sha256': hashlib.sha256(raw).hexdigest(),
                  'poster_bytes_preserved': True, 'poster_front_uv': 'PASS', 'all_materials_opaque': True,
                  'texture_sources': texture_sources, 'derived_normals': derived_normals,
                  'normal_pixel_check': 'NOT RUN in --source mode; run system Python for pixels' if '--source' in sys.argv else 'PASS',
                  **counts, 'materials': len(gltf['materials']), 'embedded_images': len(gltf['images']),
                  'bounds_gltf': {'min': [min(p[i] for p in all_positions) for i in range(3)],
                                  'max': [max(p[i] for p in all_positions) for i in range(3)]}}
if '--source' in sys.argv:
    import bpy
    master = ROOT/'source-assets/buildings/V-15/mirror-hall-facade.blend'
    bpy.ops.wm.open_mainfile(filepath=str(master))
    for stem in ('Plaster001', 'Concrete034', 'WoodSiding009'):
        shader = bpy.data.materials[f'V15_ambientCG_{stem}'].node_tree.nodes['Principled BSDF']
        assert len(shader.inputs['Normal'].links) == 1, 'source normal must be bound to shader'
        normal = shader.inputs['Normal'].links[0].from_node
        assert normal.type == 'NORMAL_MAP' and not normal.inputs['Strength'].is_linked and normal.inputs['Strength'].default_value == 1
        assert len(normal.inputs['Color'].links) == 1
        texture = normal.inputs['Color'].links[0].from_node
        assert texture.type == 'TEX_IMAGE' and texture.image and texture.image.packed_file
        image = texture.image
        derived = MATERIALS/f'{stem}-NormalGL-scale035.png'
        assert image.colorspace_settings.name == 'Non-Color' and Path(bpy.path.abspath(image.filepath)).resolve() == derived.resolve()
        assert hashlib.sha256(image.packed_file.data).hexdigest() == hashlib.sha256(derived.read_bytes()).hexdigest(), 'source packed normal differs from shared PNG'
    report['blender_source'] = {'status': 'PASS', 'scope': 'three bound packed normal maps, shared paths, Non-Color and Strength 1 only', 'version': bpy.app.version_string, 'source_sha256': hashlib.sha256(master.read_bytes()).hexdigest()}
print(json.dumps(report, indent=2))
