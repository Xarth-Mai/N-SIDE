"""Inspect exported binary accessors and the static Scene0 import contract"""
import hashlib
import json
import math
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[3]
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
assert counts['degenerate_triangles'] == 0, counts
assert counts['vertices'] < 20000 and counts['triangles'] < 16000
assert min(row[1] for row in all_positions) >= -1e-5
assert max(row[1] for row in all_positions) < 12
print(json.dumps({'result': 'PASS', 'file': str(PATH.relative_to(ROOT)), 'sha256': hashlib.sha256(raw).hexdigest(),
                  **counts, 'materials': len(gltf['materials']), 'embedded_images': len(gltf['images']),
                  'bounds_gltf': {'min': [min(p[i] for p in all_positions) for i in range(3)],
                                  'max': [max(p[i] for p in all_positions) for i in range(3)]}}, indent=2))
