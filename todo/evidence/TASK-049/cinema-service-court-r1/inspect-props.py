"""Read actual existing GLBs; apply their node and proposed prop transforms without editing assets."""
import hashlib
import json
import math
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'tools'))
from glb import read_glb, require, values
HERE=Path(__file__).resolve().parent
proposal=json.loads((HERE/'candidate.json').read_text())
appearance=json.loads((ROOT/'source-assets/district-scene/appearance.json').read_text())
assets=json.loads((ROOT/'source-assets/environment-kit/asset-manifest.json').read_text())
runtime=json.loads((ROOT/'game/assets/environment/manifest.json').read_text())
results=[]
for prop in proposal['props']:
    model=appearance['models'][prop['model']]; path=ROOT/'game/assets'/model['file']
    doc,binary=read_glb(path);digest=hashlib.sha256(path.read_bytes()).hexdigest()
    record=next(r for r in assets['files'] if r['output']==model['file'].removeprefix('environment/'))
    installed=next(r for r in runtime['files'] if r['file']==record['output'])
    require(installed['sha256']==digest,'runtime differs from recorded environment export')
    require(len(doc['nodes'])==1 and len(doc['meshes'])==1 and doc['scenes'][doc.get('scene',0)]['nodes']==[0],'expected known single-mesh prop')
    node=doc['nodes'][0]
    require(set(node)<={'mesh','name','translation','scale'},'unexpected node transform; do not silently ignore rotation')
    scale=node.get('scale',[1,1,1]);translation=node.get('translation',[0,0,0]);points=[];count=0
    yaw=prop['world_y_rotation_radians']; co,si=math.cos(yaw),math.sin(yaw);anchor=prop['map_position']
    for primitive in doc['meshes'][0]['primitives']:
        require(primitive.get('mode',4)==4,'expected triangles')
        count+=doc['accessors'][primitive['indices']]['count']//3
        for row in values(doc,binary,doc['accessors'][primitive['attributes']['POSITION']]):
            p=[(row[k]*scale[k]+translation[k])*model['scale']*prop['scale'] for k in range(3)]
            require(all(math.isfinite(v) for v in p),'nonfinite transformed mesh')
            world_x,world_z=co*p[0]+si*p[2],-si*p[0]+co*p[2]
            points.append([anchor[0]+world_x,anchor[1]-world_z,anchor[2]+p[1]])
    bounds=[[min(p[k] for p in points),max(p[k] for p in points)] for k in range(3)]
    feet=[p for p in points if p[2]<bounds[2][0]+.001]
    result={**prop,'runtime_file':str(path.relative_to(ROOT)),'runtime_sha256':digest,'source_license':record['license'],'component_licenses':record.get('component_licenses'),'triangles':count,'map_bounds_xyz':bounds,'foot_points':feet,'materials':len(doc['materials'])}
    if prop['model']=='aircon_wall':
        require(abs(anchor[1]-.203-250)<1e-9,'actual rear packer/receiver plane must meet source north facade')
        require(bounds[0][0]>322+.725+.08 and bounds[0][1]<326-.725-.08,'unit/pipe may overlap north 1F window bays')
        require(bounds[2][1]<29-.05,'unit reaches 2F floor band')
        result['wall_contact']={'source_north':250,'frame_half_width_m':.805,'adjacent_window_x':[322,326],'margin_each_side_m':min(bounds[0][0]-322-.805,326-.805-bounds[0][1])}
    results.append(result)
report={'status':'PASS','props':results,'additional_instances':len(results),'new_model_files':0,'total_instance_triangles':sum(p['triangles'] for p in results),'scope':'Actual model/node/prop transformed vertices; appearance and source-license references. Bounds are conservative, not a Rust collision or runtime proof.'}
print(json.dumps(report,ensure_ascii=False,indent=2))
