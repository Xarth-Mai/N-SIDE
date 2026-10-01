"""Flatten two sleeve panels and direct one outer-elbow fold without inflating the figure"""
import ast
import json
import math
from pathlib import Path
import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-sleeve-r12'

def shape_sleeves(obj):
    assert obj.name == 'Jacket_ContinuousShoulders' and len(obj.data.vertices) == 519
    assert obj.get('jacket_shape_revision') == 11
    # Only five middle rings: sewn armhole, shoulder transition and cuff anchors stay fixed
    profiles = ((2, (.309, -.006, 1.191), .048 * 1.42, .94, .80),
                (3, (.329, -.006, 1.168), .046 * 1.47, .90, .78),
                (4, (.342, -.003, 1.142), .041 * 1.45, .86, .76),
                (5, (.354, -.007, 1.116), .048 * 1.40, .90, .78),
                (6, (.389, -.012, 1.039), .041 * 1.26, .96, .84))
    for side_index, sign in enumerate((1, -1)):
        across = Vector((sign * .8, 0, .6))
        along = Vector((sign * .6, 0, -.8))
        for row, (x, y, z), depth, width_scale, depth_scale in profiles:
            centre = Vector((sign * x, y, z))
            for column in range(16):
                vertex = obj.data.vertices[231 + side_index * 144 + row * 16 + column]
                delta = vertex.co - centre
                u, v, t = delta.dot(across), delta.y, delta.dot(along)
                # Broad front/back cloth planes replace a uniformly rounded cross-section
                panel = math.copysign(min(abs(v) / depth / .65, 1), v) * depth * depth_scale
                outer = max(0, min(1, u / .075)) ** 2
                crease = .006 * outer if row == 4 else 0
                diagonal = .012 * (v / depth) * outer if row == 4 else 0
                vertex.co = centre + across * (u * width_scale - crease) + Vector((0, panel, 0)) + along * (t + diagonal)
    obj.data.update()

bpy.ops.wm.open_mainfile(filepath=str(out / 'before.blend'))
obj = bpy.data.objects['Jacket_ContinuousShoulders']
before = [tuple(v.co) for v in obj.data.vertices]
shape_sleeves(obj)
changed = [i for i, v in enumerate(obj.data.vertices) if tuple(v.co) != before[i]]
bpy.ops.wm.save_as_mainfile(filepath=str(out / 'candidate.blend'))
tree = ast.parse((out / 'generator-before.py').read_text())
function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'export')
scope = {'bpy': bpy, 'RUNTIME': out}
exec(compile(ast.Module(body=[function], type_ignores=[]), '<existing export>', 'exec'), scope)
rig = bpy.data.objects['CHR001_Rig']
scope['export'](rig, [o for o in rig.children if o.type == 'MESH'])
Path(__file__).with_name('changed-vertices.json').write_text(json.dumps({'object': obj.name, 'indices': changed, 'count': len(changed)}, indent=2) + '\n')
print('Sleeve candidate only:', len(changed), 'changed vertices')
