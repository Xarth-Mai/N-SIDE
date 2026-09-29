"""Render the editable master with fixed r1 lights, samples and four review views"""
import json
import sys
from pathlib import Path

import bpy
from mathutils import Vector

root = Path(__file__).resolve().parents[4]
revision = sys.argv[sys.argv.index('--') + 1]
output = root / 'output/characters/CHR-001/model-r2' / revision
output.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.open_mainfile(filepath=str(root / 'source-assets/characters/CHR-001/model/yao-grey-study.blend'))
scene = bpy.context.scene
rig = bpy.data.objects['CHR001_Rig']
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
cam = scene.camera
views = []
for label, position, height, scale in (
    ('face-front', (0, -4, 1.61), 1.615, .39),
    ('face-quarter', (3, -4, 1.61), 1.615, .39),
    ('face-side', (4, 0, 1.61), 1.615, .39),
    ('full-body', (0, -4, 1.0), .89, 2.06),
):
    cam.location = position
    cam.rotation_euler = (Vector((0, 0, height)) - cam.location).to_track_quat('-Z', 'Y').to_euler()
    cam.data.ortho_scale = scale
    scene.render.filepath = str(output / (label + '.png'))
    bpy.ops.render.render(write_still=True)
    views.append({'label': label, 'camera_position': position, 'target_height': height, 'ortho_scale': scale, 'file': str(Path(scene.render.filepath).relative_to(root))})
report = {'revision': revision, 'engine': scene.render.engine, 'device': scene.cycles.device, 'samples': scene.cycles.samples, 'resolution': [scene.render.resolution_x, scene.render.resolution_y], 'clip': 'Idle', 'frame': 1, 'views': views}
(Path(__file__).parent / (revision + '-views.json')).write_text(json.dumps(report, indent=2) + '\n')
