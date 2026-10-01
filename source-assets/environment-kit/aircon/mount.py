"""Build/export the authored V-W10 aircon installation; --rebuild resets its master.

Original unit: Monsta3D / Poly Haven, CC0-1.0
Installation geometry and this script: N:SIDE contributors, MPL-2.0
"""
import argparse
import json
from pathlib import Path
import sys

import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / 'output/assets/public-aircon-mount-r1'
MASTER = HERE / 'wall-installation.blend'
OUTPUT = HERE / 'wall-installation.glb'
LIFT = 2.503333
WALL = -.198
# Coordinates below use the candidate GLB's +Y up and +Z outward axes
# Model hose endpoint was measured from its actual final tube rings
DRAIN = (.05780, .00110, -.01680)
MATERIALS = {}


def position(p):
    return Vector((p[0], -p[2], p[1]))


def material(name, color, roughness, metal=0):
    m = bpy.data.materials.new('NSIDE_AC_' + name)
    m.use_nodes = True
    linear = [x/12.92 if x <= .04045 else ((x+.055)/1.055)**2.4 for x in color]
    m.diffuse_color = (*linear, 1)
    node = m.node_tree.nodes['Principled BSDF']
    node.inputs['Base Color'].default_value = (*linear, 1)
    node.inputs['Roughness'].default_value = roughness
    node.inputs['Metallic'].default_value = metal
    MATERIALS[name] = m


def finish(obj, name, role):
    obj.name = name
    obj.data.materials.append(MATERIALS[role])
    obj['source'] = 'N:SIDE authored installation / MPL-2.0'
    return obj


def box(name, lo, hi, role):
    p = [(lo[i]+hi[i])/2 for i in range(3)]
    bpy.ops.mesh.primitive_cube_add(size=1, location=position(p))
    obj = bpy.context.object
    obj.dimensions = (hi[0]-lo[0], hi[2]-lo[2], hi[1]-lo[1])
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    return finish(obj, name, role)


def pipe(name, points, radius, role):
    curve = bpy.data.curves.new(name, 'CURVE')
    curve.dimensions = '3D'
    curve.resolution_u = 3
    curve.bevel_depth = radius
    curve.bevel_resolution = 2
    curve.use_fill_caps = True
    spline = curve.splines.new('BEZIER')
    spline.bezier_points.add(len(points)-1)
    for control, p in zip(spline.bezier_points, points):
        control.co = position(p)
        control.handle_left_type = control.handle_right_type = 'AUTO'
    obj = bpy.data.objects.new(name, curve)
    bpy.context.collection.objects.link(obj)
    finish(obj, name, role)
    return obj


