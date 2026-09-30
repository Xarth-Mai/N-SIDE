"""Pose evidence for this Belle conversion, using actual joint heads, not SFM tails."""
import bpy,json,math,sys
from pathlib import Path
from mathutils import Vector
base=Path('output/assets/zzz-reference/belle-player-share').resolve()
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
filename=args[0] if args else 'belle-adapted.blend'
label=args[1] if len(args)>1 else 'r3'
bpy.ops.wm.open_mainfile(filepath=str(base/filename))
scene=bpy.context.scene
rig=next(o for o in scene.objects if o.type=='ARMATURE')
actions={name:bpy.data.actions[name] for name in ['Idle','Walk','Run']}
with bpy.data.libraries.load(str(Path('source-assets/characters/CHR-002/model/ling-grey-study.blend').resolve()),link=False) as (library,loaded):
    loaded.objects=['CHR002_Rig'];loaded.actions=['Idle','Walk','Run']
source=loaded.objects[0];scene.collection.objects.link(source)
prefix='ValveBiped.Bip01_'
shape_key_basis_error=max(((vertex.co-mesh.data.shape_keys.key_blocks[0].data[vertex.index].co).length for mesh in scene.objects if mesh.type=='MESH' and mesh.data.shape_keys for vertex in mesh.data.vertices),default=0.)
report={'input':filename,'label':label,'checks':{},'max_shape_key_basis_error':shape_key_basis_error,'clips':{}}
for name,frames,src_action in zip(['Idle','Walk','Run'],[120,48,40],loaded.actions):
    rig.animation_data.action=actions[name];source.animation_data.action=src_action
    rows=[];first=None;last=None;max_root=0.;min_head=100.;min_foot=100.;max_error=0.;finite=True
    for frame in range(1,frames+2):
        scene.frame_set(frame);bpy.context.view_layer.update()
        pelvis=rig.pose.bones[prefix+'Pelvis'].head
        head=rig.pose.bones[prefix+'Head1'].head
        feet=[rig.pose.bones[prefix+s+'_Foot'].head for s in ['L','R']]
        head_gap=head.z-pelvis.z;foot_gap=min(pelvis.z-f.z for f in feet)
        root=rig.pose.bones['Root'].matrix.translation.length
        min_head=min(min_head,head_gap);min_foot=min(min_foot,foot_gap);max_root=max(max_root,root)
        errors={};angles={}
        for side in ['L','R']:
            for a,b in [('UpperArm','Forearm'),('Forearm','Hand')]:
                td=rig.pose.bones[prefix+side+'_'+b].head-rig.pose.bones[prefix+side+'_'+a].head
                sd=source.pose.bones[b+'.'+side].head-source.pose.bones[a+'.'+side].head
                errors[a+'.'+side]=math.degrees(td.angle(sd));angles[a+'.'+side]=math.degrees(td.angle(Vector((0,0,-1))))
        max_error=max(max_error,*errors.values())
        matrices={b.name:list(v for row in b.matrix for v in row) for b in rig.pose.bones}
        finite=finite and all(math.isfinite(v) for row in matrices.values() for v in row)
        if frame==1:first=matrices
        if frame==frames+1:last=matrices
        if frame in [1,1+frames//4,1+frames//2,1+3*frames//4,frames+1]:
            rows.append(dict(frame=frame,head_z=head.z,pelvis_z=pelvis.z,feet_z=[v.z for v in feet],root_translation=root,arm_from_down_degrees=angles,source_arm_error_degrees=errors))
    loop=max(abs(v-w) for key in first for v,w in zip(first[key],last[key]))
    checks={'finite':finite,'head_above_pelvis':min_head>.30,'feet_below_pelvis':min_foot>.40,'stationary_root':max_root<1e-6,'arm_matches_source':max_error<.1,'closed_loop':loop<1e-5}
    report['clips'][name]={'checks':checks,'min_head_above_pelvis':min_head,'min_pelvis_above_foot':min_foot,'max_root_translation':max_root,'max_arm_direction_error_degrees':max_error,'max_loop_matrix_difference':loop,'samples':rows}
report['checks']={'shape_key_basis_matches_mesh':shape_key_basis_error<1e-6,'pass':shape_key_basis_error<1e-6 and all(v for clip in report['clips'].values() for v in clip['checks'].values())}
(base/('pose-check-'+label+'.json')).write_text(json.dumps(report,indent=2))
print(json.dumps({name:{k:v for k,v in clip.items() if k!='samples'} for name,clip in report['clips'].items()},indent=2))
if label=='r3':assert report['checks']['pass'],report['checks']
