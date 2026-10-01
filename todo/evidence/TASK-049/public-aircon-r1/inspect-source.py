from pathlib import Path
import bpy
from mathutils import Vector
ROOT=Path.cwd();OUT=ROOT/'output/assets/public-aircon-mount-r1'
bpy.ops.wm.open_mainfile(filepath=str(ROOT/'source-assets/environment-kit/aircon/aircon-candidate.blend'))
scene=bpy.context.scene
scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=12
scene.render.resolution_x=640;scene.render.resolution_y=640;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG';scene.render.threads_mode='FIXED';scene.render.threads=2
scene.world=bpy.data.worlds.new('Neutral');scene.world.use_nodes=True
scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value=.7
for pos,power in [((1,-2,3),300),((-1,2,3),150)]:
 bpy.ops.object.light_add(type='AREA',location=pos);bpy.context.object.data.energy=power;bpy.context.object.data.size=2
bpy.ops.object.camera_add();cam=bpy.context.object;cam.data.type='ORTHO';scene.camera=cam
for name,pos,target,scale in [('valves',(1.4,-.6,.66),(.36,.02,.6),.55),('back',(.6,1.3,.62),(.23,.11,.57),.63),('drain',(.45,-.8,.10),(.058,.017,.06),.21)]:
 cam.location=pos;cam.rotation_euler=(Vector(target)-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.ortho_scale=scale
 scene.render.filepath=str(OUT/('source-'+name+'.png'));bpy.ops.render.render(write_still=True)
