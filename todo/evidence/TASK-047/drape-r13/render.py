"""Fixed full-body comparisons; never save render setup into the master"""
import hashlib
import json
import sys
from pathlib import Path
import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-drape-r13'
motion = '--motion' in sys.argv
variant_b = '--variant-b' in sys.argv
report = []
for revision in (('candidate-b',) if variant_b else ('candidate',) if '--candidate-only' in sys.argv else ('before', 'candidate')):
    master = out / (revision + '.blend')
    bpy.ops.wm.open_mainfile(filepath=str(master))
    scene = bpy.context.scene
    rig = bpy.data.objects['CHR001_Rig']
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 12
    scene.render.threads_mode = 'FIXED'
    scene.render.threads = 2
    scene.render.resolution_x, scene.render.resolution_y = 640, 480
    scene.render.resolution_percentage = 100
    cam = scene.camera
    shots = [('front', 'Idle', 1, (0, -4, 1.7)), ('quarter', 'Idle', 1, (3, -4, 1.7)), ('back-quarter', 'Idle', 1, (3, 4, 1.7)), ('back', 'Idle', 1, (0, 4, 1.7))]
    if motion:
        shots = [('run6', 'Run', 6, (2, 4, 1.6)), ('run26', 'Run', 26, (-2, 4, 1.6)), ('jump21', 'Jump', 21, (0, 4, 1.6))]
    if variant_b:
        shots = [('back', 'Idle', 1, (0, 4, 1.7)), ('back-quarter', 'Idle', 1, (3, 4, 1.7)), ('run6', 'Run', 6, (2, 4, 1.6))]
    for label, clip, frame, position in shots:
        rig.animation_data.action = bpy.data.actions[clip]
        scene.frame_set(frame)
        target = (0, 0, .90)
        cam.location = position
        cam.rotation_euler = (Vector(target) - cam.location).to_track_quat('-Z', 'Y').to_euler()
        cam.data.type = 'ORTHO'
        # At 480 px output this keeps the character roughly 300 px tall, like the real orbit
        cam.data.ortho_scale = 3.5
        path = out / f'{revision}-{label}.png'
        scene.render.filepath = str(path)
        bpy.ops.render.render(write_still=True)
        report.append({'file': str(path.relative_to(root)), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'master_sha256': hashlib.sha256(master.read_bytes()).hexdigest(), 'camera': list(position), 'target': list(target), 'ortho_scale': cam.data.ortho_scale, 'clip': clip, 'frame': frame})
Path(__file__).with_name('views-b.json' if variant_b else 'motion-views.json' if motion else 'views.json').write_text(json.dumps({'engine': 'Cycles CPU', 'threads': 2, 'samples': 12, 'resolution': [640, 480], 'lights': 'unchanged master', 'views': report}, indent=2) + '\n')
