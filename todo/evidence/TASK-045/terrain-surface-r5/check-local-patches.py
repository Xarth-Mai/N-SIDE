"""CPU contract check for the isolated shader's lattice and NormalGL rotations.
This checks sampling mathematics, not GPU execution or appearance.
"""
from pathlib import Path
import json
import math
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
case = sys.argv[1] if len(sys.argv) > 1 else 'local-patches'
assert case in ('local-patches', 'local-angles')
shader = ROOT / ('game/assets/shaders/terrain-slope.wgsl' if '--formal' in sys.argv else f'output/assets/terrain-r5-diagnostics/{case}/game/assets/shaders/terrain-slope.wgsl')
s = shader.read_text()
spacing = float(re.search(r'let position = metres / ([\d.]+);', s)[1])
assert ('quarter_turn(nt.xy, 4u - (hash & 3u))' if case == 'local-patches' else 'rotate_ground(nt.xy, -ground_angle(hash))') in s
assert s.count('let ground_n = ground_normal(') == 1
assert s.count('pbr.N = ground_normal(') == 1
assert s.count('textureSampleGrad(') == 2


def patches(x, y):
    x, y = x / spacing, y / spacing
    cx, cy = math.floor(x), math.floor(y)
    x, y = x - cx, y - cy
    if x + y < 1:
        cells, w = [(cx, cy), (cx + 1, cy), (cx, cy + 1)], [1 - x - y, x, y]
    else:
        cells, w = [(cx + 1, cy + 1), (cx, cy + 1), (cx + 1, cy)], [x + y - 1, 1 - x, 1 - y]
    w = [v * v * (3 - 2 * v) for v in w]
    return {c: v / sum(w) for c, v in zip(cells, w)}


def turn(v, n):
    x, y = v
    if case == 'local-patches':
        return [(x, y), (-y, x), (-x, -y), (y, -x)][n % 4]
    angle = n * (2 * math.pi / 65536)
    c, s = math.cos(angle), math.sin(angle)
    return c*x-s*y, s*x+c*y


def hash_cell(cell):
    value = ((cell[0] & 0xffffffff) * 1664525 + (cell[1] & 0xffffffff) * 1013904223) & 0xffffffff
    value = ((value ^ (value >> 16)) * 2246822519) & 0xffffffff
    return value ^ (value >> 13)


def surface(x, y):
    # Analytic source with directional structure, phase and orientation taken per lattice vertex
    result = 0
    for cell, weight in patches(x, y).items():
        h = hash_cell(cell)
        u, v = turn((x / 2.1, y / 2.1), h if case == "local-patches" else h & 65535)
        u += ((h >> 2) & 32767) / 32768
        v += ((h >> 17) & 32767) / 32768
        result += weight * (math.sin(2 * math.pi * u) + .3 * math.cos(6 * math.pi * v))
    return result


eps = 1e-6
worst_continuity = 0
count = 0
for cx in range(-4, 5):
    for cy in range(-4, 5):
        for t in [.05, .25, .5, .75, .95]:
            for a, b in [
                ((cx * spacing - eps, (cy + t) * spacing), (cx * spacing + eps, (cy + t) * spacing)),
                (((cx + t) * spacing, cy * spacing - eps), ((cx + t) * spacing, cy * spacing + eps)),
                (((cx + t) * spacing, (cy + 1 - t) * spacing - eps), ((cx + t) * spacing, (cy + 1 - t) * spacing + eps)),
            ]:
                diff = abs(surface(*a) - surface(*b))
                worst_continuity = max(worst_continuity, diff)
                assert diff < 2e-5, (a, b, diff)
                for p in [a, b]:
                    weights = patches(*p).values()
                    assert min(weights) >= 0 and abs(sum(weights) - 1) < 1e-12
                count += 1

worst_gradient = 0
rotations = list(range(4)) if case == "local-patches" else [hash_cell((i, 17-i)) & 65535 for i in range(17)]
for n in rotations:
    for x, y in [(-9.1, 6.8), (0, 0), (.2, .7), (12.4, -2.1)]:
        u, v = turn((x, y), n)
        gradient = (2 * math.pi * math.cos(2 * math.pi * u), -1.8 * math.pi * math.sin(6 * math.pi * v))
        transformed = turn(gradient, -n)
        def height(a, b):
            a, b = turn((a, b), n)
            return math.sin(2 * math.pi * a) + .3 * math.cos(6 * math.pi * b)
        measured = ((height(x + eps, y) - height(x - eps, y)) / (2 * eps), (height(x, y + eps) - height(x, y - eps)) / (2 * eps))
        error = max(abs(a - b) for a, b in zip(transformed, measured))
        worst_gradient = max(worst_gradient, error)
        assert error < 1e-6
        assert max(abs(a-b) for a,b in zip(turn(turn((x, y), n), -n), (x,y))) < 1e-12

result = {'status': 'PASS', 'scope': 'CPU lattice continuity, partition of unity and tangent gradient rotation; GPU and appearance separate', 'boundary_pairs': count, 'max_cross_boundary_delta': worst_continuity, 'rotation_checks': len(rotations)*4, 'max_gradient_error': worst_gradient, 'patch_metres': spacing}
(ROOT / f'todo/evidence/TASK-045/terrain-surface-r5/{case}-check.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
