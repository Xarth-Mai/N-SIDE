"""Verify the selected original files and the actual exported bench data."""
import hashlib
import io
import json
import math
from pathlib import Path
import sys

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'tools'))
from glb import read_glb, values

source = ROOT / 'source-assets/environment-kit'
manifest = json.loads((source / 'asset-manifest.json').read_text())
entry = next(f for f in manifest['files'] if f['source'] == 'street-furniture/street-bench.glb')
image_hashes = set()
for original in entry['original_files']:
    data = (source / original['path']).read_bytes()
    assert len(data) == original['bytes']
    assert hashlib.sha256(data).hexdigest() == original['sha256']
    assert hashlib.md5(data).hexdigest() == original['official_md5']
    if original['path'].endswith('.jpg'):
        image_hashes.add(hashlib.sha256(data).hexdigest())
asset = source / entry['source']
data = asset.read_bytes()
assert len(data) == entry['bytes']
assert hashlib.sha256(data).hexdigest() == entry['sha256']
assert data == (ROOT / 'game/assets/environment' / entry['output']).read_bytes()
doc, binary = read_glb(asset)
assert not doc.get('animations') and not doc.get('skins') and not doc.get('extensionsUsed')
assert len(doc['scenes']) == len(doc['nodes']) == len(doc['meshes']) == 1
assert doc['scenes'][0]['nodes'] == [0]
assert set(doc['nodes'][0]) <= {'mesh', 'name'}
points, triangles = [], 0
for primitive in doc['meshes'][0]['primitives']:
    attributes = primitive['attributes']
    assert {'POSITION', 'NORMAL', 'TANGENT', 'TEXCOORD_0'} <= attributes.keys()
    decoded = {name: values(doc, binary, doc['accessors'][index]) for name, index in attributes.items()}
    vertices = decoded['POSITION']
    assert all(math.isfinite(x) for rows in decoded.values() for row in rows for x in row)
    indices = values(doc, binary, doc['accessors'][primitive['indices']])
    assert len(indices) % 3 == 0 and all(0 <= i[0] < len(vertices) for i in indices)
    for i in range(0, len(indices), 3):
        a, b, c = [vertices[indices[i+j][0]] for j in range(3)]
        u, v = [b[k]-a[k] for k in range(3)], [c[k]-a[k] for k in range(3)]
        cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
        assert sum(x*x for x in cross) > 1e-24, 'zero-area triangle'
    for normal, tangent in zip(decoded['NORMAL'], decoded['TANGENT']):
        assert abs(sum(v*v for v in normal) - 1) < .001
        assert abs(sum(v*v for v in tangent[:3]) - 1) < .001
        assert abs(sum(a*b for a, b in zip(normal, tangent))) < .001
        assert abs(tangent[3]) == 1
    triangles += len(indices) // 3
    points.extend(vertices)
    material = doc['materials'][primitive['material']]
    pbr = material['pbrMetallicRoughness']
    assert {'baseColorTexture', 'metallicRoughnessTexture'} <= pbr.keys()
    assert 'normalTexture' in material
def bounds(rows):
    return [[min(v[i] for v in rows) for i in range(3)],
            [max(v[i] for v in rows) for i in range(3)]]
lo, hi = bounds(points)
assert abs(lo[1]) < .00001
assert abs(hi[0] - lo[0] - 1.92) < .00001
assert abs(hi[1] - .866914153) < .00001
assert abs(hi[2] - lo[2] - .670393944) < .00001
assert triangles == entry['triangles'] == 8906
embedded = set()
for image in doc['images']:
    assert image['mimeType'] == 'image/jpeg' and 'uri' not in image
    view = doc['bufferViews'][image['bufferView']]
    raw = binary[view['byteOffset']:view['byteOffset']+view['byteLength']]
    with Image.open(io.BytesIO(raw)) as img:
        assert img.size == (1024, 1024)
    embedded.add(hashlib.sha256(raw).hexdigest())
assert embedded == image_hashes and len(embedded) == 9
floor = [v for v in points if v[1] < .001]
print(json.dumps({'result': 'PASS', 'scope': 'file/geometry/UV/PBR provenance; not game runtime',
    'sha256': entry['sha256'], 'bytes': len(data), 'original_files': len(entry['original_files']),
    'triangles': triangles, 'materials': len(doc['materials']), 'embedded_images': len(embedded),
    'bounds_gltf': [lo, hi], 'left_foot': bounds([v for v in floor if v[0] < 0]),
    'right_foot': bounds([v for v in floor if v[0] > 0])}, indent=2))
