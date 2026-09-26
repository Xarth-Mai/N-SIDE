import asyncio
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('probe_skills', Path(__file__).parents[1] / 'probe_skills.py')
probe_skills = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe_skills)


class SkillDiscoveryTests(unittest.TestCase):
    def test_missing_host_is_not_run(self):
        with patch.object(probe_skills.shutil, 'which', return_value=None), \
                patch.object(probe_skills.asyncio, 'create_subprocess_exec') as start:
            result = asyncio.run(probe_skills.probe(None))
        self.assertEqual(result['status'], 'NOT RUN')
        self.assertIn('Codex CLI unavailable', result['error'])
        start.assert_not_called()

    def test_discovery_requires_exact_unique_cwds_and_nonempty_skills(self):
        skill = {'name': 'example', 'path': '/repo/.agents/skills/example/SKILL.md', 'enabled': True}
        expected = {skill['path']}
        cwds = ['/repo', '/repo/game']
        valid = {'data': [{'cwd': cwd, 'skills': [skill], 'errors': []} for cwd in cwds]}
        entries = probe_skills.check_discovery(valid, expected, cwds)
        self.assertTrue(all(not entry['missing'] and not entry['errors'] for entry in entries))
        cases = [
            (valid, set(), cwds),
            (valid, expected, ['/repo', '/repo']),
            ({'data': [valid['data'][0], valid['data'][0]]}, expected, cwds),
            ({'data': [valid['data'][0]]}, expected, cwds),
            ({'data': valid['data'] + [{'cwd': '/other', 'skills': [], 'errors': []}]}, expected, cwds),
        ]
        for response, required, requested in cases:
            with self.subTest(response=response, required=required, requested=requested), self.assertRaises(ValueError):
                probe_skills.check_discovery(response, required, requested)
        for unavailable in [[], [{**skill, 'enabled': False}]]:
            response = {'data': [{'cwd': cwd, 'skills': unavailable, 'errors': []} for cwd in cwds]}
            self.assertTrue(all(entry['missing'] == sorted(expected) for entry in probe_skills.check_discovery(response, expected, cwds)))

    def test_expected_comes_from_nonempty_required_policy_and_declarations(self):
        import json
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = root / 'third_party/skills/manifest.json'
            manifest_path.parent.mkdir(parents=True)
            skill_path = root / '.agents/skills/example/SKILL.md'
            skill_path.parent.mkdir(parents=True)
            skill_path.write_text('example')
            manifest = {'schema_version': 2, 'policy': {'required_skills': ['example']},
                        'local_skills': [{'name': 'example', 'local_path': '.agents/skills/example'}], 'skills': []}
            manifest_path.write_text(json.dumps(manifest))
            self.assertEqual(probe_skills.expected_skills(root), {str(skill_path)})
            for changed in [
                {}, {**manifest, 'schema_version': 1},
                {**manifest, 'policy': {'required_skills': []}},
                {**manifest, 'local_skills': []},
                {**manifest, 'skills': manifest['local_skills']},
            ]:
                manifest_path.write_text(json.dumps(changed))
                with self.subTest(manifest=changed), self.assertRaises(ValueError):
                    probe_skills.expected_skills(root)
            manifest_path.write_text(json.dumps(manifest))
            skill_path.unlink()
            with self.assertRaisesRegex(ValueError, 'missing'):
                probe_skills.expected_skills(root)
