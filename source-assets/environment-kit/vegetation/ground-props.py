"""Original grass and rock geometry; rock textures remain ambientCG CC0 originals.

Blender 4.5, metres, Z-up source / Y-up GLB; --export-existing preserves manual edits.
"""
import argparse
import json
import math
from pathlib import Path
import random
import sys

import bpy
from mathutils import Vector

SOURCE = Path(__file__).resolve().parent
ROOT = SOURCE.parents[2]
OUT = ROOT / "output/assets/ground-props-r1"
OUT.mkdir(parents=True, exist_ok=True)
parser = argparse.ArgumentParser()
parser.add_argument("--export-existing", action="store_true")
parser.add_argument("--render", action="store_true")
args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])


def material(name, roughness):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Roughness"].default_value = roughness
    shader.inputs["Metallic"].default_value = 0
    shader.inputs["Specular IOR Level"].default_value = .5
    return mat, shader


def grass():
    rng = random.Random(49091)
    vertices, faces, uvs = [], [], []
    # Unequal small basal fans share one footprint; curved leaves taper to real tips
    for fan, (bx, by) in enumerate([(0, 0), (.12, .07), (-.10, .05), (.02, -.13)]):
        for blade in range(15):
            angle = rng.uniform(0, math.tau)
            length = rng.uniform(.15, .32) * (1 if fan == 0 else .88)
            reach = rng.uniform(.06, .15)
            width = rng.uniform(.007, .014)
            heading = Vector((math.cos(angle), math.sin(angle), 0))
            across = Vector((-heading.y, heading.x, 0))
            base = Vector((bx + rng.uniform(-.015, .015), by + rng.uniform(-.015, .015), 0))
            start = len(vertices)
            for segment in range(7):
                t = segment / 6
                center = base + heading * reach * t * t + Vector((0, 0, length * (t - .20 * t * t)))
                breadth = width * (.32 + .68 * math.sin(math.pi * t * .75)) * (1 - t)
                for side in (-1, 0, 1):
                    vertices.append(center + across * side * breadth + Vector((0, 0, .0015 * (1 - abs(side)) * math.sin(math.pi * t))))
                    uvs.append(((side + 1) / 2, t))
            for segment in range(6):
                for side in range(2):
                    a = start + segment * 3 + side
                    # The last row meets at a single tip; avoid degenerate triangles
                    faces.append((a, a + 1, a + 4) if segment == 5 else (a, a + 1, a + 4, a + 3))
    height = max(p.z for p in vertices)
    mesh = bpy.data.meshes.new("ForestGrass")
    mesh.from_pydata([(p.x, p.y, p.z * .35 / height) for p in vertices], [], faces)
    mesh.update()
    obj = bpy.data.objects.new("ForestGrass", mesh)
    bpy.context.collection.objects.link(obj)
    uv = mesh.uv_layers.new(name="UVMap")
    for poly in mesh.polygons:
        poly.use_smooth = True
        for loop in poly.loop_indices:
            uv.data[loop].uv = uvs[mesh.loops[loop].vertex_index]
    mat, shader = material("GrassBlade", .95)
    mat.use_backface_culling = False
    image = bpy.data.images.new("GrassBladeColor", width=64, height=256, alpha=False)
    pixels = []
    for y in range(256):
        t = y / 255
        for x in range(64):
            vein = .90 + .10 * math.cos((x / 63 - .5) * math.pi * 6)
            # Authored sRGB values for the PNG; opaque edge comes from actual mesh
            color = ((.24 + .18 * t), (.34 + .17 * t), (.12 + .14 * t))
            pixels.extend([c * vein for c in color] + [1])
    image.pixels.foreach_set(pixels)
    image.filepath_raw = str(SOURCE / "grass-blade-color.png")
    image.file_format = "PNG"
    image.save()
    image.pack()
    image.filepath = "//grass-blade-color.png"
    node = mat.node_tree.nodes.new("ShaderNodeTexImage")
    node.image = image
    mat.node_tree.links.new(node.outputs["Color"], shader.inputs["Base Color"])
    mesh.materials.append(mat)
    return obj


