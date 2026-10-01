"""CPU-only visual inspection of the exported bench; not a game capture."""
from pathlib import Path
import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT / 'output/assets/public-street-r2'
OUT.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(ROOT / 'source-assets/environment-kit/street-furniture/street-bench.glb'))
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.device = 'CPU'
scene.cycles.samples = 32
scene.render.resolution_x = 1100
scene.render.resolution_y = 750
scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = 'PNG'
scene.world = bpy.data.worlds.new('Neutral inspection')
scene.world.use_nodes = True
scene.world.node_tree.nodes['Background'].inputs['Color'].default_value = (.5, .5, .5, 1)
scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = .5
bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -.003))
floor = bpy.context.object
mat = bpy.data.materials.new('Inspection ground')
mat.diffuse_color = (.21, .23, .25, 1)
floor.data.materials.append(mat)
bpy.ops.object.light_add(type='AREA', location=(-1.5, -2.5, 4))
bpy.context.object.data.energy = 500
bpy.context.object.data.size = 4
bpy.ops.object.camera_add()
cam = bpy.context.object
cam.data.type = 'ORTHO'
cam.data.ortho_scale = 2.85
scene.camera = cam
target = Vector((0, 0, .43))
for label, position in [('front', (2.8, -4.1, 2.5)), ('back', (-2.8, 4.1, 2.5))]:
    cam.location = Vector(position)
    cam.rotation_euler = (target - cam.location).to_track_quat('-Z', 'Y').to_euler()
    scene.render.filepath = str(OUT / f'bench-{label}.png')
    bpy.ops.render.render(write_still=True)
