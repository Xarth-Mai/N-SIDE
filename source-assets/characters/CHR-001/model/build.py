"""Author the CHR-001 uncoloured, rigged modelling candidate with Blender 4.5

Run from the repository root; --render adds disposable CPU Cycles review images
The .blend is the editable DCC master; this script reproduces this initial revision
"""
from pathlib import Path
import argparse
import json
import math
import sys
import bmesh
import bpy
from mathutils import Quaternion, Vector
from mathutils.bvhtree import BVHTree

ROOT = Path(__file__).resolve().parents[4]
SOURCE = Path(__file__).resolve().parent
RUNTIME = ROOT / 'game/assets/characters/CHR-001'
OUTPUT = ROOT / 'output/characters/CHR-001/model-r7/model-source'
parser = argparse.ArgumentParser()
parser.add_argument('--render', action='store_true')
parser.add_argument('--export-existing', action='store_true', help='Export the editable .blend master without rebuilding geometry')
parser.add_argument('--update-jump', action='store_true', help='Append Jump to the current master and export without rebuilding geometry')
args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else [])
for path in (SOURCE, RUNTIME, OUTPUT):
    path.mkdir(parents=True, exist_ok=True)


def author_jump(rig):
    # Pose only: the player controller supplies the 6 m/s, 18 m/s² world arc
    scene = bpy.context.scene
    scene.render.fps = 60
    rig.animation_data_clear()
    if 'Jump' in bpy.data.actions:
        bpy.data.actions.remove(bpy.data.actions['Jump'])
    scale = rig.data.bones['Hips'].head_local.z / .925
    # phase, pelvis compression, ankle lift, forward ankle offset (unscaled meters)
    poses = [(0.,0.,0.,0.),(.12,.075,.02,-.015),(.30,0.,.12,-.10),
             (.52,.025,.23,-.15),(.78,0.,.08,-.06),(.90,.055,.015,-.01),(1.,0.,0.,0.)]
    def rotate(name, axis, angle):
        bone = rig.pose.bones[name]
        bone.rotation_quaternion = Quaternion(bone.bone.matrix_local.to_quaternion().inverted() @ Vector(axis), angle)
    for frame in range(1,42):
        phase = (frame-1)/40
        scene.frame_set(frame)
        for bone in rig.pose.bones:
            bone.location=(0,0,0);bone.rotation_mode='QUATERNION';bone.rotation_quaternion=(1,0,0,0)
        left,right = next((a,b) for a,b in zip(poses,poses[1:]) if a[0] <= phase <= b[0])
        u=(phase-left[0])/(right[0]-left[0]);u=u*u*(3-2*u)
        drop,lift,forward = [(a+(b-a)*u)*scale for a,b in zip(left[1:],right[1:])]
        balance=math.sin(math.pi*phase)
        rig.pose.bones['Hips'].location.y=-drop
        rotate('Spine',(1,0,0),.08*balance)
        rotate('Head',(1,0,0),-.05*balance)
        for side,s in [('L',1),('R',-1)]:
            thigh=rig.data.bones['Thigh.'+side];shin=rig.data.bones['Shin.'+side];foot=rig.data.bones['Foot.'+side]
            upper=shin.head_local-thigh.head_local;lower=foot.head_local-shin.head_local
            a_len=math.hypot(upper.y,upper.z);b_len=math.hypot(lower.y,lower.z)
            dy=foot.head_local.y-thigh.head_local.y+forward
            dz=foot.head_local.z+lift-thigh.head_local.z+drop
            distance=math.hypot(dy,dz)
            assert abs(a_len-b_len)<distance<a_len+b_len
            angle=math.atan2(dy,-dz)-math.acos(max(-1,min(1,(a_len*a_len+distance*distance-b_len*b_len)/(2*a_len*distance))))
            bend=math.pi-math.acos(max(-1,min(1,(a_len*a_len+b_len*b_len-distance*distance)/(2*a_len*b_len))))
            thigh_angle=angle-math.atan2(upper.y,-upper.z)
            shin_angle=bend-(math.atan2(lower.y,-lower.z)-math.atan2(upper.y,-upper.z))
            rotate('Thigh.'+side,(1,0,0),thigh_angle)
            rotate('Shin.'+side,(1,0,0),shin_angle)
            rotate('Foot.'+side,(1,0,0),-thigh_angle-shin_angle+.12*balance)
            rotate('UpperArm.'+side,(0,1,0),s*(.34-.15*balance))
            axes=rig.data.bones['UpperArm.'+side].matrix_local.to_quaternion().inverted()
            rig.pose.bones['UpperArm.'+side].rotation_quaternion @= Quaternion(axes@Vector((1,0,0)),-.30*balance)
            rotate('Forearm.'+side,(1,0,0),-.10-.55*balance)
            for finger in ['Index','Middle','Ring','Pinky','Thumb']:
                rotate(finger+'.'+side,(1,0,0),.10+.12*balance)
        for bone in rig.pose.bones:
            bone.keyframe_insert('rotation_quaternion',frame=frame,group=bone.name)
            if bone.name=='Hips':bone.keyframe_insert('location',frame=frame,group=bone.name)
    action=rig.animation_data.action;action.name='Jump';action.use_fake_user=True
    for layer in action.layers:
        for strip in layer.strips:
            for curve in strip.channelbag(rig.animation_data.action_slot).fcurves:
                for key in curve.keyframe_points:key.interpolation='LINEAR'
    rig.animation_data.action=bpy.data.actions['Idle'];scene.frame_set(1)

def export(rig, objects):
    # Keep editable clothing parts in the master; batch the one-material runtime mesh
    copies=[]
    for original in sorted(objects,key=lambda obj:obj.name):
        obj=original.copy();obj.data=original.data.copy()
        bpy.context.collection.objects.link(obj);copies.append(obj)
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True);bpy.context.view_layer.objects.active=obj
        for modifier in list(obj.modifiers):
            if modifier.type=='ARMATURE': obj.modifiers.remove(modifier)
            else: bpy.ops.object.modifier_apply(modifier=modifier.name)
    bpy.ops.object.select_all(action='DESELECT')
    for obj in copies: obj.select_set(True)
    bpy.context.view_layer.objects.active=copies[0]
    bpy.ops.object.join()
    runtime=bpy.context.object;runtime.name='CHR001_RuntimeMesh';runtime.data.name='CHR001_RuntimeMesh'
    runtime.modifiers.new('Skin','ARMATURE').object=rig
    rig.select_set(True)
    bpy.context.view_layer.objects.active=rig
    bpy.ops.export_scene.gltf(filepath=str(RUNTIME/'yao-grey-study.glb'), use_selection=True, export_format='GLB', export_yup=True, export_animations=True, export_animation_mode='ACTIONS', export_anim_slide_to_zero=True, export_skins=True, export_morph=False)
    data=runtime.data
    bpy.data.objects.remove(runtime,do_unlink=True)
    bpy.data.meshes.remove(data)

