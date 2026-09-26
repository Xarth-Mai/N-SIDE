"""Read-only checks against the actual public Wiki, without browser claims."""
from concurrent.futures import ThreadPoolExecutor
from html.parser import HTMLParser
import json
from pathlib import Path
import sys
from urllib.request import urlopen
from urllib.error import HTTPError

root=Path(__file__).resolve().parents[4]
origin=sys.argv[1].rstrip('/')
quests=json.loads((root/'docs/dev/design/catalogs/quests.json').read_text())['quests']
results=[]
def fetch(path):
    try:
        with urlopen(origin+path,timeout=20) as r:
            return r.status,r.read(),r.headers.get('Content-Type','')
    except HTTPError as e:
        return e.code,e.read(),e.headers.get('Content-Type','')
class Assets(HTMLParser):
    def __init__(self):
        super().__init__();self.paths=set()
    def handle_starttag(self,tag,attrs):
        attrs=dict(attrs)
        url=attrs.get('src' if tag=='script' else 'href','')
        if tag in ('script','link') and url.startswith('/assets/'):
            self.paths.add(url)
status,legacy,_=fetch('/story/')
legacy=legacy.decode()
assert status==200 and 'location.replace("/player/encyclopedia/story/" + location.hash)' in legacy
results.append({'path':'/story/','status':status,'result':'compatibility entry points to new timeline; preserves fragment'})
status,html,_=fetch('/player/encyclopedia/story/')
html=html.decode()
assert status==200 and '横向故事时间轴' in html
for quest in quests:
    assert quest['story']['summary'] in html,quest['id']
results.append({'path':'/player/encyclopedia/story/','status':status,'result':'all 18 cards rendered'})
assets=Assets();assets.feed(html)
def detail(quest):
    path='/'+quest['source_file'].removesuffix('.md')
    status,body,_=fetch(path)
    body=body.decode()
    assert status==200 and '强制前置' in body and '后续条件影响' in body,(path,status)
    parser=Assets();parser.feed(body)
    return {'path':path,'status':status,'result':'relationship detail rendered'},parser.paths
with ThreadPoolExecutor(max_workers=6) as pool:
    for result,paths in pool.map(detail,quests):
        results.append(result);assets.paths.update(paths)
def resource(path):
    status,body,kind=fetch(path)
    assert status==200 and 'text/html' not in kind,(path,status,kind)
    return {'path':path,'status':status,'result':'asset readable','bytes':len(body)}
with ThreadPoolExecutor(max_workers=6) as pool:
    results.extend(pool.map(resource,sorted(assets.paths)))
for path in ['/dev/design/story-graph','/dev/design/catalogs/quests.json']:
    status,_,_=fetch(path)
    assert status==404,(path,status)
    results.append({'path':path,'status':status,'result':'developer source not published'})
Path(__file__).with_name('live.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
print(f'PASS: {len(results)} live HTTPS checks, including 18 details and {len(assets.paths)} assets')
