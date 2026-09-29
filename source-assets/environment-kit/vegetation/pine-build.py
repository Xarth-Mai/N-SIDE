"""Author/export the original N:SIDE pine; Blender 4.5 LTS, no add-ons."""

import argparse
import math
from pathlib import Path
import random
import sys

import bpy
from mathutils import Vector

SOURCE = Path(__file__).resolve().parent
MASTER = SOURCE / "pine-street.blend"
EXPORT = SOURCE / "pine-street.glb"
RNG = random.Random(49031)
VERTS, FACES, UVS, COLORS, SLOTS = [], [], [], [], []


def face(indices, uv, color=(1, 1, 1, 1), slot=0):
    FACES.append(indices)
    UVS.append(uv)
    COLORS.append(color)
    SLOTS.append(slot)


def branch(points, radii, sides=5):
    start = len(VERTS)
    distances = [0]
    for a, b in zip(points, points[1:]):
        distances.append(distances[-1] + (b - a).length / 0.85)
    repeats = max(0.2, math.tau * max(radii) / 0.65)
    for index, point in enumerate(points):
        direction = (points[min(index + 1, len(points) - 1)] - points[max(0, index - 1)]).normalized()
        u = direction.cross(Vector((0, 1, 0))).normalized()
        v = direction.cross(u).normalized()
        for side in range(sides):
            angle = side * math.tau / sides
            VERTS.append(tuple(point + radii[index] * (math.cos(angle) * u + math.sin(angle) * v)))
    cap = [(0.5 + math.cos(i * math.tau / sides) * 0.5,
            0.5 + math.sin(i * math.tau / sides) * 0.5) for i in range(sides)]
    face(tuple(start + i for i in reversed(range(sides))), list(reversed(cap)))
    for index in range(len(points) - 1):
        for side in range(sides):
            a = start + index * sides + side
            b = start + index * sides + (side + 1) % sides
            u0, u1 = side / sides * repeats, (side + 1) / sides * repeats
            face((a, b, b + sides, a + sides), [(u0, distances[index]), (u1, distances[index]),
                 (u1, distances[index + 1]), (u0, distances[index + 1])])
    face(tuple(start + (len(points) - 1) * sides + i for i in range(sides)), cap)


def needles(center, axis):
    """Opaque tapered needles form an open, three-dimensional terminal spray."""
    axis = axis.normalized()
    side = axis.cross(Vector((0, 0, 1))).normalized()
    up = side.cross(axis).normalized()
    color = RNG.choice([(0.075, 0.17, 0.072, 1), (0.09, 0.20, 0.080, 1),
                        (0.12, 0.23, 0.085, 1), (0.070, 0.15, 0.090, 1)])
    for needle in range(12):
        angle = needle * 2.39996 + RNG.uniform(-0.25, 0.25)
        radial = side * math.cos(angle) + up * math.sin(angle)
        direction = (axis * RNG.uniform(0.35, 0.95) + radial).normalized()
        base = center + axis * RNG.uniform(-0.065, 0.065)
        blade_side = direction.cross(radial + axis).normalized()
        length = RNG.uniform(0.20, 0.30)
        width = RNG.uniform(0.012, 0.020)
        start = len(VERTS)
        VERTS.extend([tuple(base - blade_side * width), tuple(base + blade_side * width),
                      tuple(base + direction * length)])
        face((start, start + 1, start + 2), [(0, 0), (1, 0), (0.5, 1)], color, 1)


