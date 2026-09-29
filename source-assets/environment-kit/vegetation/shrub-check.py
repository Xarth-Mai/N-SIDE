"""Check the actual opaque, metric shrub against the existing placement envelope."""

import hashlib
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools'))
from glb import read_glb, require, values


def check(path):
    data, binary = read_glb(path)
    require(not data.get('animations') and not data.get('skins'), 'shrub must remain static')
    require(not data.get('images') and not data.get('textures'), 'vertex-colored shrub has no texture dependencies')
    roots = data['scenes'][data.get('scene', 0)]['nodes']
    require(len(roots) == 1, 'expected one direct-mesh root')
    node = data['nodes'][roots[0]]
    require(set(node) <= {'name', 'mesh'}, 'root must have baked metric coordinates and identity transform')
    require(len(data['materials']) == 1, 'expected one shared stem/leaf material')
    material = data['materials'][0]
    require(material.get('alphaMode', 'OPAQUE') == 'OPAQUE', 'shrub must be opaque')
    require(material.get('doubleSided'), 'geometric leaves require double-sided rendering')
    pbr = material['pbrMetallicRoughness']
    require(pbr.get('metallicFactor') == 0 and pbr.get('roughnessFactor', 1) >= 0.9, 'shrub must be matte nonmetal')
    require(pbr.get('baseColorFactor', [1, 1, 1, 1])[3] == 1, 'material alpha must remain 1')
    primitives = data['meshes'][node['mesh']]['primitives']
    require(len(primitives) == 1, 'expected a single draw primitive')
    primitive = primitives[0]
    require(primitive.get('mode', 4) == 4, 'expected triangle topology')
    attrs = {k: values(data, binary, data['accessors'][v]) for k, v in primitive['attributes'].items()}
    require({'POSITION', 'NORMAL', 'TEXCOORD_0', 'COLOR_0'} <= attrs.keys(), 'missing geometry/UV/color attributes')
    points = attrs['POSITION']
    require(all(len(rows) == len(points) for rows in attrs.values()), 'attribute counts differ')
    require(all(math.isfinite(x) for rows in attrs.values() for row in rows for x in row), 'nonfinite vertex data')
    require(all(abs(sum(x*x for x in n)-1) < 0.002 for n in attrs['NORMAL']), 'normals must be normalized')
    require(all(0 <= x <= 1 for c in attrs['COLOR_0'] for x in c), 'vertex colors out of range')
    require(all(len(c) == 3 or c[3] == 1 for c in attrs['COLOR_0']), 'vertex alpha must remain 1')
    indices = [r[0] for r in values(data, binary, data['accessors'][primitive['indices']])]
    require(len(indices) % 3 == 0 and all(0 <= i < len(points) for i in indices), 'invalid triangle indices')
    for a, b, c in zip(indices[::3], indices[1::3], indices[2::3]):
        u = [points[b][i]-points[a][i] for i in range(3)]
        v = [points[c][i]-points[a][i] for i in range(3)]
        cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
        require(sum(x*x for x in cross) > 1e-17, 'degenerate triangle')
    low = [min(p[i] for p in points) for i in range(3)]
    high = [max(p[i] for p in points) for i in range(3)]
    radius = max(math.hypot(p[0], p[2]) for p in points)
    require(abs(low[1]) < 1e-5 and abs(high[1]-0.8) < 1e-5, 'expected Y-up 0.8m height with grounded pivot')
    require(radius <= 0.669339, 'shrub exceeds replaced model horizontal envelope')
    triangles = len(indices)//3
    require(1000 < triangles <= 6000, 'shrub exceeds this sample 6000 triangle ceiling')
    return {'status': 'PASS', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'bytes': path.stat().st_size, 'bounds': [low, high], 'radius_m': radius,
            'triangles': triangles, 'primitives': 1, 'materials': 1, 'images': 0,
            'alpha': 'OPAQUE vertex-colored geometry; double sided; no cutout', 'animations': 'none'}


if __name__ == '__main__':
    try:
        print(json.dumps(check(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name('shrub-courtyard.glb')), indent=2))
    except (ValueError, KeyError, IndexError, OSError) as error:
        print(f'FAIL: {error}', file=sys.stderr)
        raise SystemExit(1)
