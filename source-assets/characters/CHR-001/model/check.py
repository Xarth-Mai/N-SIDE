"""Blender checks for the actual edited master: rig, loops and shoe contact

blender -b --python source-assets/characters/CHR-001/model/check.py
"""
import json
import math
import argparse
from pathlib import Path
import sys

import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

source = Path(__file__).resolve().parent
root = source.parents[3]
parser = argparse.ArgumentParser()
parser.add_argument('--master', type=Path, default=source / 'yao-grey-study.blend')
parser.add_argument('--output', type=Path, default=root / 'todo/evidence/TASK-047/model-r7/dcc-check.json')
args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else [])
bpy.ops.wm.open_mainfile(filepath=str(args.master))
rig = bpy.data.objects['CHR001_Rig']
scene = bpy.context.scene
results = []
meshes = [obj for obj in rig.children if obj.type == 'MESH']
assert all(len(obj.material_slots) == 1 for obj in meshes), 'Every character part must use one material slot'
materials = {obj.material_slots[0].material for obj in meshes}
assert len(materials) == 1, 'Character must retain one shared material'
material = next(iter(materials))
bsdf = material.node_tree.nodes.get('Principled BSDF')
def upstream(socket, node_type, output):
    assert len(socket.links) == 1, f'{socket.name}: expected one material input'
    link = socket.links[0]
    assert link.from_node.type == node_type and link.from_socket.name == output, f'{socket.name}: wrong material channel'
    return link.from_node
base = upstream(bsdf.inputs['Base Color'], 'TEX_IMAGE', 'Color').image
channels = upstream(bsdf.inputs['Roughness'], 'SEPARATE_COLOR', 'Green')
assert upstream(bsdf.inputs['Metallic'], 'SEPARATE_COLOR', 'Blue') == channels, 'Metallic must use the same packed map'
surface = upstream(channels.inputs['Color'], 'TEX_IMAGE', 'Color').image
normal_node = upstream(bsdf.inputs['Normal'], 'NORMAL_MAP', 'Normal')
assert normal_node.space == 'TANGENT' and normal_node.inputs['Strength'].default_value == 1, 'Normal amplitude must be baked into tangent RGB'
normal = upstream(normal_node.inputs['Color'], 'TEX_IMAGE', 'Color').image
assert base.colorspace_settings.name == 'sRGB', 'Base color must remain sRGB'
for image in (surface, normal):
    assert image.colorspace_settings.name == 'Non-Color', f'{image.name}: data texture must be linear'
for image in (base, surface, normal):
    assert tuple(image.size) == (1024, 1024) and image.packed_file, f'{image.name}: missing packed 1K image'
