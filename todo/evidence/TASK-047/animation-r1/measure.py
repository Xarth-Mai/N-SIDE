"""Read exported GLB keyframes and report planted-foot travel, without changing assets."""
import copy
import bisect
import argparse
import hashlib
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'tools'))
from glb import read_glb, values
from validate_character import transformed

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--asset',type=Path,default=ROOT/'game/assets/characters/CHR-001/yao-grey-study.glb')
parser.add_argument('--output',type=Path,default=Path(__file__).with_name('exported-foot-tracks.json'))
parser.add_argument('--check',action='store_true',help='fail if locomotion stance feet travel forward instead of backward')
parser.add_argument('--fps',type=int,default=30)
parser.add_argument('--gait',action='store_true',help='use authored Walk 1/3 and Run 1/5 stance phases rather than ankle height heuristic')
args=parser.parse_args()
path = args.asset
doc, binary = read_glb(path)
parents = {child: i for i, node in enumerate(doc['nodes']) for child in node.get('children', [])}
feet = {n['name']: i for i, n in enumerate(doc['nodes']) if n.get('name') in ('Foot.L', 'Foot.R', 'Toe.L', 'Toe.R')}
report = {'asset_sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'forward': '+Z', 'up': '+Y', 'units': 'meters', 'sampling': f'actual exported glTF STEP/LINEAR channels, quaternion shortest-arc interpolation; sampled {args.fps}fps', 'stance':'authored phase' if args.gait else 'ankle height heuristic', 'clips': []}
for animation in doc['animations']:
    samplers = [(s.get('interpolation', 'LINEAR'), [x[0] for x in values(doc,binary,doc['accessors'][s['input']])], values(doc,binary,doc['accessors'][s['output']])) for s in animation['samplers']]
    duration = max(s[1][-1] for s in samplers)
    frames = round(duration * args.fps)
    samples=[]
    for frame in range(frames+1):
        t=frame/args.fps
        nodes=copy.deepcopy(doc['nodes'])
        for channel in animation['channels']:
            mode, times, data = samplers[channel['sampler']]
            assert mode in ('LINEAR','STEP'), mode
            index=max(0,bisect.bisect_right(times,t+1e-7)-1)
            value=data[index]
            if mode=='LINEAR' and index+1<len(times) and abs(times[index]-t)>1e-7:
                a,b=data[index],data[index+1]
                u=(t-times[index])/(times[index+1]-times[index])
                if channel['target']['path']=='rotation':
                    dot=sum(x*y for x,y in zip(a,b))
                    if dot<0: b=tuple(-x for x in b);dot=-dot
                    if dot<.9995:
                        angle=math.acos(min(1.,dot))
                        wa=math.sin((1-u)*angle)/math.sin(angle)
                        wb=math.sin(u*angle)/math.sin(angle)
                        value=tuple(x*wa+y*wb for x,y in zip(a,b))
                    else:
                        value=tuple(x*(1-u)+y*u for x,y in zip(a,b))
                    length=math.sqrt(sum(x*x for x in value))
                    value=tuple(x/length for x in value)
                else:
                    value=tuple(x*(1-u)+y*u for x,y in zip(a,b))
            nodes[channel['target']['node']][channel['target']['path']]=value
        points={}
        for name,index in feet.items():
            point=(0.,0.,0.)
            while True:
                point=transformed(point,nodes[index])
                if index not in parents: break
                index=parents[index]
            points[name]=point
        samples.append({'time':t,'feet':points})
    contact=[]
    speed={'Idle':0.,'Walk':3.2,'Run':5.6}[animation['name']]
    for side in ('L','R'):
        key='Foot.'+side
        floor=min(s['feet'][key][1] for s in samples)
        intervals=[]
        for a,b in zip(samples,samples[1:]):
            if args.gait and animation['name']!='Idle':
                duty={'Walk':1/3,'Run':.2}[animation['name']]
                offset=0 if side=='L' else .5
                stance=all((row['time']/duration+offset+1e-7)%1<=duty+1e-6 for row in (a,b))
            else:
                stance=max(a['feet'][key][1],b['feet'][key][1])<=floor+.015
            if stance:
                v=(b['feet'][key][2]-a['feet'][key][2])/(b['time']-a['time'])
                intervals.append({'from':a['time'],'to':b['time'],'foot_forward_velocity':v,'world_sliding_velocity_at_current_playback':v+speed})
        velocities=[i['foot_forward_velocity'] for i in intervals]
        drift=0.;max_drift=0.;last_end=None
        for interval in intervals:
            if last_end is None or abs(interval['from']-last_end)>1e-7:drift=0.
            drift+=interval['world_sliding_velocity_at_current_playback']*(interval['to']-interval['from'])
            max_drift=max(max_drift,abs(drift));last_end=interval['to']
        contact.append({'side':side,'minimum_ankle_height':floor,'horizontal_range':[min(s['feet'][key][2] for s in samples),max(s['feet'][key][2] for s in samples)],'near_ground_threshold':None if args.gait else .015,'max_stance_position_drift_m':max_drift,'near_ground_intervals':intervals,'mean_forward_velocity':sum(velocities)/len(velocities) if velocities else None,'max_world_sliding_velocity':max(abs(v+speed) for v in velocities) if velocities else None,'rms_world_sliding_velocity':math.sqrt(sum((v+speed)**2 for v in velocities)/len(velocities)) if velocities else None})
    direction_ok=animation['name']=='Idle' or all(f['mean_forward_velocity'] is not None and f['mean_forward_velocity'] < -.01 for f in contact)
    report['clips'].append({'name':animation['name'],'duration':duration,'current_body_speed':speed,'stance_direction_valid':direction_ok,'foot_contacts':contact,'samples':samples})
report['stance_direction_valid']=all(c['stance_direction_valid'] for c in report['clips'])
if args.gait:
    report['speed_and_drift_valid']=all(abs(f['mean_forward_velocity']+c['current_body_speed'])<.01 and f['max_stance_position_drift_m']<.001 for c in report['clips'] for f in c['foot_contacts'])
out=args.output
out.write_text(json.dumps(report,indent=2)+'\n')
for c in report['clips']:
 print(c['name'],c['duration'],[(f['side'],f['horizontal_range'],f['mean_forward_velocity']) for f in c['foot_contacts']])
if args.check and not report['stance_direction_valid']:
    print('FAIL: planted feet move forward with the body; positive playback scaling cannot cancel sliding',file=sys.stderr)
    sys.exit(1)

if args.check and args.gait and not report['speed_and_drift_valid']:
    print('FAIL: authored stance must match controller speed within .01 m/s and drift less than 1 mm',file=sys.stderr)
    sys.exit(1)
