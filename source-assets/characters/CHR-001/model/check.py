"""Blender checks for the actual edited master: rig, loops and shoe contact

blender -b --python source-assets/characters/CHR-001/model/check.py
"""
import json
import math
from pathlib import Path

import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

source = Path(__file__).resolve().parent
root = source.parents[3]
bpy.ops.wm.open_mainfile(filepath=str(source / 'yao-grey-study.blend'))
rig = bpy.data.objects['CHR001_Rig']
scene = bpy.context.scene
results = []
for name, period in (('Idle', 60), ('Walk', 30), ('Run', 20)):
    rig.animation_data.action = bpy.data.actions[name]
    loop_matrices = []
    contact_errors = []
    floor_min = math.inf
    for frame in range(1, period+2):
        scene.frame_set(frame)
        for bone in rig.pose.bones:
            assert all(math.isfinite(x) for row in bone.matrix for x in row), f'{name} frame {frame}: nonfinite {bone.name}'
        root_matrix = rig.pose.bones['Root'].matrix
        assert (root_matrix.translation - rig.data.bones['Root'].matrix_local.translation).length < .00001, f'{name} frame {frame}: Root moved'
        if frame in (1, period+1):
            loop_matrices.append([x for bone in rig.pose.bones for row in bone.matrix for x in row])
        graph = bpy.context.evaluated_depsgraph_get()
        phase = (frame-1)/period*math.tau
        for side, offset in (('L',0), ('R',math.pi)):
            shoe = bpy.data.objects['Sneaker_Sole.'+side].evaluated_get(graph)
            evaluated = shoe.to_mesh()
            minimum = min((shoe.matrix_world @ vertex.co).z for vertex in evaluated.vertices)
            shoe.to_mesh_clear()
            floor_min = min(floor_min, minimum)
            if name == 'Idle' or math.cos(phase+offset) <= 1e-7:
                contact_errors.append(abs(minimum))
    loop_error = max(abs(a-b) for a,b in zip(*loop_matrices))
    assert loop_error < .00001, f'{name}: loop joint matrix mismatch {loop_error}'
    assert floor_min > -.015, f'{name}: floor penetration {floor_min} m'
    assert max(contact_errors) < .015, f'{name}: stance-foot ground distance {max(contact_errors)} m'
    results.append({'clip': name, 'sampled_frames': period+1, 'loop_matrix_max_delta': loop_error, 'minimum_sole_height_m': floor_min, 'max_stance_height_error_m': max(contact_errors)})
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
surface = BVHTree.FromObject(bpy.data.objects['Face_Head'], bpy.context.evaluated_depsgraph_get())
feature_gaps = {}
for name in ('Mouth', 'Eye_White.L', 'Eye_White.R', 'Iris_hair.L', 'Iris_hair.R', 'Iris_ink.L', 'Iris_ink.R', 'Eye_Glint.L', 'Eye_Glint.R'):
    gaps = []
    data = bpy.data.objects[name].data
    samples = [v.co for v in data.vertices]
    if name != 'Mouth':
        samples += [sum((data.vertices[i].co for i in polygon.vertices), Vector()) / len(polygon.vertices) for polygon in data.polygons]
    for point in samples:
        hit, _, _, _ = surface.ray_cast(Vector((point.x, -1, point.z)), Vector((0, 1, 0)))
        assert hit is not None, f'{name}: feature misses face surface'
        gaps.append(hit.y - point.y)
    limit = .0041 if name != 'Mouth' else .0021
    assert max(gaps) < limit, f'{name}: floating facial feature {max(gaps)} m'
    assert min(gaps) > -.0006, f'{name}: facial feature sinks into face {min(gaps)} m'
    if name != 'Mouth':
        assert min(gaps) > 0, f'{name}: eye patch crosses face surface'
    if name.startswith('Eye_White'):
        # The white is a shallow convex surface, but its open boundary must sit
        # against the face rather than floating the entire eye like a sticker
        edge_counts = {}
        for polygon in data.polygons:
            for edge in polygon.edge_keys:
                edge_counts[edge] = edge_counts.get(edge, 0) + 1
        boundary = {i for edge, count in edge_counts.items() if count == 1 for i in edge}
        assert boundary, f'{name}: eye has no open lid boundary'
        assert max(gaps[i] for i in boundary) < .0011, f'{name}: floating eye boundary'
        assert max(gaps) > .002, f'{name}: eye surface lost its convex volume'
    if name.startswith('Iris_'):
        white = BVHTree.FromObject(bpy.data.objects['Eye_White.' + name[-1]], bpy.context.evaluated_depsgraph_get())
        for point in samples:
            hit, _, _, _ = white.ray_cast(Vector((point.x, -1, point.z)), Vector((0, 1, 0)))
            assert hit is not None, f'{name}: iris extends beyond the lid opening'
            assert 0 < hit.y - point.y < .0018, f'{name}: iris crosses or floats above the eye surface'
    feature_gaps[name] = {'minimum_m': min(gaps), 'maximum_m': max(gaps)}
assert 'Nose' not in bpy.data.objects, 'Nose must be part of the continuous head mesh'
nasal_profile = []
for z in (1.538, 1.543, 1.552, 1.563):
    points = [surface.ray_cast(Vector((x, -1, z)), Vector((0, 1, 0)))[0] for x in (-.02, 0, .02)]
    assert all(point is not None for point in points), 'Nasal profile missed head mesh'
    nasal_profile.append({'height_m': z, 'projection_from_cheeks_m': (points[0].y + points[2].y) / 2 - points[1].y})
assert .004 < nasal_profile[1]['projection_from_cheeks_m'] < .018, 'Integrated nose projection outside this candidate range'
report = {'status':'PASS','asset':'CHR-001 modelling candidate r4','scope':'DCC joint transforms, loop endpoints, sole ground distance, facial fit, eye convexity/lid coverage and continuous nasal profile; excludes game-controller speed and artistic acceptance','clips':results,'facial_surface_gaps':feature_gaps,'nasal_profile':nasal_profile}
print(json.dumps(report,indent=2))
(root/'todo/evidence/TASK-047/model-r4/dcc-check.json').write_text(json.dumps(report,indent=2)+'\n')
