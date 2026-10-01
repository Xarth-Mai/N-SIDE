"""Two directed lower-back cloth folds, on an isolated r12 master copy"""
import ast
import json
import math
from pathlib import Path
import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-drape-r13'


def shape_lower_jacket(obj):
    assert obj.name == 'Jacket_ContinuousShoulders'
    assert obj.get('jacket_shape_revision') == 12 and len(obj.data.vertices) == 519
    old = obj.data
    vertex_groups = [(g.name, g.lock_weight) for g in obj.vertex_groups]
    vertices = [v.co.copy() for v in old.vertices]
    weights = [{g.group: g.weight for g in v.groups} for v in old.vertices]
    faces = []
    uv = []
    face_properties = []
    # Only the broad body band between z ~1.065 and ~1.19 needs support loops
    lower = list(range(66, 99))
    upper = list(range(99, 132))
    rings = [lower]
    for t in (.24, .48, .72):
        ring = []
        for a, b in zip(lower, upper):
            ring.append(len(vertices))
            vertices.append(vertices[a].lerp(vertices[b], t))
            groups = weights[a].keys() | weights[b].keys()
            weights.append({g: weights[a].get(g, 0) * (1 - t) + weights[b].get(g, 0) * t for g in groups})
        rings.append(ring)
    rings.append(upper)
    replaced = []
    for polygon in old.polygons:
        indices = tuple(polygon.vertices)
        tex = [Vector(old.uv_layers.active.data[i].uv) for i in polygon.loop_indices]
        column = indices[0] - 66
        if 0 <= column < 32 and indices == (66 + column, 67 + column, 100 + column, 99 + column):
            replaced.append(polygon.index)
            for j, (lo, hi) in enumerate(zip(rings, rings[1:])):
                t0, t1 = (0, .24, .48, .72, 1)[j:j + 2]
                faces.append((lo[column], lo[column + 1], hi[column + 1], hi[column]))
                uv.append((tex[0].lerp(tex[3], t0), tex[1].lerp(tex[2], t0), tex[1].lerp(tex[2], t1), tex[0].lerp(tex[3], t1)))
                face_properties.append((polygon.material_index, polygon.use_smooth))
        else:
            faces.append(indices)
            uv.append(tex)
            face_properties.append((polygon.material_index, polygon.use_smooth))
    assert len(replaced) == 32
    changed = []
    for i, p in enumerate(vertices):
        # Keep the entire original zipper, hem edge, upper back and both sleeves
        if i < 519 and (i >= 99 or i < 33 or i % 33 in (0, 1, 31, 32)):
            continue
        if i >= 519 and (i - 519) % 33 in (0, 1, 31, 32):
            continue
        if p.y <= .006:
            continue
        t = max(0, min(1, (p.z - 1.022) / .168))
        envelope = math.sin(math.pi * t)
        back = max(0, min(1, (p.y - .006) / .055))
        centre = .09 + .10 * t
        distance = abs(p.x) - centre
        # A broad raised fold and its shallow neighbouring valley produce one cloth turn
        displacement = (.025 * math.exp(-(distance / .025) ** 2)
                        - .007 * math.exp(-((distance - .040) / .023) ** 2)) * envelope * back
        p.y += displacement
        if abs(displacement) > 1e-8:
            changed.append({'index': i, 'displacement_m': displacement})
    data = bpy.data.meshes.new(old.name + '_drape_candidate')
    data.from_pydata(vertices, [], faces)
    for material in old.materials:
        data.materials.append(material)
    layer = data.uv_layers.new(name=old.uv_layers.active.name)
    for polygon, tex, (material_index, smooth) in zip(data.polygons, uv, face_properties):
        polygon.material_index = material_index
        polygon.use_smooth = smooth
        for loop, value in zip(polygon.loop_indices, tex):
            layer.data[loop].uv = value
    obj.data = data
    obj.vertex_groups.clear()
    for name, locked in vertex_groups:
        obj.vertex_groups.new(name=name).lock_weight = locked
    for i, groups in enumerate(weights):
        for group, weight in groups.items():
            if weight > 0:
                obj.vertex_groups[group].add([i], weight, 'REPLACE')
    data.update()
    bpy.data.meshes.remove(old)
    return {'old_vertices': 519, 'new_vertices': len(vertices), 'replaced_faces': replaced,
            'added_support_heights_fraction': [.24, .48, .72], 'displaced_vertices': changed,
            'scope': 'No global width, hem, zipper, shoulder or sleeve movement; lower-back folds only'}


bpy.ops.wm.open_mainfile(filepath=str(out / 'before.blend'))
report = shape_lower_jacket(bpy.data.objects['Jacket_ContinuousShoulders'])
bpy.ops.wm.save_as_mainfile(filepath=str(out / 'candidate.blend'))
tree = ast.parse((out / 'generator-before.py').read_text())
function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'export')
scope = {'bpy': bpy, 'RUNTIME': out}
exec(compile(ast.Module(body=[function], type_ignores=[]), '<existing exporter>', 'exec'), scope)
rig = bpy.data.objects['CHR001_Rig']
scope['export'](rig, [o for o in rig.children if o.type == 'MESH'])
Path(__file__).with_name('geometry-change.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'vertices': report['new_vertices'], 'replaced_faces': len(report['replaced_faces']),
                  'changed_points': len(report['displaced_vertices'])}))
