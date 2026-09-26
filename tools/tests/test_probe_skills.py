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
