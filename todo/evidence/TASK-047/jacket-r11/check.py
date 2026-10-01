"""Exact preserved-data comparisons and 28-pose jacket contact diagnostics"""
import ast
import hashlib
import json
import math
from pathlib import Path
import sys
import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-jacket-r11'
formal = '--formal' in sys.argv
master = root / 'source-assets/characters/CHR-001/model/yao-grey-study.blend' if formal else out / 'candidate.blend'
runtime = root / 'game/assets/characters/CHR-001/yao-grey-study.glb' if formal else out / 'yao-grey-study.glb'
sys.path.insert(0, str(root / 'tools'))
from glb import read_glb, values

source = ast.parse((root / 'todo/evidence/TASK-047/model-r9/check-contract.py').read_text())
scope = {'bpy': bpy, 'hashlib': hashlib, 'changed': ('Jacket_ContinuousShoulders',), 'values': values}
for name in ('snapshot', 'accessor', 'animation', 'textures'):
    function = next(n for n in source.body if isinstance(n, ast.FunctionDef) and n.name == name)
    exec(compile(ast.Module(body=[function], type_ignores=[]), '<existing comparison>', 'exec'), scope)
before = scope['snapshot'](out / 'before.blend')
after = scope['snapshot'](master)
checks = {name + '_identical': before[name] == after[name] for name in before}
bd, bb = read_glb(out / 'before.glb')
ad, ab = read_glb(runtime)
accessor, animation, textures = (scope[n] for n in ('accessor', 'animation', 'textures'))
checks.update({
    'GLB_nodes_identical': bd['nodes'] == ad['nodes'],
    'joint_indices_identical': bd['skins'][0]['joints'] == ad['skins'][0]['joints'],
    'inverse_binds_identical': accessor(bd, bb, bd['skins'][0]['inverseBindMatrices']) == accessor(ad, ab, ad['skins'][0]['inverseBindMatrices']),
    'all_exported_clip_samples_identical': animation(bd, bb) == animation(ad, ab),
    'three_embedded_textures_identical': len(ad['images']) == 3 and textures(bd, bb) == textures(ad, ab),
    'materials_identical': bd['materials'] == ad['materials'],
})

def jacket(path):
    bpy.ops.wm.open_mainfile(filepath=str(path))
    obj = bpy.data.objects['Jacket_ContinuousShoulders']
    rig = bpy.data.objects['CHR001_Rig']
    coordinates = [list(v.co) for v in obj.data.vertices]
    anchors = []
    for i, p in enumerate(coordinates):
        if i < 231:
            row, column = divmod(i, 33)
            fixed = row >= 4 or column in (0, 1, 31, 32) or (p[1] > 0 and p[2] > 1.15)
        else:
            fixed = ((i - 231) % 144) // 16 in (0, 7, 8)
        if fixed:
            anchors.append([i, p])
    data = {'anchors': anchors, 'faces': [list(p.vertices) for p in obj.data.polygons],
            'uv': [list(u.uv) for u in obj.data.uv_layers.active.data],
            'weights': [[[g.group, g.weight] for g in v.groups] for v in obj.data.vertices],
            'groups': [g.name for g in obj.vertex_groups]}
    poses = []
    related = ('Hood', 'TShirt', 'TShirt_Collar', 'Face_Head', 'Hair_Cap', 'Hair_CrownBase', 'Palm.L', 'Palm.R', 'Sleeve_Cuff.L', 'Sleeve_Cuff.R')
    for clip, frames in (('Idle', (1, 31, 61, 91, 121)), ('Walk', (1, 9, 17, 25, 33, 41, 49)), ('Run', (1, 6, 11, 16, 21, 26, 31, 36, 41)), ('Jump', (1, 6, 13, 21, 31, 37, 41))):
        rig.animation_data.action = bpy.data.actions[clip]
        for frame in frames:
            bpy.context.scene.frame_set(frame)
            graph = bpy.context.evaluated_depsgraph_get()
            surface = BVHTree.FromObject(obj, graph)
            collisions = {name: len(surface.overlap(BVHTree.FromObject(bpy.data.objects[name], graph))) for name in related}
            poses.append({'clip': clip, 'frame': frame, 'BVH_overlap_pairs': collisions})
    return data, poses

old, old_poses = jacket(out / 'before.blend')
new, new_poses = jacket(master)
checks.update({'jacket_' + name + '_identical': old[name] == new[name] for name in old})
checks['head_hair_collar_clear_in_28_poses'] = all(p['BVH_overlap_pairs'][name] == 0 for p in new_poses for name in ('Face_Head', 'Hair_Cap', 'Hair_CrownBase', 'TShirt_Collar'))
baseline = json.loads(Path(__file__).with_name('source-baseline.json').read_text())
if formal:
    checks['promoted_GLB_matches_reviewed_candidate'] = runtime.read_bytes() == (out / 'yao-grey-study.glb').read_bytes()
    checks['CHR002_files_unchanged'] = all(hashlib.sha256((root / p).read_bytes()).hexdigest() == sha for p, sha in baseline['files'].items() if 'CHR-002' in p)
    tree = ast.parse((root / 'source-assets/characters/CHR-001/model/build.py').read_text())
    function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'update_jacket')
    functions = {'math': math, 'Vector': Vector}
    exec(compile(ast.Module(body=[function], type_ignores=[]), '<formal jacket update>', 'exec'), functions)
    obj = bpy.data.objects['Jacket_ContinuousShoulders']
    promoted_vertices = [tuple(v.co) for v in obj.data.vertices]
    functions['update_jacket'](obj)
    checks['repeat_update_keeps_promoted_vertices'] = [tuple(v.co) for v in obj.data.vertices] == promoted_vertices
    bpy.ops.wm.open_mainfile(filepath=str(out / 'before.blend'))
    obj = bpy.data.objects['Jacket_ContinuousShoulders']
    functions['update_jacket'](obj)
    checks['formal_function_reproduces_promoted_vertices'] = [tuple(v.co) for v in obj.data.vertices] == promoted_vertices
else:
    checks['formal_files_unchanged'] = all(hashlib.sha256((root / p).read_bytes()).hexdigest() == sha for p, sha in baseline['files'].items())
report = {'status': 'PASS' if all(checks.values()) else 'FAIL', 'checks': checks, 'before_poses': old_poses, 'candidate_poses': new_poses,
          'scope': 'Exact unchanged mesh/UV/weight/rig/animation/image data and fixed jacket anchors; BVH contacts are diagnostics, not visual acceptance',
          'sha256': {name: hashlib.sha256((out / name).read_bytes()).hexdigest() for name in ('before.blend', 'before.glb', 'candidate.blend', 'yao-grey-study.glb')}}
report['checked_source'] = str(master.relative_to(root))
report['checked_runtime'] = str(runtime.relative_to(root))
report['source_sha256'] = hashlib.sha256(master.read_bytes()).hexdigest()
report['runtime_sha256'] = hashlib.sha256(runtime.read_bytes()).hexdigest()
Path(__file__).with_name('promoted-contract.json' if formal else 'preserved-contract.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'status': report['status'], 'checks': checks}))
for name in new_poses[0]['BVH_overlap_pairs']:
    print(name, 'max overlap pairs', max(p['BVH_overlap_pairs'][name] for p in old_poses), '->', max(p['BVH_overlap_pairs'][name] for p in new_poses))
assert all(checks.values()), 'Jacket edit changed a preserved contract'
