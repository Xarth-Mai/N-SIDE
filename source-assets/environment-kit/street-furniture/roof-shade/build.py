"""Edit/export a two-post slatted roof shade; --rebuild authors its initial master."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import bmesh
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
MASTER = HERE / 'roof-shade.blend'
GLB = HERE / 'roof-shade.glb'
MATERIALS = ROOT / 'source-assets/environment-kit/materials'
COLLECTION = 'RoofShade_EXPORT'


def uv_metric(obj, wood=False):
    mesh = obj.data
    uv = mesh.uv_layers.new(name='UVMap')
    for face in mesh.polygons:
        axis = max(range(3), key=lambda i: abs(face.normal[i]))
        axes = ((1, 2), (0, 2), (0, 1))[axis]
        for index in face.loop_indices:
            p = mesh.vertices[mesh.loops[index].vertex_index].co
            # Wood grain runs along each horizontal board; choose a strip between the source siding seams
            uv.data[index].uv = (p[axes[0]] / 2, p[axes[1]] + .1) if wood else (p[axes[0]], p[axes[1]])


def box(name, center, size, material, bevel=.004, rotation=None):
    bpy.ops.mesh.primitive_cube_add(size=1, location=center)
    obj = bpy.context.object
    obj.name = name
    obj.dimensions = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    modifier = obj.modifiers.new('Small manufactured edge', 'BEVEL')
    modifier.width = bevel
    modifier.segments = 1
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    obj.data.materials.append(material)
    uv_metric(obj, material.name == 'RoofShade_Wood')
    if rotation is not None:
        obj.rotation_mode = 'QUATERNION'
        obj.rotation_quaternion = rotation
    for collection in list(obj.users_collection):
        collection.objects.unlink(obj)
    bpy.data.collections[COLLECTION].objects.link(obj)
    return obj


def brace(name, a, b, material):
    a, b = Vector(a), Vector(b)
    return box(name, (a+b)/2, (.065, (b-a).length, .065), material,
               rotation=(b-a).to_track_quat('Y', 'Z'))


def author():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    collection = bpy.data.collections.new(COLLECTION)
    bpy.context.scene.collection.children.link(collection)
    metal = bpy.data.materials.new('RoofShade_Metal')
    metal.use_nodes = True
    shader = metal.node_tree.nodes['Principled BSDF']
    appearance = json.loads((ROOT / 'source-assets/district-scene/appearance.json').read_text())['materials']['metal']
    def linear(v):
        return v/12.92 if v <= .04045 else ((v+.055)/1.055)**2.4
    color = tuple(linear(v) for v in appearance['color'][:3]) + (1,)
    shader.inputs['Base Color'].default_value = color
    shader.inputs['Roughness'].default_value = appearance['roughness']
    shader.inputs['Metallic'].default_value = appearance['metallic']
    metal.diffuse_color = color
    wood = bpy.data.materials.new('RoofShade_Wood')
    wood.use_nodes = True
    shader = wood.node_tree.nodes['Principled BSDF']
    shader.inputs['Roughness'].default_value = .78
    for filename, space, socket in [('WoodSiding009_1K-JPG_Color.jpg', 'sRGB', 'Base Color'),
                                    ('WoodSiding009-NormalGL-scale035.png', 'Non-Color', 'Normal')]:
        tex = wood.node_tree.nodes.new('ShaderNodeTexImage')
        tex.image = bpy.data.images.load(str(MATERIALS / filename))
        tex.image.colorspace_settings.name = space
        tex.image.pack()
        if socket == 'Normal':
            normal = wood.node_tree.nodes.new('ShaderNodeNormalMap')
            normal.inputs['Strength'].default_value = 1
            wood.node_tree.links.new(tex.outputs['Color'], normal.inputs['Color'])
            wood.node_tree.links.new(normal.outputs['Normal'], shader.inputs['Normal'])
        else:
            wood.node_tree.links.new(tex.outputs['Color'], shader.inputs[socket])
    for side, x in [('west', -1.75), ('east', 1.75)]:
        box(f'{side} foot plate', (x, 1.30, .0125), (.26, .26, .025), metal, .003)
        box(f'{side} rear post', (x, 1.30, 1.245), (.095, .12, 2.44), metal)
        brace(f'{side} rear knee', (x, 1.30, 1.92), (x, .65, 2.405), metal)
        for dx in (-.085, .085):
            for dy in (-.085, .085):
                bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=.012, depth=.012, location=(x+dx, 1.30+dy, .031))
                bolt = bpy.context.object
                bolt.name = f'{side} base fixing {dx} {dy}'
                bolt.data.materials.append(metal)
                uv_metric(bolt)
                for current in list(bolt.users_collection): current.objects.unlink(bolt)
                collection.objects.link(bolt)
    box('rear cross member', (0, 1.30, 2.405), (3.595, .12, .12), metal)
    for i, x in enumerate((-1.75, 0, 1.75)):
        box(f'cantilever arm {i+1}', (x, .18, 2.405), (.08, 2.56, .12), metal)
    for i in range(10):
        box(f'wood shade slat {i+1:02}', (0, -1.01+i*.235, 2.4925), (4.04, .15, .055), wood, .003)
    scene = bpy.context.scene
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1
    scene['asset_id'] = 'AST-ROOF-SHADE'
    scene['pivot'] = 'Ground at existing seat centre; Blender +Y is rear, front is -Y'
    scene['placement_map_xyz'] = [312, 237, 37]
    scene['scope'] = 'Open two-post shade only; existing public bench remains a separate reused asset'
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=str(MASTER))


def export():
    copies = []
    for source in sorted(bpy.data.collections[COLLECTION].objects, key=lambda obj: obj.name):
        obj = source.copy()
        obj.data = source.data.copy()
        bpy.context.scene.collection.objects.link(obj)
        copies.append(obj)
    bpy.ops.object.select_all(action='DESELECT')
    for obj in copies: obj.select_set(True)
    bpy.context.view_layer.objects.active = copies[0]
    bpy.ops.object.convert(target='MESH')
    bpy.ops.object.join()
    obj = bpy.context.object
    bpy.context.scene.cursor.location = (0, 0, 0)
    bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    obj.name = obj.data.name = 'RoofShade'
    mesh = bmesh.new()
    mesh.from_mesh(obj.data)
    bmesh.ops.triangulate(mesh, faces=list(mesh.faces))
    mesh.to_mesh(obj.data)
    mesh.free()
    bpy.ops.export_scene.gltf(filepath=str(GLB), export_format='GLB', use_selection=True, export_yup=True,
                              export_animations=False, export_cameras=False, export_lights=False,
                              export_extras=False, export_materials='EXPORT', export_tangents=True)
    bpy.data.objects.remove(obj, do_unlink=True)
    print(json.dumps({'status': 'PASS', 'blender': bpy.app.version_string, 'source': str(MASTER.relative_to(ROOT)),
                      'source_sha256': hashlib.sha256(MASTER.read_bytes()).hexdigest(),
                      'glb': str(GLB.relative_to(ROOT)), 'glb_sha256': hashlib.sha256(GLB.read_bytes()).hexdigest(),
                      'bytes': GLB.stat().st_size, 'render': 'NOT RUN'}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rebuild', action='store_true')
    args = parser.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])
    if args.rebuild: author()
    else: bpy.ops.wm.open_mainfile(filepath=str(MASTER))
    export()
