"""Edit only the current Hood mesh; export to an ignored candidate directory"""
import ast
import math
from pathlib import Path
import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

root = Path(__file__).resolve().parents[4]
out = root / 'output/characters/CHR-001/hood-r10'
bpy.ops.wm.open_mainfile(filepath=str(out / 'before.blend'))
hood = bpy.data.objects['Hood']
old = hood.data
assert [group.name for group in hood.vertex_groups] == ['Chest', 'Neck']
assert len(old.vertices) == 145 and len(old.polygons) == 112
# Keep the two mouth loops verbatim; lower cloth drops through unequal support loops
profiles = (
    (.071, 1.273, .017, .016),
    (.076, 1.286, .055, .023),
    (.072, 1.305, .088, .034),
    (.062, 1.330, .105, .049),
    (.050, 1.355, .112, .058),
    (.039, 1.380, .106, .063),
    (.030, 1.412, .091, .061),
)
vertices, uv, faces = [], [], []
old_uv = {}
for loop in old.loops:
    old_uv[loop.vertex_index] = tuple(old.uv_layers.active.data[loop.index].uv)
rig = bpy.data.objects['CHR001_Rig']
pose_position = rig.data.pose_position
rig.data.pose_position = 'REST'
bpy.context.view_layer.update()
jacket = BVHTree.FromObject(bpy.data.objects['Jacket_ContinuousShoulders'], bpy.context.evaluated_depsgraph_get())
for y, z, rx, ry in profiles:
    for j in range(29):
        angle = .55 + (math.tau - 1.1) * j / 28
        point = Vector((rx * math.sin(angle), y - ry * math.cos(angle), z))
        hit, _, _, _ = jacket.ray_cast(Vector((point.x, 1, point.z)), Vector((0, -1, 0)))
        if hit is not None:
            blend = max(0, min(1, -math.cos(angle) * 3))
            point.y += max(0, hit.y + .008 - point.y) * blend
        vertices.append(tuple(point))
        # Retain the jacket atlas region and the mouth's original UV spacing
        v = (point.z - 1.265) / (1.444 - 1.265) * .75
        uv.append(((16 + j / 28 * 480) / 1024, (256 + 16 + v * 224) / 1024))
rig.data.pose_position = pose_position
bpy.context.view_layer.update()
for index in range(87, 145):
    vertices.append(tuple(old.vertices[index].co))
    uv.append(old_uv[index])
for k in range(len(profiles) + 1):
    for j in range(28):
        a = k * 29 + j
        faces.append((a, a + 1, a + 30, a + 29))
mesh = bpy.data.meshes.new('Hood_r10')
mesh.from_pydata(vertices, [], faces)
mesh.update()
for material in old.materials:
    mesh.materials.append(material)
layer = mesh.uv_layers.new(name=old.uv_layers.active.name)
for poly in mesh.polygons:
    poly.use_smooth = True
    for loop in poly.loop_indices:
        layer.data[loop].uv = uv[mesh.loops[loop].vertex_index]
hood.data = mesh
for name in ('Chest', 'Neck'):
    if hood.vertex_groups.get(name) is None:
        hood.vertex_groups.new(name=name)
for i, (_, _, z) in enumerate(vertices):
    t = max(0, min(1, (z - 1.37) / .12))
    if t < 1:
        hood.vertex_groups['Chest'].add([i], 1 - t, 'REPLACE')
    if t > 0:
        hood.vertex_groups['Neck'].add([i], t, 'REPLACE')
bpy.data.meshes.remove(old)
# Save before exporting: the existing exporter uses transient joined copies
bpy.ops.wm.save_as_mainfile(filepath=str(out / 'candidate.blend'))
tree = ast.parse((root / 'source-assets/characters/CHR-001/model/build.py').read_text())
export_function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'export')
scope = {'bpy': bpy, 'RUNTIME': out}
exec(compile(ast.Module(body=[export_function], type_ignores=[]), '<existing export>', 'exec'), scope)
rig = bpy.data.objects['CHR001_Rig']
scope['export'](rig, [o for o in rig.children if o.type == 'MESH'])
print('Hood candidate only:', len(vertices), 'vertices,', len(faces), 'quads')
