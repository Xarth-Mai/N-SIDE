"""Compare r7 DCC and GLB contracts against the 13397c5 baseline"""
import hashlib
import json
from pathlib import Path
import sys

import bpy

root=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(root/'tools'))
from glb import read_glb, values
output=root/'output/characters/CHR-001/model-r7'
changed=('TShirt','Hood','Jacket_','Zip_Tape.','Tee_','Hair_Cap','Bangs_','Temple_')
def snapshot(path):
    bpy.ops.wm.open_mainfile(filepath=str(path))
    rig=bpy.data.objects['CHR001_Rig']
    meshes={}
    for obj in rig.children:
        if obj.type!='MESH' or obj.name.startswith(changed):continue
        meshes[obj.name]={
            'vertices':[list(v.co) for v in obj.data.vertices],
            'faces':[list(p.vertices) for p in obj.data.polygons],
            'uv':[list(uv.uv) for uv in obj.data.uv_layers.active.data],
            'weights':[[[g.group,g.weight] for g in v.groups] for v in obj.data.vertices],
            'groups':[g.name for g in obj.vertex_groups],
        }
    bones={b.name:{'parent':b.parent.name if b.parent else None,'matrix':[list(row) for row in b.matrix_local]} for b in rig.data.bones}
    images={image.name:hashlib.sha256(image.packed_file.data).hexdigest() for image in bpy.data.images if image.packed_file}
    return {'meshes':meshes,'bones':bones,'images':images}
before=snapshot(output/'baseline.blend')
after=snapshot(root/'source-assets/characters/CHR-001/model/yao-grey-study.blend')
checks={'untouched_source_meshes_identical':before['meshes']==after['meshes'],'source_bones_rest_and_hierarchy_identical':before['bones']==after['bones'],'packed_source_image_bytes_identical':before['images']==after['images']}
bd,bb=read_glb(output/'baseline.glb')
ad,ab=read_glb(root/'game/assets/characters/CHR-001/yao-grey-study.glb')
def accessor(doc,binary,index):return values(doc,binary,doc['accessors'][index])
def animation(doc,binary):
    return {clip['name']:{doc['nodes'][c['target']['node']]['name']+'.'+c['target']['path']:{'interpolation':clip['samplers'][c['sampler']].get('interpolation','LINEAR'),'input':accessor(doc,binary,clip['samplers'][c['sampler']]['input']),'output':accessor(doc,binary,clip['samplers'][c['sampler']]['output'])} for c in clip['channels']} for clip in doc['animations']}
def textures(doc,binary):
    result=[]
    for image in doc['images']:
        view=doc['bufferViews'][image['bufferView']]
        offset=view.get('byteOffset',0)
        result.append(hashlib.sha256(binary[offset:offset+view['byteLength']]).hexdigest())
    return result
checks.update({'GLB_nodes_identical':bd['nodes']==ad['nodes'],'GLB_joint_names_identical':bd['skins'][0]['joints']==ad['skins'][0]['joints'],'inverse_binds_identical':accessor(bd,bb,bd['skins'][0]['inverseBindMatrices'])==accessor(ad,ab,ad['skins'][0]['inverseBindMatrices']),'all_clip_samples_identical':animation(bd,bb)==animation(ad,ab),'three_embedded_texture_bytes_identical':len(ad['images'])==3 and textures(bd,bb)==textures(ad,ab),'material_identical':bd['materials']==ad['materials']})
report={'status':'PASS' if all(checks.values()) else 'FAIL','baseline_commit':'13397c5','checks':checks,'unchanged_source_meshes':sorted(after['meshes']),'packed_source_images':after['images'],'embedded_GLB_images':textures(ad,ab),'scope':'Exact source meshes outside declared edits, 32-bone rest/bind hierarchy, all exported animation samples, material and embedded texture bytes; excludes artistic acceptance and real game behavior'}
Path(__file__).with_name('preserved-contract.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
assert all(checks.values()), 'Preserved DCC or GLB contract changed'
