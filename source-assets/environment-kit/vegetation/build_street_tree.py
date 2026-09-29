"""Author N:SIDE's early-autumn street tree with Blender 4.5 LTS, without add-ons.

Run from the repository root to regenerate the authored form.
The saved .blend is editable; the fixed-seed script preserves the first authored form.
"""

import math
from pathlib import Path
import random

import bpy
from mathutils import Vector

SOURCE = Path(__file__).resolve().parent
RNG = random.Random(45019)
VERTS, FACES, SLOTS, SMOOTH = [], [], [], []


def face(indices, slot=0, smooth=True):
    FACES.append(indices)
    SLOTS.append(slot)
    SMOOTH.append(smooth)


def branch(points, radii, sides=7):
    start = len(VERTS)
    for index, point in enumerate(points):
        direction = (points[min(index + 1, len(points) - 1)] - points[max(0, index - 1)]).normalized()
        u = direction.cross(Vector((0, 1, 0))).normalized()
        v = direction.cross(u).normalized()
        for side in range(sides):
            angle = side * math.tau / sides
            VERTS.append(tuple(point + radii[index] * (math.cos(angle) * u + math.sin(angle) * v)))
    face(tuple(start + side for side in reversed(range(sides))))
    for index in range(len(points) - 1):
        for side in range(sides):
            a = start + index * sides + side
            b = start + index * sides + (side + 1) % sides
            face((a, b, b + sides, a + sides))
    face(tuple(start + (len(points) - 1) * sides + side for side in range(sides)))


def leaf(base, direction, length):
    # Curved opaque geometry: a central ridge catches light, no cutout texture/alpha.
    direction.normalize()
    sideways = direction.cross(Vector((0, 0, 1)))
    if sideways.length < 0.01:
        sideways = Vector((1, 0, 0))
    sideways.normalize()
    normal = sideways.cross(direction).normalized()
    width = length * RNG.uniform(0.27, 0.39)
    start = len(VERTS)
    VERTS.extend(tuple(p) for p in (
        base,
        base + direction * length * 0.25 + sideways * width * 0.76,
        base + direction * length * 0.68 + sideways * width,
        base + direction * length,
        base + direction * length * 0.68 - sideways * width,
        base + direction * length * 0.25 - sideways * width * 0.76,
        base + direction * length * 0.48 + normal * length * 0.045,
    ))
    slot = RNG.choices([1, 2, 3, 4], [41, 31, 22, 6])[0]
    for index in range(6):
        face((start + index, start + (index + 1) % 6, start + 6), slot, True)


bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
trunk = [Vector(p) for p in [(0, 0, 0), (0.025, 0.015, 0.7), (-0.045, 0.015, 1.6),
                           (0.045, 0.035, 2.45), (0.10, 0.025, 3.25), (0.22, 0.03, 4.05), (0.18, 0.08, 4.8)]]
branch(trunk, [0.18, 0.15, 0.125, 0.108, 0.075, 0.043, 0.012], 10)
# Bough endpoints deliberately form a rising asymmetric crown, not concentric spheres.
BOUGHS = [(-1.90, -0.40, 3.50), (1.82, 0.28, 3.73), (-0.35, 1.90, 3.83),
          (0.70, -1.92, 3.87), (-1.45, 1.38, 4.40), (1.37, -1.15, 4.56),
          (1.29, 1.37, 4.72), (-1.38, -1.25, 4.88), (-0.50, 0.30, 5.17),
          (0.58, -0.20, 5.36), (0.11, 1.12, 5.16)]
for index, endpoint in enumerate(BOUGHS):
    end = Vector(endpoint)
    start = Vector((0.02, 0.015, 2.27 + index * 0.18))
    middle = start.lerp(end, 0.52) + Vector((0, 0, -0.13))
    branch([start, middle, end], [0.075 - index * 0.003, 0.038, 0.009])
    for twig in range(11):
        angle = twig * 2.39996 + index * 0.7
        radial = RNG.uniform(0.30, 0.87)
        tip = end + Vector((math.cos(angle) * radial, math.sin(angle) * radial,
                            RNG.uniform(-0.65, 0.50)))
        base = middle.lerp(end, RNG.uniform(0.46, 0.94))
        bend = base.lerp(tip, 0.55) + Vector((0, 0, 0.08))
        branch([base, bend, tip], [0.014, 0.009, 0.0035], 5)
        along = (tip - base).normalized()
        side = Vector((-along.y, along.x, 0)).normalized()
        for pair in range(3):
            position = bend.lerp(tip, 0.12 + pair * 0.30)
            for sign in [-1, 1]:
                direction = along * RNG.uniform(0.20, 0.65) + side * sign + Vector((0, 0, RNG.uniform(-0.16, 0.65)))
                leaf(position, direction, RNG.uniform(0.30, 0.46))
        leaf(tip, along + Vector((0, 0, 0.23)), 0.35)

bottom = min(p[2] for p in VERTS)
height = max(p[2] for p in VERTS) - bottom
VERTS = [(p[0] * 6 / height, p[1] * 6 / height, (p[2] - bottom) * 6 / height) for p in VERTS]
mesh = bpy.data.meshes.new("StreetTreeEarlyAutumn")
mesh.from_pydata(VERTS, [], FACES)
mesh.update()
tree = bpy.data.objects.new("StreetTreeEarlyAutumn", mesh)
bpy.context.collection.objects.link(tree)
colors = [(0.115, 0.071, 0.044, 1), (0.13, 0.27, 0.052, 1), (0.20, 0.33, 0.070, 1),
          (0.30, 0.36, 0.092, 1), (0.43, 0.25, 0.066, 1)]
for name, color in [("Bark", colors[0]), ("EarlyAutumnLeaves", (1, 1, 1, 1))]:
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    material.diffuse_color = color
    shader = material.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = color
    shader.inputs["Roughness"].default_value = 0.92
    material.use_backface_culling = name == "Bark"
    if name != "Bark":
        vertex_color = material.node_tree.nodes.new("ShaderNodeVertexColor")
        vertex_color.layer_name = "LeafColor"
        material.node_tree.links.new(vertex_color.outputs["Color"], shader.inputs["Base Color"])
    mesh.materials.append(material)
leaf_colors = mesh.color_attributes.new(name="LeafColor", type="FLOAT_COLOR", domain="CORNER")
for polygon, slot, smooth in zip(mesh.polygons, SLOTS, SMOOTH):
    polygon.material_index = min(slot, 1)
    polygon.use_smooth = smooth
    for loop in polygon.loop_indices:
        leaf_colors.data[loop].color = colors[slot] if slot else (1, 1, 1, 1)
# A usable source UV layer; no bitmap is currently needed for these material colors.
uv = mesh.uv_layers.new(name="UVMap")
for polygon in mesh.polygons:
    for loop in polygon.loop_indices:
        p = mesh.vertices[mesh.loops[loop].vertex_index].co
        uv.data[loop].uv = (p.x * 0.5, p.z * 0.5)
bpy.context.view_layer.objects.active = tree
tree.select_set(True)
bpy.context.scene.unit_settings.system = "METRIC"
bpy.context.scene.unit_settings.scale_length = 1
bpy.context.scene.render.fps = 30
bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE / "street-tree.blend"))
bpy.ops.export_scene.gltf(filepath=str(SOURCE / "street-tree.glb"), export_format="GLB",
    use_selection=True, export_animations=False, export_yup=True, export_cameras=False,
    export_lights=False, export_materials="EXPORT", export_extras=False)
print(f"TREE vertices={len(mesh.vertices)} triangles={sum(len(p.vertices)-2 for p in mesh.polygons)} height_m=6 seed=45019")
