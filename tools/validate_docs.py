#!/usr/bin/env python3
"""Check project Markdown, stable identifiers, and local link targets."""
from __future__ import annotations

import argparse
import json
import re
import unicodedata
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlsplit

SCOPES = ('docs', 'todo', '.agents/skills', 'source-assets')
TOP_FILES = ('README.md', 'AGENTS.md', 'THIRD_PARTY_NOTICES.md', 'tools/README.md', 'game/README.md')
DOCUMENT_ID = re.compile(r'DOC-[A-Z0-9-]+\Z')
SUBJECT_ID = re.compile(r'(?:QST|ENM|CHR|LOC|WORLD|EVT|AST)-\d{3,}\Z')
DESIGN_STATES = {'draft', 'accepted', 'superseded'}
INLINE_LINK = re.compile(r'!?\[[^\]\n]*\]\(\s*(?:<([^>\n]+)>|([^\s)]+))[^\n)]*\)')
REFERENCE_LINK = re.compile(r'^\s*\[[^\]\n]+\]:\s*(?:<([^>\n]+)>|(\S+))', re.M)


def metadata(text: str) -> tuple[dict[str, Any], str]:
    """Read the project's single-line front matter format."""
    lines = text.splitlines()
    if not lines or lines[0] != '---':
        return {}, text
    try:
        end = lines.index('---', 1)
    except ValueError as exc:
        raise ValueError('front matter 结束标记缺失') from exc
    values: dict[str, Any] = {}
    for line in lines[1:end]:
        if not line.strip():
            continue
        key, sep, raw = line.partition(':')
        key, raw = key.strip(), raw.strip()
        if not sep or not re.fullmatch(r'[a-z][a-z0-9_]*', key):
            raise ValueError(f'字段格式：{line}')
        if key in values:
            raise ValueError(f'字段重复：{key}')
        if raw.startswith(('"', '[')):
            try:
                value = json.loads(raw)
            except json.JSONDecodeError as exc:
                raise ValueError(f'{key} 的字符串或数组格式有误') from exc
        else:
            value = raw
        if not isinstance(value, (str, int, float, list)) or isinstance(value, bool):
            raise ValueError(f'{key} 使用单行标量或数组')
        values[key] = value
    return values, '\n' * (end + 1) + '\n'.join(lines[end + 1:])


def prose(text: str, *, keep_inline: bool = False) -> str:
    """Extract prose while preserving line positions around fenced code."""
    result: list[str] = []
    fence_char, fence_size = '', 0
    for line in text.splitlines():
        start = re.match(r'^\s{0,3}(`{3,}|~{3,})', line)
        if fence_char:
            closing = re.match(r'^\s{0,3}(' + re.escape(fence_char) + r'{3,})\s*$', line)
            if closing and len(closing.group(1)) >= fence_size:
                fence_char, fence_size = '', 0
            result.append('')
        elif start:
            fence_char, fence_size = start.group(1)[0], len(start.group(1))
            result.append('')
        else:
            result.append(line if keep_inline else re.sub(r'(`+).*?\1', '', line))
    return '\n'.join(result)


def body_only(text: str) -> str:
    """Leave task and Skill YAML validation to their Bun owners."""
    if text.startswith('---\n'):
        end = text.find('\n---', 4)
        if end >= 0:
            return '\n' * text[:end + 4].count('\n') + text[end + 4:]
    return text


def anchors(text: str, *, vitepress: bool = False) -> set[str]:
    found: set[str] = set()
    counts: dict[str, int] = {}
    for match in re.finditer(r'^#{1,6}\s+(.+?)\s*#*$', prose(body_only(text), keep_inline=True), re.M):
        heading = match.group(1)
        explicit = re.search(r'\{#([^}]+)\}', heading)
        if explicit:
            found.add(explicit.group(1))
            continue
        heading = re.sub(r'<[^>]*>', '', heading)
        heading = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', heading)
        heading = re.sub(r'`([^`]+)`', r'\1', heading)
        heading = re.sub(r'(\*\*|__|[~*])(.+?)\1', r'\2', heading)
        if vitepress:
            # Match the locked VitePress 1.6 slug rules, including numeric headings
            slug = unicodedata.normalize('NFKD', heading)
            slug = re.sub(r'[\u0300-\u036f\u0000-\u001f]', '', slug)
            slug = re.sub(r"[\s~`!@#$%^&*()\-_+=[\]{}|\\;:\"'“”‘’<>,.?/]+", '-', slug).strip('-').lower()
            slug = re.sub(r'^(\d)', r'_\1', slug)
        else:
            slug = ''.join(char for char in heading.lower() if char.isalnum() or char in '_-' or char.isspace()).strip()
            slug = re.sub(r'\s', '-', slug)
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        found.add(slug + (f'-{count}' if count else ''))
    found.update(re.findall(r'<[^>]+\b(?:id|name)=["\']([^"\']+)["\']', text))
    return found


