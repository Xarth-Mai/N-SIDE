"""Blender checks for the actual edited master: rig, loops and shoe contact

blender -b --python source-assets/characters/CHR-001/model/check.py
"""
import json
import math
from pathlib import Path

import bpy

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
report = {'status':'PASS','asset':'CHR-001 modelling candidate','scope':'DCC joint transforms, loop endpoints and sole ground distance; excludes game-controller speed and artistic acceptance','clips':results}
print(json.dumps(report,indent=2))
(root/'todo/evidence/TASK-047/model-r1/dcc-check.json').write_text(json.dumps(report,indent=2)+'\n')
