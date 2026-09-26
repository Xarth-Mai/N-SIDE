"""Probe the real isolated VitePress servers; terminate only processes started here."""
from pathlib import Path
import hashlib
import json
import os
import signal
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
checks = []

def request(port, path, method='GET'):
    try:
        with urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{port}{path}', method=method), timeout=15) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()

for profile, port in [('player', 5187), ('dev', 5188)]:
    with (OUT / f'wiki-http-{profile}.log').open('w') as log:
        process = subprocess.Popen(['bun', 'tools/wiki.mjs', 'dev', profile, str(port)], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            for _ in range(100):
                if process.poll() is not None:
                    raise RuntimeError(f'{profile} server exited: {(OUT / f"wiki-http-{profile}.log").read_text()}')
                try:
                    if request(port, '/')[0] == 200:
                        break
                except OSError:
                    pass
                time.sleep(.1)
            else:
                raise RuntimeError(f'{profile} server timeout')
            routes = [('/player/encyclopedia/locations/n-district', [200]),
                      ('/player/encyclopedia/locations/n-district.md?import', [200]),
                      ('/@localSearchIndexroot', [200]),
                      ('/locations/n-district.html', [200]),
                      ('/project-assets/district-map/map.json', [200]),
                      ('/dev/design/quests/QST-002/narrative.json', [404] if profile == 'player' else [200]),
                      ('/%64ev/design/quests/QST-002/narrative.json', [404] if profile == 'player' else [200]),
                      ('/production/assets.html', [404] if profile == 'player' else [200]),
                      ('/project-assets/district-map/district.json', [404] if profile == 'player' else [200]),
                      ('/@fs/' + str(ROOT / 'docs/dev/design/quests/QST-002/narrative.json'), [403, 404]),
                      ('/@fs/' + str(ROOT / 'source-assets/district-map/district.json'), [403, 404])]
            for path, expected in routes:
                status, body = request(port, path)
                assert status in expected, (profile, path, status, body[:300])
                if profile == 'player':
                    for marker in [b'resolution_committed', b'QST-002-B001', b'DOCS-PIPELINE', b'dev/design/catalogs']:
                        assert marker not in body, (profile, path, marker)
                if path.endswith('/map.json'):
                    data = json.loads(body)
                    assert set(data) == {'groups', 'places', 'scene'} and len(data['places']) == 91
                if path == '/locations/n-district.html':
                    assert b'location.hash' in body and b'/player/encyclopedia/locations/n-district' in body
                checks.append({'profile': profile, 'path': path, 'status': status, 'expected': expected, 'bytes': len(body), 'sha256': hashlib.sha256(body).hexdigest(), 'result': 'PASS'})
            status, body = request(port, '/dev/design/quests/QST-002/narrative.json', 'HEAD')
            assert status == (404 if profile == 'player' else 200) and not body
            checks.append({'profile': profile, 'path': '/dev/design/quests/QST-002/narrative.json', 'method': 'HEAD', 'status': status, 'result': 'PASS'})
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
            process.wait(timeout=15)
result = {'command': 'python3 todo/evidence/TASK-009/r3/wiki-http.py', 'result': 'PASS', 'checks': checks}
(OUT / 'wiki-http.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'result': 'PASS', 'requests': len(checks)}))
