#!/usr/bin/env python3
"""Ask the installed Codex app-server to discover repo Skills, without starting an AI turn."""
from __future__ import annotations

import argparse
import asyncio
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


async def probe(log) -> dict:
    if not shutil.which('codex'):
        return {'status': 'NOT RUN', 'error': 'Codex CLI unavailable; install or use the configured host'}
    process = await asyncio.create_subprocess_exec(
        'codex', 'app-server', '--stdio', cwd=ROOT,
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
        stderr=log)

    async def request(message):
        process.stdin.write((json.dumps(message) + '\n').encode())
        await process.stdin.drain()
        while line := await process.stdout.readline():
            response = json.loads(line)
            if response.get('id') == message.get('id'):
                if 'error' in response:
                    raise RuntimeError(str(response['error']))
                return response['result']
        raise RuntimeError(f'Codex app-server closed before response {message["id"]}')

    try:
        initialize = await asyncio.wait_for(request({
            'id': 1, 'method': 'initialize',
            'params': {'clientInfo': {'name': 'nside-skill-discovery', 'version': '1'}}}), 30)
        process.stdin.write(b'{"method":"initialized"}\n')
        await process.stdin.drain()
        response = await asyncio.wait_for(request({
            'id': 2, 'method': 'skills/list',
            'params': {'cwds': [str(ROOT), str(ROOT / 'game')], 'forceReload': True}}), 30)
        expected = {str(path.resolve()) for path in (ROOT / '.agents/skills').glob('*/SKILL.md')}
        entries = []
        for entry in response['data']:
            skills = [s for s in entry['skills'] if str(s['path']) in expected]
            missing = expected - {str(s['path']) for s in skills if s['enabled']}
            errors = [e for e in entry['errors'] if str(ROOT) in str(e['path'])]
            entries.append({'cwd': str(Path(entry['cwd']).relative_to(ROOT)) or '.',
                            'skills': [{'name': s['name'], 'path': str(Path(s['path']).relative_to(ROOT)), 'enabled': s['enabled']} for s in skills],
                            'missing': [str(Path(path).relative_to(ROOT)) for path in sorted(missing)], 'errors': errors})
        return {'status': 'PASS' if len(entries) == 2 and all(not e['missing'] and not e['errors'] for e in entries) else 'FAIL',
                'codex': initialize.get('userAgent', 'see installed codex --version'),
                'scope': 'Host discovery only; no AI turn or implicit routing claim', 'entries': entries}
    finally:
        if process.returncode is None:
            process.terminate()
            try:
                await asyncio.wait_for(process.wait(), 5)
            except asyncio.TimeoutError:
                process.kill()
                await process.wait()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    try:
        with args.output.with_suffix('.log').open('w', encoding='utf-8') as log:
            result = asyncio.run(probe(log))
    except (OSError, ValueError, KeyError, RuntimeError, asyncio.TimeoutError) as error:
        result = {'status': 'FAIL', 'error': str(error) or 'Codex app-server timed out'}
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'{result["status"]}: {args.output}')
    return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())
