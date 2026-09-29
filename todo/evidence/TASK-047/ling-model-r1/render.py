"""Render the editable master with fixed lights; --motion checks side and gait views"""
import hashlib
import json
import sys
from pathlib import Path

import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
revision = sys.argv[sys.argv.index('--') + 1]
output = root / 'output/characters/CHR-002/ling-model-r1' / revision
output.mkdir(parents=True, exist_ok=True)
master = Path(sys.argv[sys.argv.index('--master') + 1]) if '--master' in sys.argv else root / 'output/characters/CHR-002/ling-model-r1' / ('baseline.blend' if revision == 'before' else revision+'.blend')
bpy.ops.wm.open_mainfile(filepath=str(master))
scene = bpy.context.scene
scene.render.threads_mode = 'FIXED'
scene.render.threads = 2
rig = bpy.data.objects['CHR002_Rig']
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
cam = scene.camera
views = []
motion = '--motion' in sys.argv
if motion:
    scene.render.resolution_x=480;scene.render.resolution_y=640;scene.cycles.samples=16
shots = tuple((f'{clip.lower()}-{i:02}',(4,0,1.05),.82,1.88,clip,1+i*period//8) for clip,period in (('Walk',48),('Run',40)) for i in range(8)) + tuple((f'run-front-{frame:02}',(0,-4,1.05),.82,1.88,'Run',frame) for frame in (1,7,13)) if motion else (
    ('body-front',(0,-4,.95),.82,1.88,'Idle',1),
    ('body-quarter',(3,-4,1.05),.82,1.88,'Idle',1),
    ('body-side',(4,0,.95),.82,1.88,'Idle',1),
    ('body-back',(0,4,.95),.82,1.88,'Idle',1),
    ('face-front',(0,-4,1.52),1.515,.38,'Idle',1),
    ('hair-quarter',(3,4,1.43),1.43,.62,'Idle',1),
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
