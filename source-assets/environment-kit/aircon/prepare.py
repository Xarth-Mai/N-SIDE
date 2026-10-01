"""Rebuild the clean Poly Haven aircon candidate; original downloads stay intact."""
import json
import tempfile
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parent
original = ROOT / "original"
data = json.loads((original / "exterior_aircon_unit_1k.gltf").read_text())
data["nodes"] = [data["nodes"][0]]
assert data["nodes"][0]["name"] == "exterior_aircon_unit"
data["scenes"] = [{"nodes": [0]}]
data["scene"] = 0
for item in data["buffers"] + data["images"]:
    item["uri"] = str(original / item["uri"])

bpy.ops.wm.read_factory_settings(use_empty=True)
with tempfile.TemporaryDirectory(prefix="nside-aircon-") as directory:
    selection = Path(directory) / "selection.gltf"
    selection.write_text(json.dumps(data))
    bpy.ops.import_scene.gltf(filepath=str(selection))
objects = [o for o in bpy.context.scene.objects if o.type == "MESH"]
assert len(objects) == 1 and objects[0].name == "exterior_aircon_unit"

# The upstream BLEND material puts an opaque JPEG mask in Base Color
# Reconnect the actual color and mask without changing the original source files
material = bpy.data.materials["exterior_aircon_unit_02"]
nodes, links = material.node_tree.nodes, material.node_tree.links
bsdf = next(n for n in nodes if n.type == "BSDF_PRINCIPLED")
mask = next(n for n in nodes if n.type == "TEX_IMAGE" and "opacity" in n.image.name)
for name in ("Base Color", "Alpha"):
    for link in list(bsdf.inputs[name].links):
        links.remove(link)
color = nodes.new("ShaderNodeTexImage")
color.image = bpy.data.images.load(str(original / "textures/exterior_aircon_unit_02_diff_1k.jpg"))
links.new(color.outputs["Color"], bsdf.inputs["Base Color"])
mask.image.colorspace_settings.name = "Non-Color"
threshold = nodes.new("ShaderNodeMath")
threshold.operation = "GREATER_THAN"
threshold.inputs[1].default_value = .5
links.new(mask.outputs["Color"], threshold.inputs[0])
links.new(threshold.outputs[0], bsdf.inputs["Alpha"])
material.surface_render_method = "DITHERED"

bpy.context.view_layer.update()
points = [o.matrix_world @ v.co for o in objects for v in o.data.vertices]
low = Vector([min(p[i] for p in points) for i in range(3)])
high = Vector([max(p[i] for p in points) for i in range(3)])
origin = Vector(((low.x + high.x) / 2, (low.y + high.y) / 2, low.z))
for obj in objects:
    obj.location -= origin
    obj.select_set(True)
bpy.context.view_layer.objects.active = objects[0]
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
obj = bpy.context.object
obj.name = obj.data.name = "CandidateAircon"
bpy.context.scene.unit_settings.system = "METRIC"
bpy.context.scene.unit_settings.scale_length = 1
bpy.ops.file.pack_all()
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT / "aircon-candidate.blend"))
bpy.ops.export_scene.gltf(
    filepath=str(ROOT / "aircon-candidate.glb"), export_format="GLB",
    use_selection=True, export_animations=False, export_yup=True,
    export_tangents=True, export_image_format="AUTO",
)
