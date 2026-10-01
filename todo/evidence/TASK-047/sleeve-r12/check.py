"""Check the sleeve candidate with the existing preservation and contact comparisons"""
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
out = root / 'output/yao-sleeve-r12'
formal = '--formal' in sys.argv
master = root / 'source-assets/characters/CHR-001/model/yao-grey-study.blend' if formal else out / 'candidate.blend'
runtime = root / 'game/assets/characters/CHR-001/yao-grey-study.glb' if formal else out / 'yao-grey-study.glb'
sys.path.insert(0, str(root / 'tools'))
from glb import read_glb, values
scope = {'bpy': bpy, 'hashlib': hashlib, 'BVHTree': BVHTree,
         'changed': ('Jacket_ContinuousShoulders',), 'values': values}
for path, names in (('model-r9/check-contract.py', ('snapshot', 'accessor', 'animation', 'textures')),
                    ('jacket-r11/check.py', ('jacket',))):
    tree = ast.parse((root / 'todo/evidence/TASK-047' / path).read_text())
    for name in names:
        function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == name)
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
old, old_poses = scope['jacket'](out / 'before.blend')
old_coordinates = [tuple(v.co) for v in bpy.data.objects['Jacket_ContinuousShoulders'].data.vertices]
new, new_poses = scope['jacket'](master)
new_coordinates = [tuple(v.co) for v in bpy.data.objects['Jacket_ContinuousShoulders'].data.vertices]
checks.update({'jacket_' + name + '_identical': old[name] == new[name] for name in old})
allowed = {231 + side * 144 + row * 16 + col for side in range(2) for row in range(2, 7) for col in range(16)}
checks['only_five_middle_sleeve_rings_changed'] = all(a == b for i, (a, b) in enumerate(zip(old_coordinates, new_coordinates)) if i not in allowed)
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
report = {'status': 'PASS' if all(checks.values()) else 'FAIL', 'checks': checks,
          'before_poses': old_poses, 'candidate_poses': new_poses,
          'scope': 'Only five middle sleeve rings deform; numerical contacts do not establish visual quality; candidate and promoted checks report their actual paths',
          'sha256': {name: hashlib.sha256((out / name).read_bytes()).hexdigest() for name in ('before.blend', 'before.glb', 'candidate.blend', 'yao-grey-study.glb')}}
report['checked_source'] = str(master.relative_to(root))
report['checked_runtime'] = str(runtime.relative_to(root))
report['source_sha256'] = hashlib.sha256(master.read_bytes()).hexdigest()
report['runtime_sha256'] = hashlib.sha256(runtime.read_bytes()).hexdigest()
Path(__file__).with_name('promoted-contract.json' if formal else 'preserved-contract.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'status': report['status'], 'checks': checks}))
for name in new_poses[0]['BVH_overlap_pairs']:
    print(name, 'max overlap pairs', max(p['BVH_overlap_pairs'][name] for p in old_poses), '->', max(p['BVH_overlap_pairs'][name] for p in new_poses))
assert all(checks.values()), 'Sleeve edit changed a preserved contract'
