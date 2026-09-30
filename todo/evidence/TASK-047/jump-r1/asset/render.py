"""Disposable CPU pose review; no world-space jump trajectory is added"""
from pathlib import Path
import bpy,json
from mathutils import Vector
root=Path(__file__).resolve().parents[5];out=root/'output/jump-r1';records=[]
for cid,stem in [('CHR-001','yao'),('CHR-002','ling')]:
 bpy.ops.wm.open_mainfile(filepath=str(root/'source-assets/characters'/cid/'model'/(stem+'-grey-study.blend')))
 scene=bpy.context.scene;rig=bpy.data.objects['CHR'+cid[-3:]+'_Rig'];camera=scene.camera
 scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=12
 scene.render.resolution_x=416;scene.render.resolution_y=560;scene.render.resolution_percentage=100
 camera.location=(3,-4,1.3);camera.rotation_euler=(Vector((0,0,.89))-camera.location).to_track_quat('-Z','Y').to_euler();camera.data.ortho_scale=2.05
 rig.animation_data.action=bpy.data.actions['Jump']
 for frame in [1,6,21,37]:
  scene.frame_set(frame);scene.render.filepath=str(out/f'{cid}-jump-{frame}.png');bpy.ops.render.render(write_still=True)
  records.append({'character':cid,'frame':frame,'image':scene.render.filepath})
(root/'todo/evidence/TASK-047/jump-r1/asset/render.json').write_text(json.dumps({'engine':'Cycles CPU','samples':12,'resolution':[416,560],'views':records},indent=2)+'\n')
