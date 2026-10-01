"""Check actual GLB installation geometry and preservation of the selected CC0 unit."""
import hashlib
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT.parents[2] / 'tools'))
from glb import read_glb, require, values

source, source_binary = read_glb(ROOT / 'aircon-candidate.glb')
path = ROOT / 'wall-installation.glb'
doc, binary = read_glb(path)
require(len(doc['nodes']) == len(doc['meshes']) == 1 and not any(k in doc['nodes'][0] for k in ('matrix','translation','rotation','scale')), 'expected one static identity root')
require(not doc.get('skins') and not doc.get('animations') and not doc.get('extensionsRequired'), 'expected uncompressed static export')
materials = {m['name']: i for i,m in enumerate(doc['materials'])}
require(set(materials) == {'exterior_aircon_unit_01','exterior_aircon_unit_02','NSIDE_AC_Metal','NSIDE_AC_PVC','NSIDE_AC_Insulation'}, 'unexpected material set')
all_positions, counts, shading_delta = [], {}, {}
for primitive in doc['meshes'][0]['primitives']:
    attrs = {name: values(doc,binary,doc['accessors'][a]) for name,a in primitive['attributes'].items()}
    require({'POSITION','NORMAL','TEXCOORD_0'} <= attrs.keys(), 'required attributes missing')
    require(all(math.isfinite(x) for rows in attrs.values() for row in rows for x in row), 'nonfinite vertex data')
    require(all(abs(sum(v*v for v in row)-1) < 1e-4 for row in attrs['NORMAL']), 'nonunit normals')
    ps = attrs['POSITION']
    ids = [v[0] for v in values(doc,binary,doc['accessors'][primitive['indices']])]
    require(primitive.get('mode',4)==4 and len(ids)%3==0 and all(0<=i<len(ps) for i in ids), 'invalid triangle indices')
    for a,b,c in zip(ids[::3],ids[1::3],ids[2::3]):
        u=[ps[b][i]-ps[a][i] for i in range(3)];v=[ps[c][i]-ps[a][i] for i in range(3)]
        cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        require(sum(x*x for x in cross) > 4e-24, 'degenerate triangle')
    all_positions.extend(ps)
    name = doc['materials'][primitive['material']]['name']
    counts[name] = len(ids)//3
    if name.startswith('exterior_aircon_unit'):
        src_i=next(i for i,m in enumerate(source['materials']) if m['name']==name)
        src=next(p for p in source['meshes'][0]['primitives'] if p['material']==src_i)
        src_ids=[v[0] for v in values(source,source_binary,source['accessors'][src['indices']])]
        require(len(ids)==len(src_ids), f'{name}: upstream triangles changed')
        for key in ('POSITION','TEXCOORD_0'):
            before=values(source,source_binary,source['accessors'][src['attributes'][key]])
            require(all(abs(attrs[key][a][axis]-before[b][axis]-(2.503333 if key=='POSITION' and axis==1 else 0)) < 1e-6
                for a,b in zip(ids,src_ids) for axis in range(len(before[0]))), f'{name}: geometry, normals or UV changed')
        before=values(source,source_binary,source['accessors'][src['attributes']['NORMAL']])
        angles=[]
        for a,b in zip(ids,src_ids):
            u,v=attrs['NORMAL'][a],before[b]
            cosine=sum(x*y for x,y in zip(u,v))/math.sqrt(sum(x*x for x in u)*sum(y*y for y in v))
            angles.append(math.degrees(math.acos(min(1,max(-1,cosine)))))
        shading_delta[name]=max(angles)
        require(max(angles)<.1, f'{name}: normal direction changed beyond export rounding')
        require(doc['materials'][primitive['material']]==source['materials'][src_i], f'{name}: material changed')

def image_hashes(document,blob):
    hashes=[]
    for img in document['images']:
        require('uri' not in img, 'external runtime image')
        view=document['bufferViews'][img['bufferView']];offset=view.get('byteOffset',0)
        hashes.append(hashlib.sha256(blob[offset:offset+view['byteLength']]).hexdigest())
    return hashes

require(image_hashes(source,source_binary)==image_hashes(doc,binary), 'reviewed textures changed')
low=[min(p[i] for p in all_positions) for i in range(3)]
high=[max(p[i] for p in all_positions) for i in range(3)]
require(abs(low[1])<1e-6 and high[1]<3.44, 'grade pivot or upper floor clearance changed')
require(low[0]>-.401 and high[0]<.401 and low[2]>-.236 and high[2]<.188, 'unexpected installation envelope')
require(sum(counts.values()) < 11000, 'installation exceeds the local single-unit allowance')
print(json.dumps({'status':'PASS','file':str(path.relative_to(ROOT.parents[2])),
    'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
    'triangles_by_material':counts,'triangles':sum(counts.values()),'materials':len(materials),
    'embedded_images':len(doc['images']),'bounds_gltf':[low,high],
    'selected_unit_geometry_uv_materials_images':'preserved except +2.503333m height',
    'reexport_normal_max_angle_degrees':shading_delta,
    'finite_attributes_valid_indices_unit_normals_zero_degenerate_triangles':'PASS',
    'runtime':'NOT RUN'},indent=2))
