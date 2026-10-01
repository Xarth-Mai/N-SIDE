"""Deform only the frozen jacket; keep its sewn boundaries, UVs and skin"""
import ast
import json
import math
from pathlib import Path
import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-jacket-r11'

def update_jacket(obj):
    assert obj.name == 'Jacket_ContinuousShoulders' and len(obj.data.vertices) == 519
    # Torso keeps its upper shoulder/neck rows and the entire zipper border
    for vertex in list(obj.data.vertices)[:231]:
        row, column = divmod(vertex.index, 33)
        if row >= 4 or column in (0, 1, 31, 32):
            continue
        p = vertex.co
        # The hood lies against the central upper back; keep its support unchanged
        if p.y > 0 and p.z > 1.15:
            continue
        angle = .48 + (math.tau - .96) * column / 32
        side = abs(math.sin(angle)) ** 2
        amount = (.25, .60, 1.0, .9)[row]
        p.x += math.copysign(.029 * amount * side, p.x)
        p.y += -math.cos(angle) * .008 * amount * side
    # Expand sleeve cross-sections rather than scaling hands or moving the sewn armhole
    centres = ((.203, 0, 1.327), (.25, -.002, 1.264), (.309, -.006, 1.191),
               (.329, -.006, 1.168), (.342, -.003, 1.142), (.354, -.007, 1.116),
               (.389, -.012, 1.039), (.407, -.017, 1.005), (.414, -.016, .980))
    fullness = (0, .22, .42, .47, .45, .40, .26, 0, 0)
    for side_index, sign in enumerate((1, -1)):
        for row, ((x, y, z), amount) in enumerate(zip(centres, fullness)):
            if amount == 0:
                continue
            centre = Vector((sign * x, y, z))
            for column in range(16):
                vertex = obj.data.vertices[231 + side_index * 144 + row * 16 + column]
                vertex.co = centre + (vertex.co - centre) * (1 + amount)
    obj.data.update()

bpy.ops.wm.open_mainfile(filepath=str(out / 'before.blend'))
obj = bpy.data.objects['Jacket_ContinuousShoulders']
before = [tuple(v.co) for v in obj.data.vertices]
update_jacket(obj)
changed = [i for i, v in enumerate(obj.data.vertices) if tuple(v.co) != before[i]]
bpy.ops.wm.save_as_mainfile(filepath=str(out / 'candidate.blend'))
tree = ast.parse((out / 'generator-before.py').read_text())
function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'export')
scope = {'bpy': bpy, 'RUNTIME': out}
exec(compile(ast.Module(body=[function], type_ignores=[]), '<existing export>', 'exec'), scope)
rig = bpy.data.objects['CHR001_Rig']
scope['export'](rig, [o for o in rig.children if o.type == 'MESH'])
Path(__file__).with_name('changed-vertices.json').write_text(json.dumps({'object': obj.name, 'indices': changed, 'count': len(changed)}, indent=2) + '\n')
print('Jacket candidate only:', len(changed), 'edited of', len(before), 'vertices')
