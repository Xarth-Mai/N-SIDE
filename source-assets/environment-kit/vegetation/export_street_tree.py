"""Export the editable .blend master through Blender, without regenerating the tree."""

from pathlib import Path
import math
import sys
from mathutils import Vector
import bpy

source = Path(__file__).resolve().parent
bpy.ops.wm.open_mainfile(filepath=str(source / "street-tree.blend"))
bpy.ops.object.select_all(action="DESELECT")
tree = bpy.data.objects["StreetTreeEarlyAutumn"]
tree.select_set(True)
bpy.context.view_layer.objects.active = tree
bpy.ops.export_scene.gltf(filepath=str(source / "street-tree.glb"), export_format="GLB",
    use_selection=True, export_animations=False, export_yup=True, export_cameras=False,
    export_lights=False, export_materials="EXPORT", export_extras=False)

OUT = source.parents[2] / "output/assets/task045-vegetation-r1"
if "--render" in sys.argv:
    OUT.mkdir(parents=True, exist_ok=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 24
    scene.cycles.use_denoising = True
    scene.render.resolution_x = 720
    scene.render.resolution_y = 840
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
    camera.data.ortho_scale = 7.6
    scene.camera = camera
    for index, angle in enumerate([0, 120, 240]):
        radians = math.radians(angle)
        camera.location = (11 * math.sin(radians), -11 * math.cos(radians), 5.1)
        camera.rotation_euler = (Vector((0, 0, 3)) - camera.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = str(OUT / f"tree-{index}.png")
        bpy.ops.render.render(write_still=True)
