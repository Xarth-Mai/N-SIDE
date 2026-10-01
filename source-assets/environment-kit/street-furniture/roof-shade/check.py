"""Inspect actual shade triangles, copied runtime, textures and roof placement."""
import hashlib
import json
import math
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'tools'))
from glb import read_glb, values

sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
sub = lambda a,b: tuple(x-y for x,y in zip(a,b))
dot = lambda a,b: sum(x*y for x,y in zip(a,b))
length = lambda a: math.sqrt(dot(a,a))
cross = lambda a,b: (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
bounds = lambda points: ([min(p[i] for p in points) for i in range(3)], [max(p[i] for p in points) for i in range(3)])


def overlap(a,b):
    return all(a[0][i] < b[1][i]-1e-6 and a[1][i] > b[0][i]+1e-6 for i in range(3))


def point_segment(p,a,b):
    d=sub(b,a)
    t=max(0,min(1,dot(sub(p,a),d)/dot(d,d))) if dot(d,d) else 0
    return length(sub(p,tuple(a[i]+t*d[i] for i in range(2))))


def rectangle_segment(lo,hi,a,b):
    # A slab intersection detects a road crossing the rectangle without an endpoint inside
    enter,leave=0,1
    for i in range(2):
        d=b[i]-a[i]
        if abs(d)<1e-9:
            if not lo[i]<=a[i]<=hi[i]: break
        else:
            x,y=sorted(((lo[i]-a[i])/d,(hi[i]-a[i])/d))
            enter,leave=max(enter,x),min(leave,y)
            if enter>leave: break
    else:
        return 0
    corners=[lo,(hi[0],lo[1]),hi,(lo[0],hi[1])]
    return min([point_segment(p,a,b) for p in corners]+[
        point_segment(p,corners[i],corners[(i+1)%4]) for p in (a,b) for i in range(4)])


def mesh_triangles(path):
    doc,binary=read_glb(path)
    triangles=[]
    for mesh in doc['meshes']:
        for primitive in mesh['primitives']:
            read=lambda index:values(doc,binary,doc['accessors'][index])
            positions=read(primitive['attributes']['POSITION'])
            indices=[v[0] for v in read(primitive['indices'])]
            triangles.extend([positions[i] for i in indices[start:start+3]] for start in range(0,len(indices),3))
    return doc,binary,triangles


def main():
    path=HERE/'roof-shade.glb'
    doc,binary,triangles=mesh_triangles(path)
    assert doc.get('scene',0)==0 and len(doc['scenes'])==len(doc['nodes'])==len(doc['meshes'])==1
    assert set(doc['nodes'][0]) <= {'mesh','name'}
    assert not any(doc.get(k) for k in ('skins','animations','cameras','extensionsUsed'))
    assert len(doc['materials'])==2 and len(doc['images'])==2 and len(triangles)<=1800
    assert all(m.get('alphaMode','OPAQUE')=='OPAQUE' for m in doc['materials'])
    vertex_count=0
    for primitive in doc['meshes'][0]['primitives']:
        attrs=primitive['attributes'];assert {'POSITION','NORMAL','TEXCOORD_0','TANGENT'}<=attrs.keys()
        read=lambda name:values(doc,binary,doc['accessors'][attrs[name]])
        ps,ns,uv,ts=(read(k) for k in ('POSITION','NORMAL','TEXCOORD_0','TANGENT'))
        assert len(ps)==len(ns)==len(uv)==len(ts)
        assert all(math.isfinite(v) for rows in (ps,ns,uv,ts) for row in rows for v in row)
        assert all(abs(length(n)-1)<1e-4 for n in ns)
        assert all(abs(length(t[:3])-1)<1e-4 and abs(dot(n,t[:3]))<1e-4 for n,t in zip(ns,ts))
        ids=[v[0] for v in values(doc,binary,doc['accessors'][primitive['indices']])]
        assert all(0<=i<len(ps) for i in ids) and len(ids)%3==0
        for start in range(0,len(ids),3):
            a,b,c=ids[start:start+3];normal=cross(sub(ps[b],ps[a]),sub(ps[c],ps[a]));size=length(normal)
            assert size>1e-9
            assert all(dot(normal,ns[i])/size>.99 for i in (a,b,c))
        vertex_count+=len(ps)
    roles={m['name']:m for m in doc['materials']};wood=roles['RoofShade_Wood']
    assert wood['normalTexture'].get('scale',1)==1
    embedded={}
    for slot,filename in ((wood['normalTexture'],'WoodSiding009-NormalGL-scale035.png'),
                          (wood['pbrMetallicRoughness']['baseColorTexture'],'WoodSiding009_1K-JPG_Color.jpg')):
        image=doc['images'][doc['textures'][slot['index']]['source']];view=doc['bufferViews'][image['bufferView']]
        start=view.get('byteOffset',0);data=binary[start:start+view['byteLength']]
        source=ROOT/'source-assets/environment-kit/materials'/filename
        assert data==source.read_bytes();embedded[filename]=sha(source)
    registry=json.loads((ROOT/'source-assets/environment-kit/asset-manifest.json').read_text())
    registered=[v for v in registry['files'] if v['source']=='street-furniture/roof-shade/roof-shade.glb']
    assert len(registered)==1 and registered[0]['sha256']==sha(path)
    runtime_path=ROOT/'game/assets/environment'/registered[0]['output']
    assert runtime_path.read_bytes()==path.read_bytes(), 'runtime GLB differs from source export'
    appearance=json.loads((ROOT/'source-assets/district-scene/appearance.json').read_text())
    assert appearance['models']['roof_shade']=={'file':'environment/street-furniture/roof-shade.glb','scene':0,'scale':1}
    color=next(v for v in registry['files'] if v['source']=='materials/WoodSiding009_1K-JPG_Color.jpg')
    normal=next(v for v in registry['derived_normals']['files'] if v['output'].endswith('WoodSiding009-NormalGL-scale035.png'))
    assert color['license']==normal['license']=='CC0-1.0'
    assert color['sha256']==embedded['WoodSiding009_1K-JPG_Color.jpg']
    assert normal['output_sha256']==embedded['WoodSiding009-NormalGL-scale035.png']
    assert sha(ROOT/normal['source'])==normal['source_sha256']
    # glTF X,Y,Z -> local source X,north,height; anchor chosen to reuse the existing bench
    local=[[(p[0],-p[2],p[1]) for p in tri] for tri in triangles]
    local_bounds=bounds([p for tri in local for p in tri])
    expected=((-2.02,-1.10,0),(2.02,1.46,2.52))
    assert all(abs(a-b)<1e-5 for row,target in zip(local_bounds,expected) for a,b in zip(row,target))
    bench_path=ROOT/'game/assets/environment/street-furniture/street-bench.glb'
    _,_,bench=mesh_triangles(bench_path)
    bench_bounds=bounds([(p[0],-p[2],p[1]) for tri in bench for p in tri])
    tri_bounds=[bounds(tri) for tri in local]
    assert not any(overlap(b,bench_bounds) for b in tri_bounds), 'shade triangle overlaps the actual bench envelope'
    clear={'front_seat_approach':((-1.55,-2.1,0),(1.55,-.39,1.85)),
           'west_seat_side':((-1.65,-.75,0),(-.99,.70,1.85)),
           'east_seat_side':((.99,-.75,0),(1.65,.70,1.85))}
    for name,volume in clear.items():
        assert not any(overlap(b,volume) for b in tri_bounds),name
    map_data=json.loads((ROOT/'source-assets/district-map/district.json').read_text())
    anchor=(312,237,37)
    world=[tuple(v+anchor[i] for i,v in enumerate(p)) for tri in local for p in tri]
    world_bounds=bounds(world)
    assert all(309.8<=p[0]<=314.2 and 234.8<=p[1]<=239.5 and p[2]>=37-1e-6 for p in world)
    reserve=next(s for s in map_data['surfaces'] if s.get('id')=='cinema-roof-planting-reserve')
    assert reserve['kind']=='park' and reserve['building']=='V-15' and reserve['elevated']
    assert reserve['elevation']==anchor[2]
    reserve_bounds=([min(p[i] for p in reserve['polygon']) for i in range(2)],
                    [max(p[i] for p in reserve['polygon']) for i in range(2)])
    reserve_gap=length([max(reserve_bounds[0][i]-world_bounds[1][i],world_bounds[0][i]-reserve_bounds[1][i],0) for i in range(2)])
    assert reserve_gap>.32, 'shade envelope intrudes into reserved planting ground'
    roads=[]
    for index in (168,169,170,238,240,770):
        road=map_data['roads'][index];points=[map_data['nodes'][n][:2] for n in road['nodes']]
        margin=min(rectangle_segment(world_bounds[0][:2],world_bounds[1][:2],a,b) for a,b in zip(points,points[1:]))-road['width']/2
        assert margin>.32,(index,margin)
        roads.append({'source':f'/roads/{index}','nodes':road['nodes'],'width':road['width'],'plan_edge_margin_m':margin})
    print(json.dumps({'status':'PASS','source_sha256':sha(HERE/'roof-shade.blend'),'glb_sha256':sha(path),
                      'runtime_sha256':sha(runtime_path),'runtime_copy_and_model_binding':'PASS',
                      'bytes':path.stat().st_size,'triangles':len(triangles),'vertices':vertex_count,'materials':2,'images':2,
                      'bounds_gltf':bounds([p for tri in triangles for p in tri]),'bounds_map':world_bounds,
                      'anchor_map':anchor,'source_texture_hashes':embedded,'normal_scale':1,
                      'bench_sha256':sha(bench_path),'bench_local_bounds':bench_bounds,'bench_triangle_aabb_separation':'PASS',
                      'clear_approach_volumes_local':clear,'rf_road_margins':roads,
                      'map_source_sha256':sha(ROOT/'source-assets/district-map/district.json'),
                      'planting_reserve':{'source_id':reserve['id'],'polygon':reserve['polygon'],'elevation':reserve['elevation'],
                                          'plan_envelope_margin_m':reserve_gap,'status':'PASS'},
                      'quest_boundary':'R6 physical planting reserve is read from the map; no QST-025 planting geometry or story-state changes in this asset',
                      'visual_review':'NOT RUN; source and runtime screenshot acceptance pending'},indent=2))


if __name__=='__main__':main()