assert abs(bsdf.inputs['Specular IOR Level'].default_value - .24) < 1e-6, 'Unexpected candidate dielectric specular level'
pixels = list(surface.pixels)
assert all(abs(value) < 1e-6 for value in pixels[2::4]), 'Packed metallic B must be zero for all regions'
regions = [('skin', .64), ('hair', .52), ('jacket', .80), ('shirt', .88), ('pants', .76), ('shoe', .62), ('ink', .72), ('white', .86)]
surface_ranges = {}
for index, (name, expected) in enumerate(regions):
    x0, y0 = (index % 2) * 512, (index // 2) * 256
    samples = [pixels[(y * 1024 + x) * 4 + 1] for y in range(y0, y0 + 256) for x in range(x0, x0 + 512)]
    assert expected - .016 <= min(samples) <= max(samples) <= expected + .016, f'{name}: packed roughness values outside candidate range'
    surface_ranges[name] = {'minimum': min(samples), 'maximum': max(samples)}
pixels = list(normal.pixels)
assert max(abs(value - .5) for value in pixels[0::4] + pixels[1::4]) < .018, 'Seam normal exceeds the shallow candidate amplitude'
assert min(pixels[2::4]) > .99, 'Tangent normal must face outward'
material_check = {'status': 'PASS', 'slots': 1, 'embedded_source_images': 3, 'surface_channels': 'G roughness, B metallic zero', 'roughness_ranges': surface_ranges, 'normal': 'linear tangent +Y; small seam-only slopes; scale 1'}
assert scene.render.fps == 60, 'Character action source must use 60 FPS'
for name, period in (('Idle', 120), ('Walk', 48), ('Run', 40)):
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
        phase = (frame-1)/period
        for side, offset in (('L',0), ('R',.5)):
            shoe = bpy.data.objects['Sneaker_Sole.'+side].evaluated_get(graph)
            evaluated = shoe.to_mesh()
            minimum = min((shoe.matrix_world @ vertex.co).z for vertex in evaluated.vertices)
            shoe.to_mesh_clear()
            floor_min = min(floor_min, minimum)
            if name == 'Idle' or (phase+offset) % 1 <= (1/3 if name == 'Walk' else .2) + 1e-7:
                contact_errors.append(abs(minimum))
    loop_error = max(abs(a-b) for a,b in zip(*loop_matrices))
    assert loop_error < .00001, f'{name}: loop joint matrix mismatch {loop_error}'
    assert floor_min > -.005, f'{name}: floor penetration {floor_min} m'
    assert max(contact_errors) < .005, f'{name}: stance-foot ground distance {max(contact_errors)} m'
    results.append({'clip': name, 'sampled_frames': period+1, 'loop_matrix_max_delta': loop_error, 'minimum_sole_height_m': floor_min, 'max_stance_height_error_m': max(contact_errors)})
assert not any(obj.name.startswith('Zip_Tape.') for obj in meshes), 'Zipper tape must share the jacket mesh'
print_gaps=[]
for clip, frames in (('Idle',(1,)),('Walk',(8,)),('Run',(1,6,11,16,21,26,31,36))):
    rig.animation_data.action=bpy.data.actions[clip]
    for frame in frames:
        scene.frame_set(frame)
        graph=bpy.context.evaluated_depsgraph_get()
        cloth=BVHTree.FromObject(bpy.data.objects['TShirt'],graph)
        for obj in (o for o in meshes if o.name.startswith('Tee_')):
            evaluated=obj.evaluated_get(graph)
            data=evaluated.to_mesh()
            samples=[v.co for v in data.vertices]
            samples += [sum((data.vertices[i].co for i in polygon.vertices),Vector())/len(polygon.vertices) for polygon in data.polygons]
            for point in samples:
                hit,normal,_,distance=cloth.find_nearest(point)
                signed_gap=(point-hit).dot(normal)
                assert 0 < signed_gap < .002, f'{clip} {frame} {obj.name}: printed mark crosses or floats above shirt: {signed_gap}'
                print_gaps.append(signed_gap)
            evaluated.to_mesh_clear()
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
idle_matrices = [x for bone in rig.pose.bones for row in bone.matrix for x in row]
rig.animation_data.action = bpy.data.actions['Jump']
assert tuple(bpy.data.actions['Jump'].frame_range) == (1.,41.), 'Jump must span 40/60 seconds'
jump_samples=[]
for frame in range(1,42):
    scene.frame_set(frame)
    for bone in rig.pose.bones:
        assert all(math.isfinite(x) for row in bone.matrix for x in row), f'Jump {frame}: nonfinite {bone.name}'
    root_matrix=rig.pose.bones['Root'].matrix
    assert max(abs(root_matrix[r][c]-rig.data.bones['Root'].matrix_local[r][c]) for r in range(4) for c in range(4)) < 1e-5, 'Jump must not move or rotate Root'
    hips=rig.pose.bones['Hips'].head.z-rig.data.bones['Hips'].head_local.z
    assert -.1 < hips < 1e-5, 'Jump must compress the pose without adding world ascent'
    soles=[]
    graph=bpy.context.evaluated_depsgraph_get()
    for side in ['L','R']:
        shoe=bpy.data.objects['Sneaker_Sole.'+side].evaluated_get(graph);data=shoe.to_mesh()
        soles.append(min((shoe.matrix_world @ vertex.co).z for vertex in data.vertices));shoe.to_mesh_clear()
    if frame in [1,41]:
        values=[x for bone in rig.pose.bones for row in bone.matrix for x in row]
        assert max(abs(a-b) for a,b in zip(values,idle_matrices)) < .0001, 'Jump endpoints must return to Idle pose'
        assert max(abs(z) for z in soles) < .005, 'Jump endpoint sole is not grounded'
    jump_samples.append({'frame':frame,'hips_offset_m':hips,'sole_height_m':soles})
assert min(jump_samples[20]['sole_height_m']) > .15, 'Jump apex must visibly tuck both legs'
assert min(s['hips_offset_m'] for s in jump_samples[:8]) < -.04, 'Jump needs takeoff compression'
assert min(s['hips_offset_m'] for s in jump_samples[32:]) < -.03, 'Jump needs landing compression'
results.append({'clip':'Jump','duration_seconds':40/60,'source_fps':60,'loop':False,'root_static':True,'world_ascent_baked':False,'samples':jump_samples})
rig.animation_data.action = bpy.data.actions['Idle']
scene.frame_set(1)
surface = BVHTree.FromObject(bpy.data.objects['Face_Head'], bpy.context.evaluated_depsgraph_get())
feature_gaps = {}
for name in ('Mouth', 'Eye_White.L', 'Eye_White.R', 'Iris_hair.L', 'Iris_hair.R', 'Iris_ink.L', 'Iris_ink.R', 'Eye_Glint.L', 'Eye_Glint.R'):
    gaps = []
    data = bpy.data.objects[name].data
    samples = [v.co for v in data.vertices]
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
# Check actual subdivided group surfaces, including the small crown closure
hair_vertices,hair_faces=[],[]
cap_gaps=[]
for name in ('Hair_Cap','Hair_CrownBase'):
    assert all(.5 < uv.uv.x < 1 and 0 < uv.uv.y < .25 for uv in bpy.data.objects[name].data.uv_layers.active.data), f'{name}: UV escapes the gray atlas hair region'
    cap=bpy.data.objects[name].evaluated_get(bpy.context.evaluated_depsgraph_get())
    cap_mesh=cap.to_mesh()
    points=[v.co.copy() for v in cap_mesh.vertices]
    offset=len(hair_vertices)
    hair_vertices.extend(points)
    hair_faces.extend([tuple(i+offset for i in p.vertices) for p in cap_mesh.polygons])
    points += [sum((cap_mesh.vertices[i].co for i in p.vertices), Vector()) / len(p.vertices) for p in cap_mesh.polygons]
    for point in points:
        origin=Vector((0,.018,point.z));direction=(point-origin).normalized()
        hit,_,_,_=surface.ray_cast(origin,direction)
        if hit is not None:cap_gaps.append((point-origin).length-(hit-origin).length)
    cap.to_mesh_clear()
assert cap_gaps and min(cap_gaps)>-.0003, f'Hair intersects the head after subdivision: {min(cap_gaps)} m'
# Cover the back and crown with hair, not an exposed skin hole between roots
hair_surface=BVHTree.FromPolygons(hair_vertices,hair_faces)
coverage=[]
for z in (1.61,1.65,1.69,1.71):
    for index in range(9):
        angle=1.8+index*.325
        radial=Vector((math.sin(angle),-math.cos(angle),0))
        origin=Vector((0,.018,z))+radial*.4
        head_hit=surface.ray_cast(origin,-radial)[0]
        hair_hit=hair_surface.ray_cast(origin,-radial)[0]
        if head_hit is not None:
            assert hair_hit is not None, f'Uncovered back/crown at {z} m, angle {angle}'
            gap=(head_hit-origin).length-(hair_hit-origin).length
            assert gap>.0005, f'Hair does not cover head at {z} m, angle {angle}: {gap} m'
            coverage.append(gap)
top_origin=Vector((0,.018,1.85))
top_hair=hair_surface.ray_cast(top_origin,Vector((0,0,-1)))[0]
top_head=surface.ray_cast(top_origin,Vector((0,0,-1)))[0]
assert top_hair is not None and top_head is not None and top_hair.z>top_head.z+.001, 'Open crown exposes the head from above'
report = {'status':'PASS','asset':'CHR-001 modelling candidate r7 / material r1','scope':'DCC material channels and color space, joint transforms, loop endpoints, sole ground distance, facial fit, eye convexity/lid coverage, continuous nasal profile, subdivided hair clearance/coverage, hair UV and chest print fit in 10 poses; excludes game-controller speed and artistic acceptance','material':material_check,'clips':results,'facial_surface_gaps':feature_gaps,'nasal_profile':nasal_profile,'hair_cap_min_clearance_m':min(cap_gaps),'back_crown_coverage_samples':len(coverage),'back_crown_min_coverage_m':min(coverage),'top_crown_clearance_m':top_hair.z-top_head.z,'hair_uv_region':'PASS','chest_print_gap_m':{'minimum':min(print_gaps),'maximum':max(print_gaps),'poses':10}}
print(json.dumps(report,indent=2))
args.output.write_text(json.dumps(report,indent=2)+'\n')
