"""Author the original N:SIDE courtyard shrub; rebuilding replaces manual .blend edits."""

import math
from pathlib import Path
import random

import bpy
from mathutils import Vector

SOURCE = Path(__file__).resolve().parent
RNG = random.Random(49031)
VERTS, FACES, COLORS, SMOOTH, UVS = [], [], [], [], []
PALETTE = [(0.075, 0.205, 0.052, 1), (0.13, 0.285, 0.075, 1),
           (0.20, 0.34, 0.11, 1), (0.26, 0.36, 0.12, 1)]


def face(indices, color, smooth, uv):
    FACES.append(indices)
    COLORS.append(color)
    SMOOTH.append(smooth)
    UVS.append(uv)


def stem(points, radii, sides=4):
    first = len(VERTS)
    for i, p in enumerate(points):
        direction = (points[min(i + 1, len(points) - 1)] - points[max(i - 1, 0)]).normalized()
        across = direction.cross(Vector((0, 1, 0))).normalized()
        up = direction.cross(across)
        for j in range(sides):
            angle = j * math.tau / sides
            VERTS.append(p + radii[i] * (across * math.cos(angle) + up * math.sin(angle)))
    brown = (0.14, 0.095, 0.055, 1)
    for i in range(len(points) - 1):
        for j in range(sides):
            a = first + i * sides + j
            b = first + i * sides + (j + 1) % sides
            face((a, b, b + sides, a + sides), brown, True,
                 [(j / sides, i), ((j + 1) / sides, i), ((j + 1) / sides, i + 1), (j / sides, i + 1)])
    face(tuple(first + j for j in reversed(range(sides))), brown, False,
         [(0.5 + 0.5 * math.cos(j * math.tau / sides), 0.5 + 0.5 * math.sin(j * math.tau / sides)) for j in reversed(range(sides))])
    face(tuple(first + (len(points) - 1) * sides + j for j in range(sides)), brown, False,
         [(0.5 + 0.5 * math.cos(j * math.tau / sides), 0.5 + 0.5 * math.sin(j * math.tau / sides)) for j in range(sides)])


def leaf(base, direction, length, color):
    direction.normalize()
    side = direction.cross(Vector((0, 0, 1)))
    if side.length < 0.01:
        side = Vector((1, 0, 0))
    side.normalize()
    normal = side.cross(direction).normalized()
    roll = RNG.uniform(-0.85, 0.85)
    side = side * math.cos(roll) + normal * math.sin(roll)
    normal = side.cross(direction).normalized()
    start = len(VERTS)
    outline = [(0, 0), (0.28, 0.78), (0.65, 1), (1, 0), (0.65, -1), (0.28, -0.78)]
    for along, across in outline:
        VERTS.append(base + direction * length * along + side * length * 0.32 * across
                     - normal * length * (0.07 * along * along + 0.09 * abs(across)))
    for j in range(1, 5):
        indices = [0, j, j + 1]
        face(tuple(start + k for k in indices), color, False,
             [(0.5 + outline[k][1] * 0.5, outline[k][0]) for k in indices])


bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
# Unequal basal shoots carry low, outward foliage and narrower rising tips
ENDS = []
for i in range(11):
    angle = i * 2.39996
    reach = [0.43, 0.39, 0.47, 0.36][i % 4]
    ENDS.append(Vector((math.cos(angle) * reach, math.sin(angle) * reach,
                        [0.44, 0.54, 0.48, 0.61][i % 4])))
