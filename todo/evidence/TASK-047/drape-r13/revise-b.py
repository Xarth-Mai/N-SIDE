"""One asymmetric cloth correction on the existing r13 topology; never touch formal assets"""
import ast
import json
import math
from pathlib import Path
import bpy

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-drape-r13'
bpy.ops.wm.open_mainfile(filepath=str(out / 'before.blend'))
base = [v.co.copy() for v in bpy.data.objects['Jacket_ContinuousShoulders'].data.vertices]
for t in (.24, .48, .72):
    base.extend(base[66 + j].lerp(base[99 + j], t) for j in range(33))
bpy.ops.wm.open_mainfile(filepath=str(out / 'candidate.blend'))
obj = bpy.data.objects['Jacket_ContinuousShoulders']
assert len(obj.data.vertices) == len(base) == 618
# Restore the undecorated cloth position on this topology before a single new correction
for vertex, point in zip(obj.data.vertices, base):
    vertex.co = point
profile = ((-.042, 0), (-.007, -.017), (.018, .007), (.049, 0))
changed = []
for vertex in obj.data.vertices:
    i, p = vertex.index, vertex.co
    if i < 519 and (i >= 99 or i < 33 or i % 33 in (0, 1, 31, 32)):
        continue
    if i >= 519 and (i - 519) % 33 in (0, 1, 31, 32):
        continue
    if p.y <= .006:
        continue
    t = max(0, min(1, (p.z - 1.022) / .168))
    centre = .085 + .10 * t
    distance = abs(p.x) - centre
    for (x0, y0), (x1, y1) in zip(profile, profile[1:]):
        if x0 <= distance <= x1:
            amount = y0 + (y1 - y0) * (distance - x0) / (x1 - x0)
            amount *= math.sin(math.pi * t) * max(0, min(1, (p.y - .006) / .055))
            amount *= 1 if p.x > 0 else .23
            p.y += amount
            changed.append({'index': i, 'displacement_m': amount})
            break
obj.data.update()
bpy.ops.wm.save_as_mainfile(filepath=str(out / 'candidate-b.blend'))
tree = ast.parse((out / 'generator-before.py').read_text())
function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'export')
runtime = out / 'variant-b'
runtime.mkdir(exist_ok=True)
scope = {'bpy': bpy, 'RUNTIME': runtime}
exec(compile(ast.Module(body=[function], type_ignores=[]), '<existing exporter>', 'exec'), scope)
rig = bpy.data.objects['CHR001_Rig']
scope['export'](rig, [o for o in rig.children if o.type == 'MESH'])
report = {'topology': 'unchanged from r13 A', 'profile_m': profile,
          'secondary_strength': .23, 'primary_side': '+X', 'changed': changed,
          'scope': 'One narrow inward fold and shallow raised flank; no global width or hem movement'}
Path(__file__).with_name('geometry-change-b.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'status': 'candidate only', 'changed_vertices': len(changed)}))
