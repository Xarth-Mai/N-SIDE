import bpy, json, math
from pathlib import Path
from mathutils import Vector
base=Path('output/assets/zzz-reference/belle-player-share').resolve()
bpy.ops.wm.open_mainfile(filepath=str(base/'belle-import.blend'))
target=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE')
with bpy.data.libraries.load(str(Path('source-assets/characters/CHR-002/model/ling-grey-study.blend').resolve()),link=False) as (library, loaded):
    loaded.objects=['CHR002_Rig']; loaded.actions=['Idle','Walk','Run']
source=loaded.objects[0];bpy.context.scene.collection.objects.link(source)
def bone_data(rig, name):
    bone=rig.data.bones[name]
    return dict(head=list(bone.head_local),tail=list(bone.tail_local),quat=list(bone.matrix_local.to_quaternion()),parent=bone.parent.name if bone.parent else None)
names=['Hips','Spine','Chest','Head','Clavicle.L','UpperArm.L','Forearm.L','Hand.L','Thigh.L','Shin.L','Foot.L']
target_names=['Pelvis','Spine','Spine2','Head1','L_Clavicle','L_UpperArm','L_Forearm','L_Hand','L_Thigh','L_Calf','L_Foot']
report={'rest':{a:{'source':bone_data(source,a),'target':bone_data(target,'ValveBiped.Bip01_'+b)} for a,b in zip(names,target_names)},'samples':{}}
for action in loaded.actions:
    source.animation_data.action=action
    report['samples'][action.name]=[]
    for frame in [1,11,21,31]:
        bpy.context.scene.frame_set(frame);bpy.context.view_layer.update()
        points={name:list(source.pose.bones[name].head) for name in names}
        def angle(a,b):return math.degrees((source.pose.bones[b].head-source.pose.bones[a].head).angle(Vector((0,0,-1))))
        report['samples'][action.name].append({'frame':frame,'points':points,'upperarm_from_down':angle('UpperArm.L','Forearm.L'),'forearm_from_down':angle('Forearm.L','Hand.L')})
(base/'rest-diagnostic-r3.json').write_text(json.dumps(report,indent=2))
print(json.dumps(report,indent=2))
