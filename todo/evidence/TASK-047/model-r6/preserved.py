"""Compare the edited master against this round's isolated baseline, read-only"""
import hashlib
import json
from pathlib import Path

import bpy

root=Path(__file__).resolve().parents[4]
hair=('Hair_','Bangs_','Temple_','Crown_','Nape_','Back_Lock_')
def snapshot(path):
    bpy.ops.wm.open_mainfile(filepath=str(path))
    rig=bpy.data.objects['CHR001_Rig']
    meshes={}
    for obj in rig.children:
        if obj.type!='MESH' or obj.name.startswith(hair):continue
        meshes[obj.name]={
            'vertices':[list(v.co) for v in obj.data.vertices],
            'faces':[list(p.vertices) for p in obj.data.polygons],
            'uv':[list(uv.uv) for uv in obj.data.uv_layers.active.data],
            'weights':[[[group.group,group.weight] for group in v.groups] for v in obj.data.vertices],
            'groups':[group.name for group in obj.vertex_groups],
        }
    bones={b.name:{'parent':b.parent.name if b.parent else None,'matrix':[list(row) for row in b.matrix_local]} for b in rig.data.bones}
    return {'meshes':meshes,'bones':bones}
baseline=snapshot(root/'output/characters/CHR-001/model-r6/baseline.blend')
current=snapshot(root/'source-assets/characters/CHR-001/model/yao-grey-study.blend')
assert baseline==current,'Head, clothes, non-hair UV/weights, or rest rig changed'
atlas=root/'source-assets/characters/CHR-001/model/grey-study.png'
sha=hashlib.sha256(atlas.read_bytes()).hexdigest()
inputs=json.loads((Path(__file__).parent/'input-hashes.json').read_text())
assert sha==inputs[str(atlas.relative_to(root))],'Grayscale atlas changed'
report={'status':'PASS','baseline_commit':'f93af27bfda9650ceb2687b822bda5367540ef1a','non_hair_mesh_count':len(current['meshes']),'joint_count':len(current['bones']),'compared':['source mesh coordinates','polygon indices','UV','vertex weights and groups','bone hierarchy and rest matrices','gray atlas bytes'],'snapshot_sha256':hashlib.sha256(json.dumps(current,sort_keys=True).encode()).hexdigest(),'atlas_sha256':sha,'scope':'Static DCC master data only; Run animation changed separately in animation-r2'}
(Path(__file__).parent/'preserved.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
