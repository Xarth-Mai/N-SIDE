#!/usr/bin/env python3
"""Ask the installed Codex app-server to discover repo Skills, without starting an AI turn."""
from __future__ import annotations

import argparse
import asyncio
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def expected_skills(root: Path) -> set[str]:
    manifest = json.loads((root / 'third_party/skills/manifest.json').read_text())
    if not isinstance(manifest, dict) or manifest.get('schema_version') != 2:
        raise ValueError('Skill manifest schema_version must be 2')
    policy = manifest.get('policy')
    if not isinstance(policy, dict):
        raise ValueError('Skill manifest policy must be an object')
    required = policy.get('required_skills')
    if not isinstance(required, list) or not required or any(not isinstance(name, str) or not name for name in required):
        raise ValueError('policy.required_skills must be a nonempty list of names')
    if len(required) != len(set(required)):
        raise ValueError('policy.required_skills must be unique')
    declarations = manifest.get('local_skills', []) + manifest.get('skills', [])
    names = [item['name'] for item in declarations]
    if len(names) != len(set(names)) or not set(required) <= set(names):
        raise ValueError('Skill declarations must be unique and include every required Skill')
    expected = {str((root / item['local_path'] / 'SKILL.md').resolve()) for item in declarations}
    if not expected or len(expected) != len(declarations):
        raise ValueError('Expected Skill paths must be nonempty and unique')
    missing = sorted(path for path in expected if not Path(path).is_file())
    if missing:
        raise ValueError(f'Declared Skill files missing: {missing}')
    return expected


def check_discovery(response: dict, expected: set[str], cwds: list[str]) -> list[dict]:
    if not expected:
        raise ValueError('Expected Skill set must not be empty')
    requested = [str(Path(cwd).resolve()) for cwd in cwds]
    if not requested or len(requested) != len(set(requested)):
        raise ValueError('Requested cwds must be nonempty and unique')
    data = response['data']
    returned = [str(Path(entry['cwd']).resolve()) for entry in data]
    if len(returned) != len(set(returned)) or set(returned) != set(requested):
        raise ValueError(f'Returned cwds must match requested cwds exactly: expected={requested}, actual={returned}')
    entries = []
    for entry, cwd in zip(data, returned):
        skills = [skill for skill in entry['skills'] if str(Path(skill['path']).resolve()) in expected]
        paths = [str(Path(skill['path']).resolve()) for skill in skills]
        if len(paths) != len(set(paths)):
            raise ValueError(f'Duplicate Skill results for cwd {cwd}')
        missing = expected - {str(Path(skill['path']).resolve()) for skill in skills if skill['enabled'] is True}
        entries.append({'cwd': cwd, 'skills': [{key: skill[key] for key in ('name', 'path', 'enabled')} for skill in skills],
                        'missing': sorted(missing), 'errors': entry['errors']})
    return entries


async def probe(log) -> dict:
    if not shutil.which('codex'):
        return {'status': 'NOT RUN', 'error': 'Codex CLI unavailable; install or use the configured host'}
    expected = expected_skills(ROOT)
    cwds = [str(ROOT), str(ROOT / 'game')]
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
            'params': {'cwds': cwds, 'forceReload': True}}), 30)
        entries = check_discovery(response, expected, cwds)
        return {'status': 'PASS' if all(not e['missing'] and not e['errors'] for e in entries) else 'FAIL',
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
    except (OSError, ValueError, KeyError, TypeError, RuntimeError, asyncio.TimeoutError) as error:
        result = {'status': 'FAIL', 'error': str(error) or 'Codex app-server timed out'}
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'{result["status"]}: {args.output}')
    return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())
