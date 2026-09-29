"""Read actual uncompressed DDS mips; report directional color/normal variation, not visual acceptance."""
from pathlib import Path
import hashlib
import json
import struct
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
def srgb(value):
    return np.where(value <= .04045, value / 12.92, ((value + .055) / 1.055) ** 2.4)

def profiles(value):
    return {'std':float(value.std()), 'row_mean_std':float(value.mean(axis=1).std()),
            'column_mean_std':float(value.mean(axis=0).std()),
            'min':float(value.min()), 'max':float(value.max())}

out = {'scope':'CPU measurements of actual source/DDS; no GPU or image-quality claim', 'files':{}}
for name, role in [('Ground037_1K-JPG_Color.jpg','ground-color'), ('Ground037_1K-JPG_NormalGL.jpg','ground-normal')]:
    source = ROOT / 'source-assets/environment-kit/materials' / name
    path = ROOT / 'game/assets/environment/materials' / (role+'.dds')
    data = path.read_bytes()
    assert data[:4] == b'DDS ' and data[84:88] == bytes(4)
    height,width = struct.unpack_from('<II',data,12)
    count, = struct.unpack_from('<I',data,28)
    assert struct.unpack_from('<5I',data,88) == (32,0xff0000,0xff00,0xff,0xff000000)
    offset=128
    levels=[]
    for level in range(count):
        w,h=max(1,width>>level),max(1,height>>level)
        rgb=np.frombuffer(data[offset:offset+w*h*4],dtype=np.uint8).reshape(h,w,4)[:,:, [2,1,0]]
        offset += w*h*4
        if level==0:
            assert np.array_equal(rgb,np.asarray(Image.open(source).convert('RGB'))), 'mip0 must equal the decoded JPG source'
        values=rgb.astype(np.float64)/255
        measurements={'level':level,'size':[w,h]}
        if role.endswith('color'):
            measurements['linear_luminance']=profiles(srgb(values) @ np.array([.2126,.7152,.0722]))
            measurements['green_minus_red']=profiles(values[:,:,1]-values[:,:,0])
        else:
            nt=values*2-1
            measurements['tangent_x']=profiles(nt[:,:,0]); measurements['tangent_y']=profiles(nt[:,:,1])
            measurements['mean_length']=float(np.linalg.norm(nt,axis=2).mean())
        levels.append(measurements)
    assert offset==len(data)
    out['files'][role]={'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(), 'dds_sha256':hashlib.sha256(data).hexdigest(),'mip0_matches_source':True,'levels':levels}
path=ROOT/'todo/evidence/TASK-045/terrain-surface-r5/texture-analysis.json'
path.write_text(json.dumps(out,indent=2)+'\n')
for role, item in out['files'].items():
    print(role)
    for level in item['levels'][::2]:
        measurement=level.get('linear_luminance',level.get('tangent_x'))
        print(level['level'],level['size'],{k:round(v,5) for k,v in measurement.items()})
