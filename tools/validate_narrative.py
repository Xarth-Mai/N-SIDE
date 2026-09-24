#!/usr/bin/env python3
"""Validate N:SIDE narrative design data and explicitly modelled information paths."""
from __future__ import annotations

import argparse
import csv
import json
import math
import re
from collections import deque
from pathlib import Path
from typing import Any

KNOWLEDGE = {'hidden', 'hinted', 'suspected', 'confirmed', 'disproved'}
IMPORTANCE = {'critical', 'supporting', 'optional'}
DIALOGUE_FIELDS = {'line_id', 'speaker_id', 'text_zh_cn', 'context', 'recording_status', 'audio_path'}


def validate_data(data: Any, directory: Path, max_states: int = 50000) -> dict[str, Any]:
    """Inspect one quest. `next` edges enumerate all modelled routes.

    Beat requirements are checked at entry, deliveries at completion. Analysis
    explores (beat, knowledge-vector) states. Engine predicates and the meaning
    of prose remain outside this design-graph analysis.
    """
    issues: list[dict[str, Any]] = []
    result: dict[str, Any] = {'quest_id': None, 'issues': issues, 'states_checked': 0,
                              'complete': True, 'ok': False}

    def add(code: str, where: str, message: str, **details: Any) -> None:
        issues.append({'code': code, 'location': where, 'message': message, **details})

    def text(value: Any, where: str) -> bool:
        if not isinstance(value, str) or not value.strip():
            add('field', where, '字段使用有内容的字符串')
            return False
        return True

    def strings(value: Any, where: str) -> bool:
        if not isinstance(value, list) or any(not isinstance(x, str) or not x for x in value):
            add('field', where, '字段使用字符串数组')
            return False
        if len(set(value)) != len(value):
            add('duplicate', where, '数组项重复')
        return True

    def enum(value: Any, values: set[str], where: str) -> None:
        if not isinstance(value, str) or value not in values:
            add('enum', where, f'字段值范围：{", ".join(sorted(values))}')

    if not isinstance(max_states, int) or isinstance(max_states, bool) or max_states < 1:
        add('budget', 'max_states', '分析预算使用正整数')
        result['complete'] = False
        return result
    if not isinstance(data, dict):
        add('field', '$', '顶层使用 JSON object')
        return result
    if type(data.get('schema_version')) is not int or data['schema_version'] != 1:
        add('version', 'schema_version', '当前结构版本为整数 1')
    qid = data.get('quest_id')
    if not isinstance(qid, str) or not re.fullmatch(r'QST-\d{3,}', qid):
        add('id', 'quest_id', '任务 ID 使用 QST-编号')
        return result
    result['quest_id'] = qid

    def records(field: str, kind: str) -> dict[str, dict[str, Any]]:
        value = data.get(field)
        indexed: dict[str, dict[str, Any]] = {}
        if not isinstance(value, list):
            add('field', field, '字段使用 object 数组')
            return indexed
        for number, record in enumerate(value):
            loc = f'{field}[{number}]'
            if not isinstance(record, dict):
                add('field', loc, '条目使用 object')
                continue
            rid = record.get('id')
            if not isinstance(rid, str) or not re.fullmatch(re.escape(qid) + '-' + kind + r'\d{3,}', rid):
                add('id', loc, f'局部 ID 使用 {qid}-{kind}编号')
            elif rid in indexed:
                add('duplicate', loc, f'ID 重复：{rid}')
            else:
                indexed[rid] = record
        return indexed

    info = records('information', 'I')
    beats = records('beats', 'B')
    choices = records('choices', 'C')
    scenes = records('scenes', 'S')
    for iid, item in info.items():
        for field in ('proposition', 'world_truth'):
            text(item.get(field), f'{iid}.{field}')
        enum(item.get('importance'), IMPORTANCE, f'{iid}.importance')
        enum(item.get('initial_state'), KNOWLEDGE, f'{iid}.initial_state')
    if not beats:
        add('field', 'beats', '流程包含起点与结束节点')
    text(data.get('entry_beat'), 'entry_beat')
    strings(data.get('terminal_beats'), 'terminal_beats')
    if data.get('terminal_beats') == []:
        add('field', 'terminal_beats', '流程至少包含一个结束节点')

    for bid, beat in beats.items():
        for field in ('dramatic_change', 'player_goal', 'player_action'):
            text(beat.get(field), f'{bid}.{field}')
        strings(beat.get('next'), f'{bid}.next')
        if not isinstance(beat.get('state_changes'), dict):
            add('field', f'{bid}.state_changes', '状态效果使用 object')
        duration = beat.get('target_seconds')
        if duration is not None and (type(duration) not in (int, float) or (isinstance(duration, float) and not math.isfinite(duration)) or duration < 0):
            add('field', f'{bid}.target_seconds', '时长使用有限非负数或 null')
        for field in ('requires', 'delivers'):
            mapping = beat.get(field)
            if not isinstance(mapping, dict):
                add('field', f'{bid}.{field}', '信息效果使用 object')
                continue
            for iid, value in mapping.items():
                if iid not in info:
                    add('reference', f'{bid}.{field}', f'信息引用目标：{iid}')
                if field == 'requires':
                    if strings(value, f'{bid}.{field}.{iid}'):
                        if not value:
                            add('field', f'{bid}.{field}.{iid}', '进入条件至少包含一个认知状态')
                        for state in value:
                            enum(state, KNOWLEDGE, f'{bid}.{field}.{iid}')
                else:
                    enum(value, KNOWLEDGE, f'{bid}.{field}.{iid}')
    for cid, choice in choices.items():
        text(choice.get('beat_id'), f'{cid}.beat_id')
        options = choice.get('options')
        if not isinstance(options, list) or len(options) < 2:
            add('field', f'{cid}.options', '显式选择具有至少两个选项')
            continue
        option_ids: set[str] = set()
        for idx, option in enumerate(options):
            loc = f'{cid}.options[{idx}]'
            if not isinstance(option, dict):
                add('field', loc, '选项使用 object')
                continue
            for field in ('id', 'to', 'acknowledgement', 'consequence'):
                text(option.get(field), f'{loc}.{field}')
            oid = option.get('id')
            if isinstance(oid, str):
                if oid in option_ids:
                    add('duplicate', loc, f'选项 ID 重复：{oid}')
                option_ids.add(oid)
    for sid, scene in scenes.items():
        for field in ('beat_id', 'intent', 'entry_state', 'exit_state'):
            text(scene.get(field), f'{sid}.{field}')
        for field in ('cast', 'line_ids'):
            strings(scene.get(field), f'{sid}.{field}')
        if isinstance(scene.get('cast'), list):
            for speaker in scene['cast']:
                if not isinstance(speaker, str) or not re.fullmatch(r'CHR-\d{3,}|NARRATOR|SYSTEM', speaker):
                    add('id', f'{sid}.cast', f'角色 ID：{speaker}')
        if isinstance(scene.get('line_ids'), list):
            for line in scene['line_ids']:
                if not isinstance(line, str) or not re.fullmatch(re.escape(qid) + r'-L\d{3,}', line):
                    add('id', f'{sid}.line_ids', f'台词 ID：{line}')
    if issues:
        return result

    # All fields above are now shape-checked.
    entry = data['entry_beat']
    terminals = set(data['terminal_beats'])
    for bid in [entry, *data['terminal_beats']]:
        if bid not in beats:
            add('reference', 'entry / terminal', f'节点引用目标：{bid}')
    for bid, beat in beats.items():
        for nxt in beat['next']:
            if nxt not in beats:
                add('reference', f'{bid}.next', f'节点引用目标：{nxt}')
        if bid in terminals and beat['next']:
            add('terminal', bid, '结束节点的 next 为 []')
        if bid not in terminals and not beat['next']:
            add('terminal', bid, '零后继节点登记在 terminal_beats')
    for cid, choice in choices.items():
        bid = choice['beat_id']
        if bid not in beats:
            add('reference', cid, f'选择所属节点：{bid}')
        for option in choice['options']:
            if option['to'] not in beats:
                add('reference', cid, f'选项目标：{option["to"]}')
            elif bid in beats and option['to'] not in beats[bid]['next']:
                add('choice_edge', cid, f'选项后继登记在 {bid}.next：{option["to"]}')
    for sid, scene in scenes.items():
        if scene['beat_id'] not in beats:
            add('reference', sid, f'场景所属节点：{scene["beat_id"]}')
    required = {iid for beat in beats.values() for iid in beat['requires']}
    for iid, item in info.items():
        if item['importance'] == 'critical' and iid not in required:
            add('critical_use', iid, '关键信息在 requires 中登记使用节点')

    # Dialogue has one physical source; structured scenes carry only line IDs.
    line_scenes = [(sid, scene) for sid, scene in scenes.items() if scene['line_ids']]
    if line_scenes:
        dialogue = directory / 'dialogue.csv'
        rows: dict[str, dict[str, Any]] = {}
        try:
            with dialogue.open(encoding='utf-8-sig', newline='') as stream:
                reader = csv.DictReader(stream)
                if not DIALOGUE_FIELDS.issubset(set(reader.fieldnames or [])):
                    add('dialogue_columns', 'dialogue.csv', '对白源采用项目既有六列字段')
                else:
                    for line_number, row in enumerate(reader, 2):
                        lid = row.get('line_id') or ''
                        if not re.fullmatch(re.escape(qid) + r'-L\d{3,}', lid):
                            add('id', f'dialogue.csv:{line_number}', f'台词 ID：{lid}')
                        elif lid in rows:
                            add('duplicate', f'dialogue.csv:{line_number}', f'台词 ID 重复：{lid}')
                        else:
                            rows[lid] = row
        except (OSError, UnicodeError, csv.Error) as exc:
            add('dialogue_read', 'dialogue.csv', str(exc))
        for sid, scene in line_scenes:
            for lid in scene['line_ids']:
                if lid not in rows:
                    add('dialogue_reference', sid, f'对白源中的台词目标：{lid}')
                else:
                    speaker = rows[lid].get('speaker_id')
                    if speaker not in set(scene['cast']) | {'NARRATOR', 'SYSTEM'}:
                        add('speaker', sid, f'场景参与者包含台词说话者：{lid} / {speaker}')
    if issues:
        return result

    def reachable(starts: set[str], adjacency: dict[str, list[str]]) -> set[str]:
        found: set[str] = set()
        queue = list(starts)
        while queue:
            bid = queue.pop()
            if bid not in found:
                found.add(bid)
                queue.extend(adjacency[bid])
        return found

    graph = {bid: beat['next'] for bid, beat in beats.items()}
    reached = reachable({entry}, graph)
    reverse: dict[str, list[str]] = {bid: [] for bid in beats}
    for bid, after in graph.items():
        for nxt in after:
            reverse[nxt].append(bid)
    ending_reachable = reachable(terminals, reverse)
    for bid in sorted(beats.keys() - reached):
        add('unreachable', bid, '节点与起点之间的可达路径待连接')
    for bid in sorted(reached - ending_reachable):
        add('no_ending', bid, '节点到结束节点的可达路径待连接')

    ids = sorted(info)
    position = {iid: idx for idx, iid in enumerate(ids)}
    initial = (entry, tuple(info[iid]['initial_state'] for iid in ids))
    queue = deque([initial])
    parents: dict[tuple[str, tuple[str, ...]], tuple[str, tuple[str, ...]] | None] = {initial: None}
    reported: set[tuple[str, str, str]] = set()

    def route(key: tuple[str, tuple[str, ...]]) -> list[str]:
        path: list[str] = []
        cursor: tuple[str, tuple[str, ...]] | None = key
        while cursor is not None:
            path.append(cursor[0])
            cursor = parents[cursor]
        return list(reversed(path))

    while queue:
        if result['states_checked'] >= max_states:
            result['complete'] = False
            add('incomplete', 'analysis', f'已使用分析预算：{max_states} 个状态')
            break
        key = queue.popleft()
        bid, knowledge = key
        result['states_checked'] += 1
        beat = beats[bid]
        for iid, expected in beat['requires'].items():
            actual = knowledge[position[iid]]
            signature = (bid, iid, actual)
            if actual not in expected and signature not in reported:
                reported.add(signature)
                add('knowledge_order', bid, f'{iid}：进入时为 {actual}，需要 {expected}',
                    info_id=iid, actual=actual, expected=expected, route=route(key))
        after = list(knowledge)
        for iid, state in beat['delivers'].items():
            after[position[iid]] = state
        after_tuple = tuple(after)
        for nxt in beat['next']:
            next_key = (nxt, after_tuple)
            if next_key not in parents:
                # Bound queued states as well as processed states.
                if len(parents) >= max_states:
                    result['complete'] = False
                    if not any(item['code'] == 'incomplete' for item in issues):
                        add('incomplete', 'analysis', f'已达到分析预算：{max_states} 个状态')
                    queue.clear()
                    break
                parents[next_key] = key
                queue.append(next_key)
        if not result['complete']:
            break
    result['ok'] = not issues and result['complete']
    return result


