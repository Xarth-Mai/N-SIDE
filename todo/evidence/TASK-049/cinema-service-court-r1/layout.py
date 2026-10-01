"""Draw the measured candidate plan, not a game screenshot."""
import hashlib,json
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[4];HERE=Path(__file__).resolve().parent
c=json.loads((HERE/'candidate.json').read_text());r=json.loads((HERE/'check.json').read_text());p=json.loads((HERE/'props-report.json').read_text())
data=json.loads((ROOT/'source-assets/district-map/district.json').read_text())
out=ROOT/'output/cinema-service-court-r1';out.mkdir(parents=True,exist_ok=True)
im=Image.new('RGB',(1000,800),'#f4f3ed');draw=ImageDraw.Draw(im)
font_path=ROOT/'source-assets/ui-kit/fonts/NotoSansSC-VF.ttf'
font=ImageFont.truetype(str(font_path),19);small=ImageFont.truetype(str(font_path),15)
# Diagram extent is deliberately local; north is up
xy=lambda p:(60+(p[0]-289)*18,660-(p[1]-237)*18)
polygon=lambda ps,fill,outline=None:draw.polygon([xy(p) for p in ps],fill=fill,outline=outline,width=2)
for s in data['surfaces']:
 if s.get('id') in ['cinema-arrival-court','cinema-west-garden-north']:
  polygon(s['polygon'],'#ccd5b1' if s['kind']=='park' else '#ddd5c4','#ada38e')
polygon([[300,237],[340,237],[340,250],[300,250]],'#9b9c9a','#656866')
polygon(c['surface']['polygon'],'#e6c792','#a37c3f')
for road in r['overlapping_roads']:
 polygon(road['polygon'],'#c5c8c5','#6f7670')
# The existing west/north service link supplies context outside the new patch
road=data['roads'][236]
for a,b in zip(road['nodes'],road['nodes'][1:]):draw.line([xy(data['nodes'][a]),xy(data['nodes'][b])],fill='#68767b',width=4)
for a in c['reserved_areas']:
 ps=[xy(q) for q in a['polygon']];draw.line(ps+[ps[0]],fill='#54885d' if a['name'].startswith('unloading') else '#3275a3',width=3)
for prop in p['props']:
 (x0,x1),(y0,y1),_=prop['map_bounds_xyz'];polygon([[x0,y0],[x1,y0],[x1,y1],[x0,y1]],'#874c8f','#55365b')
for name,label in [('cinema_loading','卸货节点'),('cinema_service_entry','原后台门')]:
 q=xy(data['nodes'][name]);draw.ellipse((q[0]-4,q[1]-4,q[0]+4,q[1]+4),fill='#25384a');draw.text((q[0]+9,q[1]-8),label,font=small,fill='#25384a')
draw.text((55,28),'镜厅北后勤前场候选 · 地图几何示意',font=font,fill='#202624')
draw.text((55,60),'25 m ｜ 198 m² ｜ 原服务路与门位不改 ｜ 未进入游戏',font=small,fill='#4b5350')
draw.text(xy([302,245]),'V-15 镜厅 · 北墙',font=font,fill='#fffdf5')
draw.text(xy([306,260]),'原 3 m 后台来路',font=small,fill='#34494e')
draw.text(xy([326.7,260.5]),'3.6 m 到门净空',font=small,fill='#3275a3')
draw.text(xy([326.7,263.2]),'8 × 8.5 m 卸货净空',font=small,fill='#54885d')
draw.text(xy([312,263]),'路灯',font=small,fill='#55365b')
draw.text(xy([325.2,252]),'墙挂空调与收水口',font=small,fill='#55365b')
draw.text((55,713),'已有资产：Monsta3D / Poly Haven 空调装配；Kenney City Kit 路灯',font=small,fill='#4b5350')
draw.text((55,742),'紫色为实际 GLB 投影包络；蓝/绿为保持空出的活动区域，不是新道路或停车位',font=small,fill='#4b5350')
path=out/'layout.png';im.save(path)
(out/'layout-metadata.json').write_text(json.dumps({'path':str(path.relative_to(ROOT)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':path.stat().st_size,'pixels':list(im.size),'classification':'source plan diagram, not runtime render'},ensure_ascii=False,indent=2)+'\n')
