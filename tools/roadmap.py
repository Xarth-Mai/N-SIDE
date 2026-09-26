#!/usr/bin/env python3
"""Validate the N:SIDE progress ledger and render its deterministic Markdown view.

The ledger is edited deliberately by a person or an agent. This tool does not
approve designs, run the game, or mutate source progress/acceptance records.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import sys
from typing import Any

DATA = Path('todo/demo-progress.json')
REPORT = Path('todo/demo-progress.md')
STATES = {'backlog', 'ready', 'in_progress', 'blocked', 'done', 'cancelled'}
KINDS = {'decision': '决策', 'loop': '迭代', 'delivery': '制作', 'gate': '阶段验收'}
ROLES = {'author': '作者', 'codex': 'Codex 技术核验', 'playtester': '实际试玩者'}
LABELS = {'backlog': '待安排', 'ready': '可开始', 'in_progress': '进行中',
          'blocked': '受阻', 'done': '已验收', 'cancelled': '已批准移出范围'}
STEPS = {'orient': '定位与讲解', 'design': '决策与设计', 'build': '制作',
         'verify': '客观检查', 'review': '体验／作者验收', 'record': '记录与下一步'}
RESOLVED = {'done', 'cancelled'}


def text(value: Any, where: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f'{where}: 需要非空文本')
    return value


def ref_path(ref: str, root: Path) -> Path:
    """Evidence uses repository-relative files; explicit anchors are checked."""
    text(ref, 'reference')
    part, sep, anchor = ref.partition('#')
    if not part or '\\' in part or ':' in part or Path(part).is_absolute():
        raise ValueError(f'引用不是仓库相对文件: {ref}')
    path = (root / part).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f'引用越出仓库: {ref}')
    if not path.is_file():
        raise ValueError(f'证据文件不存在: {ref}')
    if sep:
        if not anchor or not re.search(
            r'<a\s+id=[\"\']' + re.escape(anchor) + r'[\"\']\s*>',
            path.read_text(encoding='utf-8'),
        ):
            raise ValueError(f'证据缺少显式锚点: {ref}')
    return path


def acceptance(record: Any, role: str, where: str, root: Path) -> None:
    if not isinstance(record, dict):
        raise ValueError(f'{where}: 缺少验收／范围批准记录')
    if record.get('role') != role:
        raise ValueError(f'{where}: 需要 {role} 的实际记录')
    for key in ('by', 'record', 'revision', 'result'):
        text(record.get(key), f'{where}.{key}')
    if record['result'] != 'pass':
        raise ValueError(f'{where}: 未通过的记录不能结项')
    if not re.fullmatch(r'[0-9a-f]{7,64}', record['revision']):
        raise ValueError(f'{where}: revision 应为提交／输入校验和')
    ref_path(record['record'], root)


def validate(data: Any, root: Path) -> dict[str, dict[str, Any]]:
    if not isinstance(data, dict) or data.get('schema_version') != 1:
        raise ValueError('不支持的进度数据格式')
    if type(data.get('plan_revision')) is not int or data['plan_revision'] < 1:
        raise ValueError('plan_revision 应为正整数')
    if not re.fullmatch(r'[0-9a-f]{40}', text(data.get('baseline_commit'), 'baseline_commit')):
        raise ValueError('baseline_commit 应为完整提交 SHA')
    ref_path(data.get('source_roadmap'), root)
    text(data.get('note'), 'note')
    related = data.get('related_tasks')
    if not isinstance(related, list) or len(set(related)) != len(related):
        raise ValueError('related_tasks 应为不重复的路径数组')
    for ref in related:
        ref_path(ref, root)
    stages = data.get('stages')
    if not isinstance(stages, list) or [s.get('id') for s in stages] != [f'M{i}' for i in range(8)]:
        raise ValueError('阶段应保持 M0—M7 顺序')
    items: dict[str, dict[str, Any]] = {}
    for stage in stages:
        text(stage.get('title'), f"{stage['id']}.title")
        text(stage.get('learn'), f"{stage['id']}.learn")
        rows = stage.get('items')
        if not isinstance(rows, list) or len(rows) < 2:
            raise ValueError(f"{stage['id']}: 缺少工作包和阶段验收")
        for row in rows:
            ident = text(row.get('id'), 'item.id')
            if not re.fullmatch(stage['id'] + r'-[0-9]{2}', ident) or ident in items:
                raise ValueError(f'重复或不合法的工作包 ID: {ident}')
            items[ident] = row
            for key in ('title', 'decide', 'deliver', 'verify'):
                text(row.get(key), f'{ident}.{key}')
            if row.get('kind') not in KINDS or row.get('status') not in STATES or row.get('reviewer') not in ROLES:
                raise ValueError(f'{ident}: 未知类型、状态或验收角色')
            if row['kind'] == 'gate' and row['reviewer'] != 'author':
                raise ValueError(f'{ident}: 阶段放行由作者确认')
            deps = row.get('depends_on')
            if not isinstance(deps, list) or not all(isinstance(d, str) for d in deps) or len(set(deps)) != len(deps):
                raise ValueError(f'{ident}: 依赖应为不重复的 ID 数组')
            evidence = row.get('evidence', [])
            if not isinstance(evidence, list) or not all(isinstance(e, str) for e in evidence):
                raise ValueError(f'{ident}: evidence 应为路径数组')
            for ref in evidence:
                ref_path(ref, root)
            if row['status'] == 'done':
                if not evidence:
                    raise ValueError(f'{ident}: 已验收项必须有证据')
                acceptance(row.get('accepted'), row['reviewer'], f'{ident}.accepted', root)
            elif row.get('accepted') is not None:
                raise ValueError(f'{ident}: 重开项将旧验收留在日志，不保留有效 accepted')
            if row['status'] == 'cancelled':
                if row['kind'] == 'gate':
                    raise ValueError(f'{ident}: 阶段验收不能取消')
                acceptance(row.get('scope_change'), 'author', f'{ident}.scope_change', root)
                text(row.get('reason'), f'{ident}.reason')
            if row['status'] == 'blocked':
                text(row.get('reason'), f'{ident}.reason')
            work = row.get('work')
            if work is not None:
                if row['status'] not in {'in_progress', 'blocked', 'done'}:
                    raise ValueError(f'{ident}: 当前轮次与状态不一致')
                if not isinstance(work, dict) or type(work.get('round')) is not int or work['round'] < 1 or work.get('step') not in STEPS:
                    raise ValueError(f'{ident}: 非法轮次或 loop 步骤')
                if row['status'] == 'done' and work['step'] != 'record':
                    raise ValueError(f'{ident}: 结项轮次应停在 record')
                ref_path(work.get('record'), root)
                text(work.get('next_action'), f'{ident}.work.next_action')
                if work['step'] == 'review' and not evidence:
                    raise ValueError(f'{ident}: 提交人工验收前需要交付证据')
        if sum(r['kind'] == 'gate' for r in rows) != 1 or rows[-1]['kind'] != 'gate':
            raise ValueError(f"{stage['id']}: 最后一项应为唯一阶段验收")
        if set(rows[-1]['depends_on']) != {r['id'] for r in rows[:-1]}:
            raise ValueError(f"{stage['id']}: 阶段验收必须覆盖本阶段全部工作包")
    for ident, row in items.items():
        for dep in row['depends_on']:
            if dep not in items:
                raise ValueError(f'{ident}: 未知依赖 {dep}')
        if row['status'] in {'ready', 'in_progress', 'done'}:
            pending = [d for d in row['depends_on'] if items[d]['status'] not in RESOLVED]
            if pending:
                raise ValueError(f"{ident}: 前置尚未通过 {', '.join(pending)}；先处理依赖或标记 blocked")
    visiting, visited = set(), set()
    def visit(ident: str) -> None:
        if ident in visiting:
            raise ValueError(f'依赖形成环: {ident}')
        if ident in visited:
            return
        visiting.add(ident)
        for dep in items[ident]['depends_on']:
            visit(dep)
        visiting.remove(ident)
        visited.add(ident)
    for ident in items:
        visit(ident)
    if data.get('focus') not in items:
        raise ValueError('focus 未引用有效工作包')
    return items


def effective(row: dict[str, Any], items: dict[str, dict[str, Any]]) -> str:
    if row['status'] == 'backlog' and all(items[d]['status'] in RESOLVED for d in row['depends_on']):
        return 'ready'
    return row['status']


def summary(data: dict[str, Any], items: dict[str, dict[str, Any]]) -> dict[str, Any]:
    stats = []
    for s in data['stages']:
        rows = s['items']
        stats.append({'id': s['id'], 'title': s['title'], 'done': sum(r['status'] == 'done' for r in rows),
                      'total': len(rows), 'cancelled': sum(r['status'] == 'cancelled' for r in rows),
                      'gate_passed': rows[-1]['status'] == 'done',
                      'active': [r['id'] for r in rows if r['status'] == 'in_progress'],
                      'blocked': [r['id'] for r in rows if r['status'] == 'blocked']})
    candidates = [ident for ident, row in items.items() if effective(row, items) in {'ready', 'in_progress'}]
    focus = data['focus']
    active = [ident for ident in candidates if items[ident]['status'] == 'in_progress']
    pool = active or candidates
    next_item = focus if focus in pool else (pool[0] if pool else None)
    return {'stages': stats, 'passed_stages': sum(s['gate_passed'] for s in stats),
            'focus': focus, 'next': next_item,
            'ready_or_active': candidates}


def link(ref: str, label: str | None = None) -> str:
    path, sep, anchor = ref.partition('#')
    relative = os.path.relpath(path, 'todo').replace(os.sep, '/')
    target = relative + (f'#{anchor}' if sep else '')
    return f'[{label or ref}]({target})'


def render_stage(stage: dict[str, Any], items: dict[str, dict[str, Any]]) -> str:
    rows = stage['items']
    done = sum(r['status'] == 'done' for r in rows)
    out = [f'<a id="{stage["id"].lower()}"></a>', f'## {stage["id"]} {stage["title"]}｜已验收 {done}/{len(rows)}',
           '', f'先理解：{stage["learn"]}', '']
    for index, row in enumerate(rows, 1):
        out += [f'<a id="{row["id"].lower()}"></a>',
                f'### {row["id"]} {row["title"]}（清单第 {index}/{len(rows)} 项）', '',
                f'状态：**{LABELS[effective(row, items)]}**；类型：{KINDS[row["kind"]]}；验收：{ROLES[row["reviewer"]]}',
                f'前置：{", ".join(row["depends_on"]) or "无"}', '',
                f'**需要决定／设计：** {row["decide"]}', '',
                f'**Codex 交付：** {row["deliver"]}', '', f'**完成条件：** {row["verify"]}', '']
        if row.get('reason'):
            out += [f'当前说明：{row["reason"]}', '']
        work = row.get('work')
        if work:
            pos = list(STEPS).index(work['step']) + 1
            out += [f'本轮：第 {work["round"]} 轮，步骤 {pos}/6「{STEPS[work["step"]]}」',
                    f'下一动作：{work["next_action"]}', f'轮次记录：{link(work["record"])}', '']
        if row.get('evidence'):
            out += ['证据：' + '；'.join(link(e) for e in row['evidence']), '']
        accepted = row.get('accepted') or row.get('scope_change')
        if accepted:
            out += [f'记录：{accepted["by"]}（{ROLES[accepted["role"]]}），输入 `{accepted["revision"]}`；{link(accepted["record"])}', '']
    return '\n'.join(out).rstrip() + '\n'


def render(data: dict[str, Any], items: dict[str, dict[str, Any]]) -> str:
    stats = summary(data, items)
    focus = items[data['focus']]
    stage = next(s for s in data['stages'] if focus in s['items'])
    position = stage['items'].index(focus) + 1
    out = ['<!-- Generated by tools/roadmap.py. Edit demo-progress.json, then run bun run roadmap:sync. -->',
           '# N:SIDE Demo 细化进度', '',
           '来源：[总路线图](demo-roadmap.md) · [进度数据](demo-progress.json) · [个人开发工作流](../docs/production/solo-workflow.md) · [本轮与证据记录](demo-worklog.md)', '',
           f'计划修订：{data["plan_revision"]}；初始化核对基线：`{data["baseline_commit"]}`', '', data['note'], '',
           f'**阶段放行：{stats["passed_stages"]}/8。** 工作包数量不是工时或整体完成百分比', '',
           f'关注项：**{focus["id"]} {focus["title"]}**，清单位置 {position}/{len(stage["items"])}；这是位置，不是已完成数量', '',
           f'下一可推进项：**{stats["next"] or "暂无；需解除阻塞、验收或更新 focus"}**', '',
           '已有城市／Viewer 成果见归档；剩余工作按总路线图承接，不折算为 M0—M7 已完成数量：' + '、'.join(link(p) for p in data['related_tasks']) + '', '',
           '| 阶段 | 已验收 | 进行中 | 受阻 | 批准移出 | 阶段放行 |', '| --- | --- | --- | --- | --- | --- |']
    for s in stats['stages']:
        out.append(f'| [{s["id"]} {s["title"]}](#{s["id"].lower()}) | {s["done"]}/{s["total"]} | {", ".join(s["active"]) or "—"} | {", ".join(s["blocked"]) or "—"} | {s["cancelled"]} | {"通过" if s["gate_passed"] else "未通过"} |')
    out += ['', '计数以通过验收的稳定工作包为单位。`in_progress`、等待作者确认和仅写完文档均不计为完成。Loop 的轮数开放，只显示“第几轮、当前步骤 1/6—6/6”；取消项仍留在原分母，单列范围变化', '']
    for s in data['stages']:
        out += [render_stage(s, items).rstrip(), '']
    return '\n'.join(out).rstrip() + '\n'


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--write', action='store_true', help='只更新生成的 Markdown 看板')
    mode.add_argument('--check', action='store_true', help='检查数据及看板是否同步')
    mode.add_argument('--json', action='store_true', help='输出机器可读的汇总')
    mode.add_argument('--stage', choices=[f'M{i}' for i in range(8)], help='输出指定阶段细项')
    args = parser.parse_args(argv)
    try:
        root = args.root.resolve()
        data = json.loads((root / DATA).read_text(encoding='utf-8'))
        items = validate(data, root)
        report = render(data, items)
        dest = root / REPORT
        if args.write:
            dest.parent.mkdir(parents=True, exist_ok=True)
            temp = dest.with_suffix('.md.tmp')
            temp.write_text(report, encoding='utf-8')
            temp.replace(dest)
            print(f'已生成 {REPORT}；源进度与验收状态未修改')
        elif args.check:
            if not dest.is_file() or dest.read_text(encoding='utf-8') != report:
                raise ValueError('看板缺失或过期；运行 bun run roadmap:sync 后再检查')
            print('PASS: 进度结构、依赖、证据引用与生成看板一致；不代表游戏或人工验收通过')
        elif args.stage:
            print(render_stage(next(s for s in data['stages'] if s['id'] == args.stage), items), end='')
        else:
            result = summary(data, items)
            if args.json:
                print(json.dumps(result, ensure_ascii=False, indent=2))
            else:
                print(f"N:SIDE：阶段已放行 {result['passed_stages']}/8")
                for s in result['stages']:
                    print(f"{s['id']} {s['title']}：已验收 {s['done']}/{s['total']}；进行中 {len(s['active'])}；受阻 {len(s['blocked'])}；移出 {s['cancelled']}；{'阶段通过' if s['gate_passed'] else '未放行'}")
                print(f"关注项：{result['focus']}；下一可推进项：{result['next'] or '暂无'}")
                if result['next']:
                    row = items[result['next']]
                    stage = next(s for s in data['stages'] if row in s['items'])
                    print(f"当前入口：{row['id']} {row['title']}；清单第 {stage['items'].index(row) + 1}/{len(stage['items'])} 项")
                    work = row.get('work')
                    if work:
                        print(f"本轮：第 {work['round']} 轮，步骤 {list(STEPS).index(work['step']) + 1}/6「{STEPS[work['step']]}」；下一动作：{work['next_action']}")
                    else:
                        print('当前没有迭代记录')
                print('查看细项：bun run roadmap --stage M0；查看可读看板：todo/demo-progress.md')
        return 0
    except (OSError, UnicodeError, ValueError, TypeError, KeyError, AttributeError) as exc:
        print(f'ROADMAP ERROR: {exc}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
