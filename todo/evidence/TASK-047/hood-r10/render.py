"""Fixed-light before/after Hood views; run only in the coordinated CPU window"""
import hashlib
import json
import sys
from pathlib import Path
import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
output = root / 'output/characters/CHR-001/hood-r10'
views = []
for revision in (('candidate',) if '--candidate-only' in sys.argv else ('before', 'candidate')):
    master = output / (revision + '.blend')
    bpy.ops.wm.open_mainfile(filepath=str(master))
    scene = bpy.context.scene
    rig = bpy.data.objects['CHR001_Rig']
    rig.animation_data.action = bpy.data.actions['Idle']
    scene.frame_set(1)
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 12
    scene.render.threads_mode = 'FIXED'
    scene.render.threads = 2
    scene.render.resolution_x, scene.render.resolution_y = 360, 480
    scene.render.resolution_percentage = 100
    cam = scene.camera
    for label, position in (('side', (4, 0, 1.39)), ('quarter', (3, 4, 1.39)), ('back', (0, 4, 1.39))):
        cam.location = position
        cam.rotation_euler = (Vector((0, 0, 1.39)) - cam.location).to_track_quat('-Z', 'Y').to_euler()
        cam.data.ortho_scale = .55
        image = output / (revision + '-' + label + '.png')
        scene.render.filepath = str(image)
        bpy.ops.render.render(write_still=True)
        views.append({'file': str(image.relative_to(root)), 'sha256': hashlib.sha256(image.read_bytes()).hexdigest(), 'master_sha256': hashlib.sha256(master.read_bytes()).hexdigest(), 'camera_position': position, 'target': [0, 0, 1.39], 'orthographic_scale': .55, 'clip': 'Idle', 'frame': 1})
report = {'engine': 'Cycles CPU', 'threads': 2, 'samples': 12, 'resolution': [360, 480], 'lights': 'Unchanged source master', 'views': views}
Path(__file__).with_name('views.json').write_text(json.dumps(report, indent=2) + '\n')
