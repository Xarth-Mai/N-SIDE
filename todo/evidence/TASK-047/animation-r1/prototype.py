"""Evaluate the build.py gait on the existing master in memory; never save over it."""
from pathlib import Path
import ast
import argparse
import sys
import bpy
import json
import math
from mathutils import Quaternion,Vector

ROOT=Path(__file__).resolve().parents[4]
SOURCE=ROOT/'source-assets/characters/CHR-001/model'
parser=argparse.ArgumentParser()
parser.add_argument('--render',action='store_true',help='render one full side-view cycle at 30 FPS using CPU')
args=parser.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])
OUTPUT=ROOT/'output/characters/CHR-001/animation-r1/linear'
OUTPUT.mkdir(parents=True,exist_ok=True)
RUNTIME=OUTPUT
bpy.ops.wm.open_mainfile(filepath=str(SOURCE/'yao-grey-study.blend'))
rig=bpy.data.objects['CHR001_Rig'];scene=bpy.context.scene
rig.animation_data_clear()
for a in list(bpy.data.actions):bpy.data.actions.remove(a)
script=(SOURCE/'build.py').read_text()
section=script[script.index('# In-place motion:'):script.index('# Ground/cameras/lights')]
exec(section)
report={'scope':'prototype master evaluated in memory; no production asset overwritten','clips':[]}
for name,(period,stride) in CLIPS.items():
    rig.animation_data.action=bpy.data.actions[name]
    samples=[];matrices=[];min_sole=100.;max_contact_height=0.;max_contact_error=0.
    for step in range(period*2+1):
        frame=1+step/2;scene.frame_set(int(frame),subframe=frame%1)
        graph=bpy.context.evaluated_depsgraph_get()
        row={'time':step/(2*FPS),'feet':{}}
        if step in (0,period*2):matrices.append([x for b in rig.pose.bones for m in b.matrix for x in m])
        for side,offset in (('L',0),('R',.5)):
            p=((step/2)/period+offset)%1
            ankle=rig.pose.bones['Foot.'+side].matrix.translation
            shoe=bpy.data.objects['Sneaker_Sole.'+side].evaluated_get(graph);mesh=shoe.to_mesh()
            floor=min((shoe.matrix_world@v.co).z for v in mesh.vertices);shoe.to_mesh_clear()
            min_sole=min(min_sole,floor)
            stance=name=='Idle' or p*period<=GAITS[name][1]+1e-7
            row['feet'][side]={'ankle':list(ankle),'sole_height':floor,'stance':stance}
            if stance:
                max_contact_height=max(max_contact_height,abs(floor))
                if name!='Idle':
                    y,z=foot_target(name,p)
                    max_contact_error=max(max_contact_error,abs(ankle.y-y),abs(ankle.z-z))
        samples.append(row)
    loop_error=max(abs(a-b) for a,b in zip(*matrices))
    print(name,'floor',min_sole,'contact',max_contact_height,'target-error',max_contact_error,'loop',loop_error,flush=True)
    assert min_sole>-.005,(name,'floor',min_sole)
    assert max_contact_height<.005,(name,'contact',max_contact_height)
    assert loop_error<1e-5,(name,'loop',loop_error)
    report['clips'].append({'name':name,'duration':period/FPS,'sample_fps':2*FPS,'minimum_sole':min_sole,'max_contact_height':max_contact_height,'max_ankle_target_error':max_contact_error,'loop_matrix_delta':loop_error,'samples':samples})
(OUTPUT/'prototype-check.json').write_text(json.dumps(report,indent=2)+'\n')
rig.animation_data.action=bpy.data.actions['Idle'];scene.frame_set(1)
export=next(n for n in ast.parse(script).body if isinstance(n,ast.FunctionDef) and n.name=='export')
exec(compile(ast.Module(body=[export],type_ignores=[]),'build.py export','exec'))
export(rig,[o for o in rig.children if o.type=='MESH'])
if not args.render:
    print('PROTOTYPE PASS',flush=True)
    raise SystemExit(0)
cam=bpy.data.objects['Review_Camera'];scene.camera=cam
scene.render.resolution_x=384;scene.render.resolution_y=512;scene.render.resolution_percentage=100
scene.render.engine='CYCLES';scene.cycles.device='CPU';scene.cycles.samples=12
scene.render.threads_mode='FIXED';scene.render.threads=4
for name in ('Walk','Run'):
    rig.animation_data.action=bpy.data.actions[name]
    directory=OUTPUT/name.lower();directory.mkdir(exist_ok=True)
    cam.location=(-4,0,1.0);cam.rotation_euler=(Vector((0,0,.86))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=2.0
    for index,frame in enumerate(range(1,CLIPS[name][0]+1,2)):
        scene.frame_set(frame)
        scene.render.filepath=str(directory/f'{index:03}.png');bpy.ops.render.render(write_still=True)
print('PROTOTYPE PASS',flush=True)