def rock():
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=3, radius=1)
    obj = bpy.context.object
    obj.name = "ForestRock"
    for vertex in obj.data.vertices:
        p = vertex.co.copy()
        scale = 1 + .075 * math.sin(p.x * 7 + p.y * 4) * math.cos(p.z * 5 - p.y * 3)
        vertex.co = (p.x * .71 * scale, p.y * .48 * scale, max(-.36, p.z * .45 * scale))
    low = min(v.co.z for v in obj.data.vertices)
    high = max(v.co.z for v in obj.data.vertices)
    for vertex in obj.data.vertices:
        vertex.co.z = (vertex.co.z - low) * .65 / (high - low)
    obj.data.update()
    uv = obj.data.uv_layers.new(name="UVMap")
    for poly in obj.data.polygons:
        poly.use_smooth = True
        axis = max(range(3), key=lambda a: abs(poly.normal[a]))
        a, b = ((1, 2), (0, 2), (0, 1))[axis]
        for loop in poly.loop_indices:
            p = obj.data.vertices[obj.data.loops[loop].vertex_index].co
            uv.data[loop].uv = ((p[a] + .8) / 1.8, (p[b] + .6) / 1.8)
    mat, shader = material("Rock043L", .93)
    for suffix, socket in (("Color", "Base Color"), ("NormalGL", None)):
        image = bpy.data.images.load(str(SOURCE.parent / "materials" / f"Rock043L_1K-JPG_{suffix}.jpg"))
        image.colorspace_settings.name = "sRGB" if socket else "Non-Color"
        image.pack()
        image.filepath = f"//../materials/Rock043L_1K-JPG_{suffix}.jpg"
        node = mat.node_tree.nodes.new("ShaderNodeTexImage")
        node.image = image
        if socket:
            mat.node_tree.links.new(node.outputs["Color"], shader.inputs[socket])
        else:
            normal = mat.node_tree.nodes.new("ShaderNodeNormalMap")
            mat.node_tree.links.new(node.outputs["Color"], normal.inputs["Color"])
            mat.node_tree.links.new(normal.outputs["Normal"], shader.inputs["Normal"])
    obj.data.materials.append(mat)
    return obj


reports = []
for name, build in (("grass-forest", grass), ("rock-forest", rock)):
    if args.export_existing:
        bpy.ops.wm.open_mainfile(filepath=str(SOURCE / f"{name}.blend"))
        obj = next(o for o in bpy.context.scene.objects if o.type == "MESH")
    else:
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False)
        obj = build()
        bpy.context.scene.unit_settings.system = "METRIC"
        bpy.context.preferences.filepaths.save_version = 0
        bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE / f"{name}.blend"))
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.export_scene.gltf(filepath=str(SOURCE / f"{name}.glb"), export_format="GLB",
                             use_selection=True, export_yup=True, export_animations=False)
    obj.data.calc_loop_triangles()
    reports.append({"name": name, "triangles": len(obj.data.loop_triangles),
                    "height": max(v.co.z for v in obj.data.vertices),
                    "radius": max(math.hypot(v.co.x, v.co.y) for v in obj.data.vertices)})
    if args.render:
        scene = bpy.context.scene
        scene.render.engine = "CYCLES"
        scene.cycles.device = "CPU"
        scene.cycles.samples = 24
        scene.render.threads_mode = "FIXED"
        scene.render.threads = 2
        scene.render.resolution_x, scene.render.resolution_y = 720, 540
        scene.render.resolution_percentage = 100
        scene.world.color = (.3, .3, .3)
        bpy.ops.object.light_add(type="AREA", location=(-2, -3, 4))
        bpy.context.object.data.energy = 350
        bpy.context.object.data.shape = "DISK"
        bpy.context.object.data.size = 4
        bpy.context.object.rotation_euler = (Vector((0, 0, .2)) - bpy.context.object.location).to_track_quat("-Z", "Y").to_euler()
        scene.world.use_nodes = True
        scene.world.node_tree.nodes.get("Background").inputs["Color"].default_value = (.3, .3, .3, 1)
        bpy.ops.object.camera_add(location=(1.35, -1.8, 1.15))
        scene.camera = bpy.context.object
        scene.camera.rotation_euler = (Vector((0, 0, .2)) - scene.camera.location).to_track_quat("-Z", "Y").to_euler()
        scene.camera.data.type = "ORTHO"
        scene.camera.data.ortho_scale = 1.8 if name.startswith("rock") else .9
        scene.render.filepath = str(OUT / f"{name}.png")
        bpy.ops.render.render(write_still=True)
(OUT / "report.json").write_text(json.dumps(reports, indent=2) + "\n")
print(json.dumps(reports))
