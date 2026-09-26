"""Check emitted story pages over the real preview servers; this is not a browser test."""
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
quests = json.loads((ROOT / 'docs/dev/design/catalogs/quests.json').read_text())['quests']
results = []

def request(port, path):
    try:
        with urllib.request.urlopen(f'http://127.0.0.1:{port}{path}', timeout=10) as response:
            return response.status, response.read().decode()
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode()

for profile, port in [('player', 5197), ('dev', 5198)]:
    with (OUT / f'preview-{profile}.log').open('w') as log:
        process = subprocess.Popen(['bun', 'tools/wiki.ts', 'preview', profile, str(port)], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            for _ in range(80):
                if process.poll() is not None:
                    raise RuntimeError(f'{profile} exited: {process.returncode}')
                try:
                    if request(port, '/')[0] == 200:
                        break
                except OSError:
                    pass
                time.sleep(.1)
            else:
                raise RuntimeError('Preview did not become ready')
            for quest in quests:
                path = '/' + quest['source_file'].removesuffix('.md')
                status, body = request(port, path)
                assert status == 200 and '故事关系' in body and '强制前置' in body and '后续条件影响' in body, (profile, path)
                assert quest['title'] in body and 'condition-' not in body.split('<title>')[0]
                results.append({'profile': profile, 'path': path, 'status': status, 'detail': True})
            for path, role in [('/player/encyclopedia/story/', None), ('/player/encyclopedia/story/main/', 'main'), ('/player/encyclopedia/story/daily/', 'side')]:
                status, body = request(port, path)
                assert status == 200
                for quest in quests:
                    if role is None or quest['story']['role'] == role:
                        assert quest['title'] in body and quest['story']['summary'] in body, (profile, path, quest['id'])
                results.append({'profile': profile, 'path': path, 'status': status, 'all_cards': True})
            status, body = request(port, '/dev/design/story-graph')
            assert status == (200 if profile == 'dev' else 404)
            if profile == 'dev':
                assert '逐篇关系审查' in body
                for quest in quests:
                    assert quest['story']['review']['pending'][0] in body, quest['id']
            results.append({'profile': profile, 'path': '/dev/design/story-graph', 'status': status})
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
            process.wait(timeout=10)
(OUT / 'http.json').write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
print(f'PASS: {len(results)} real HTTP checks; all 18 details, indices and audience boundaries')
