"""Verify the complete published Wiki, not just the newly added story feature."""
from concurrent.futures import ThreadPoolExecutor
from html.parser import HTMLParser
from pathlib import Path
from urllib.request import urlopen
import json,sys
root=Path(__file__).resolve().parents[4]
origin=sys.argv[1].rstrip('/')
manifest=json.loads((root/'docs/.vitepress/cache/wiki/dev/manifest.json').read_text())
results=[]
class Links(HTMLParser):
    def __init__(self):super().__init__();self.assets=set()
    def handle_starttag(self,tag,attrs):
        d=dict(attrs);url=d.get('src' if tag=='script' else 'href','')
        if tag in ('script','link') and url.startswith('/assets/'):self.assets.add(url)
def get(path):
    with urlopen(origin+path,timeout=20) as r:
        data=r.read();assert r.status==200,path
        return data,r.headers.get('Content-Type','')
body,_=get('/');home=body.decode()
for text in ['核心体验','都市与梦境','开发资料','系统设计','开发手册','大共梦事件']:
    assert text in home,text
results.append({'path':'/','result':'overview, encyclopedia, development navigation and event entry present'})
assets=Links();assets.feed(home)
def page(name):
    path='/'+name.removesuffix('.md')
    body,kind=get(path);assert 'text/html' in kind,path
    text=body.decode();parser=Links();parser.feed(text)
    if name.endswith('story/index.md'):assert '横向故事时间轴' in text
    if name.endswith('world/great-shared-dream.md'):
        for phrase in ['三个条件相遇','独立验证与源头终止','此后的日子与今天的新联动']:assert phrase in text
    return {'path':path,'result':'page available'},parser.assets
with ThreadPoolExecutor(max_workers=8) as pool:
    for result,paths in pool.map(page,manifest['pages']):results.append(result);assets.assets.update(paths)
def alias(entry):
    path='/'+entry['old'];body,_=get(path)
    assert 'location.replace('+json.dumps(entry['to'])+' + location.hash)' in body.decode(),path
    return {'path':path,'target':entry['to'],'result':'legacy link retained'}
with ThreadPoolExecutor(max_workers=8) as pool:results.extend(pool.map(alias,manifest['aliases']))
def asset(path):
    body,kind=get(path);assert body and 'text/html' not in kind,path
    return {'path':path,'result':'asset available'}
with ThreadPoolExecutor(max_workers=8) as pool:results.extend(pool.map(asset,sorted(assets.assets)))
for path in ['/story/','/production/wiki','/narrative/workflow','/conventions']:
    body,_=get(path);assert '页面已迁移' in body.decode(),path
    results.append({'path':path,'result':'original extensionless entry available'})
body,_=get('/dev/design/catalogs/quests.json');assert len(json.loads(body)['quests'])==18
results.append({'path':'/dev/design/catalogs/quests.json','result':'developer catalog available'})
Path(__file__).with_name('live.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
print(f'PASS: {len(manifest["pages"])} pages, {len(manifest["aliases"])} legacy routes, {len(assets.assets)} assets, complete home and developer catalog; {len(results)} HTTPS checks')
