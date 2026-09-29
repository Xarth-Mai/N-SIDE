"""Same-pose CPU review of the frozen b948cab master and material-only candidate."""
from pathlib import Path
import argparse
import hashlib
import json
import math
import sys

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT / 'output/characters/CHR-001/material-r1'
parser = argparse.ArgumentParser()
parser.add_argument('variant', choices=['baseline', 'candidate', 'final'])
args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
master = (ROOT / 'source-assets/characters/CHR-001/model/yao-grey-study.blend') if args.variant == 'final' else OUT / 'baseline/yao-grey-study.blend'
bpy.ops.wm.open_mainfile(filepath=str(master))
SOURCE = OUT / args.variant
SOURCE.mkdir(exist_ok=True, parents=True)
if args.variant == 'candidate':
    text = (ROOT / 'source-assets/characters/CHR-001/model/build.py').read_text()
    code = text[text.index('# Values distinguish'):text.index("\nrig = bpy.data.objects.new")]
    exec(compile(code, 'build.py:material', 'exec'))
    for obj in bpy.data.objects['CHR001_Rig'].children:
        if obj.type == 'MESH':
            obj.data.materials.clear()
            obj.data.materials.append(material)
scene = bpy.context.scene
scene.cycles.samples = 32
scene.render.threads_mode = 'FIXED'
scene.render.threads = 2
scene.render.resolution_x = 720
scene.render.resolution_y = 960
rig = bpy.data.objects['CHR001_Rig']
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
cam = scene.camera
views = [('whole', (2.5, -4, 1.4), (0, 0, .9), 2.06), ('upper', (2, -4, 1.6), (0, 0, 1.45), .73)]
for name, position, target, scale in views:
    cam.location = position
    cam.rotation_euler = (Vector(target) - cam.location).to_track_quat('-Z', 'Y').to_euler()
    cam.data.ortho_scale = scale
    scene.render.filepath = str(SOURCE / (name + '.png'))
    bpy.ops.render.render(write_still=True)
used_material = next(o for o in rig.children if o.type == 'MESH').material_slots[0].material
used_bsdf = used_material.node_tree.nodes.get('Principled BSDF')
report = {'variant': args.variant, 'master': str(master.relative_to(ROOT)), 'backend': 'Cycles CPU', 'threads': 2, 'samples': 32, 'size': [720, 960], 'clip': 'Idle', 'frame': 1, 'light': 'unchanged master area lights and world', 'views': views, 'roughness_regions': globals().get('ROUGHNESS'), 'specular_IOR_level': used_bsdf.inputs['Specular IOR Level'].default_value, 'sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in SOURCE.glob('*.png')}}
(ROOT / 'todo/evidence/TASK-047/material-r1' / (args.variant + '-render.json')).write_text(json.dumps(report, indent=2) + '\n')