def markdown_files(root: Path) -> list[Path]:
    # 上游原版遵守其自身 Markdown / YAML 格式，由 validate-skills.ts 单独校验
    manifest = root / 'third_party/skills/manifest.json'
    originals = set()
    if manifest.is_file():
        imports = json.loads(manifest.read_text(encoding='utf-8'))
        originals = {root / skill['local_path'] / file['path']
                     for skill in imports['skills'] if skill['mode'] in {'verbatim', 'patched'}
                     for file in skill['files']}
    files = {root / name for name in TOP_FILES if (root / name).is_file()}
    for scope in SCOPES:
        files.update(path for path in (root / scope).rglob('*.md')
                     if not any(part in {'.vitepress', 'node_modules', 'public'}
                                for part in path.relative_to(root / scope).parts))
    return sorted(files - originals)


def validate(root: Path) -> dict[str, Any]:
    root = root.resolve()
    issues: list[dict[str, Any]] = []
    records: dict[str, Path] = {}
    subjects: dict[str, list[Path]] = {}
    designs: list[tuple[Path, dict[str, Any]]] = []
    try:
        documents = markdown_files(root)
    except (OSError, UnicodeError, ValueError, KeyError, TypeError) as exc:
        return {'files': 0, 'records': 0, 'ok': False, 'issues': [
            {'file': 'third_party/skills/manifest.json', 'line': 1, 'message': str(exc)}]}

    def issue(path: Path, message: str, line: int = 1) -> None:
        issues.append({'file': path.relative_to(root).as_posix(), 'line': line, 'message': message})

    catalogues: dict[str, set[str]] = {}
    for prefix, filename, collection in [('CHR-', 'characters.json', 'characters'), ('QST-', 'quests.json', 'quests')]:
        catalogue = root / 'docs/dev/design/catalogs' / filename
        if catalogue.is_file():
            try:
                data = json.loads(catalogue.read_text(encoding='utf-8'))
                catalogues[prefix] = {item['id'] for item in data[collection]}
            except (OSError, UnicodeError, ValueError, KeyError, TypeError) as exc:
                issue(catalogue, f'对象目录读取失败：{exc}')

    for path in documents:
        relative = path.relative_to(root)
        try:
            text = path.read_bytes().decode('utf-8')
        except (OSError, UnicodeError) as exc:
            issue(path, f'UTF-8 读取失败：{exc}')
            continue
        legacy = relative.parts[:3] == ('todo', 'archive', 'legacy')
        own_metadata = relative.parts[:2] not in {('todo', 'tasks'), ('.agents', 'skills')} and not legacy
        if not legacy:
            if '\r' in text:
                issue(path, '行尾格式使用 LF')
            for number, line in enumerate(text.splitlines(), 1):
                if line.endswith((' ', '\t')):
                    issue(path, '行尾存在空格', number)
        try:
            fields, body = metadata(text) if own_metadata else ({}, body_only(text))
        except ValueError as exc:
            issue(path, str(exc))
            fields, body = {}, text
        visible = prose(body)
        if not re.search(r'^#\s+\S', visible, re.M):
            issue(path, '一级标题缺失')
        template = relative.parts[:4] == ('docs', 'dev', 'handbook', 'templates')
        if relative.parts[0] == 'docs' and not template:
            if 'id' in fields:
                issue(path, '文档使用 document_id，对象引用使用 subject_id')
            for field, pattern in [('document_id', DOCUMENT_ID), ('subject_id', SUBJECT_ID)]:
                value = fields.get(field)
                if value is None:
                    continue
                if not isinstance(value, str) or not pattern.fullmatch(value):
                    issue(path, f'{field} 格式：{value}')
                elif field == 'document_id':
                    if value in records:
                        issue(path, f'ID 重复：{value}，另见 {records[value].relative_to(root).as_posix()}')
                    else:
                        records[value] = path
                else:
                    subjects.setdefault(value, []).append(path)
                    for prefix, ids in catalogues.items():
                        if value.startswith(prefix) and value not in ids:
                            issue(path, f'对象未登记在权威目录：{value}')
            if 'status' in fields and (not isinstance(fields['status'], str) or fields['status'] not in DESIGN_STATES):
                issue(path, f'设计状态值：{fields["status"]}')
            designs.append((path, fields))
        for pattern in (INLINE_LINK, REFERENCE_LINK):
            for match in pattern.finditer(visible):
                target = match.group(1) or match.group(2)
                try:
                    url = urlsplit(target)
                except ValueError:
                    issue(path, f'链接格式：{target}')
                    continue
                if url.scheme or url.netloc:
                    continue
                destination = unquote(url.path)
                wiki = relative.parts[0] == 'docs'
                if not destination:
                    linked = path
                elif wiki and destination.startswith('/'):
                    public_file = root / 'docs/public' / destination.lstrip('/')
                    linked = public_file if public_file.exists() else root / 'docs' / destination.lstrip('/')
                else:
                    linked = root / destination.lstrip('/') if destination.startswith('/') else path.parent / destination
                if wiki and not linked.exists():
                    candidates = [linked.with_suffix('.md')] if linked.suffix == '.html' else []
                    if not linked.suffix:
                        candidates.extend([linked.with_suffix('.md'), linked / 'index.md'])
                    linked = next((item for item in candidates if item.exists()), linked)
                if wiki and linked.is_dir():
                    linked /= 'index.md'
                dev = relative.parts[:2] == ('docs', 'dev')
                player = relative.parts[:2] == ('docs', 'player')
                resolved = linked.resolve()
                if wiki and not resolved.is_relative_to(root / 'docs') and not (dev and resolved.is_relative_to(root)):
                    issue(path, f'Wiki 页面链接使用 docs/ 内的发布路径：{target}', visible[:match.start()].count('\n') + 1)
                elif player and not any(resolved.is_relative_to(root / 'docs' / allowed) for allowed in ('player', 'public')):
                    issue(path, f'玩家页面链接越出发布受众：{target}', visible[:match.start()].count('\n') + 1)
                elif not linked.exists():
                    issue(path, f'链接目标缺失：{target}', visible[:match.start()].count('\n') + 1)
                elif url.fragment and linked.is_file() and linked.suffix == '.md':
                    try:
                        if unquote(url.fragment) not in anchors(linked.read_text(encoding='utf-8'), vitepress=resolved.is_relative_to(root / 'docs')):
                            issue(path, f'链接锚点缺失：{target}', visible[:match.start()].count('\n') + 1)
                    except (OSError, UnicodeError) as exc:
                        issue(path, f'锚点读取失败：{target}：{exc}')

    for subject_id, paths in subjects.items():
        authority = [path for path in paths if path.relative_to(root).parts[:2] == ('docs', 'player') and path.relative_to(root).parts[2] != 'guide']
        if len(authority) > 1:
            issue(authority[-1], f'对象权威页重复：{subject_id}')
    known = records.keys() | subjects.keys()
    for ids in catalogues.values():
        known |= ids
    for path, fields in designs:
        dependencies = fields.get('depends_on', [])
        if not isinstance(dependencies, list) or any(not isinstance(x, str) for x in dependencies):
            issue(path, 'depends_on 使用字符串数组')
            continue
        for dependency in dependencies:
            if dependency == fields.get('document_id'):
                issue(path, f'依赖指向自身：{dependency}')
            elif dependency not in known:
                issue(path, f'依赖目标缺失：{dependency}')
    return {'files': len(documents), 'records': len(known), 'issues': issues, 'ok': not issues}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('.'), help='Project root')
    parser.add_argument('--json', action='store_true', help='Print JSON results')
    args = parser.parse_args()
    if not args.root.is_dir():
        parser.error(f'目录缺失：{args.root}')
    result = validate(args.root)
    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print(f"{'PASS' if result['ok'] else 'FAIL'}: {result['files']} Markdown files, {result['records']} IDs")
        for item in result['issues']:
            print(f"{item['file']}:{item['line']}: {item['message']}")
    return 0 if result['ok'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