if args.export_existing or args.update_jump:
    bpy.ops.wm.open_mainfile(filepath=str(SOURCE/'yao-grey-study.blend'))
    rig=bpy.data.objects['CHR001_Rig']
    if args.update_jump:
        author_jump(rig)
        bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE/'yao-grey-study.blend'))
    export(rig,[o for o in rig.children if o.type=='MESH'])
    raise SystemExit(0)

bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
for data in list(bpy.data.actions):
    bpy.data.actions.remove(data)
scene = bpy.context.scene
scene.unit_settings.system = 'METRIC'
scene.unit_settings.scale_length = 1
scene.render.fps = 30
scene.render.engine = 'CYCLES'
scene.cycles.device = 'CPU'
scene.cycles.samples = 24
scene.render.threads_mode = 'FIXED'
scene.render.threads = 2
scene.view_settings.view_transform = 'Standard'
scene.world.color = (0.35, 0.35, 0.35)

# Values distinguish material regions, not an approved hair, eye or clothing palette
VALUES = {'skin': .78, 'hair': .40, 'jacket': .58, 'shirt': .88,
          'pants': .36, 'shoe': .71, 'ink': .065, 'white': .96}
REGIONS = list(VALUES)
SIZE = 1024
image = bpy.data.images.new('CHR001_GreyStudy_BaseColor', width=SIZE, height=SIZE, alpha=False)
pixels = [0.0] * (SIZE * SIZE * 4)
for y in range(SIZE):
    for x in range(SIZE):
        region = REGIONS[(y // 256) * 2 + x // 512]
        value = VALUES[region]
        # Subtle woven strips and seam allowance, with a 12 px guard around each island
        u, v = x % 512, y % 256
        if region in ('jacket', 'pants', 'shoe') and (u in range(24, 28) or v in range(24, 28)):
            value *= .88
        if region in ('shirt', 'jacket', 'pants') and (x + y) % 8 == 0:
            value *= .988
        i = (y * SIZE + x) * 4
        pixels[i:i+4] = [value, value, value, 1]
image.pixels.foreach_set(pixels)
image.filepath_raw = str(SOURCE / 'grey-study.png')
image.file_format = 'PNG'
image.save()
image.pack()
image.filepath='//grey-study.png'
# Reuse the same padded islands for material response; no palette or UV change
ROUGHNESS = {'skin': .64, 'hair': .52, 'jacket': .80, 'shirt': .88,
             'pants': .76, 'shoe': .62, 'ink': .72, 'white': .86}
surface_pixels, normal_pixels = [], []
for y in range(SIZE):
    for x in range(SIZE):
        region = REGIONS[(y // 256) * 2 + x // 512]
        u, v = x % 512, y % 256
        roughness = ROUGHNESS[region]
        nx = ny = 0.0
        if region in ('jacket', 'shirt', 'pants'):
            # Cloth variation lives in roughness, keeping distant silhouettes quiet
            roughness += .012 * math.sin(u * math.pi / 8) * math.sin(v * math.pi / 8)
        if region in ('jacket', 'pants', 'shoe'):
            # A shallow ridge follows the existing atlas seam, not random scratches
            dx, dy = (u - 26) / 4, (v - 26) / 4
            nx = .045 * dx * math.exp(-dx * dx / 2)
            ny = .045 * dy * math.exp(-dy * dy / 2)
        length = math.sqrt(nx * nx + ny * ny + 1)
        surface_pixels.extend((1, roughness, 0, 1))  # glTF: G roughness, B metalness
        normal_pixels.extend((.5 + nx / length / 2, .5 + ny / length / 2, .5 + .5 / length, 1))
def data_image(name, filename, pixels):
    result = bpy.data.images.new(name, width=SIZE, height=SIZE, alpha=False)
    result.colorspace_settings.name = 'Non-Color'
    result.pixels.foreach_set(pixels)
    result.filepath_raw = str(SOURCE / filename)
    result.file_format = 'PNG'
    result.save()
    result.pack()
    result.filepath = '//' + filename
    return result
surface = data_image('CHR001_GreyStudy_Surface', 'grey-study-surface.png', surface_pixels)
normal = data_image('CHR001_GreyStudy_Normal', 'grey-study-normal.png', normal_pixels)
material = bpy.data.materials.new('CHR001_GreyStudy_Atlas')
material.use_nodes = True
bsdf = material.node_tree.nodes.get('Principled BSDF')
bsdf.inputs['Roughness'].default_value = 1
bsdf.inputs['Specular IOR Level'].default_value = .24
tex = material.node_tree.nodes.new('ShaderNodeTexImage')
tex.image = image
material.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
surface_tex = material.node_tree.nodes.new('ShaderNodeTexImage')
surface_tex.image = surface
channels = material.node_tree.nodes.new('ShaderNodeSeparateColor')
material.node_tree.links.new(surface_tex.outputs['Color'], channels.inputs['Color'])
material.node_tree.links.new(channels.outputs['Green'], bsdf.inputs['Roughness'])
material.node_tree.links.new(channels.outputs['Blue'], bsdf.inputs['Metallic'])
normal_tex = material.node_tree.nodes.new('ShaderNodeTexImage')
normal_tex.image = normal
normal_map = material.node_tree.nodes.new('ShaderNodeNormalMap')
# Bake the small amplitude into RGB: Bevy 0.19.1 glTF does not apply normal.scale
normal_map.inputs['Strength'].default_value = 1
material.node_tree.links.new(normal_tex.outputs['Color'], normal_map.inputs['Color'])
material.node_tree.links.new(normal_map.outputs['Normal'], bsdf.inputs['Normal'])

rig = bpy.data.objects.new('CHR001_Rig', bpy.data.armatures.new('CHR001_Skeleton'))
bpy.context.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
rig.select_set(True)
bpy.ops.object.mode_set(mode='EDIT')
BONES = {}
def bone(name, start, end, parent=None):
    b = rig.data.edit_bones.new(name)
    b.head, b.tail = start, end
    b.align_roll(Vector((0, -1, 0)))
    if parent:
        b.parent = rig.data.edit_bones[parent]
    BONES[name] = (Vector(start), Vector(end))
bone('Root', (0, 0, 0), (0, 0, .12))
bone('Hips', (0, 0, .925), (0, 0, 1.06), 'Root')
bone('Spine', (0, 0, 1.06), (0, 0, 1.21), 'Hips')
bone('Chest', (0, 0, 1.21), (0, 0, 1.37), 'Spine')
bone('Neck', (0, 0, 1.37), (0, 0, 1.495), 'Chest')
bone('Head', (0, 0, 1.495), (0, 0, 1.735), 'Neck')
for side, s in (('L', 1), ('R', -1)):
    bone('Clavicle.'+side, (s*.03, 0, 1.355), (s*.185, 0, 1.375), 'Chest')
    bone('UpperArm.'+side, (s*.185, 0, 1.375), (s*.335, -.005, 1.155), 'Clavicle.'+side)
    bone('Forearm.'+side, (s*.335, -.005, 1.155), (s*.414, -.016, .965), 'UpperArm.'+side)
    bone('Hand.'+side, (s*.414, -.016, .965), (s*.435, -.019, .892), 'Forearm.'+side)
    for n, dx, length in (('Index', -.023, .062), ('Middle', -.008, .071), ('Ring', .008, .064), ('Pinky', .024, .05)):
        start = (s*(.435+dx), -.019, .900)
        end = (s*(.447+dx), -.028, .900-length)
        bone(n+'.'+side, start, end, 'Hand.'+side)
    bone('Thumb.'+side, (s*.409, -.022, .929), (s*.381, -.042, .88), 'Hand.'+side)
    bone('Thigh.'+side, (s*.095, 0, .925), (s*.105, -.017, .505), 'Hips')
    bone('Shin.'+side, (s*.105, -.017, .505), (s*.108, .015, .105), 'Thigh.'+side)
    bone('Foot.'+side, (s*.108, .015, .105), (s*.108, -.125, .045), 'Shin.'+side)
    bone('Toe.'+side, (s*.108, -.125, .045), (s*.108, -.195, .035), 'Foot.'+side)
bpy.ops.object.mode_set(mode='OBJECT')
rig.show_in_front = True
rig.data.display_type = 'STICK'
objects = []

def atlas_uv(region, u, v):
    i = REGIONS.index(region)
    return ((i % 2 * 512 + 16 + u*480) / SIZE, (i // 2*256 + 16 + v*224) / SIZE)

def mesh(name, vertices, faces, region, weights, uv=None, subdiv=0):
    data = bpy.data.meshes.new(name)
    data.from_pydata(vertices, [], faces)
    data.update()
    obj = bpy.data.objects.new(name, data)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    layer = data.uv_layers.new(name='UVMap')
    if uv is None:
        lo = [min(v[a] for v in vertices) for a in range(3)]
        hi = [max(v[a] for v in vertices) for a in range(3)]
        uv = [((v[0]-lo[0])/max(hi[0]-lo[0], .001), (v[2]-lo[2])/max(hi[2]-lo[2], .001)) for v in vertices]
    for poly in data.polygons:
        poly.use_smooth = True
        for loop in poly.loop_indices:
            layer.data[loop].uv = atlas_uv(region, *uv[data.loops[loop].vertex_index])
    groups = {}
    for i, position in enumerate(vertices):
        influence = weights(Vector(position)) if callable(weights) else weights[i] if isinstance(weights,list) else {weights: 1}
        assert 0 < len(influence) <= 4 and abs(sum(influence.values())-1) < 1e-6
        for joint, w in influence.items():
            if joint not in groups:
                groups[joint] = obj.vertex_groups.new(name=joint)
            if w > 0:
                groups[joint].add([i], w, 'REPLACE')
    obj.parent = rig
    mod = obj.modifiers.new('Skin', 'ARMATURE')
    mod.object = rig
    if subdiv:
        mod = obj.modifiers.new('Surface subdivision', 'SUBSURF')
        mod.levels = subdiv
        mod.render_levels = subdiv
    objects.append(obj)
    return obj

def blend(a, b, t):
    t = max(0, min(1, t))
    return {a: 1-t, b: t} if 0 < t < 1 else {b if t else a: 1}

def body_weights(p):
    z = p.z
    if z < 1.07:
        return blend('Hips', 'Spine', (z-.99)/.12)
    return blend('Spine', 'Chest', (z-1.14)/.18)

# Elliptical ring lofts have deliberate joint loops and a UV seam on the back
# Profiles: centre X/Y/Z, lateral radius, depth radius
def loft(name, rings, region, weights, sides=24, start=0, end=2*math.pi, caps=True, fold=0, subdiv=0):
    verts, faces, uv = [], [], []
    for k, (x, y, z, rx, ry) in enumerate(rings):
        for j in range(sides+1):
            angle = start+(end-start)*j/sides
            ripple = 1 + fold*math.sin(angle*5 + k*1.7)
            verts.append((x+rx*math.sin(angle)*ripple, y-ry*math.cos(angle)*ripple, z))
            uv.append((j/sides, k/max(1,len(rings)-1)))
    for k in range(len(rings)-1):
        for j in range(sides):
            a = k*(sides+1)+j
            faces.append((a, a+1, a+sides+2, a+sides+1))
    if caps and abs(end-start-2*math.pi) < .001:
        faces.append(tuple(reversed(range(sides))))
        faces.append(tuple((len(rings)-1)*(sides+1)+j for j in range(sides)))
    return mesh(name, verts, faces, region, weights, uv, subdiv)

def tube(name, points, widths, depths, region, weights, sides=12):
    verts, faces, uv = [], [], []
    for k, point in enumerate(points):
        p = Vector(point)
        tangent = Vector(points[min(k+1, len(points)-1)])-Vector(points[max(0, k-1)])
        tangent.normalize()
        depth_axis = Vector((0, -1, 0))
        if abs(tangent.dot(depth_axis)) > .9:
            depth_axis = Vector((0, 0, 1))
        width_axis = tangent.cross(depth_axis).normalized()
        depth_axis = width_axis.cross(tangent).normalized()
        for j in range(sides+1):
            a = j/sides*math.tau
            v = p+width_axis*widths[k]*math.cos(a)+depth_axis*depths[k]*math.sin(a)
            verts.append(v)
            uv.append((j/sides, k/(len(points)-1)))
    for k in range(len(points)-1):
        for j in range(sides):
            a = k*(sides+1)+j
            faces.append((a, a+1, a+sides+2, a+sides+1))
    faces.append(tuple(reversed(range(sides))))
    faces.append(tuple((len(points)-1)*(sides+1)+j for j in range(sides)))
    return mesh(name, verts, faces, region, weights, uv)

# Underlying neck and torso; hidden body faces stay under the separate clothing

shirt=loft('TShirt', [(0,0,.955,.198,.130),(0,0,.974,.198,.130),(0,-.002,1.00,.185,.118),(0,0,1.08,.143,.086),(0,0,1.22,.165,.089),(0,0,1.31,.163,.081),(0,.004,1.370,.143,.061),(0,.008,1.437,.043,.038)], 'shirt', body_weights, fold=.012, caps=False)
# The hem wraps both trouser hips; a round single-body ellipse cut through the
# two-lobed hip volume at the sides, although the centre-front render was clear
for k in range(2):
    for j in range(25):
        cosine=math.cos(j/24*math.tau)
        shirt.data.vertices[k*25+j].co.y=-.130*math.copysign(abs(cosine)**.65,cosine)
shirt.data.update()
loft('TShirt_Collar', [(0,.008,1.433,.045,.040),(0,.008,1.443,.043,.038)], 'shirt', 'Neck', caps=False)
# Raised crew neck and a closer shoulder slope keep the exposed neck compact
jacket = loft('Jacket_Body', [(0,.008,1.01,.179,.111),(0,.008,1.033,.179,.111),(0,.009,1.065,.182,.116),(0,.008,1.19,.179,.109),(0,.006,1.29,.173,.094),(0,.008,1.376,.166,.078),(0,.012,1.438,.072,.054)], 'jacket', body_weights, sides=32, start=.48, end=math.tau-.48, caps=False, fold=.015)

# Sleeve branches share the torso boundary rather than intersecting capped tubes
jverts=[tuple(v.co) for v in jacket.data.vertices]
# The zipper tape is part of the cloth boundary, sharing its skinning and thickness
for k in range(7):
    for edge, inside in ((0,1),(32,31)):
        jverts[k*33+inside]=tuple(Vector(jverts[k*33+edge]).lerp(Vector(jverts[k*33+inside]),.14))
juv=[(j/32,k/6) for k in range(7) for j in range(33)]
jweights=[body_weights(Vector(v)) for v in jverts]
jfaces=[tuple(p.vertices) for p in jacket.data.polygons if not (4 <= p.index//32 < 6 and (4 <= p.index%32 < 10 or 22 <= p.index%32 < 28))]
objects.remove(jacket);bpy.data.objects.remove(jacket,do_unlink=True)

# Hood is an open, flattened pouch behind the nape rather than a torus
hood = loft('Hood', [(0,.075,1.265,.053,.023),(0,.078,1.30,.104,.044),(0,.052,1.365,.117,.072),(0,.027,1.444,.077,.058),(0,.021,1.453,.061,.051)], 'jacket', lambda p: blend('Chest','Neck',(p.z-1.37)/.12), sides=28, start=.55, end=math.tau-.55, caps=False)
solid = hood.modifiers.new('Hood lining thickness', 'SOLIDIFY'); solid.thickness = .005
for s, side in ((1,'L'),(-1,'R')):
    # The torso opening has 16 ordered vertices around each shoulder/armhole
    columns=list(range(4,11)) if s==1 else list(range(28,21,-1))
    boundary=[4*33+j for j in columns]+[k*33+columns[-1] for k in (5,6)]+[6*33+j for j in reversed(columns[:-1])]+[5*33+columns[0]]
    previous=boundary
    arm_points=[(s*.203,0,1.327,.043,.044),(s*.25,-.002,1.264,.048,.045),(s*.309,-.006,1.191,.047,.048),(s*.329,-.006,1.168,.052,.046),(s*.342,-.003,1.142,.047,.041),(s*.354,-.007,1.116,.055,.048),(s*.389,-.012,1.039,.046,.041),(s*.407,-.017,1.005,.048,.039),(s*.414,-.016,.980,.036,.032)]
    for k,(x,y,z,rw,rd) in enumerate(arm_points):
        current=[]
        for j in range(16):
            a=math.pi*1.25-j/16*math.tau
            point=Vector((x,y,z))+Vector((s*.80,0,.60))*rw*math.cos(a)+Vector((0,1,0))*rd*math.sin(a)
            current.append(len(jverts));jverts.append(tuple(point));juv.append((j/16,(k+1)/len(arm_points)))
            jweights.append(blend('Forearm.'+side,'UpperArm.'+side,(z-1.10)/.10))
        for j in range(16):jfaces.append((previous[j],previous[(j+1)%16],current[(j+1)%16],current[j]))
        previous=current
    tube('Sleeve_Cuff.'+side, [(s*.408,-.016,.991),(s*.419,-.017,.969),(s*.42,-.017,.965)], [.045,.038,.036], [.041,.033,.032], 'pants', 'Forearm.'+side)
    tube('Hood_Cord.'+side, [(s*.047,-.045,1.436),(s*.058,-.068,1.386),(s*.069,-.078,1.326),(s*.075,-.084,1.296)], [.0017,.0017,.0017,.0023], [.0017,.0017,.0017,.0023], 'ink', body_weights, sides=6)
    # Palm and individually posed fingers, with finger bone groups
    tube('Palm.'+side, [(s*.419,-.017,.969),(s*.429,-.018,.94),(s*.436,-.019,.903),(s*.436,-.021,.890)], [.03,.034,.034,.028], [.022,.025,.022,.016], 'skin', 'Hand.'+side, sides=16)
    for n in ('Index','Middle','Ring','Pinky','Thumb'):
        a,b = BONES[n+'.'+side]
        d = b-a
        tube(n+'_Mesh.'+side, [a,a+d*.35,a+d*.72,b], [.009,.009,.007,.0028] if n != 'Thumb' else [.011,.01,.008,.003], [.009,.009,.007,.003], 'skin', n+'.'+side, sides=10)
    # Trouser leg contours, with controlled knee and ankle folds
    rings = [(s*.089,0,.995,.096,.105),(s*.096,0,.926,.098,.106),(s*.101,0,.84,.090,.092),(s*.104,-.005,.68,.076,.078),(s*.103,-.015,.552,.069,.071),(s*.105,-.024,.525,.070,.075),(s*.108,-.010,.492,.073,.067),(s*.108,-.017,.465,.071,.073),(s*.11,.004,.32,.064,.068),(s*.106,.014,.206,.060,.061),(s*.111,.006,.180,.070,.066),(s*.108,-.003,.164,.067,.070),(s*.110,.016,.145,.057,.050),(s*.109,.015,.127,.052,.050)]
    def leg_weights(p,side=side):
        if p.z > .88:
            return blend('Thigh.'+side,'Hips',(p.z-.88)/.14)
        return blend('Shin.'+side,'Thigh.'+side,(p.z-.46)/.09)
    loft('Trousers.'+side, rings, 'pants', leg_weights, fold=.035)
    # Sole and layered shoe upper preserve a flat ground contact, frontal toe and heel
    x=s*.109
    loft('Sneaker_Sole.'+side, [(x,-.068,0,.063,.142),(x,-.068,.024,.069,.15),(x,-.068,.043,.071,.15),(x,-.064,.059,.066,.143)], 'white', 'Foot.'+side, sides=24)
    loft('Sneaker_Upper.'+side, [(x,-.064,.05,.064,.14),(x,-.052,.078,.063,.133),(x,-.017,.113,.056,.098),(x,.008,.153,.042,.052),(x,.012,.156,.043,.05)], 'shoe', 'Foot.'+side, sides=24)
    loft('Sock.'+side, [(x,.015,.125,.042,.042),(x,.015,.175,.041,.041)], 'ink', 'Shin.'+side, sides=16)
    # Shoe laces are short ribbons on the sloped instep, avoiding procedural tiny knots
    for k in range(4):
        z=.093+k*.012; y=-.145+k*.021
        tube(f'Lace_{k}.{side}', [(x-.036,y,z),(x,y-.003,z+.007),(x+.036,y,z)], [.0023]*3,[.002]*3,'white','Foot.'+side,sides=6)
    # Heel pull tab and side panel edge read at ordinary walking distance
    tube('Heel_Tab.'+side, [(x,.071,.078),(x,.073,.125),(x,.058,.157)], [.012]*3,[.003]*3,'ink','Foot.'+side,sides=6)


jacket=mesh('Jacket_ContinuousShoulders',jverts,jfaces,'jacket',jweights,juv)
for polygon in jacket.data.polygons:
    if all(index < 231 and index % 33 in (0,1,31,32) for index in polygon.vertices):
        for loop in polygon.loop_indices:
            jacket.data.uv_layers.active.data[loop].uv=atlas_uv('ink',.5,.5)
solid=jacket.modifiers.new('Cloth thickness','SOLIDIFY');solid.thickness=.003
mod=jacket.modifiers.new('Cloth surface','SUBSURF');mod.levels=1;mod.render_levels=1

# Soft adolescent face with jaw/cheek/temple loops and a small, modelled nose
head = loft('Face_Head', [(0,.005,1.35,.061,.045),(0,.012,1.395,.044,.037),(0,.024,1.435,.032,.030),(0,.022,1.515,.030,.029),(0,-.003,1.520,.037,.052),(0,-.006,1.523,.043,.058),(0,-.004,1.535,.055,.064),(0,.004,1.552,.077,.073),(0,.007,1.560,.088,.076),(0,.008,1.570,.093,.080),(0,.008,1.581,.096,.084),(0,.010,1.596,.097,.086),(0,.012,1.617,.098,.087),(0,.012,1.651,.097,.087),(0,.016,1.692,.083,.075),(0,.017,1.721,.048,.047),(0,.017,1.734,.006,.008)], 'skin',lambda p: blend('Neck','Head',(p.z-1.435)/.055),sides=48,subdiv=1)
# The throat rises behind the chin; separate underside and outer-jaw loops turn
# upward toward the ear instead of extending the cheek as a cone into the neck
for ring, front_drop, back_rise in ((3,.020,.020),(4,.019,.017),(5,.020,.016),(6,.019,.009)):
    for j in range(49):
        facing=math.cos(j/48*math.tau)
        head.data.vertices[ring*49+j].co.z += -front_drop*max(0,facing)+back_rise*max(0,-facing)
# Keep a continuous bridge, nose tip and upper-lip/chin profile; shallow sockets
# and the cheek transition carry the shape instead of painted face shadows
for vertex in head.data.vertices:
    p=vertex.co
    if p.y < 0:
        p.y -= .025*math.exp(-(p.x/.012)**2-((p.z-1.561)/.011)**2)
        p.y -= .008*math.exp(-(p.x/.011)**2-((p.z-1.582)/.021)**2)
        p.y += .0045*math.exp(-((abs(p.x)-.044)/.028)**2-((p.z-1.598)/.011)**2)
        p.y -= .0025*math.exp(-((abs(p.x)-.062)/.023)**2-((p.z-1.570)/.013)**2)
        p.y -= .003*math.exp(-(p.x/.026)**2-((p.z-1.537)/.008)**2)
head.data.update()
for side,s in (('L',1),('R',-1)):
    tube('Ear.'+side, [(s*.087,.009,1.568),(s*.100,.010,1.572),(s*.103,.012,1.594),(s*.097,.012,1.605),(s*.088,.010,1.600)], [.007,.010,.010,.008,.006],[.007,.007,.007,.006,.005],'skin','Head',sides=16)
    tube('Ear_Fold.'+side,[(s*.097,.000,1.580),(s*.099,-.001,1.590),(s*.098,-.001,1.598),(s*.093,-.001,1.600)],[.002,.0025,.002,.001],[.0015]*4,'skin','Head',sides=8)
# Project against the welded, subdivided surface used by the export. The loft's
# duplicate seam vertices otherwise make a different face along the nose/mouth
head_mesh=bmesh.new();head_mesh.from_mesh(head.data)
bmesh.ops.remove_doubles(head_mesh,verts=list(head_mesh.verts),dist=.000001)
head_mesh.to_mesh(head.data);head_mesh.free();head.data.update()
# Face overlays are thin curved geometry, never a whole opaque facial billboard
bpy.context.view_layer.update()
face_surface=BVHTree.FromObject(head,bpy.context.evaluated_depsgraph_get())
def face_y(x,z,offset=0):
    hit,_,_,_=face_surface.ray_cast(Vector((x,-1,z)),Vector((0,1,0)))
    return (hit.y if hit is not None else -.065) + offset

def patch(name, outline, region, offset=-.005, surface=face_y):
    cx=sum(x for x,z in outline)/len(outline);cz=sum(z for x,z in outline)/len(outline)
    # A middle ring keeps the surface chords above the face at sub-mm offsets
    points=[(cx,cz)]+[(cx+(x-cx)*r,cz+(z-cz)*r) for r in (.5,1) for x,z in outline]
    verts=[(x,surface(x,z,offset),z) for x,z in points]
    n=len(outline)
    faces=[(0,i+1,(i+1)%n+1) for i in range(n)]
    faces += [(i+1,n+i+1,n+(i+1)%n+1,(i+1)%n+1) for i in range(n)]
    return mesh(name,verts,faces,region,'Head')
for s,side in ((1,'L'),(-1,'R')):
    cx=s*.041
    def eye_bounds(x):
        u=max(-1,min(1,(s*x-.041)/.025))
        arch=max(0,1-u*u)**.60
        mid=1.597+.0032*u
        return mid-.0082*arch,mid+.0108*arch
    def eye_y(x,z,offset=0):
        low,high=eye_bounds(x)
        u=(s*x-.041)/.025
        v=(z-(low+high)/2)/max(.0001,(high-low)/2)
        bulge=.0022*max(0,1-u*u)*max(0,1-v*v)
        return face_y(x,z,offset-bulge)
    outline=[]
    for j in range(25):
        a=j/24*math.tau
        x=cx+s*.025*math.cos(a)
        z=eye_bounds(x)[1 if math.sin(a)>=0 else 0]
        outline.append((x,z))
    patch('Eye_White.'+side,outline,'white',-.00065,surface=eye_y)
    # The upper lid overlaps the iris; the gaze is not an untouched round disk
    for region,rx,rz,offset in [('hair',.0110,.0120,-.0010),('ink',.0040,.0082,-.0013)]:
        iris=[]
        for j in range(32):
            x=cx-s*.0008+rx*math.cos(j/32*math.tau)
            low,high=eye_bounds(x)
            z=max(low+.00035,min(high-.00035,1.600+rz*math.sin(j/32*math.tau)))
            iris.append((x,z))
        patch('Iris_'+region+'.'+side,iris,region,offset,surface=eye_y)
    patch('Eye_Glint.'+side,[(cx-.003+.0015*math.cos(a/12*math.tau),1.602+.0015*math.sin(a/12*math.tau)) for a in range(12)],'white',-.0016,surface=eye_y)
    for label,indices,radius,region in [('Upper_Lid',range(13),.0012,'ink'),('Lower_Lid',range(20,25),.00035,'hair')]:
        lid=[(outline[i][0],face_y(*outline[i],-.0011),outline[i][1]) for i in indices]
        widths=[radius*(.5+.6*math.sin(math.pi*i/(len(lid)-1))) for i in range(len(lid))]
        tube(label+'.'+side,lid,widths,[w*.7 for w in widths],region,'Head',sides=6)
    brow=[(cx+s*x,face_y(cx+s*x,1.622+z,-.001),1.622+z) for x,z in [(-.021,-.001),(-.006,.002),(.012,.003 if s==1 else .002),(.025,.001)]]
    tube('Eyebrow.'+side,brow,[.0007,.0017,.0013,.0003],[.0006]*4,'hair','Head',sides=6)
mouth=[(-.017,1.5357),(-.006,1.5362),(.006,1.5358),(.016,1.5365)]
mouth=[(x+(nx-x)*t/4,z+(nz-z)*t/4) for (x,z),(nx,nz) in zip(mouth,mouth[1:]) for t in range(4)]+[mouth[-1]]
# Resample the mouth onto the actual lip surface; four long chords cut behind it
tube('Mouth',[(x,face_y(x,z,-.00065),z) for x,z in mouth],[.0003+.00035*math.sin(math.pi*i/(len(mouth)-1)) for i in range(len(mouth))],[.00035]*len(mouth),'pants','Head',sides=6)
# Swept group surfaces all start at the crown and overlap their neighbors.
# The back has no smooth carrier shell with independent short islands on top.
sectors=(0,.52,1.13,1.58,2.00,2.55,2.93,3.48,3.83,4.36,4.73,5.27,5.78,math.tau)
tip_offsets=(.0,.009,-.008,.015,-.004,.012,-.009,.016,-.011,.008,-.004,.011,.0)
levels=(0,.07,.20,.36,.53,.69,.82,.93,1)
cap_vertices,cap_faces,cap_uv=[],[],[]
for group,(left,right) in enumerate(zip(sectors,sectors[1:])):
    centre=(left+right)/2
    front=max(0,math.cos(centre))**.6
    back=max(0,-math.cos(centre))**.5
    end=1.585+.085*front-.037*back+tip_offsets[group]
    start=len(cap_vertices)
    for k,t in enumerate(levels):
        taper=.72+.025*(group%3)
        spread=min(1,.09+t/.15)*(1-max(0,(t-taper)/(1-taper))**1.3)
        z=1.744+(end-1.744)*t
        centre_sweep=centre+.30*(1-t)**.8+(.05,.17,-.08)[group%3]*t
        for j in range(5):
            cross=j/2-1
            angle=centre_sweep+(right-left)*.80*spread*cross
            radius=math.sqrt(max(0,1-((z-1.630)/.119)**2))
            ridge=(1-cross*cross)*(.004+.0015*(group%3))*math.sin(math.pi*t)**.7
            # Root rows meet the common crown; broad midsections continue into
            # tapered ends, with one edge above the adjoining group rather than
            # a detached leaf perimeter surrounded by visible scalp
            overlap=.004*(cross+1)/2*math.sin(math.pi*t)
            radial=ridge+overlap
            point=Vector(((.108*radius+radial)*math.sin(angle),.018-(.102*radius+radial)*math.cos(angle),z))
            point.x-=.012*(1-t)**2
            origin=Vector((0,.018,z));direction=(point-origin).normalized()
            hit,_,_,_=face_surface.ray_cast(origin,direction)
            if hit is not None and (point-origin).length < (hit-origin).length+.006:
                point=hit+direction*.006
            cap_vertices.append(point);cap_uv.append((j/4,t))
    for k in range(len(levels)-1):
        for j in range(4):
            a=start+k*5+j
            cap_faces.append((a,a+1,a+6,a+5))
cap=mesh('Hair_Cap',cap_vertices,cap_faces,'hair','Head',cap_uv,subdiv=1)
solid=cap.modifiers.new('Hair group thickness','SOLIDIFY');solid.thickness=.0015;solid.offset=-1
# A small crown closure sits under converging roots, not under the full back
crown=loft('Hair_CrownBase',[(0,.018,1.701,.080,.074),(0,.018,1.720,.066,.061),(0,.017,1.737,.045,.042),(0,.016,1.748,.004,.004)],'hair','Head',sides=32,caps=False,subdiv=1)
crown_mesh=bmesh.new();crown_mesh.from_mesh(crown.data);crown_mesh.verts.ensure_lookup_table()
top=crown_mesh.faces.new([crown_mesh.verts[i] for i in range(99,131)])
for loop in top.loops:loop[crown_mesh.loops.layers.uv.active].uv=atlas_uv('hair',.5,.5)
crown_mesh.to_mesh(crown.data);crown_mesh.free()
def hair_lock(name, points, width):
    # Lens-shaped strand groups with a narrow tip; one closed mesh, not hair particles
    verts,faces,uv=[],[],[]
    a,b,c,d=map(Vector,points)
    count=13
    for i in range(count):
        t=i/(count-1)
        p=(1-t)**3*a+3*(1-t)**2*t*b+3*(1-t)*t*t*c+t**3*d
        tangent=(3*(1-t)**2*(b-a)+6*(1-t)*t*(c-b)+3*t*t*(d-c)).normalized()
        out=Vector((p.x,p.y-.012,(p.z-1.60)*.3)).normalized()
        side=tangent.cross(out).normalized()
        out=side.cross(tangent).normalized()
        w=width*(.70+.38*math.sin(math.pi*t))*(1-t**1.35)+.0004
        if name.startswith('Crown_'):
            w=width*(.40+.65*math.sin(math.pi*t))*(1-t**2.1)+.0004
        for j in range(8):
            angle=j/8*math.tau
            v=p+side*(w*math.cos(angle))+out*((w*.10+.0004)*math.sin(angle))
            verts.append(v);uv.append((j/7,t))
    for i in range(count-1):
        for j in range(8):
            faces.append((i*8+j,i*8+(j+1)%8,(i+1)*8+(j+1)%8,(i+1)*8+j))
    faces.extend([tuple(reversed(range(8))),tuple((count-1)*8+j for j in range(8))])
    return mesh(name,verts,faces,'hair','Head',uv)
# A broad swept group, shorter counter-sweep and thin secondary tips share one cap
bangs=[
    ([(-.032,.019,1.733),(-.076,-.058,1.734),(-.114,-.079,1.665),(-.099,-.058,1.596)],.021),
    ([(-.024,.007,1.736),(-.045,-.085,1.733),(-.092,-.101,1.663),(-.066,-.084,1.620)],.032),
    ([(-.016,.010,1.738),(-.015,-.100,1.723),(-.025,-.111,1.661),(-.007,-.087,1.619)],.032),
    ([(-.005,.014,1.739),(.046,-.075,1.748),(.025,-.111,1.697),(.004,-.096,1.650)],.030),
    ([(.016,.018,1.733),(.061,-.054,1.734),(.076,-.109,1.669),(.059,-.087,1.629)],.022),
    ([(.023,.025,1.728),(.080,-.033,1.710),(.106,-.067,1.649),(.092,-.053,1.586)],.019)]
for i,(path,width) in enumerate(bangs):hair_lock(f'Bangs_{i:02}',path,width)
for s,side in ((1,'L'),(-1,'R')):
    for i,(y,width,z) in enumerate(((-.014,.022,1.586),(.026,.026,1.608),(.065,.018,1.596))):
        hair_lock(f'Temple_{i}.{side}',[(s*.049,y*.5,1.716),(s*.100,y,1.710),(s*.124,y-.010,1.645),(s*(.104+.006*i),y+.004,z)],width)
hair_lock('Crown_0.L',[(-.018,.022,1.732),(-.035,.016,1.768),(-.054,-.003,1.760),(-.068,-.028,1.730)],.015)
hair_lock('Crown_0.R',[(.015,.032,1.732),(.053,.036,1.750),(.078,.040,1.740),(.090,.043,1.714)],.016)
# Minimal original double-dot / short bar chest graphic, all editable mesh
for z in (1.275,1.246):
    patch('Tee_Dot_'+str(z),[(-.018+.008*math.cos(i/20*math.tau),z+.008*math.sin(i/20*math.tau)) for i in range(20)],'hair',-.014)
mesh('Tee_Bar',[(.005+.055*j/6,-.091,z) for z in (1.253,1.267) for j in range(7)],[(j,j+1,j+8,j+7) for j in range(6)],'hair',body_weights)
# Printed marks use the same spine/chest blend as the underlying cloth
for obj in objects:
    if obj.name.startswith('Tee_'):
        obj.vertex_groups.clear()
        for joint in ('Spine','Chest'):
            group=obj.vertex_groups.new(name=joint)
            for vertex in obj.data.vertices:
                group.add([vertex.index],body_weights(vertex.co).get(joint,0),'REPLACE')

for obj in objects:
    if obj.name.startswith(('Sleeve.', 'Jacket_Body', 'TShirt', 'Trousers.', 'Palm.', 'Ear.')):
        mod=obj.modifiers.new('Cloth surface', 'SUBSURF');mod.levels=1;mod.render_levels=1

# Printed marks follow the finished shirt surface instead of floating as plaques
bpy.context.view_layer.update()
shirt_surface=BVHTree.FromObject(shirt,bpy.context.evaluated_depsgraph_get())
for obj in objects:
    if obj.name.startswith('Tee_'):
        for vertex in obj.data.vertices:
            hit,_,_,_=shirt_surface.ray_cast(Vector((vertex.co.x,-1,vertex.co.z)),Vector((0,1,0)))
            assert hit is not None, 'Chest graphic must lie on the shirt'
            vertex.co.y=hit.y-.0006


# Lower the cranial mass and merge its transition into the neck, without flattening a chin cap
for obj in objects:
    for v in obj.data.vertices:
        if obj.name=='Face_Head':
            v.co.z -= .018*max(0,min(1,(v.co.z-1.43)/.06))
        elif obj.name.startswith(('Ear.','Ear_Fold.','Eye_','Iris_','Upper_Lid.','Lower_Lid.','Eyebrow.','Nose','Mouth','Hair_','Bangs_','Temple_','Crown_','Nape_','Back_Lock_')):
            v.co.z -= .018

# Consistent outward normals after negative-side profile construction
for obj in objects:
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True);bpy.context.view_layer.objects.active=obj
    bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.remove_doubles(threshold=.000001);bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT')

# In-place motion: stance feet sweep against the real controller velocity
# At 3.2 m/s the Walk input reads as a light jog, not a leisurely walking gait
FPS = 60
scene.render.fps = FPS
CLIPS = {'Idle': (120, 0), 'Walk': (48, .47), 'Run': (40, .83)}
GAITS = {'Walk': (3.2, 16, .09), 'Run': (5.6, 8, .16)}

def foot_target(name, phase):
    speed, stance_frames, lift = GAITS[name]
    period = CLIPS[name][0] / FPS
    stance = stance_frames / FPS
    time = phase % 1 * period
    reach = speed * stance / 2
    if time <= stance:
        return -reach + speed * time, .105
    swing_time = time - stance
    swing = period - stance
    edge = 1 / 30
    extra = speed * edge / 2
    # Match stance velocity at lift-off and landing; the small overshoot happens in the air
    if swing_time < edge:
        y = reach + speed * (swing_time - swing_time**2 / (2 * edge))
    elif swing_time > swing - edge:
        t = swing_time - (swing - edge)
        y = -reach - extra + speed * t*t / (2 * edge)
    else:
        u = (swing_time-edge)/(swing-2*edge)
        y = (reach+extra) * (1-2*u*u*(3-2*u))
    return y, .105 + lift * math.sin(math.pi * swing_time/swing)**2

def foot_pitch(name, phase):
    if name!='Run':return 0.
    period=CLIPS[name][0]
    stance=GAITS[name][1]/period
    p=phase%1
    if p<=stance:return 0.
    swing=(p-stance)/(1-stance)
    # Roll only the airborne foot; the verified flat-ground stance stays planted
    return .28*math.sin(2*math.pi*swing)*math.sin(math.pi*swing)

def rotate(name, axis, angle):
    pb=rig.pose.bones[name]
    pb.rotation_mode='QUATERNION'
    local=rig.data.bones[name].matrix_local.to_quaternion().inverted() @ Vector(axis)
    pb.rotation_quaternion=Quaternion(local,angle)

for name,(period,stride) in CLIPS.items():
    rig.animation_data_clear()
    for frame in range(1,period+2):
        phase=(frame-1)/period
        t=phase*math.tau
        for pb in rig.pose.bones:
            pb.location=(0,0,0);pb.rotation_mode='QUATERNION';pb.rotation_quaternion=(1,0,0,0)
        targets={side:foot_target(name,phase+offset) for side,offset in (('L',0),('R',.5))} if stride else {}
        upper=math.hypot(.42,.017); lower=math.hypot(.4,.032)
        if stride:
            # Lower the pelvis only as far as the authored contact/return poses require
            height=min(.875+.018*math.sin(2*t)**2, *(z+math.sqrt((upper+lower-.007)**2-y*y) for y,z in targets.values()))
            bob=height-.925
        else:
            bob=.001*(1-math.cos(t))
        rig.pose.bones['Hips'].location.y=bob
        rotate('Chest',(0,0,1),(-.075 if name=='Run' else .025 if stride else .008)*math.sin(t))
        rotate('Head',(0,0,1),(.03 if name=='Run' else -.015)*math.sin(t))
        if name=='Run':
            rotate('Spine',(1,0,0),.19+.035*math.cos(2*t))
            for bone,pitch in (('Chest',.025*math.cos(2*t+.5)),('Head',-.10-.025*math.cos(2*t))):
                axis=rig.data.bones[bone].matrix_local.to_quaternion().inverted()@Vector((1,0,0))
                rig.pose.bones[bone].rotation_quaternion @= Quaternion(axis,pitch)
        for side,s in (('L',1),('R',-1)):
            arm_phase=math.sin(t+(0 if s==1 else math.pi))
            if stride:
                dy,z=targets[side]
                dz=z-(.925+bob)
                distance=math.hypot(dy,dz)
                assert distance < upper+lower-.0069, (name,frame,side,'overextended IK',distance)
                target=math.atan2(dy,-dz)
                a=target-math.acos(max(-1,min(1,(upper*upper+distance*distance-lower*lower)/(2*upper*distance))))
                bend=math.pi-math.acos(max(-1,min(1,(upper*upper+lower*lower-distance*distance)/(2*upper*lower))))
                thigh=a-math.atan2(-.017,.42)
                shin=bend-(math.atan2(.032,.4)-math.atan2(-.017,.42))
                rotate('Thigh.'+side,(1,0,0),thigh)
                rotate('Shin.'+side,(1,0,0),shin)
                rotate('Foot.'+side,(1,0,0),-thigh-shin+foot_pitch(name,phase+(0 if s==1 else .5)))
                speed,stance_frames,_=GAITS[name]
                arm_phase=dy/(speed*stance_frames/(2*FPS))
            pb=rig.pose.bones['UpperArm.'+side]
            axes=rig.data.bones[pb.name].matrix_local.to_quaternion().inverted()
            pb.rotation_quaternion=Quaternion(axes@Vector((0,1,0)),s*.34) @ Quaternion(axes@Vector((1,0,0)),-stride*.65*arm_phase)
            flex=.10 if name=='Idle' else .28 if name=='Walk' else 1.30+.20*arm_phase
            rotate('Forearm.'+side,(1,0,0),-flex)
            if name=='Run':
                # Pronation is shared by forearm and wrist, leaving the palms facing inward
                rig.pose.bones['Forearm.'+side].rotation_quaternion @= Quaternion((0,1,0),s*1.40)
                rig.pose.bones['Hand.'+side].rotation_quaternion=Quaternion((0,1,0),s*.40)
            for finger in ('Index','Middle','Ring','Pinky','Thumb'):
                curl=.10 if name=='Idle' else .2 if name=='Walk' else -.40 if finger=='Thumb' else -.75
                rotate(finger+'.'+side,(1,0,0),curl)
        for pb in rig.pose.bones:
            pb.keyframe_insert('rotation_quaternion',frame=frame,group=pb.name)
            if pb.name=='Hips':pb.keyframe_insert('location',frame=frame,group=pb.name)
    action=rig.animation_data.action
    action.name=name;action.use_fake_user=True
    # Match the exported sampled glTF interpolation between authored poses
    for layer in action.layers:
        for strip in layer.strips:
            for curve in strip.channelbag(rig.animation_data.action_slot).fcurves:
                for point in curve.keyframe_points:
                    point.interpolation='LINEAR'
author_jump(rig)
rig.animation_data.action=bpy.data.actions['Idle']
scene.frame_start=1;scene.frame_end=CLIPS['Idle'][0]+1;scene.frame_set(1)

# Ground/cameras/lights are review aids, excluded from GLB
bpy.ops.mesh.primitive_plane_add(size=200, location=(0,0,0))
floor=bpy.context.object;floor.name='Review_Ground'
mat=bpy.data.materials.new('Review_Ground_Material');mat.diffuse_color=(.22,.22,.22,1);floor.data.materials.append(mat)
for name,pos,power,size in [('Key',(-3,-4,5),450,4),('Fill',(3,-2,3),200,3),('Rim',(0,3,4),350,3)]:
    data=bpy.data.lights.new('Review_'+name,'AREA');data.energy=power;data.shape='DISK';data.size=size
    obj=bpy.data.objects.new('Review_'+name,data);bpy.context.collection.objects.link(obj);obj.location=pos
    obj.rotation_euler=(Vector((0,0,1))-obj.location).to_track_quat('-Z','Y').to_euler()
cam=bpy.data.objects.new('Review_Camera',bpy.data.cameras.new('Review_Camera'))
bpy.context.collection.objects.link(cam);scene.camera=cam
cam.data.type='ORTHO';cam.data.ortho_scale=2.06
scene.render.resolution_x=720;scene.render.resolution_y=960;scene.render.resolution_percentage=100
bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE/'yao-grey-study.blend'))
export(rig,objects)
report={'status':'exported-candidate','blender':bpy.app.version_string,'units':'meters','dcc_forward':'-Y','gltf_forward':'+Z','face_hair_revision':'r7','material_revision':'r1','age_actual':20,'age_appearance':18,'palette':'unapproved grayscale study; grayscale hair is not a character color decision','bones':len(rig.data.bones),'editable_meshes':len(objects),'source_vertices':sum(len(o.data.vertices) for o in objects),'clips':{n:{'seconds':p/FPS,'fps':FPS,'in_place':True,'loop':n!='Jump'} for n,(p,_) in (CLIPS|{'Jump':(40,0)}).items()},'runtime':'NOT RUN; root integrates and captures the real Bevy path','views':[]}
if args.render:
    for label,pos,clip,frame in [('front',(0,-4,1.0),'Idle',1),('three-quarter',(3,-4,1.3),'Idle',1),('back',(0,4,1.0),'Idle',1),('walk-contact',(3,-4,1.2),'Walk',8),('run-contact',(3,-4,1.2),'Run',6),('face',(0,-4,1.61),'Idle',1),('walk-lift',(3,-4,1.2),'Walk',1),('run-lift',(3,-4,1.2),'Run',1)]:
        rig.animation_data.action=bpy.data.actions[clip];scene.frame_set(frame)
        target=Vector((0,0,1.615 if label=='face' else .89))
        cam.location=pos;cam.rotation_euler=(target-cam.location).to_track_quat('-Z','Y').to_euler()
        cam.data.ortho_scale=.39 if label=='face' else 2.06
        scene.render.filepath=str(OUTPUT/(label+'.png'))
        bpy.ops.render.render(write_still=True)
        report['views'].append({'label':label,'clip':clip,'frame':frame,'file':str(Path(scene.render.filepath).relative_to(ROOT))})
(OUTPUT/'model-report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
