"""Compare the isolated Hood candidate with its exact frozen master and GLB"""
import ast
import hashlib
import json
import math
from pathlib import Path
import sys
import bpy
from mathutils.bvhtree import BVHTree

root = Path(__file__).resolve().parents[4]
out = root / 'output/characters/CHR-001/hood-r10'
formal = '--formal' in sys.argv
master = root / 'source-assets/characters/CHR-001/model/yao-grey-study.blend' if formal else out / 'candidate.blend'
runtime = root / 'game/assets/characters/CHR-001/yao-grey-study.glb' if formal else out / 'yao-grey-study.glb'
sys.path.insert(0, str(root / 'tools'))
from glb import read_glb, values
# Reuse the previous selective-edit comparison, changing only the declared object
previous = ast.parse((root / 'todo/evidence/TASK-047/model-r9/check-contract.py').read_text())
scope = {'bpy': bpy, 'hashlib': hashlib, 'changed': ('Hood',), 'values': values}
for name in ('snapshot', 'accessor', 'animation', 'textures'):
    function = next(n for n in previous.body if isinstance(n, ast.FunctionDef) and n.name == name)
    exec(compile(ast.Module(body=[function], type_ignores=[]), '<existing comparison>', 'exec'), scope)
before = scope['snapshot'](out / 'before.blend')
after = scope['snapshot'](master)
checks = {key + '_identical': before[key] == after[key] for key in before}
bd, bb = read_glb(out / 'before.glb')
ad, ab = read_glb(runtime)
accessor, animation, textures = (scope[n] for n in ('accessor', 'animation', 'textures'))
checks.update({
    'GLB_nodes_identical': bd['nodes'] == ad['nodes'],
    'joint_names_identical': bd['skins'][0]['joints'] == ad['skins'][0]['joints'],
    'inverse_binds_identical': accessor(bd, bb, bd['skins'][0]['inverseBindMatrices']) == accessor(ad, ab, ad['skins'][0]['inverseBindMatrices']),
    'all_exported_clip_samples_identical': animation(bd, bb) == animation(ad, ab),
    'embedded_textures_identical': len(ad['images']) == 3 and textures(bd, bb) == textures(ad, ab),
    'materials_identical': bd['materials'] == ad['materials'],
})

def hood_surface(path):
    bpy.ops.wm.open_mainfile(filepath=str(path))
    obj = bpy.data.objects['Hood']
    rows = len(obj.data.vertices) // 29
    assert len(obj.data.vertices) == rows * 29
    uv = {}
    for loop in obj.data.loops:
        uv[loop.vertex_index] = list(obj.data.uv_layers.active.data[loop.index].uv)
    mouth = [{'co': list(v.co), 'uv': uv[v.index], 'weights': [[g.group, g.weight] for g in v.groups]} for v in list(obj.data.vertices)[-58:]]
    rig = bpy.data.objects['CHR001_Rig']
    poses = []
    related = ('Jacket_ContinuousShoulders', 'Face_Head', 'TShirt_Collar', 'Hair_Cap', 'Hair_CrownBase')
    for clip, frames in (('Idle', (1, 31, 61, 91, 121)), ('Walk', (1, 9, 17, 25, 33, 41, 49)), ('Run', (1, 6, 11, 16, 21, 26, 31, 36, 41)), ('Jump', (1, 6, 13, 21, 31, 37, 41))):
        rig.animation_data.action = bpy.data.actions[clip]
        bpy.context.scene.frame_set(frame=frames[0])
        for frame in frames:
            bpy.context.scene.frame_set(frame)
            graph = bpy.context.evaluated_depsgraph_get()
            hood = BVHTree.FromObject(obj, graph)
            collisions = {}
            for name in related:
                tree = BVHTree.FromObject(bpy.data.objects[name], graph)
                collisions[name] = len(hood.overlap(tree))
            poses.append({'clip': clip, 'frame': frame, 'BVH_overlap_pairs': collisions})
    return {'mouth': mouth, 'poses': poses, 'vertices': len(obj.data.vertices), 'quads': len(obj.data.polygons),
            'bounds': [[min(v.co[i] for v in obj.data.vertices) for i in range(3)], [max(v.co[i] for v in obj.data.vertices) for i in range(3)]]}
old = hood_surface(out / 'before.blend')
new = hood_surface(master)
checks['upper_two_loops_position_UV_weights_identical'] = old['mouth'] == new['mouth']
checks['head_hair_collar_surfaces_clear_in_28_poses'] = all(count == 0 for pose in new['poses'] for name, count in pose['BVH_overlap_pairs'].items() if name != 'Jacket_ContinuousShoulders')
if formal:
    checks['promoted_GLB_matches_reviewed_candidate'] = runtime.read_bytes() == (out / 'yao-grey-study.glb').read_bytes()
else:
    checks['source_master_untouched'] = hashlib.sha256((root / 'source-assets/characters/CHR-001/model/yao-grey-study.blend').read_bytes()).hexdigest() == hashlib.sha256((out / 'before.blend').read_bytes()).hexdigest()
    checks['runtime_master_untouched'] = hashlib.sha256((root / 'game/assets/characters/CHR-001/yao-grey-study.glb').read_bytes()).hexdigest() == hashlib.sha256((out / 'before.glb').read_bytes()).hexdigest()
for row in (old, new):
    del row['mouth']
report = {'status': 'PASS' if all(checks.values()) else 'FAIL', 'checks': checks, 'before': old, 'candidate': new,
          'scope': 'Exact unchanged source meshes, rig rest, keyframe curves, GLB nodes/binds, sampled animation data, material and three embedded images. Evaluated BVH overlap is a diagnostic, not a visual acceptance.',
          'sha256': {name: hashlib.sha256((out / name).read_bytes()).hexdigest() for name in ('before.blend', 'before.glb', 'candidate.blend', 'yao-grey-study.glb')}}
report['checked_master'] = str(master.relative_to(root))
report['checked_runtime'] = str(runtime.relative_to(root))
report['checked_source_sha256'] = hashlib.sha256(master.read_bytes()).hexdigest()
report['checked_runtime_sha256'] = hashlib.sha256(runtime.read_bytes()).hexdigest()
Path(__file__).with_name('promoted-contract.json' if formal else 'preserved-contract.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'status': report['status'], 'checks': checks, 'source_vertices': [old['vertices'], new['vertices']]}))
assert all(checks.values()), 'Hood change affected a preserved contract'
