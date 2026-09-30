import bpy,json,math
from pathlib import Path
from mathutils import Vector
base=Path('output/assets/zzz-reference/belle-player-share').resolve()
bpy.ops.wm.open_mainfile(filepath=str(base/'belle-adapted.blend'))
scene=bpy.context.scene
rig=next(o for o in scene.objects if o.type=='ARMATURE')
scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=16
scene.render.resolution_x=480;scene.render.resolution_y=640;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG'
scene.world=bpy.data.worlds.new('PoseWorld');scene.world.color=(.18,.18,.18)
scene.view_settings.view_transform='Standard'
for location,power,size in [((2,-3,4),350,4),((-3,-1,2),200,3),((1,2,3),250,3)]:
    data=bpy.data.lights.new('PoseLight','AREA');data.energy=power;data.shape='DISK';data.size=size
    light=bpy.data.objects.new('PoseLight',data);scene.collection.objects.link(light);light.location=location
    light.rotation_euler=(Vector((0,0,.9))-light.location).to_track_quat('-Z','Y').to_euler()
data=bpy.data.cameras.new('PoseCamera');camera=bpy.data.objects.new('PoseCamera',data);scene.collection.objects.link(camera)
data.type='ORTHO';data.ortho_scale=2.;scene.camera=camera
views=[('idle-front','Idle',1,(0,-4,1.0)),('idle-quarter','Idle',1,(2.5,-4,1.6)),('walk-quarter','Walk',13,(2.5,-4,1.6)),('run-quarter-11','Run',11,(2.5,-4,1.6)),('run-quarter-31','Run',31,(2.5,-4,1.6)),('run-side-11','Run',11,(4,0,1.1))]
for name,action,frame,position in views:
    rig.animation_data.action=bpy.data.actions[action];scene.frame_set(frame);bpy.context.view_layer.update()
    camera.location=position;camera.rotation_euler=(Vector((0,0,.86))-camera.location).to_track_quat('-Z','Y').to_euler()
    scene.render.filepath=str(base/('pose-r4-'+name+'.png'));bpy.ops.render.render(write_still=True)
(base/'render-pose-r4.json').write_text(json.dumps({'engine':'Cycles CPU','samples':16,'resolution':[480,640],'views':views,'status':'rendered; visual review recorded separately'},indent=2))