ENDS.extend(Vector(p) for p in [(0.10, 0.04, 0.72), (-0.09, 0.16, 0.67), (0.08, -0.15, 0.66)])
for i, tip in enumerate(ENDS):
    base = Vector((tip.x * 0.14, tip.y * 0.14, 0.008))
    bend = base.lerp(tip, 0.48) + Vector((0, 0, 0.055))
    stem([base, bend, tip], [0.009, 0.006, 0.0025], 5)
    axis = (tip - bend).normalized()
    sideways = Vector((-tip.y, tip.x, 0)).normalized()
    outward = Vector((tip.x, tip.y, 0)).normalized()
    for j in range(7):
        t = 0.23 + j * 0.115
        at = base.lerp(bend, t / 0.48) if t < 0.48 else bend.lerp(tip, (t - 0.48) / 0.52)
        sign = -1 if j % 2 else 1
        reach = RNG.uniform(0.08, 0.13) if j < 3 else RNG.uniform(0.02, 0.07)
        end = at + sideways * sign * RNG.uniform(0.12, 0.21) + outward * reach
        end.z += RNG.uniform(-0.07, 0.10)
        middle = at.lerp(end, 0.55) + Vector((0, 0, 0.018))
        stem([at, middle, end], [0.0038, 0.0026, 0.001], 3)
        twig_axis = (end - at).normalized()
        leaf_side = Vector((-twig_axis.y, twig_axis.x, 0)).normalized()
        shade = PALETTE[(i + j // 3) % len(PALETTE)]
        for k in range(9):
            point = at.lerp(end, 0.13 + k * 0.10)
            direction = twig_axis * RNG.uniform(0.2, 0.6) + leaf_side * (-1 if k % 2 else 1)
            direction.z += RNG.uniform(-0.4, 0.7)
            leaf(point, direction, RNG.uniform(0.055, 0.082), shade)
        leaf(end, twig_axis + Vector((0, 0, 0.22)), 0.065, shade)
    for direction in [axis, axis + sideways, axis - sideways]:
        leaf(tip, direction, 0.067, PALETTE[i % len(PALETTE)])

low = min(p.z for p in VERTS)
height = max(p.z for p in VERTS) - low
radius = max(math.hypot(p.x, p.y) for p in VERTS)
# Preserve the old normalized shrub's 0.8 m height and 0.669339 m horizontal envelope
xy_scale = min(1, 0.65 / radius)
VERTS = [(p.x * xy_scale, p.y * xy_scale, (p.z - low) * 0.8 / height) for p in VERTS]
mesh = bpy.data.meshes.new("CourtyardShrub")
mesh.from_pydata(VERTS, [], FACES)
mesh.update()
obj = bpy.data.objects.new("CourtyardShrub", mesh)
bpy.context.collection.objects.link(obj)
material = bpy.data.materials.new("ShrubVertexColor")
material.use_nodes = True
material.use_backface_culling = False
material.diffuse_color = PALETTE[1]
shader = material.node_tree.nodes.get("Principled BSDF")
shader.inputs["Roughness"].default_value = 0.95
shader.inputs["Metallic"].default_value = 0
color = material.node_tree.nodes.new("ShaderNodeVertexColor")
color.layer_name = "PlantColor"
material.node_tree.links.new(color.outputs["Color"], shader.inputs["Base Color"])
mesh.materials.append(material)
colors = mesh.color_attributes.new(name="PlantColor", type="FLOAT_COLOR", domain="CORNER")
uv = mesh.uv_layers.new(name="UVMap")
for polygon, tint, smooth, coords in zip(mesh.polygons, COLORS, SMOOTH, UVS):
    polygon.use_smooth = smooth
    for loop, coord in zip(polygon.loop_indices, coords):
        colors.data[loop].color = tint
        uv.data[loop].uv = coord
bpy.context.view_layer.objects.active = obj
obj.select_set(True)
bpy.context.scene.unit_settings.system = "METRIC"
bpy.context.scene.unit_settings.scale_length = 1
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE / "shrub-courtyard.blend"))
bpy.ops.export_scene.gltf(filepath=str(SOURCE / "shrub-courtyard.glb"), export_format="GLB",
    use_selection=True, export_animations=False, export_yup=True, export_cameras=False,
    export_lights=False, export_materials="EXPORT", export_extras=False)
print(f"SHRUB vertices={len(mesh.vertices)} triangles={sum(len(p.vertices)-2 for p in mesh.polygons)} leaves={len(ENDS)*73} height_m=0.8 radius_m={radius*xy_scale:.6f} seed=49031")