def rebuild():
    bpy.ops.wm.open_mainfile(filepath=str(HERE / 'aircon-candidate.blend'))
    original = bpy.data.objects['CandidateAircon']
    original.location.z += LIFT
    original['source'] = 'Monsta3D / Poly Haven, CC0-1.0; retained clean candidate'
    material('PVC', (.68,.70,.65), .78)
    material('Metal', (.34,.38,.37), .46, .65)
    material('Insulation', (.17,.19,.18), .92)
    # Packers span the measured back plates at z=-.176136 to the actual brick face
    for i, x in enumerate((-.26577,.25120)):
        box(f'MountWallPacker_{i}', (x-.03,LIFT+.31,WALL-.005), (x+.03,LIFT+.69,-.176), 'Metal')
    x,y,z = DRAIN
    pipe('DrainHoseCoupler', [(x,LIFT+y+.015,z),(x,LIFT+y-.032,z)], .012, 'PVC')
    # This shallow offset turns the inherited hanging hose into a supported wall run
    pipe('DrainWallRun', [(x,LIFT-.017,z),(x,LIFT-.060,-.022),(x,LIFT-.105,-.075),
                         (x,LIFT-.140,-.154),(x,LIFT-.19,-.164),(x,.155,-.164)], .010, 'PVC')
    for height in (.55,1.20,1.90):
        box(f'DrainClip_{height}', (x-.026,height-.012,WALL-.005), (x+.026,height+.012,-.145), 'Metal')
    # A covered receiver provides a real visual endpoint at the grade anchor
    box('DrainReceiverBottom', (x-.10,0,WALL-.005), (x+.10,.02,-.008), 'Metal')
    for side, bounds in [('left',(x-.10,x-.086)),('right',(x+.086,x+.10))]:
        box('DrainReceiver_'+side,(bounds[0],.02,WALL-.005),(bounds[1],.165,-.008),'Metal')
    box('DrainReceiverFront',(x-.086,.02,-.022),(x+.086,.165,-.008),'Metal')
    box('DrainReceiverBack',(x-.086,.02,WALL-.005),(x+.086,.165,WALL+.009),'Metal')
    for i in range(7):
        xx=x-.085+i*.026
        # The rear pipe has its own small opening between the centre cover ribs
        back = -.135 if abs(xx-x) < .025 else WALL+.009
        box(f'DrainCover_{i}',(xx,.165,back),(xx+.014,.18,-.022),'Metal')
    # The existing copper lines already enter this rear bundle; extend its actual mouth
    # Ring bounds: x[-.012439,.050649], y[.414325,.477412], z[-.187080,-.170698]
    sx, sy = .019105, LIFT+.445869
    pipe('ExistingBundleWallSleeve', [(sx,sy,-.171),(sx,sy,-.235)], .027, 'Insulation')
    for name,lo,hi in [
        ('left',(sx-.065,sy-.065),(sx-.033,sy+.065)),
        ('right',(sx+.033,sy-.065),(sx+.065,sy+.065)),
        ('top',(sx-.033,sy+.033),(sx+.033,sy+.065)),
        ('bottom',(sx-.033,sy-.065),(sx+.033,sy-.033))]:
        box('WallSleeveCover_'+name, (lo[0],lo[1],WALL-.006),
            (hi[0],hi[1],WALL+.014), 'PVC')
    bpy.ops.file.pack_all()
    bpy.context.scene.unit_settings.system = 'METRIC'
    bpy.context.scene.unit_settings.scale_length = 1
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))


def export():
    bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    bpy.ops.object.select_all(action='DESELECT')
    objects = [o for o in bpy.context.scene.objects if o.type in ('MESH','CURVE')]
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.convert(target='MESH')
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.join()
    obj = bpy.context.object
    obj.name = 'VW10_AirconInstallation'
    bpy.ops.export_scene.gltf(filepath=str(OUTPUT), export_format='GLB', use_selection=True,
        export_animations=False, export_yup=True, export_tangents=True, export_image_format='AUTO')


def preview():
    bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 12
    scene.render.threads_mode = 'FIXED'
    scene.render.threads = 2
    scene.render.resolution_x, scene.render.resolution_y = 700, 800
    scene.render.resolution_percentage = 100
    scene.world = bpy.data.worlds.new('Inspection world')
    scene.world.use_nodes = True
    scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = .65
    wall = bpy.data.materials.new('Inspection neutral wall')
    wall.diffuse_color = (.40,.39,.36,1)
    bpy.ops.mesh.primitive_cube_add(size=1, location=(0,-WALL+.04,1.8))
    obj = bpy.context.object
    obj.dimensions = (2,.08,3.6)
    obj.data.materials.append(wall)
    bpy.ops.mesh.primitive_plane_add(size=6,location=(0,0,-.004))
    bpy.context.object.data.materials.append(wall)
    for p,power in [((1,-3,5),500),((-2,-1,3),250)]:
        bpy.ops.object.light_add(type='AREA',location=p)
        bpy.context.object.data.energy=power
        bpy.context.object.data.size=3
    bpy.ops.object.camera_add()
    camera=bpy.context.object
    camera.data.type='ORTHO'
    scene.camera=camera
    OUT.mkdir(parents=True,exist_ok=True)
    for label,p,target,scale in [('mounted',(3,-6,3.5),(0,0,1.65),3.8),
            ('connections',(1.5,-2.5,3.7),(.20,.02,3.05),1.18),
            ('drain',(1,-2,1.8),(.058,.12,1.1),2.5)]:
        camera.location=p
        camera.rotation_euler=(Vector(target)-camera.location).to_track_quat('-Z','Y').to_euler()
        camera.data.ortho_scale=scale
        scene.render.filepath=str(OUT/(label+'.png'))
        bpy.ops.render.render(write_still=True)


args=argparse.ArgumentParser()
args.add_argument('--rebuild',action='store_true')
args.add_argument('--preview',action='store_true')
parsed=args.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])
if parsed.rebuild:
    rebuild()
if parsed.preview:
    preview()
else:
    export()
