"""Render the editable master with fixed lights; --motion checks side and gait views"""
import hashlib
import json
import sys
from pathlib import Path

import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
revision = sys.argv[sys.argv.index('--') + 1]
output = root / 'output/characters/CHR-001/model-r6' / revision
output.mkdir(parents=True, exist_ok=True)
master = Path(sys.argv[sys.argv.index('--master') + 1]) if '--master' in sys.argv else root / 'output/characters/CHR-001/model-r6' / ('baseline.blend' if revision == 'before' else revision+'.blend')
bpy.ops.wm.open_mainfile(filepath=str(master))
scene = bpy.context.scene
scene.render.threads_mode = 'FIXED'
scene.render.threads = 2
rig = bpy.data.objects['CHR001_Rig']
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
cam = scene.camera
views = []
motion = '--motion' in sys.argv
shots = (
    ('body-side', (4, 0, 1.0), .89, 2.06, 'Idle', 1),
    ('walk-quarter', (3, -4, 1.2), .89, 2.06, 'Walk', 8),
    ('run-quarter', (3, -4, 1.2), .89, 2.06, 'Run', 6),
) if motion else (
    ('face-front', (0, -4, 1.61), 1.615, .39, 'Idle', 1),
    ('face-quarter', (3, -4, 1.61), 1.615, .39, 'Idle', 1),
    ('face-side', (4, 0, 1.61), 1.615, .39, 'Idle', 1),
    ('head-back', (0, 4, 1.61), 1.615, .39, 'Idle', 1),
    ('hair-quarter', (3, 4, 1.61), 1.615, .39, 'Idle', 1),
    ('hair-top', (0, .018, 3), 1.615, .39, 'Idle', 1),
)
for label, position, height, scale, clip, frame in shots:
    rig.animation_data.action = bpy.data.actions[clip]
    scene.frame_set(frame)
    cam.location = position
    cam.rotation_euler = (Vector((0, 0, height)) - cam.location).to_track_quat('-Z', 'Y').to_euler()
    cam.data.ortho_scale = scale
    scene.render.filepath = str(output / (label + '.png'))
    bpy.ops.render.render(write_still=True)
    views.append({'label': label, 'camera_position': position, 'target_height': height, 'ortho_scale': scale, 'clip': clip, 'frame': frame, 'file': str(Path(scene.render.filepath).relative_to(root))})
report = {'revision': revision, 'master_sha256': hashlib.sha256(master.read_bytes()).hexdigest(), 'engine': scene.render.engine, 'device': scene.cycles.device, 'samples': scene.cycles.samples, 'resolution': [scene.render.resolution_x, scene.render.resolution_y], 'views': views}
(Path(__file__).parent / (revision + ('-motion-views.json' if motion else '-views.json'))).write_text(json.dumps(report, indent=2) + '\n')
