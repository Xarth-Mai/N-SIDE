"""V-15 additive cinema facade, authored in metres in the source map frame

blender --background --python source-assets/buildings/V-15/build.py -- --render
Use --export-existing to export the edited .blend without rebuilding geometry
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import sys

import bpy
import bmesh
from mathutils import Vector

SOURCE = Path(__file__).resolve().parent
ROOT = SOURCE.parents[2]
BLEND = SOURCE / 'mirror-hall-facade.blend'
GLB = ROOT / 'game/assets/environment/buildings/v15-mirror-hall-facade.glb'
OUTPUT = ROOT / 'output/buildings/V-15/r1'
MATERIALS = ROOT / 'source-assets/environment-kit/materials'
FONT = ROOT / 'source-assets/ui-kit/fonts/NotoSansSC-VF.ttf'
parser = argparse.ArgumentParser()
parser.add_argument('--render', action='store_true')
parser.add_argument('--export-existing', action='store_true')
args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else [])
GLB.parent.mkdir(parents=True, exist_ok=True)
OUTPUT.mkdir(parents=True, exist_ok=True)


def material(name, color, roughness=.65, metallic=0, texture=None, repeat=(2, 2), emission=False):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.diffuse_color = (*color, 1)
    bsdf = mat.node_tree.nodes.get('Principled BSDF')
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Roughness'].default_value = roughness
    bsdf.inputs['Metallic'].default_value = metallic
    if emission:
        bsdf.inputs['Emission Color'].default_value = (*color, 1)
        bsdf.inputs['Emission Strength'].default_value = 1
    if texture:
        for suffix, space in [('Color', 'sRGB'), ('NormalGL', 'Non-Color')]:
            node = mat.node_tree.nodes.new('ShaderNodeTexImage')
            node.image = bpy.data.images.load(str(MATERIALS / f'{texture}_1K-JPG_{suffix}.jpg'), check_existing=True)
            node.image.colorspace_settings.name = space
            if suffix == 'Color':
                mat.node_tree.links.new(node.outputs['Color'], bsdf.inputs['Base Color'])
            else:
                normal = mat.node_tree.nodes.new('ShaderNodeNormalMap')
                normal.inputs['Strength'].default_value = .35
                mat.node_tree.links.new(node.outputs['Color'], normal.inputs['Color'])
                mat.node_tree.links.new(normal.outputs['Normal'], bsdf.inputs['Normal'])
    mat['metres_per_repeat'] = repeat
    return mat


def put(obj, collection):
    for current in list(obj.users_collection):
        current.objects.unlink(obj)
    collection.objects.link(obj)
    return obj


def box(name, low, high, mat, collection, bevel=.015):
    centre = (Vector(low) + Vector(high)) / 2
    size = Vector(high) - Vector(low)
    assert min(size) > 0
    bpy.ops.mesh.primitive_cube_add(size=1, location=centre)
    obj = put(bpy.context.object, collection)
    obj.name = name
    obj.data.name = name
    obj.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(mat)
    uv = obj.data.uv_layers.active
    repeat = mat['metres_per_repeat']
    for poly in obj.data.polygons:
        axis = max(range(3), key=lambda i: abs(poly.normal[i]))
        axes = ((1, 2), (0, 2), (0, 1))[axis]
        for loop in poly.loop_indices:
            point = obj.data.vertices[obj.data.loops[loop].vertex_index].co + centre
            uv.data[loop].uv = (point[axes[0]] / repeat[0], point[axes[1]] / repeat[1])
    if bevel:
        mod = obj.modifiers.new('Small manufactured edge', 'BEVEL')
        mod.width = min(bevel, min(size) / 4)
        mod.segments = 1
    return obj


def text(name, body, position, size, mat, collection, font=None, max_width=None, height=None):
    curve = bpy.data.curves.new(name, 'FONT')
    curve.body = body
    curve.size = size
    curve.extrude = .012 if size > 1 else .002
    curve.resolution_u = 3
    if font:
        curve.font = font
        curve.offset = .012
    obj = bpy.data.objects.new(name, curve)
    collection.objects.link(obj)
    obj.location = position
    obj.rotation_euler[0] = math.pi / 2
    curve.materials.append(mat)
    bpy.context.view_layer.update()
    if height:
        world_bounds=[obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
        obj.scale *= height / (max(p.z for p in world_bounds)-min(p.z for p in world_bounds))
        bpy.context.view_layer.update()
    if max_width and obj.dimensions.x > max_width:
        obj.scale.x *= max_width / obj.dimensions.x
    return obj


def beam(name, a, b, width, mat, collection):
    obj = box(name, (-width/2, -width/2, 0), (width/2, width/2, (Vector(b)-Vector(a)).length), mat, collection)
    obj.location = (Vector(a) + Vector(b))/2
    obj.rotation_euler = (Vector(b)-Vector(a)).to_track_quat('Z', 'Y').to_euler()
    return obj


def poster(name, x, bottom, width, height, face, accent, mat, collection):
    # Original abstract programme graphics, not a new film title or licensed illustration
    box(name+' backing', (x-width/2-.085, -.59, bottom-.085), (x+width/2+.085, -.24, bottom+height+.085), mat['metal'], collection)
    box(name+' print', (x-width/2, -.614, bottom), (x+width/2, -.594, bottom+height), face, collection, 0)
    box(name+' horizon', (x-width*.44, -.64, bottom+height*.22), (x+width*.44, -.621, bottom+height*.265), accent, collection, 0)
    box(name+' composition', (x-width*.28, -.642, bottom+height*.32), (x+width*.22, -.622, bottom+height*.73), accent, collection, 0)
    box(name+' light stripe', (x-width*.03, -.662, bottom+height*.38), (x+width*.045, -.643, bottom+height*.91), mat['letter'], collection, 0)
    text(name+' footer', 'MIRROR HALL', (x-width*.43, -.668, bottom+height*.08), width*.11, mat['letter'], collection, max_width=width*.85)


def author():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.name = 'Scene0'
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1
    facade = bpy.data.collections.new('V15_EXPORT_Facade')
    context = bpy.data.collections.new('REVIEW_ONLY_Map_context')
    scene.collection.children.link(facade)
    scene.collection.children.link(context)
    mats = {
        'concrete': material('V15_ambientCG_Concrete034', (.52,.50,.47), texture='Concrete034', repeat=(1.1,.55)),
        'plaster': material('V15_ambientCG_Plaster001', (.67,.65,.60), texture='Plaster001'),
        'wood': material('V15_ambientCG_WoodSiding009', (.26,.16,.08), texture='WoodSiding009', repeat=(2,1)),
        'metal': material('V15_Charcoal_metal', (.055,.07,.08), .38, .5),
        'bronze': material('V15_Warm_trim', (.27,.17,.08), .36, .6),
        'letter': material('V15_Warm_letter_and_diffuser', (.90,.70,.37), .43, .05, emission=True),
        'blue': material('V15_Programme_blue', (.035,.10,.18)),
        'rust': material('V15_Programme_red', (.36,.07,.045)),
    }
    font = bpy.data.fonts.load(str(FONT))
    # Only thin attached skin, never a second 40 x 30 x 12 m building
    box('Portal left masonry reveal', (.60,-.42,0), (1.05,.04,11.65), mats['concrete'], facade)
    box('Portal right masonry reveal', (17.55,-.42,0), (18.0,.04,11.65), mats['concrete'], facade)
    box('Cinema sign wall', (1.05,-.27,3.58), (17.55,-.045,11.35), mats['plaster'], facade)
    box('Cinema sign upper coping', (.58,-.46,11.35), (18.02,.08,11.66), mats['metal'], facade)
    for x in [3.3, 10.4, 14.1]:
        box('Vertical panel joint', (x,-.285,3.62), (x+.025,-.273,11.30), mats['metal'], facade, 0)
    for z in [7.7, 9.55]:
        box('Horizontal panel joint', (1.10,-.285,z), (17.50,-.273,z+.022), mats['metal'], facade, 0)
    # Soffit lights bottom at 3.106 m; the entrance walk strip remains 3 m wide and empty
    box('Deep canopy folded fascia', (.60,-3.0,3.16), (18.0,.06,3.58), mats['metal'], facade)
    box('Recessed timber soffit', (.87,-2.84,3.12), (17.72,-.20,3.17), mats['wood'], facade)
    box('Canopy bronze drip edge', (.65,-3.035,3.20), (17.95,-3.005,3.24), mats['bronze'], facade, .003)
    for x in [2,5,8,11,14,17]:
        box('Soffit recessed light', (x-.21,-2.45,3.106), (x+.21,-2.22,3.122), mats['letter'], facade, .004)
    for x in [3.90,7.86]:
        box('Deep entrance jamb', (x,-.66,0), (x+.24,.045,3.12), mats['concrete'], facade)
        box('Entrance bronze reveal', (x+.05,-.675,.12), (x+.10,-.661,2.93), mats['bronze'], facade, .003)
    box('Deep entrance lintel', (3.90,-.66,2.92), (8.10,.045,3.12), mats['concrete'], facade)
    text('Mirror Hall raised Chinese title', '镜厅', (4.12,-.335,5.63), 2.35, mats['letter'], facade, font=font, height=1.8)
    text('Mirror Hall raised English title', 'MIRROR HALL', (4.20,-.332,5.01), .42, mats['metal'], facade)
    text('Mirror Hall programme line', 'CINEMA  /  CULTURE', (4.20,-.332,4.49), .24, mats['metal'], facade)
    poster('Tall civic poster', 2.16,4.14,1.46,5.93,mats['blue'],mats['rust'],mats,facade)
    for i,x in enumerate([11.25,13.48,15.71]):
        poster('Street programme '+str(i+1),x,.45,1.42,2.25,mats['blue'] if i!=1 else mats['rust'],mats['rust'] if i!=1 else mats['blue'],mats,facade)
    box('Canopy drain', (17.23,-.52,.14), (17.36,-.36,3.15), mats['metal'], facade)
    # Below the existing 37 m deck: keep the whole 4 m rooftop route free above z=12
    box('Upper street connection bearing', (39.98,20.0,11.22), (40.38,30.0,11.83), mats['concrete'], facade)
    for y in [20.5,29.85]:
        box('Upper street edge beam', (40.12,y-.12,11.48), (43.15,y+.12,11.80), mats['metal'], facade)
        beam('Upper street diagonal bracket', (40.24,y,9.18), (43.0,y,11.58), .18, mats['metal'], facade)
        box('Upper street bracket wall plate', (39.98,y-.23,9.05), (40.28,y+.23,9.85), mats['bronze'], facade)
    # Context is a labelled DCC measuring aid and is excluded by collection from export
    context_mat = material('REVIEW_ONLY source shell', (.36,.39,.42))
    ground_mat = material('REVIEW_ONLY ground', (.24,.28,.30))
    glass_mat = material('REVIEW_ONLY existing opaque door', (.08,.18,.21), .24, .2)
    box('REVIEW_ONLY existing V15 shell', (0,.06,0), (40,30,12), context_mat, context, 0)
    box('REVIEW_ONLY ground datum 25m', (-6,-16,-.2), (57,42,-.06), ground_mat, context, 0)
    box('REVIEW_ONLY existing entry glass', (4.2,-.13,.04), (7.8,-.07,2.75), glass_mat, context, 0)
    box('REVIEW_ONLY existing door frame', (5.96,-.20,0), (6.04,-.13,2.80), mats['metal'], context, 0)
    district=json.loads((ROOT/'source-assets/district-map/district.json').read_text())
    platform=next(surface for surface in district['surfaces'] if surface.get('id')=='cinema_upper_platform')
    polygon=[(x-300,y-220) for x,y in platform['polygon']]
    n=len(polygon)
    vertices=[(x,y,z) for z in [11.60,12] for x,y in polygon]
    faces=[tuple(reversed(range(n))),tuple(range(n,2*n))]
    faces += [(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    mesh=bpy.data.meshes.new('REVIEW_ONLY source platform polygon');mesh.from_pydata(vertices,[],faces);mesh.update()
    deck=bpy.data.objects.new('REVIEW_ONLY existing upper street platform',mesh);context.objects.link(deck)
    mesh.materials.append(context_mat)
    box('REVIEW_ONLY 1.7m scale person', (7.05,-5.6,0), (7.5,-5.28,1.7), mats['rust'], context, .07)
    scene.world = bpy.data.worlds.new('Review daylight')
    scene.world.use_nodes = True
    scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.50,.65,.80,1)
    scene.world.node_tree.nodes['Background'].inputs[1].default_value=.55
    sun_data=bpy.data.lights.new('Review sun','SUN');sun_data.energy=2.0;sun_data.angle=math.radians(12)
    sun=bpy.data.objects.new('REVIEW_ONLY sun',sun_data);context.objects.link(sun)
    sun.rotation_euler=(math.radians(25),math.radians(-20),math.radians(-25))
    camera_data=bpy.data.cameras.new('Review camera')
    camera=bpy.data.objects.new('REVIEW_ONLY camera',camera_data);context.objects.link(camera);scene.camera=camera
    camera.location=(23,-31,13)
    camera.rotation_euler=(Vector((8.8,0,5.2))-camera.location).to_track_quat('-Z','Y').to_euler()
    camera.data.lens=44
    scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=32
    scene.cycles.use_denoising=True
    scene.render.resolution_x=1440;scene.render.resolution_y=960;scene.render.resolution_percentage=100
    scene.view_settings.view_transform='AgX'
    scene.render.image_settings.file_format='PNG'
    scene['asset_id']='BLD-V15-FACADE-001'
    scene['source_origin_xyz']=[300.0,220.0,25.0]
    scene['runtime_anchor_xyz']=[300.0,25.0,-220.0]
    scene['export_collection']='V15_EXPORT_Facade'
    bpy.ops.file.pack_all()
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(BLEND))


def export():
    facade=bpy.data.collections['V15_EXPORT_Facade']
    copies=[]
    for original in sorted(facade.objects,key=lambda obj:obj.name):
        obj=original.copy();obj.data=original.data.copy();bpy.context.scene.collection.objects.link(obj)
        copies.append(obj)
    bpy.ops.object.select_all(action='DESELECT')
    for obj in copies:obj.select_set(True)
    bpy.context.view_layer.objects.active=copies[0]
    bpy.ops.object.convert(target='MESH')
    bpy.ops.object.join()
    runtime=bpy.context.object;runtime.name='V15_MirrorHall_Facade';runtime.data.name=runtime.name
    bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
    mesh=bmesh.new();mesh.from_mesh(runtime.data)
    bmesh.ops.triangulate(mesh,faces=list(mesh.faces))
    # Font outline tessellation can leave zero-area slivers; remove them on the export copy
    bmesh.ops.delete(mesh,geom=[face for face in mesh.faces if face.calc_area()<1e-8],context='FACES')
    mesh.to_mesh(runtime.data);mesh.free()
    bpy.ops.export_scene.gltf(filepath=str(GLB),export_format='GLB',use_selection=True,export_yup=True,
                              export_animations=False,export_cameras=False,export_lights=False,export_materials='EXPORT',export_extras=False)
    bpy.data.objects.remove(runtime,do_unlink=True)


def inspect_components():
    bounds={}
    depsgraph=bpy.context.evaluated_depsgraph_get()
    for obj in bpy.data.collections['V15_EXPORT_Facade'].objects:
        evaluated=obj.evaluated_get(depsgraph)
        mesh=evaluated.to_mesh()
        points=[obj.matrix_world @ vertex.co for vertex in mesh.vertices]
        bounds[obj.name]={'min':[min(p[i] for p in points) for i in range(3)],
                          'max':[max(p[i] for p in points) for i in range(3)]}
        evaluated.to_mesh_clear()
    clearances={
        'south_3m_approach':([4.5,-4,0],[7.5,0,2.75]),
        'east_dining_door':([40,10.2,4],[41,13.8,6.75]),
        'north_service_door':([19.35,30,0],[20.65,31,2.35]),
        'east_roof_route':([38,0,12],[42,30,14]),
        'upper_platform_connection':([40,20,12],[44,30,14]),
    }
    for name,(low,high) in clearances.items():
        hits=[obj for obj,b in bounds.items() if all(b['min'][i]<high[i]-1e-5 and b['max'][i]>low[i]+1e-5 for i in range(3))]
        assert not hits,(name,hits)
    (OUTPUT/'component-bounds.json').write_text(json.dumps({'frame':'Blender local x,y,z metres','bounds':bounds,'clear_volumes':clearances},indent=2)+'\n')


def report():
    raw=GLB.read_bytes();size,kind=struct.unpack_from('<I4s',raw,12)
    assert kind==b'JSON'
    gltf=json.loads(raw[20:20+size])
    primitives=[p for mesh in gltf['meshes'] for p in mesh['primitives']]
    pos=[gltf['accessors'][p['attributes']['POSITION']] for p in primitives]
    info={'blender':bpy.app.version_string,'glb_sha256':hashlib.sha256(raw).hexdigest(),'glb_bytes':len(raw),
          'scene_count':len(gltf['scenes']),'mesh_count':len(gltf['meshes']),'primitives':len(primitives),
          'vertices':sum(a['count'] for a in pos),
          'triangles':sum(gltf['accessors'][p['indices']]['count']//3 for p in primitives),
          'bounds_gltf':{'min':[min(a['min'][i] for a in pos) for i in range(3)],'max':[max(a['max'][i] for a in pos) for i in range(3)]},
          'materials':[m['name'] for m in gltf['materials']],'embedded_images':len(gltf.get('images',[])),
          'extensions_used':gltf.get('extensionsUsed',[]),'nodes':gltf['nodes']}
    assert set(info['extensions_used']) <= {'KHR_materials_unlit','KHR_texture_transform'}
    assert info['vertices']<20000 and info['triangles']<16000 and len(primitives)<=8
    assert all('material' in p and 'NORMAL' in p['attributes'] and 'TEXCOORD_0' in p['attributes'] for p in primitives)
    assert all('bufferView' in image for image in gltf.get('images',[]))
    assert not gltf.get('animations') and not gltf.get('skins') and not gltf.get('cameras')
    (OUTPUT/'export-report.json').write_text(json.dumps(info,indent=2)+'\n')
    print(json.dumps(info,indent=2))


def render():
    scene=bpy.context.scene
    for name,position,target,lens in [
        ('south-entry', (23,-31,13),(8.8,0,5.2),44),
        ('entry-human-height', (11,-17,1.75),(7.5,0,4.2),30),
        ('upper-connection',(56,6,18),(41.2,24,10),42),
    ]:
        scene.camera.location=position
        scene.camera.rotation_euler=(Vector(target)-scene.camera.location).to_track_quat('-Z','Y').to_euler()
        scene.camera.data.lens=lens
        scene.render.filepath=str(OUTPUT/(name+'.png'))
        bpy.ops.render.render(write_still=True)


if args.export_existing:
    bpy.ops.wm.open_mainfile(filepath=str(BLEND))
else:
    author()
inspect_components()
export()
report()
if args.render:
    render()
