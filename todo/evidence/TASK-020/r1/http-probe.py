"""Check the real Wiki servers and stop only the processes started by this probe."""
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
results = []

def request(port, path):
    try:
        with urllib.request.urlopen(f'http://127.0.0.1:{port}{path}', timeout=10) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()

for mode in ['preview', 'dev']:
    for profile, port in [('player', 5187), ('dev', 5188)]:
        with (OUT / f'http-{mode}-{profile}.log').open('w') as log:
            process = subprocess.Popen(['bun', 'tools/wiki.ts', mode, profile, str(port)], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                for _ in range(80):
                    if process.poll() is not None:
                        raise RuntimeError(f'{mode}/{profile} exited: {process.returncode}')
                    try:
                        if request(port, '/')[0] == 200:
                            break
                    except OSError:
                        pass
                    time.sleep(.1)
                else:
                    raise RuntimeError('Server did not become ready')
                paths = ['/', '/player/encyclopedia/locations/n-district', '/project-assets/district-map/map.json', '/dev/handbook/wiki', '/project-assets/district-map/district.json']
                if mode == 'preview':
                    paths += ['/' + str(p.relative_to(ROOT / 'output/wiki' / profile / 'dist')) for p in (ROOT / 'output/wiki' / profile / 'dist/assets').glob('search-index.*.json')]
                else:
                    paths += ['/@localSearchIndex', '/@fs' + str(ROOT / 'docs/dev/handbook/wiki.md')]
                for path in paths:
                    status, body = request(port, path)
                    expected = 403 if path.startswith('/@fs') else 404 if profile == 'player' and (path.startswith('/dev/') or path.endswith('/district.json')) else 200
                    assert status == expected, (mode, profile, path, status, expected)
                    if path.endswith('/map.json'):
                        data = json.loads(body)
                        assert set(data) == {'groups', 'places', 'scene'}
                        assert len(data['places']) == 91 and data['scene']['objects']
                    results.append({'mode': mode, 'profile': profile, 'path': path, 'status': status, 'bytes': len(body)})
            finally:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=10)
(OUT / 'http.json').write_text(json.dumps(results, ensure_ascii=False, indent=2) + '\n')
print(f'PASS: {len(results)} real HTTP checks')
