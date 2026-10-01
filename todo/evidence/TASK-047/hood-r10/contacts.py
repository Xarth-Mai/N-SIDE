"""Locate evaluated triangle-overlap diagnostics in the candidate's Idle frame"""
from pathlib import Path
import json
import bpy
from mathutils.bvhtree import BVHTree
root = Path(__file__).resolve().parents[4]
output = root / 'output/characters/CHR-001/hood-r10'
bpy.ops.wm.open_mainfile(filepath=str(output / 'candidate.blend'))
rig = bpy.data.objects['CHR001_Rig']
rig.animation_data.action = bpy.data.actions['Idle']
bpy.context.scene.frame_set(1)
graph = bpy.context.evaluated_depsgraph_get()
obj = bpy.data.objects['Hood']
hood = BVHTree.FromObject(obj, graph)
evaluated = obj.evaluated_get(graph)
mesh = evaluated.to_mesh()
mesh.calc_loop_triangles()
records = {}
for name in ('Jacket_ContinuousShoulders', 'TShirt_Collar'):
    pairs = hood.overlap(BVHTree.FromObject(bpy.data.objects[name], graph))
    indices = sorted({a for a, b in pairs})
    coords = []
    # BVHTree.FromObject indexes evaluated polygons; read their actual vertex positions
    for index in indices:
        face = mesh.polygons[index]
        coords.extend(list(mesh.vertices[i].co) for i in face.vertices)
    records[name] = {'pairs': len(pairs), 'hood_polygons': indices,
                     'bounds': [[min(p[i] for p in coords) for i in range(3)], [max(p[i] for p in coords) for i in range(3)]] if coords else None}
evaluated.to_mesh_clear()
print(json.dumps(records, indent=2))
Path(__file__).with_name('contacts.json').write_text(json.dumps(records, indent=2) + '\n')
