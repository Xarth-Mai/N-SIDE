#!/usr/bin/env python3
"""Compare imported files and patch bases with pinned upstream Git objects, not local hashes."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import difflib
import hashlib
import json
from pathlib import Path
import subprocess
import urllib.error
import urllib.parse
import urllib.request


def git(checkout, *args):
    return subprocess.check_output(['git', '-C', str(checkout), *args], stderr=subprocess.PIPE)


def request(url):
    with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'N-SIDE-source-check'}), timeout=40) as response:
        return response.read()


def source_tree(source, checkout=None):
    commit = source['commit']
    if checkout:
        if git(checkout, 'rev-parse', f'{commit}^{{commit}}').decode().strip() != commit:
            raise ValueError('checkout does not contain the exact pinned commit')
        paths = git(checkout, 'ls-tree', '-rz', '--name-only', commit).decode().split('\0')
        return {path for path in paths if path}, lambda path: git(checkout, 'show', f'{commit}:{path}'), 'git-object'
    repo = source['repo'].removeprefix('https://github.com/')
    tree = json.loads(request(f'https://api.github.com/repos/{repo}/git/trees/{commit}?recursive=1'))
    if tree.get('truncated'):
        raise ValueError('upstream tree is truncated; use a Git checkout')
    objects = {entry['path']: entry['sha'] for entry in tree['tree'] if entry['type'] == 'blob'}
    def read(path):
        data = request(f'https://raw.githubusercontent.com/{repo}/{commit}/{urllib.parse.quote(path)}')
        blob = hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()
        if blob != objects[path]:
            raise ValueError(f'raw content differs from upstream Git object: {path}')
        return data
    return set(objects), read, 'github-tree-and-blob'


def check(root, checkouts=None):
    root = Path(root).resolve()
    checkouts = checkouts or {}
    manifest = json.loads((root / 'third_party/skills/manifest.json').read_text())
    results = []
    for source in manifest['sources']:
        entry = {'source': source['id'], 'commit': source['commit'], 'status': 'PASS', 'files': 0, 'patches': 0, 'pinned_links': 0, 'issues': []}
        try:
            tree, read, method = source_tree(source, checkouts.get(source['id']))
            entry['method'] = method
            units = [unit for unit in manifest['skills'] + manifest['references'] if unit['source'] == source['id']]
            needed = {file['upstream_path'] for file in source['license_files']}
            checks = [(file['upstream_path'], root / file['local_path'], None, file['upstream_path']) for file in source['license_files']]
            for unit in units:
                if unit['mode'] == 'adapted':
                    for path in unit.get('upstream_paths', [unit['upstream_path']]):
                        if not any(file.startswith(path + '/') for file in tree):
                            entry['issues'].append(f'adaptation source missing: {path}')
                    continue
                prefix = unit['upstream_path'] + '/'
                actual = {path.removeprefix(prefix) for path in tree if path.startswith(prefix)}
                expected = {file['path'] for file in unit['files']}
                if actual != expected:
                    entry['issues'].append(f"directory coverage {unit['name']}: unrecorded={sorted(actual - expected)}, absent upstream={sorted(expected - actual)}")
                for file in unit['files']:
                    upstream = prefix + file['path']
                    needed.add(upstream)
                    checks.append((upstream, root / unit['local_path'] / file['path'], file.get('patch'), file['path']))
                for link in unit.get('pinned_links', []):
                    prefix_url = f"{source['repo']}/blob/{source['commit']}/"
                    target = urllib.parse.unquote(link['url'].removeprefix(prefix_url)).split('#')[0]
                    if not link['url'].startswith(prefix_url) or target not in tree:
                        entry['issues'].append(f"pinned reference not in upstream tree: {link['url']}")
                    else:
                        entry['pinned_links'] += 1
            missing = needed - tree
            if missing:
                entry['issues'].append(f'missing upstream files: {sorted(missing)}')
            available = sorted(needed & tree)
            with ThreadPoolExecutor(max_workers=8) as pool:
                originals = dict(zip(available, pool.map(read, available)))
            for upstream, local, patch, relative in checks:
                if upstream not in originals:
                    continue
                original = originals[upstream]
                current = local.read_bytes()
                if patch:
                    entry['patches'] += 1
                    if hashlib.sha256(original).hexdigest() != patch['upstream_sha256']:
                        entry['issues'].append(f'patch base mismatch: {local.relative_to(root)}')
                    expected = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), current.decode().splitlines(keepends=True), fromfile='a/' + relative, tofile='b/' + relative))
                    if not expected or (root / patch['file']).read_text() != expected:
                        entry['issues'].append(f'patch does not explain local changes: {local.relative_to(root)}')
                    if b'Modified for N:SIDE' not in current:
                        entry['issues'].append(f'modified file lacks visible attribution: {local.relative_to(root)}')
                elif original != current:
                    entry['issues'].append(f'not byte-identical to pinned source: {local.relative_to(root)}')
                entry['files'] += 1
        except (urllib.error.URLError, TimeoutError, subprocess.CalledProcessError) as error:
            entry['status'] = 'NOT RUN'
            entry['issues'].append(f'upstream unavailable: {error}')
        except (OSError, ValueError, KeyError, TypeError) as error:
            entry['status'] = 'FAIL'
            entry['issues'].append(str(error))
        if entry['issues'] and entry['status'] == 'PASS':
            entry['status'] = 'FAIL'
        results.append(entry)
    return {'status': 'PASS' if results and all(entry['status'] == 'PASS' for entry in results) else 'FAIL', 'sources': results, 'scope': 'Independent pinned source bytes, directory coverage, patch bases/diffs and pinned sibling targets; adapted prose is not claimed verbatim'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('.'))
    parser.add_argument('--checkouts', type=Path, help='Optional JSON map of source IDs to independent Git checkouts; otherwise GitHub fixed objects are fetched')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    try:
        result = check(args.root, json.loads(args.checkouts.read_text()) if args.checkouts else None)
    except (OSError, ValueError, KeyError, TypeError) as error:
        result = {'status': 'FAIL', 'issues': [f'invalid source manifest: {error}']}
    payload = json.dumps(result, ensure_ascii=False, indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload)
    print(payload, end='')
    return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())
