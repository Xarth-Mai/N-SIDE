"""Assemble one CC0 Poly Haven straight bench; run with Blender --background."""
import json
import math
from pathlib import Path

import bpy
import bmesh
from mathutils import Quaternion, Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / "output/assets/public-street-r2"
OUT.mkdir(parents=True, exist_ok=True)
original = HERE / "original"
data = json.loads((original / "modular_street_seating_1k.gltf").read_text())
# Upstream lays out individual modules, so keep only parts used by this bench
names = {"crossbar", "legs_single", "back_support_r", "back_support_l",
         "arm_rest_01", "seat", "seat_back"}
data["nodes"] = [node for node in data["nodes"] if node["name"] in names]
assert {node["name"] for node in data["nodes"]} == names
data["scenes"] = [{"nodes": list(range(len(data["nodes"])))}]
data["scene"] = 0
for buffer in data["buffers"]:
    buffer["uri"] = str(original / buffer["uri"])
for image in data["images"]:
    image["uri"] = str(original / image["uri"])
subset = OUT / "bench-selection.gltf"
subset.write_text(json.dumps(data))
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(subset))
objects = {obj.name: obj for obj in bpy.context.scene.objects if obj.type == "MESH"}
assert set(objects) == names
for name in ("crossbar", "seat", "seat_back"):
    objects[name].location.x = 0
objects["legs_single"].location.x = -.92
objects["back_support_r"].location.x = -.84
objects["back_support_l"].location.x = .84
objects["arm_rest_01"].location.x = -.92
objects["arm_rest_01"].location.z = .45
for name in ("legs_single", "arm_rest_01"):
    source = objects[name]
    right = source.copy()
    right.data = source.data.copy()
    bpy.context.collection.objects.link(right)
    right.name = name + "_right"
    right.location.x = .92
    right.rotation_mode = 'QUATERNION'
    right.rotation_quaternion = Quaternion((0, 0, 1), math.pi) @ right.rotation_quaternion
bpy.context.view_layer.update()
meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
points = [obj.matrix_world @ v.co for obj in meshes for v in obj.data.vertices]
lo = Vector([min(p[i] for p in points) for i in range(3)])
hi = Vector([max(p[i] for p in points) for i in range(3)])
origin = Vector(((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, lo.z))
for obj in meshes:
    obj.location -= origin
    obj.select_set(True)
bpy.context.view_layer.objects.active = meshes[0]
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
bpy.ops.object.join()
bench = bpy.context.object
bench.name = "StreetBench"
bench.data.name = "StreetBench"
# Each copied upstream armrest contains one collinear triangle
mesh = bmesh.new()
mesh.from_mesh(bench.data)
degenerate = [face for face in mesh.faces if face.calc_area() < 1e-12]
assert len(degenerate) == 2
bmesh.ops.delete(mesh, geom=degenerate, context='FACES_ONLY')
mesh.to_mesh(bench.data)
mesh.free()
bpy.context.scene.unit_settings.system = "METRIC"
bpy.context.scene.unit_settings.scale_length = 1
bpy.ops.file.pack_all()
bpy.ops.wm.save_as_mainfile(filepath=str(HERE / "street-bench.blend"))
bpy.ops.export_scene.gltf(filepath=str(HERE / "street-bench.glb"), export_format="GLB",
                          use_selection=True, export_animations=False, export_yup=True,
                          export_tangents=True, export_image_format="AUTO")
print("N:SIDE_BENCH", json.dumps({"upstream_parts": sorted(names),
    "dimensions_xyz_blender": list(hi - lo), "pivot_shift": list(origin),
    "seat_height_m": .45, "front_gltf": "+Z", "up_gltf": "+Y"}))
