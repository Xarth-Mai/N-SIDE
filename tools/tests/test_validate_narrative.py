"""Executable examples for the N:SIDE narrative design graph."""
from __future__ import annotations

import copy
import csv
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[1] / 'validate_narrative.py'
SPEC = importlib.util.spec_from_file_location('validate_narrative', TOOL)
assert SPEC and SPEC.loader
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)
Q = 'QST-001'
I = Q + '-I001'
B1, B2, B3, B4 = [Q + '-B' + f'{i:03d}' for i in range(1, 5)]


def beat(bid, *, after=None, delivers=None, requires=None):
    return {'id': bid, 'dramatic_change': 'A clue changes the explanation.',
            'player_goal': 'Inspect the clue.', 'player_action': 'Inspect and compare.',
            'requires': requires or {}, 'delivers': delivers or {}, 'state_changes': {},
            'next': after or [], 'target_seconds': None}


def example():
    return {'schema_version': 1, 'quest_id': Q, 'entry_beat': B1, 'terminal_beats': [B2],
            'information': [{'id': I, 'proposition': 'The item was packed.',
                             'world_truth': 'The dated seal shows packing.',
                             'importance': 'critical', 'initial_state': 'hidden'}],
            'beats': [beat(B1, after=[B2], delivers={I: 'confirmed'}),
                      beat(B2, requires={I: ['confirmed']})],
            'choices': [], 'scenes': []}


class NarrativeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.data = example()

    def check(self, **kwargs):
        return M.validate_data(self.data, self.root, **kwargs)

    def codes(self, **kwargs):
        return {x['code'] for x in self.check(**kwargs)['issues']}

    def write(self, relative='narrative.json', data=None):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(self.data if data is None else data), encoding='utf-8')
        return path

    def branch(self):
        self.data['beats'] = [beat(B1, after=[B2, B3]),
                              beat(B2, after=[B4], delivers={I: 'confirmed'}),
                              beat(B3, after=[B4]),
                              beat(B4, requires={I: ['confirmed']})]
        self.data['terminal_beats'] = [B4]

    def add_scene(self):
        self.data['scenes'] = [{'id': Q + '-S001', 'beat_id': B1,
                               'intent': 'Compare evidence.', 'entry_state': 'Investigating.',
                               'exit_state': 'A new lead.', 'cast': ['CHR-001'],
                               'line_ids': [Q + '-L001']}]

    def write_dialogue(self, speaker='CHR-001', duplicate=False):
        with (self.root / 'dialogue.csv').open('w', encoding='utf-8', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['line_id', 'speaker_id', 'text_zh_cn', 'context', 'recording_status', 'audio_path'])
            row = [Q + '-L001', speaker, '封条日期在这里。', 'Inspect seal.', 'temp', '']
            writer.writerow(row)
            if duplicate:
                writer.writerow(row)

    def test_linear(self):
        r = self.check()
        self.assertTrue(r['ok'], r)
        self.assertEqual(r['states_checked'], 2)

    def test_initial_knowledge(self):
        self.data['information'][0]['initial_state'] = 'confirmed'
        self.data['beats'][0]['delivers'] = {}
        self.assertTrue(self.check()['ok'])

    def test_late_delivery(self):
        self.data['beats'][0]['delivers'] = {}
        self.data['beats'][1]['delivers'] = {I: 'confirmed'}
        self.assertIn('knowledge_order', self.codes())

    def test_requires_before_delivery(self):
        self.data['beats'][0]['requires'] = {I: ['confirmed']}
        self.assertIn('knowledge_order', self.codes())

    def test_branch_bypasses_clue(self):
        self.branch()
        r = self.check()
        issue = next(x for x in r['issues'] if x['code'] == 'knowledge_order')
        self.assertEqual(issue['route'], [B1, B3, B4])

    def test_recovery_branch(self):
        self.branch()
        self.data['beats'][2]['delivers'] = {I: 'confirmed'}
        self.assertTrue(self.check()['ok'])

    def test_multiple_accepted_states(self):
        self.data['beats'][0]['delivers'] = {I: 'suspected'}
        self.data['beats'][1]['requires'] = {I: ['suspected', 'confirmed']}
        self.assertTrue(self.check()['ok'])

    def test_reinterpretation(self):
        self.data['beats'][0]['delivers'] = {I: 'disproved'}
        self.data['beats'][1]['requires'] = {I: ['disproved']}
        self.assertTrue(self.check()['ok'])

    def test_loop_terminates_analysis(self):
        self.data['beats'][0]['next'] = [B1, B2]
        r = self.check()
        self.assertTrue(r['ok'])
        self.assertEqual(r['states_checked'], 3)

    def test_loop_overwrites_knowledge(self):
        self.data['terminal_beats'] = [B3]
        self.data['beats'][1]['next'] = [B2, B3]
        self.data['beats'][1]['delivers'] = {I: 'disproved'}
        self.data['beats'].append(beat(B3))
        r = self.check()
        self.assertIn('knowledge_order', {x['code'] for x in r['issues']})
        self.assertTrue(any(x.get('actual') == 'disproved' for x in r['issues']))

    def test_unreachable_node(self):
        self.data['beats'].append(beat(B3))
        self.data['terminal_beats'].append(B3)
        self.assertIn('unreachable', self.codes())

    def test_cycle_without_ending_route(self):
        self.data['beats'][0]['next'] = [B1]
        self.assertIn('no_ending', self.codes())

    def test_terminal_has_next(self):
        self.data['beats'][1]['next'] = [B1]
        self.assertIn('terminal', self.codes())

    def test_implicit_terminal(self):
        self.data['beats'][0]['next'] = []
        self.assertIn('terminal', self.codes())

    def test_missing_node(self):
        self.data['beats'][0]['next'] = [B3]
        self.assertIn('reference', self.codes())

    def test_missing_information_reference(self):
        self.data['beats'][0]['delivers'] = {Q + '-I999': 'confirmed'}
        self.assertIn('reference', self.codes())

    def test_duplicate_id(self):
        self.data['beats'].append(copy.deepcopy(self.data['beats'][0]))
        self.assertIn('duplicate', self.codes())

    def test_foreign_namespace(self):
        self.data['beats'][0]['id'] = 'QST-002-B001'
        self.assertIn('id', self.codes())

    def test_required_info_has_use(self):
        self.data['beats'][1]['requires'] = {}
        self.assertIn('critical_use', self.codes())

    def test_optional_info_has_no_use(self):
        self.data['information'][0]['importance'] = 'optional'
        self.data['beats'][1]['requires'] = {}
        self.assertTrue(self.check()['ok'])

    def test_valid_choice(self):
        self.branch()
        self.data['beats'][2]['delivers'] = {I: 'confirmed'}
        self.data['choices'] = [{'id': Q + '-C001', 'beat_id': B1,
                                'options': [{'id': 'inspect', 'to': B2, 'acknowledgement': 'Inspect animation.', 'consequence': 'Clue acquired.'},
                                            {'id': 'ask', 'to': B3, 'acknowledgement': 'Spoken reply.', 'consequence': 'Clue acquired.'}]}]
        self.assertTrue(self.check()['ok'])

    def test_choice_edge(self):
        self.data['choices'] = [{'id': Q + '-C001', 'beat_id': B1,
                                'options': [{'id': 'a', 'to': B1, 'acknowledgement': 'A', 'consequence': 'A'},
                                            {'id': 'b', 'to': B2, 'acknowledgement': 'B', 'consequence': 'B'}]}]
        self.assertIn('choice_edge', self.codes())

    def test_choice_acknowledgement(self):
        self.data['choices'] = [{'id': Q + '-C001', 'beat_id': B1,
                                'options': [{'id': 'a', 'to': B2, 'acknowledgement': '', 'consequence': 'A'},
                                            {'id': 'b', 'to': B2, 'acknowledgement': 'B', 'consequence': 'B'}]}]
        self.assertIn('field', self.codes())

    def test_valid_scene_and_line(self):
        self.add_scene()
        self.write_dialogue()
        self.assertTrue(self.check()['ok'])

    def test_narrator_line(self):
        self.add_scene()
        self.write_dialogue(speaker='NARRATOR')
        self.assertTrue(self.check()['ok'])

    def test_absent_speaker(self):
        self.add_scene()
        self.write_dialogue(speaker='CHR-002')
        self.assertIn('speaker', self.codes())

    def test_missing_dialogue(self):
        self.add_scene()
        self.assertIn('dialogue_read', self.codes())

    def test_missing_line(self):
        self.add_scene()
        self.write_dialogue()
        self.data['scenes'][0]['line_ids'] = [Q + '-L002']
        self.assertIn('dialogue_reference', self.codes())

    def test_duplicate_line(self):
        self.add_scene()
        self.write_dialogue(duplicate=True)
        self.assertIn('duplicate', self.codes())

    def test_scene_reference(self):
        self.add_scene()
        self.data['scenes'][0]['line_ids'] = []
        self.data['scenes'][0]['beat_id'] = B3
        self.assertIn('reference', self.codes())

    def test_scene_empty_cast_and_no_lines(self):
        self.add_scene()
        self.data['scenes'][0]['line_ids'] = []
        self.data['scenes'][0]['cast'] = []
        self.assertTrue(self.check()['ok'])

    def test_budget_reports_incomplete(self):
        r = self.check(max_states=1)
        self.assertFalse(r['ok'])
        self.assertFalse(r['complete'])
        self.assertIn('incomplete', {x['code'] for x in r['issues']})

    def test_exact_budget_finishes(self):
        r = self.check(max_states=2)
        self.assertTrue(r['complete'])
        self.assertTrue(r['ok'])

    def test_invalid_budget(self):
        self.assertIn('budget', self.codes(max_states=0))

    def test_malformed_shapes(self):
        cases = [('information', {}), ('beats', [None]), ('choices', None),
                 ('scenes', [False]), ('terminal_beats', {}), ('entry_beat', [])]
        for field, value in cases:
            with self.subTest(field=field):
                self.data = example()
                self.data[field] = value
                self.assertFalse(self.check()['ok'])

    def test_bad_nested_values(self):
        for field, value in [('requires', {I: [None]}), ('requires', {I: []}),
                             ('delivers', {I: []}), ('next', [None]),
                             ('target_seconds', True), ('state_changes', [])]:
            with self.subTest(field=field, value=value):
                self.data = example()
                self.data['beats'][0][field] = value
                self.assertFalse(self.check()['ok'])

    def test_version_type(self):
        self.data['schema_version'] = True
        self.assertIn('version', self.codes())

    def test_parse_failure(self):
        path = self.root / 'narrative.json'
        for value in ('{broken', '{"schema_version": 1, "schema_version": 2}', '{"x":NaN}'):
            path.write_text(value)
            with self.subTest(value=value):
                r = M.validate_file(path)
                self.assertFalse(r['ok'])
                self.assertEqual(r['issues'][0]['code'], 'read')

    def test_file_interface(self):
        path = self.write()
        self.assertTrue(M.validate_file(path)['ok'])

    def test_cli_empty_project(self):
        p = subprocess.run([sys.executable, '-B', str(TOOL), '--root', str(self.root)], capture_output=True, text=True)
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn('EMPTY: 0', p.stdout)

    def test_cli_project_and_json(self):
        self.write(f'docs/quests/{Q}/narrative.json')
        p = subprocess.run([sys.executable, '-B', str(TOOL), '--root', str(self.root), '--json'], capture_output=True, text=True)
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual(json.loads(p.stdout)['files'], 1)

    def test_duplicate_quest_files(self):
        self.write('docs/quests/a/narrative.json')
        self.write('docs/quests/b/narrative.json')
        p = subprocess.run([sys.executable, '-B', str(TOOL), '--root', str(self.root), '--json'], capture_output=True, text=True)
        self.assertEqual(p.returncode, 1, p.stderr)
        results = json.loads(p.stdout)['results']
        self.assertTrue(any(x['code'] == 'duplicate_quest' for r in results for x in r['issues']))

    def test_cli_bad_argument(self):
        p = subprocess.run([sys.executable, '-B', str(TOOL), '--root', str(self.root), '--max-states', '0'], capture_output=True, text=True)
        self.assertEqual(p.returncode, 2)

    def test_template(self):
        template = Path(__file__).resolve().parents[2] / 'docs/templates/narrative.json'
        self.assertTrue(M.validate_file(template)['ok'])


if __name__ == '__main__':
    unittest.main()