def build():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    trunk = [Vector(p) for p in [(0, 0, 0), (0.035, 0.01, 0.8), (-0.04, 0.03, 1.7),
                               (0.10, 0.02, 2.8), (0.03, 0.06, 3.9), (-0.10, 0.10, 5),
                               (0.02, 0.12, 5.9), (0.10, 0.1, 6.35)]]
    branch(trunk, [0.15, 0.125, 0.10, 0.082, 0.062, 0.042, 0.022, 0.004], 9)
    # Deliberate uneven boughs: height, direction, reach, tip rise; no stacked crown shells
    boughs = [(3.05, 32, 1.30, 0.32), (3.12, 180, 1.16, 0.18), (3.40, 284, 1.27, 0.48),
              (3.55, 100, 1.38, 0.33), (3.74, 226, 1.21, 0.42), (3.88, 348, 1.28, 0.38),
              (4.08, 68, 1.05, 0.30), (4.28, 158, 1.18, 0.35), (4.46, 267, 1.10, 0.40),
              (4.58, 15, 1.08, 0.45), (4.76, 121, 0.90, 0.40), (4.96, 213, 0.92, 0.48),
              (5.15, 321, 0.75, 0.40), (5.29, 78, 0.62, 0.49), (5.53, 177, 0.55, 0.42),
              (5.75, 285, 0.40, 0.38), (5.96, 37, 0.27, 0.36)]
    for index, (height, angle, reach, rise) in enumerate(boughs):
        angle = math.radians(angle)
        outward = Vector((math.cos(angle), math.sin(angle), 0))
        side = Vector((-outward.y, outward.x, 0))
        segment = next(i for i in range(len(trunk) - 1) if trunk[i].z <= height <= trunk[i + 1].z)
        start = trunk[segment].lerp(trunk[segment + 1],
                                    (height - trunk[segment].z) / (trunk[segment + 1].z - trunk[segment].z))
        end = start + outward * reach + Vector((0, 0, rise))
        bend = start.lerp(end, 0.56) + Vector((0, 0, -0.14))
        branch([start, bend, end], [max(0.016, 0.040 - index * 0.0014), 0.018, 0.004])
        for twig in range(4):
            sign = 1 if twig % 2 else -1
            base = start.lerp(end, 0.35 + twig * 0.18)
            tip = base + outward * RNG.uniform(0.22, 0.40) + side * sign * RNG.uniform(0.22, 0.36)
            tip.z += RNG.uniform(-0.08, 0.24)
            branch([base, tip], [0.009, 0.0025], 4)
            for tuft in range(6):
                position = base.lerp(tip, 0.15 + tuft * 0.17)
                spray = (tip - base).normalized() + Vector((0, 0, RNG.uniform(0.25, 0.8)))
                needles(position, spray)
        needles(end, outward + Vector((0, 0, 1)))
    needles(trunk[-1], Vector((0.1, 0, 1)))
    bottom = min(p[2] for p in VERTS)
    height = max(p[2] for p in VERTS) - bottom
    radius = max(math.hypot(p[0], p[1]) for p in VERTS)
    xy_scale = min(1, 1.69 / radius)
    vertices = [(x * xy_scale, y * xy_scale, (z - bottom) * 6.5 / height) for x, y, z in VERTS]
    mesh = bpy.data.meshes.new("PineStreet")
    mesh.from_pydata(vertices, [], FACES)
    mesh.update()
    tree = bpy.data.objects.new("PineStreet", mesh)
    bpy.context.collection.objects.link(tree)
    for name in ["PineBark", "PineNeedles"]:
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        mat.use_backface_culling = name == "PineBark"
        shader = mat.node_tree.nodes.get("Principled BSDF")
        shader.inputs["Roughness"].default_value = 0.94
        if name == "PineBark":
            texture = mat.node_tree.nodes.new("ShaderNodeTexImage")
            texture.image = bpy.data.images.load(str(SOURCE / "bark-color.png"))
            texture.image.colorspace_settings.name = "sRGB"
            texture.image.pack()
            texture.image.filepath = "//bark-color.png"
            mat.node_tree.links.new(texture.outputs["Color"], shader.inputs["Base Color"])
        else:
            color = mat.node_tree.nodes.new("ShaderNodeVertexColor")
            color.layer_name = "NeedleColor"
            mat.node_tree.links.new(color.outputs["Color"], shader.inputs["Base Color"])
        mesh.materials.append(mat)
    mesh.uv_layers.new(name="UVMap")
    mesh.color_attributes.new(name="NeedleColor", type="FLOAT_COLOR", domain="CORNER")
    # Layer allocation can invalidate an earlier RNA layer reference
    uv = mesh.uv_layers["UVMap"]
    colors = mesh.color_attributes["NeedleColor"]
    for polygon, slot, face_uv, color in zip(mesh.polygons, SLOTS, UVS, COLORS):
        polygon.material_index = slot
        polygon.use_smooth = slot == 0
        for loop, point in zip(polygon.loop_indices, face_uv):
            uv.data[loop].uv = point
            colors.data[loop].color = color
    bpy.context.view_layer.objects.active = tree
    tree.select_set(True)
    bpy.context.scene.unit_settings.system = "METRIC"
    bpy.context.scene.unit_settings.scale_length = 1
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))
    print(f"PINE vertices={len(mesh.vertices)} triangles={sum(len(p.vertices)-2 for p in mesh.polygons)} seed=49031")


def export():
    bpy.ops.object.select_all(action="DESELECT")
    tree = bpy.data.objects["PineStreet"]
    tree.select_set(True)
    bpy.context.view_layer.objects.active = tree
    bpy.ops.export_scene.gltf(filepath=str(EXPORT), export_format="GLB", use_selection=True,
        export_animations=False, export_yup=True, export_cameras=False, export_lights=False,
        export_materials="EXPORT", export_extras=False)


def render(directory):
    directory.mkdir(parents=True, exist_ok=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 24
    scene.cycles.use_denoising = True
    scene.render.resolution_x, scene.render.resolution_y = 720, 840
    scene.render.resolution_percentage = 100
    scene.world.color = (0.35, 0.35, 0.35)
    scene.view_settings.view_transform = "AgX"
    bpy.ops.object.light_add(type="AREA", location=(4, -5, 9))
    bpy.context.object.data.energy = 1800
    bpy.context.object.data.shape = "DISK"
    bpy.context.object.data.size = 5
    bpy.context.object.rotation_euler = (Vector((0, 0, 3)) - bpy.context.object.location).to_track_quat("-Z", "Y").to_euler()
    bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -0.01))
    plane = bpy.context.object
    ground = bpy.data.materials.new("InspectionGround")
    ground.diffuse_color = (0.36, 0.37, 0.35, 1)
    plane.data.materials.append(ground)
    bpy.ops.object.camera_add()
    camera = bpy.context.object
    camera.data.type = "ORTHO"
    camera.data.ortho_scale = 7.8
    scene.camera = camera
    for index, angle in enumerate([0, 120, 240]):
        angle = math.radians(angle)
        camera.location = (11 * math.sin(angle), -11 * math.cos(angle), 5.3)
        camera.rotation_euler = (Vector((0, 0, 3.25)) - camera.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = str(directory / f"pine-{index}.png")
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", action="store_true", help="export saved master without rebuilding")
    parser.add_argument("--render-dir", type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    if args.export:
        bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    else:
        build()
    export()
    if args.render_dir:
        render(args.render_dir)
