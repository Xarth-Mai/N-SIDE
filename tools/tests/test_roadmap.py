"""Synthetic ledger tests; none of these records are real game acceptances."""
import copy
from contextlib import redirect_stderr, redirect_stdout
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('n_side_roadmap', ROOT / 'tools/roadmap.py')
roadmap = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(roadmap)


class RoadmapTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'todo').mkdir()
        (self.root / 'todo/proof.md').write_text('<a id="proof"></a>\nSynthetic test evidence.\n', encoding='utf-8')
        self.data = json.loads((ROOT / 'todo/demo-progress.json').read_text(encoding='utf-8'))
        for ref in [self.data['source_roadmap'], *self.data['related_tasks']]:
            path = self.root / ref
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('# Synthetic source\n', encoding='utf-8')
        # A fixture must not depend on how far the real game has progressed later.
        for stage in self.data['stages']:
            for row in stage['items']:
                row['status'] = 'backlog'
                for key in ('evidence', 'accepted', 'scope_change', 'work', 'reason'):
                    row.pop(key, None)
        self.data['focus'] = 'M0-01'

    def row(self, ident):
        return next(r for s in self.data['stages'] for r in s['items'] if r['id'] == ident)

    def record(self, role):
        return {'role': role, 'by': 'Synthetic reviewer', 'record': 'todo/proof.md#proof',
                'revision': 'abcdef1', 'result': 'pass'}

    def done(self, ident):
        row = self.row(ident)
        row.update(status='done', evidence=['todo/proof.md#proof'], accepted=self.record(row['reviewer']))

    def check(self):
        return roadmap.validate(self.data, self.root)

    def store(self):
        (self.root / roadmap.DATA).write_text(json.dumps(self.data, ensure_ascii=False), encoding='utf-8')

    def cli(self, *args):
        output, errors = io.StringIO(), io.StringIO()
        with redirect_stdout(output), redirect_stderr(errors):
            code = roadmap.main(['--root', str(self.root), *args])
        return code, output.getvalue(), errors.getvalue()

    def test_initial_counts_and_next_are_not_claimed_completion(self):
        stats = roadmap.summary(self.data, self.check())
        self.assertEqual(stats['passed_stages'], 0)
        self.assertEqual(sum(s['done'] for s in stats['stages']), 0)
        self.assertEqual([s['total'] for s in stats['stages']], [8, 8, 8, 8, 8, 8, 7, 6])
        self.assertEqual(stats['next'], 'M0-01')

    def test_accepted_item_counts_once_not_stage(self):
        self.done('M0-01')
        stats = roadmap.summary(self.data, self.check())
        self.assertEqual(stats['stages'][0]['done'], 1)
        self.assertFalse(stats['stages'][0]['gate_passed'])

    def test_done_requires_evidence(self):
        self.row('M0-01')['status'] = 'done'
        with self.assertRaisesRegex(ValueError, '必须有证据'):
            self.check()

    def test_done_requires_correct_reviewer(self):
        self.done('M0-01')
        self.row('M0-01')['accepted']['role'] = 'codex'
        with self.assertRaisesRegex(ValueError, '需要 author'):
            self.check()

    def test_missing_evidence_path_is_rejected(self):
        self.done('M0-01')
        self.row('M0-01')['evidence'] = ['todo/missing.md']
        with self.assertRaisesRegex(ValueError, '文件不存在'):
            self.check()

    def test_missing_roadmap_or_related_task_is_rejected(self):
        for field in ('source_roadmap', 'related_tasks'):
            with self.subTest(field=field):
                original = self.data[field]
                self.data[field] = 'todo/missing.md' if field == 'source_roadmap' else ['todo/missing.md']
                with self.assertRaisesRegex(ValueError, '文件不存在'):
                    self.check()
                self.data[field] = original

    def test_missing_explicit_anchor_is_rejected(self):
        self.done('M0-01')
        self.row('M0-01')['evidence'] = ['todo/proof.md#wrong']
        with self.assertRaisesRegex(ValueError, '显式锚点'):
            self.check()

    def test_path_escape_and_absolute_paths_are_rejected(self):
        for ref in ('../../outside', '/etc/passwd', 'https://example.com/proof'):
            with self.subTest(ref=ref), self.assertRaises(ValueError):
                roadmap.ref_path(ref, self.root)

    def test_symlink_escape_is_rejected(self):
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / 'proof.md'
            target.write_text('outside')
            (self.root / 'todo/escape.md').symlink_to(target)
            with self.assertRaisesRegex(ValueError, '越出仓库'):
                roadmap.ref_path('todo/escape.md', self.root)

    def test_review_and_iterations_do_not_count_as_done(self):
        self.row('M0-01').update(status='in_progress', evidence=['todo/proof.md#proof'],
            work={'round': 3, 'step': 'review', 'record': 'todo/proof.md#proof', 'next_action': 'Actual author review'})
        items = self.check()
        self.assertEqual(roadmap.summary(self.data, items)['stages'][0]['done'], 0)
        rendered = roadmap.render(self.data, items)
        self.assertIn('第 3 轮，步骤 5/6', rendered)
        self.assertIn('清单第 1/8 项', rendered)

    def test_review_without_delivery_evidence_is_rejected(self):
        self.row('M0-01').update(status='in_progress', work={'round': 1, 'step': 'review',
             'record': 'todo/proof.md#proof', 'next_action': 'Review'})
        with self.assertRaisesRegex(ValueError, '需要交付证据'):
            self.check()

    def test_loop_step_validation(self):
        self.row('M0-01').update(status='in_progress', work={'round': 0, 'step': 'unknown',
             'record': 'todo/proof.md#proof', 'next_action': 'Review'})
        with self.assertRaisesRegex(ValueError, '非法轮次'):
            self.check()

    def test_cli_shows_active_round_without_counting_it_as_done(self):
        self.row('M0-01').update(status='in_progress', evidence=['todo/proof.md#proof'],
            work={'round': 2, 'step': 'review', 'record': 'todo/proof.md#proof', 'next_action': 'Actual author review'})
        self.store()
        code, output, errors = self.cli()
        self.assertEqual(code, 0, errors)
        self.assertIn('已验收 0/8；进行中 1；受阻 0', output)
        self.assertIn('清单第 1/8 项', output)
        self.assertIn('第 2 轮，步骤 5/6', output)
        self.assertIn('Actual author review', output)

    def test_duplicate_id_is_rejected(self):
        self.row('M0-02')['id'] = 'M0-01'
        with self.assertRaisesRegex(ValueError, '重复'):
            self.check()

    def test_unknown_dependency_is_rejected(self):
        self.row('M0-01')['depends_on'] = ['M9-01']
        with self.assertRaisesRegex(ValueError, '未知依赖'):
            self.check()

    def test_dependency_cycle_is_rejected(self):
        self.row('M0-01')['depends_on'] = ['M0-02']
        with self.assertRaisesRegex(ValueError, '依赖形成环'):
            self.check()

    def test_ready_with_unmet_dependencies_is_rejected(self):
        self.row('M1-01')['status'] = 'ready'
        with self.assertRaisesRegex(ValueError, '前置尚未通过'):
            self.check()

    def test_stage_gate_must_cover_every_item(self):
        self.row('M0-08')['depends_on'].remove('M0-07')
        with self.assertRaisesRegex(ValueError, '覆盖本阶段全部'):
            self.check()

    def test_cancelled_is_not_completion_or_removed_from_denominator(self):
        self.row('M0-01').update(status='cancelled', reason='Synthetic approved scope change',
             scope_change=self.record('author'))
        stats = roadmap.summary(self.data, self.check())
        self.assertEqual((stats['stages'][0]['done'], stats['stages'][0]['total'], stats['stages'][0]['cancelled']), (0, 8, 1))
        self.assertEqual(stats['next'], 'M0-02')

    def test_cancellation_needs_actual_author_record(self):
        self.row('M0-01').update(status='cancelled', reason='Scope changed')
        with self.assertRaisesRegex(ValueError, '批准记录'):
            self.check()

    def test_cannot_cancel_stage_gate(self):
        self.row('M0-08').update(status='cancelled', reason='Invalid', scope_change=self.record('author'))
        with self.assertRaisesRegex(ValueError, '阶段验收不能取消'):
            self.check()

    def test_stage_can_pass_after_approved_scope_change_without_fake_count(self):
        self.row('M0-01').update(status='cancelled', reason='Synthetic scope change', scope_change=self.record('author'))
        for row in self.data['stages'][0]['items'][1:]:
            self.done(row['id'])
        stats = roadmap.summary(self.data, self.check())
        self.assertTrue(stats['stages'][0]['gate_passed'])
        self.assertEqual(stats['stages'][0]['done'], 7)
        self.assertEqual(stats['stages'][0]['total'], 8)

    def test_reopening_removes_current_acceptance(self):
        self.done('M0-01')
        self.row('M0-01')['status'] = 'in_progress'
        with self.assertRaisesRegex(ValueError, '重开项'):
            self.check()

    def test_downstream_acceptance_cannot_survive_invalidated_dependency(self):
        self.done('M0-01')
        self.done('M0-02')
        self.row('M0-01').update(status='in_progress')
        del self.row('M0-01')['accepted']
        with self.assertRaisesRegex(ValueError, '前置尚未通过'):
            self.check()

    def test_m2_and_m3_have_parallel_ready_items(self):
        for stage in self.data['stages'][:2]:
            for row in stage['items']:
                self.done(row['id'])
        ready = roadmap.summary(self.data, self.check())['ready_or_active']
        self.assertIn('M2-01', ready)
        self.assertIn('M3-02', ready)
        self.done('M2-01')
        ready = roadmap.summary(self.data, self.check())['ready_or_active']
        self.assertIn('M2-02', ready)
        self.assertIn('M3-01', ready)

    def test_report_is_deterministic_and_does_not_mutate_data(self):
        before = copy.deepcopy(self.data)
        items = self.check()
        self.assertEqual(roadmap.render(self.data, items), roadmap.render(self.data, items))
        self.assertEqual(before, self.data)

    def test_cli_generates_checks_and_detects_stale_report(self):
        self.store()
        self.assertEqual(self.cli('--check')[0], 1)
        self.assertEqual(self.cli('--write')[0], 0)
        self.assertEqual(self.cli('--check')[0], 0)
        (self.root / roadmap.REPORT).write_text('out of date')
        self.assertEqual(self.cli('--check')[0], 1)

    def test_read_only_cli_does_not_change_source(self):
        self.store()
        before = (self.root / roadmap.DATA).read_bytes()
        for args in ((), ('--stage', 'M0'), ('--json',), ('--write',)):
            self.assertEqual(self.cli(*args)[0], 0)
            self.assertEqual((self.root / roadmap.DATA).read_bytes(), before)

    def test_invalid_json_reports_error_without_traceback(self):
        (self.root / roadmap.DATA).write_text('{broken')
        code, _, errors = self.cli()
        self.assertEqual(code, 1)
        self.assertIn('ROADMAP ERROR', errors)
        self.assertNotIn('Traceback', errors)

    def test_resume_active_before_another_ready_focus(self):
        for ident in ('M0-01', 'M0-02', 'M0-03', 'M0-04'):
            self.done(ident)
        self.row('M0-05')['status'] = 'in_progress'
        self.data['focus'] = 'M0-06'
        self.assertEqual(roadmap.summary(self.data, self.check())['next'], 'M0-05')

    def test_unknown_schema_rejected(self):
        self.data['schema_version'] = 2
        with self.assertRaisesRegex(ValueError, '格式'):
            self.check()


if __name__ == '__main__':
    unittest.main()
