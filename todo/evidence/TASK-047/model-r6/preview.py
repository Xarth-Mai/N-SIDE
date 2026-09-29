"""Replace only hair in an isolated copy of the committed master; no formal writes"""
import ast
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Vector
from mathutils.bvhtree import BVHTree

root=Path(__file__).resolve().parents[4]
revision=sys.argv[sys.argv.index('--')+1]
output=root/'output/characters/CHR-001/model-r6'
bpy.ops.wm.open_mainfile(filepath=str(output/'baseline.blend'))
scene=bpy.context.scene
scene.render.threads_mode='FIXED';scene.render.threads=2
rig=bpy.data.objects['CHR001_Rig']
rig.animation_data.action=bpy.data.actions['Idle'];scene.frame_set(1)
material=bpy.data.materials['CHR001_GreyStudy_Atlas']
REGIONS=['skin','hair','jacket','shirt','pants','shoe','ink','white']
SIZE=1024
hair_prefixes=('Hair_','Bangs_','Temple_','Crown_','Nape_','Back_Lock_')
def preserved():
    data={obj.name:[list(v.co) for v in obj.data.vertices] for obj in rig.children if obj.type=='MESH' and not obj.name.startswith(hair_prefixes)}
    return hashlib.sha256(json.dumps(data,sort_keys=True).encode()).hexdigest()
unchanged=preserved()
for obj in list(rig.children):
    if obj.type=='MESH' and obj.name.startswith(hair_prefixes):bpy.data.objects.remove(obj,do_unlink=True)
head=bpy.data.objects['Face_Head'].evaluated_get(bpy.context.evaluated_depsgraph_get())
head_mesh=head.to_mesh()
face_surface=BVHTree.FromPolygons([v.co+Vector((0,0,.018)) for v in head_mesh.vertices],[list(p.vertices) for p in head_mesh.polygons])
head.to_mesh_clear()
objects=[]
source=root/'source-assets/characters/CHR-001/model/build.py'
code=source.read_text()
for node in ast.parse(code).body:
    if isinstance(node,ast.FunctionDef) and node.name in ('atlas_uv','mesh','loft','export'):
        exec(compile(ast.Module(body=[node],type_ignores=[]),str(source),'exec'))
hair_start=code.index('# Swept group surfaces all start')
hair_end=code.index('# Minimal original double-dot',hair_start)
exec(compile(code[hair_start:hair_end],str(source),'exec'))
for obj in objects:
    for vertex in obj.data.vertices:vertex.co.z-=.018
    bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
    bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.remove_doubles(threshold=.000001);bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT')
assert unchanged==preserved(),'Hair preview changed non-hair geometry'
bpy.ops.wm.save_as_mainfile(filepath=str(output/(revision+'.blend')))
RUNTIME=output/revision
RUNTIME.mkdir(exist_ok=True)
export(rig,[o for o in rig.children if o.type=='MESH'])
report={'revision':revision,'hair_source_sha256':hashlib.sha256(code[hair_start:hair_end].encode()).hexdigest(),'non_hair_geometry_unchanged':True,'non_hair_vertices_sha256':unchanged,'editable_hair_objects':len(objects),'source_hair_vertices':sum(len(o.data.vertices) for o in objects),'formal_master_untouched':True,'formal_glb_untouched':True}
(Path(__file__).parent/(revision+'-input.json')).write_text(json.dumps(report,indent=2)+'\n')
