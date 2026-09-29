"""Export the editable shrub master, optionally render actual CPU inspection views."""

import argparse
import math
from pathlib import Path
import sys
import bpy
from mathutils import Vector

source = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--render-dir', type=Path)
parser.add_argument('--old-model', action='store_true')
args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else [])
bpy.ops.wm.open_mainfile(filepath=str(source / 'shrub-courtyard.blend'))
bpy.ops.object.select_all(action='DESELECT')
obj = bpy.data.objects['CourtyardShrub']
obj.select_set(True)
bpy.context.view_layer.objects.active = obj
if args.old_model:
    bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.gltf(filepath=str(source.parents[2] / 'game/assets/environment/nature/plant_bushDetailed.glb'))
else:
    bpy.ops.export_scene.gltf(filepath=str(source / 'shrub-courtyard.glb'), export_format='GLB',
        use_selection=True, export_animations=False, export_yup=True, export_cameras=False,
        export_lights=False, export_materials='EXPORT', export_extras=False)
if args.render_dir:
    args.render_dir.mkdir(parents=True, exist_ok=True)
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 24
    scene.cycles.use_denoising = True
    scene.render.resolution_x = 720
    scene.render.resolution_y = 720
    scene.render.resolution_percentage = 100
    scene.world.color = (0.32, 0.32, 0.32)
    scene.view_settings.view_transform = 'AgX'
    bpy.ops.object.light_add(type='AREA', location=(2, -3, 4))
    lamp = bpy.context.object
    lamp.data.energy = 450
    lamp.data.shape = 'DISK'
    lamp.data.size = 3
    lamp.rotation_euler = (Vector((0, 0, 0.4)) - lamp.location).to_track_quat('-Z', 'Y').to_euler()
    bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -0.003))
    ground = bpy.data.materials.new('InspectionGround')
    ground.diffuse_color = (0.34, 0.35, 0.32, 1)
    bpy.context.object.data.materials.append(ground)
    bpy.ops.object.camera_add()
    camera = bpy.context.object
    camera.data.type = 'ORTHO'
    camera.data.ortho_scale = 1.55
    scene.camera = camera
    for name, angle, height in [('front', 0, 0.8), ('side', 90, 0.8), ('quarter', 45, 1.2), ('top', 10, 4.0)]:
        a = math.radians(angle)
        camera.location = (3 * math.sin(a), -3 * math.cos(a), height)
        camera.rotation_euler = (Vector((0, 0, 0.4)) - camera.location).to_track_quat('-Z', 'Y').to_euler()
        scene.render.filepath = str(args.render_dir / f'shrub-{name}.png')
        bpy.ops.render.render(write_still=True)