def validate_file(path: Path, max_states: int = 50000) -> dict[str, Any]:
    try:
        def object_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
            value: dict[str, Any] = {}
            for key, item in pairs:
                if key in value:
                    raise ValueError(f'JSON key 重复：{key}')
                value[key] = item
            return value

        def bad_constant(value: str) -> None:
            raise ValueError(f'JSON 数值：{value}')

        data = json.loads(path.read_text(encoding='utf-8'),
                          object_pairs_hook=object_pairs, parse_constant=bad_constant)
    except (OSError, UnicodeError, ValueError) as exc:
        return {'file': str(path), 'quest_id': None, 'issues': [
            {'code': 'read', 'location': '$', 'message': str(exc)}],
            'states_checked': 0, 'complete': True, 'ok': False}
    return {'file': str(path), **validate_data(data, path.parent, max_states)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group()
    source.add_argument('--root', type=Path, help='Project root; scans docs/quests')
    source.add_argument('--file', type=Path, action='append', help='Check a specific data file; repeatable')
    parser.add_argument('--max-states', type=int, default=50000)
    parser.add_argument('--json', action='store_true')
    args = parser.parse_args()
    if args.max_states < 1:
        parser.error('--max-states 使用正整数')
    if args.file:
        paths = sorted(set(p.resolve() for p in args.file))
    else:
        root = (args.root or Path('.')).resolve()
        if not root.is_dir():
            parser.error(f'项目目录：{root}')
        paths = sorted((root / 'docs/quests').rglob('narrative.json'))
    results = [validate_file(path, args.max_states) for path in paths]
    seen: dict[str, str] = {}
    for item in results:
        qid = item['quest_id']
        if qid and qid in seen:
            item['issues'].append({'code': 'duplicate_quest', 'location': 'quest_id',
                                   'message': f'任务数据 ID 重复：{qid}，另见 {seen[qid]}'})
            item['ok'] = False
        elif qid:
            seen[qid] = item['file']
    payload = {'files': len(results), 'states_checked': sum(r['states_checked'] for r in results),
               'complete': all(r['complete'] for r in results),
               'ok': all(r['ok'] for r in results), 'results': results}
    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    else:
        label = 'EMPTY' if not results else ('PASS' if payload['ok'] else 'FAIL')
        print(f"{label}: {payload['files']} narrative files, {payload['states_checked']} states")
        for item in results:
            for issue in item['issues']:
                print(f"{item['file']}:{issue['location']}: [{issue['code']}] {issue['message']}")
                if 'route' in issue:
                    print('  route: ' + ' -> '.join(issue['route']))
    return 0 if payload['ok'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
