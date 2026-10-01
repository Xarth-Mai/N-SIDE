"""Preserved contracts and existing 28-pose contact diagnostics for the isolated candidate"""
import ast
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys
import bpy
from mathutils.bvhtree import BVHTree

root = Path(__file__).resolve().parents[4]
out = root / 'output/yao-drape-r13'
variant_b = '--variant-b' in sys.argv
master = out / ('candidate-b.blend' if variant_b else 'candidate.blend')
runtime = out / ('variant-b/yao-grey-study.glb' if variant_b else 'yao-grey-study.glb')
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


def cloth_details():
    data = bpy.data.objects['Jacket_ContinuousShoulders'].data
    edge_counts = Counter(tuple(sorted((p.vertices[i], p.vertices[(i + 1) % len(p.vertices)])))
                          for p in data.polygons for i in range(len(p.vertices)))
    return {'coordinates': [tuple(v.co) for v in data.vertices],
            'surfaces': {tuple(p.vertices): {
                'uv': [tuple(data.uv_layers.active.data[i].uv) for i in p.loop_indices],
                'material': p.material_index, 'smooth': p.use_smooth} for p in data.polygons},
            'nonmanifold_edges': sum(n > 2 for n in edge_counts.values())}


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
old_cloth = cloth_details()
new, new_poses = scope['jacket'](master)
new_cloth = cloth_details()
checks['original_vertices_weights_unchanged'] = old['weights'] == new['weights'][:519]
checks['vertex_groups_unchanged'] = old['groups'] == new['groups']
checks['sewn_anchors_unchanged'] = old['anchors'] == [a for a in new['anchors'] if a[0] < 519]
checks['original_hem_zipper_front_and_sleeves_unchanged'] = all(
    a == new_cloth['coordinates'][i] for i, a in enumerate(old_cloth['coordinates'])
    if i < 33 or i >= 99 or i % 33 in (0, 1, 31, 32) or a[1] <= .006)
changed_faces = {tuple(f) for f in old['faces']} - set(new_cloth['surfaces'])
expected_faces = {(66 + c, 67 + c, 100 + c, 99 + c) for c in range(32)}
checks['only_lower_body_band_faces_replaced'] = changed_faces == expected_faces
checks['unaffected_face_uv_and_material_unchanged'] = all(
    value == new_cloth['surfaces'][face] for face, value in old_cloth['surfaces'].items() if face not in changed_faces)
checks['no_new_nonmanifold_edges'] = old_cloth['nonmanifold_edges'] == new_cloth['nonmanifold_edges'] == 0
checks['three_support_rings_only'] = len(new_cloth['coordinates']) == 519 + 99
checks['head_hair_collar_clear_in_28_poses'] = all(p['BVH_overlap_pairs'][name] == 0 for p in new_poses for name in ('Face_Head', 'Hair_Cap', 'Hair_CrownBase', 'TShirt_Collar'))
baseline = json.loads(Path(__file__).with_name('source-baseline.json').read_text())
checks['formal_files_unchanged'] = all(hashlib.sha256((root / p).read_bytes()).hexdigest() == sha for p, sha in baseline['files'].items())
report = {'status': 'PASS' if all(checks.values()) else 'FAIL', 'checks': checks,
          'before_poses': old_poses, 'candidate_poses': new_poses,
          'scope': 'Only lower-back cloth changes; contacts are diagnostic, not visual acceptance',
          'sha256': {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest() for path in (out / 'before.blend', out / 'before.glb', master, runtime)}}
Path(__file__).with_name('preserved-contract-b.json' if variant_b else 'preserved-contract.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'status': report['status'], 'checks': checks}))
for name in new_poses[0]['BVH_overlap_pairs']:
    print(name, 'max overlap pairs', max(p['BVH_overlap_pairs'][name] for p in old_poses), '->', max(p['BVH_overlap_pairs'][name] for p in new_poses))
assert all(checks.values()), 'Lower-body candidate changed a preserved contract'
