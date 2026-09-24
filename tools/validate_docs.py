#!/usr/bin/env python3
"""Check project Markdown, stable identifiers, and local link targets."""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlsplit

SCOPES = ('docs', 'todo', '.agents/skills', 'source-assets')
TOP_FILES = ('README.md', 'AGENTS.md', 'tools/README.md', 'game/README.md')
ID_PATTERN = re.compile(r'(?:DOC-[A-Z0-9-]+|(?:QST|ENM|CHR|LOC|WORLD|EVT|TASK|ADR)-\d{3,})\Z')
DESIGN_STATES = {'draft', 'approved', 'superseded'}
TASK_STATES = {'backlog', 'ready', 'in_progress', 'blocked', 'done', 'cancelled'}
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


def prose(text: str) -> str:
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
            result.append(re.sub(r'(`+).*?\1', '', line))
    return '\n'.join(result)


def markdown_files(root: Path) -> list[Path]:
    files = {root / name for name in TOP_FILES if (root / name).is_file()}
    for scope in SCOPES:
        files.update(path for path in (root / scope).rglob('*.md')
                     if not any(part in {'.vitepress', 'node_modules', 'public'}
                                for part in path.relative_to(root / scope).parts))
    return sorted(files)


def validate(root: Path) -> dict[str, Any]:
    root = root.resolve()
    issues: list[dict[str, Any]] = []
    records: dict[str, tuple[Path, dict[str, Any]]] = {}
    documents = markdown_files(root)

    def issue(path: Path, message: str, line: int = 1) -> None:
        issues.append({'file': path.relative_to(root).as_posix(), 'line': line, 'message': message})

    for path in documents:
        relative = path.relative_to(root)
        try:
            text = path.read_bytes().decode('utf-8')
        except (OSError, UnicodeError) as exc:
            issue(path, f'UTF-8 读取失败：{exc}')
            continue
        if '\r' in text:
            issue(path, '行尾格式使用 LF')
        for number, line in enumerate(text.splitlines(), 1):
            if line.endswith((' ', '\t')):
                issue(path, '行尾存在空格', number)
        try:
            fields, body = metadata(text)
        except ValueError as exc:
            issue(path, str(exc))
            fields, body = {}, text
        visible = prose(body)
        if not re.search(r'^#\s+\S', visible, re.M):
            issue(path, '一级标题缺失')
        template = relative.parts[:2] == ('docs', 'templates')
        record_id = fields.get('id')
        if record_id and not template:
            if not isinstance(record_id, str) or not ID_PATTERN.fullmatch(record_id):
                issue(path, f'ID 格式：{record_id}')
            elif record_id in records:
                previous = records[record_id][0].relative_to(root).as_posix()
                issue(path, f'ID 重复：{record_id}，另见 {previous}')
            else:
                records[record_id] = (path, fields)
            states = TASK_STATES if str(record_id).startswith('TASK-') else DESIGN_STATES
            if 'status' in fields and (not isinstance(fields['status'], str) or fields['status'] not in states):
                issue(path, f'状态值：{fields["status"]}')
        if relative.parts[:2] == ('.agents', 'skills') and path.name == 'SKILL.md':
            if fields.get('name') != path.parent.name:
                issue(path, 'Skill name 与目录名对应')
            description = fields.get('description')
            if not isinstance(description, str) or not description.strip():
                issue(path, 'Skill description 缺失')
        for pattern in (INLINE_LINK, REFERENCE_LINK):
            for match in pattern.finditer(visible):
                target = match.group(1) or match.group(2)
                try:
                    url = urlsplit(target)
                except ValueError:
                    issue(path, f'链接格式：{target}')
                    continue
                if url.scheme or url.netloc or not url.path:
                    continue
                destination = unquote(url.path)
                wiki = relative.parts[0] == 'docs'
                if wiki and destination.startswith('/'):
                    public_file = root / 'docs/public' / destination.lstrip('/')
                    linked = public_file if public_file.exists() else root / 'docs' / destination.lstrip('/')
                else:
                    linked = root / destination.lstrip('/') if destination.startswith('/') else path.parent / destination
                if wiki and not linked.exists():
                    candidates = [linked.with_suffix('.md')] if linked.suffix == '.html' else []
                    if not linked.suffix:
                        candidates.extend([linked.with_suffix('.md'), linked / 'index.md'])
                    linked = next((item for item in candidates if item.exists()), linked)
                if wiki and not linked.resolve().is_relative_to((root / 'docs').resolve()):
                    issue(path, f'Wiki 页面链接使用 docs/ 内的发布路径：{target}', visible[:match.start()].count('\n') + 1)
                elif not linked.exists():
                    issue(path, f'链接目标缺失：{target}', visible[:match.start()].count('\n') + 1)

    for record_id, (path, fields) in records.items():
        dependencies = fields.get('depends_on', [])
        if not isinstance(dependencies, list) or any(not isinstance(x, str) for x in dependencies):
            issue(path, 'depends_on 使用字符串数组')
            continue
        for dependency in dependencies:
            if dependency == record_id:
                issue(path, f'依赖指向自身：{dependency}')
            elif dependency not in records:
                issue(path, f'依赖目标缺失：{dependency}')
    return {'files': len(documents), 'records': len(records), 'issues': issues, 'ok': not issues}


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
