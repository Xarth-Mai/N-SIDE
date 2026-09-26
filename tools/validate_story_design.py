#!/usr/bin/env python3
"""Check story and character indices against the live district catalogue"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


DOCS = Path('docs')
FILES = {
    'place-catalog': DOCS / 'dev/design/catalogs/place-catalog.json',
    'city-story-map': DOCS / 'dev/design/catalogs/city-story-map.json',
    'characters': DOCS / 'dev/design/catalogs/characters.json',
    'quests': DOCS / 'dev/design/catalogs/quests.json',
}
DISTRICT = Path('source-assets/district-map/district.json')
CHARACTERS = DOCS / 'player/characters'
STORY = DOCS / 'player/story'


def character_subject(content: str) -> str | None:
    frontmatter = re.match(r'\A---\n(.*?)\n---(?:\n|$)', content, re.S)
    if not frontmatter:
        return None
    match = re.search(r'^subject_id:\s*["\']?(CHR-\d{3,})["\']?\s*$', frontmatter.group(1), re.M)
    return match.group(1) if match else None


def validate(root: Path) -> dict:
    issues = []
    result = {'ok': False, 'counts': {}, 'issues': issues}

    def add(code, location, message):
        issues.append({'code': code, 'location': location, 'message': message})

    def pairs(items):
        record = {}
        for key, value in items:
            if key in record:
                raise ValueError(f'JSON key 重复：{key}')
            record[key] = value
        return record

    def bad_constant(value):
        raise ValueError(f'JSON 数值：{value}')

    documents = {}
    for name, path in [('district', DISTRICT), *FILES.items()]:
        try:
            value = json.loads((root / path).read_text(encoding='utf-8'),
                               object_pairs_hook=pairs, parse_constant=bad_constant)
            if not isinstance(value, dict):
                raise ValueError('顶层使用 JSON object')
            documents[name] = value
        except (OSError, UnicodeError, ValueError) as exc:
            add('read', str(path), str(exc))
    if issues:
        return result

    def records(document, field, id_field='id', pattern=r'.+'):
        values = documents[document].get(field)
        indexed = {}
        if not isinstance(values, list):
            add('field', f'{document}.{field}', '字段使用 object 数组')
            return indexed
        for number, item in enumerate(values):
            location = f'{document}.{field}[{number}]'
            if not isinstance(item, dict):
                add('field', location, '条目使用 object')
                continue
            rid = item.get(id_field)
            if not isinstance(rid, str) or not re.fullmatch(pattern, rid):
                add('id', location, f'ID 格式错误：{rid}')
            elif rid in indexed:
                add('duplicate', location, f'ID 重复：{rid}')
            else:
                indexed[rid] = item
        return indexed

    places = records('district', 'places')
    blocks = records('district', 'blocks')
    buildings = records('district', 'buildings')
    catalogue = records('place-catalog', 'places', 'place_id')
    overlay = records('city-story-map', 'places', 'place_id')
    characters = records('characters', 'characters', pattern=r'CHR-\d{3,}')
    quests = records('quests', 'quests', pattern=r'QST-\d{3,}')
    result['counts'] = {'places': len(places), 'blocks': len(blocks),
                        'characters': len(characters), 'quests': len(quests)}
    if issues:
        return result

    def text(value, location):
        if not isinstance(value, str) or not value.strip():
            add('field', location, '字段使用有内容的字符串')

    def references(values, targets, location):
        if not isinstance(values, list) or any(not isinstance(v, str) for v in values):
            add('field', location, '字段使用字符串数组')
            return
        if len(values) != len(set(values)):
            add('duplicate', location, '数组项重复')
        for value in values:
            if value not in targets:
                add('reference', location, f'引用目标不存在：{value}')

    if documents['place-catalog'].get('blocks') != {k: b.get('name') for k, b in blocks.items()}:
        add('catalogue', 'place-catalog.blocks', '街坊 ID 和名称与 district.json 不一致')
    for label, index in [('place-catalog', catalogue), ('city-story-map', overlay)]:
        if index.keys() != places.keys():
            add('catalogue', label, f'场所覆盖不同：缺少 {sorted(places.keys() - index.keys())}；'
                f'多出 {sorted(index.keys() - places.keys())}')
        for pid, item in index.items():
            if pid not in places:
                continue
            source = places[pid]
            for field, source_field in [('name', 'name'), ('block_id', 'block'), ('building_id', 'building')]:
                if field not in item or item[field] != source.get(source_field):
                    add('catalogue', f'{label}.{pid}.{field}', '与 district.json 不一致')
            text(item.get('kind'), f'{label}.{pid}.kind')
    for pid in catalogue.keys() & overlay.keys():
        # kind 是作者补充分型；地图中的 space 和 role 保留各自含义
        if catalogue[pid].get('kind') != overlay[pid].get('kind'):
            add('catalogue', f'city-story-map.{pid}.kind', '与 place-catalog 分型不一致')
    building_ids = set()
    for pid, place in places.items():
        references([place.get('block')], blocks, f'district.{pid}.block')
        if place.get('building') is not None:
            references([place['building']], buildings, f'district.{pid}.building')
            if isinstance(place['building'], str):
                building_ids.add(place['building'])
    result['counts']['referenced_buildings'] = len(building_ids)
    # 这四个场所共用既有 V-15 体量，新增叙事不拆分物理建筑
    for pid in ('15', '71', '72', '75'):
        if places.get(pid, {}).get('building') != 'V-15':
            add('shared_building', f'district.{pid}', '影院、餐饮、屋顶公园与工作室共用 V-15')

    for cid, character in characters.items():
        for field in ('name', 'role', 'personal_goal', 'arc', 'relationships', 'voice'):
            text(character.get(field), f'{cid}.{field}')
        references(character.get('associated_place_ids'), places, f'{cid}.associated_place_ids')
        if character.get('home_place_id') is not None:
            references([character['home_place_id']], places, f'{cid}.home_place_id')
        source = character.get('source_file')
        if not isinstance(source, str) or Path(source).is_absolute():
            add('source', cid, '人物 source_file 使用仓库根相对路径')
            continue
        path = (root / source).resolve()
        if not path.is_relative_to((root / CHARACTERS).resolve()) or path.suffix != '.md':
            add('source', cid, '人物权威档案位于 docs/player/characters/ 内')
            continue
        try:
            if character_subject(path.read_text(encoding='utf-8')) != cid:
                add('source', cid, '人物档案 ID 与索引不一致')
        except (OSError, UnicodeError) as exc:
            add('source', cid, str(exc))
    # 百科角色正文是身份权威；开发规格可引用同一 subject_id
    canonical = {(root / item['source_file']).resolve() for item in characters.values()
                 if isinstance(item.get('source_file'), str)}
    for path in (root / CHARACTERS).rglob('*.md'):
        try:
            identity = character_subject(path.read_text(encoding='utf-8'))
            if identity and path.resolve() not in canonical:
                add('source', str(path.relative_to(root)), '角色权威正文未登记在 characters.json，不能用开发规格替代')
        except (OSError, UnicodeError) as exc:
            add('source', str(path.relative_to(root)), str(exc))
    for pid, place in overlay.items():
        for field in ('narrative_role', 'daily_activity', 'connection_reason', 'aftermath', 'access'):
            text(place.get(field), f'place.{pid}.{field}')
        for field, targets in [('character_ids', characters), ('quest_ids', quests), ('linked_place_ids', places)]:
            references(place.get(field), targets, f'place.{pid}.{field}')

    docs = (root / DOCS).resolve()
    sources = {}
    for qid, quest in quests.items():
        text(quest.get('title'), f'{qid}.title')
        references(quest.get('associated_place_ids'), places, f'{qid}.associated_place_ids')
        references(quest.get('associated_character_ids'), characters, f'{qid}.associated_character_ids')
        diver, receiver = quest.get('diver'), quest.get('reality_receiver')
        count = quest.get('max_simultaneous_sibling_divers')
        allowed_pairs = [('CHR-001', 'CHR-002'), ('CHR-002', 'CHR-001'),
                         ('player_choice_single_sibling', 'other_sibling_awake')]
        if type(count) is not int or count not in (0, 1) or (
                count == 0 and (diver is not None or receiver is not None)) or (
                count == 1 and (diver, receiver) not in allowed_pairs):
            add('diver_roles', qid, '无潜梦时分工为空；潜梦时一名兄妹入梦，另一名清醒接应')
        if count == 1:
            references(['CHR-001', 'CHR-002'], characters, f'{qid}.sibling_roles')
        source = quest.get('source_file')
        if not isinstance(source, str) or Path(source).is_absolute():
            add('source', qid, 'source_file 使用docs/ 内 Markdown 相对路径')
            continue
        path = (docs / source).resolve()
        if not path.is_relative_to((root / STORY).resolve()) or path.suffix != '.md':
            add('source', qid, f'故事正文路径超出 docs/player/story/ 或不是 Markdown：{source}')
            continue
        try:
            if source not in sources:
                sources[source] = path.read_text(encoding='utf-8')
            content = sources[source]
        except (OSError, UnicodeError) as exc:
            add('source', qid, str(exc))
            continue
        mentioned = set(re.findall(r'\b(?:QST|BC)-\d{3,}\b', content))
        references([qid], mentioned, f'{qid}.source_file')
        references(quest.get('black_cases'), {x for x in mentioned if x.startswith('BC-')}, f'{qid}.black_cases')
    result['ok'] = not issues
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('.'))
    parser.add_argument('--json', action='store_true')
    args = parser.parse_args()
    result = validate(args.root.resolve())
    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        counts = ', '.join(f'{v} {k}' for k, v in result['counts'].items())
        print(f"{'PASS' if result['ok'] else 'FAIL'}: {counts}")
        for issue in result['issues']:
            print(f"{issue['location']}: [{issue['code']}] {issue['message']}")
    return 0 if result['ok'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
