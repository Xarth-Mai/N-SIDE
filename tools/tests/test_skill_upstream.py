import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('check_skill_upstream', Path(__file__).resolve().parents[1] / 'check_skill_upstream.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class UpstreamTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'project'
        self.upstream = Path(self.temp.name) / 'upstream'
        self.upstream.mkdir()
        self.original = b'# Original\nIndependent Git source\n'
        (self.upstream / 'LICENSE').write_text('Original Author\nMIT\n')
        (self.upstream / 'example').mkdir()
        (self.upstream / 'example/SKILL.md').write_bytes(self.original)
        self.run_git('init', '-q')
        self.run_git('add', '.')
        self.run_git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.test', 'commit', '-qm', 'fixture')
        commit = self.run_git('rev-parse', 'HEAD').strip()
        self.local = self.root / '.agents/skills/example/SKILL.md'
        self.local.parent.mkdir(parents=True)
        self.local.write_bytes(self.original)
        license = self.root / 'third_party/skills/source/LICENSE'
        license.parent.mkdir(parents=True)
        license.write_bytes((self.upstream / 'LICENSE').read_bytes())
        self.manifest = {'sources': [{'id': 'source', 'commit': commit, 'repo': 'https://github.com/example/source', 'license_files': [{'upstream_path': 'LICENSE', 'local_path': 'third_party/skills/source/LICENSE'}]}], 'skills': [{'name': 'example', 'source': 'source', 'mode': 'verbatim', 'local_path': '.agents/skills/example', 'upstream_path': 'example', 'files': [{'path': 'SKILL.md', 'sha256': hashlib.sha256(self.original).hexdigest()}]}], 'references': []}

    def run_git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.upstream), *args], stderr=subprocess.PIPE, text=True)

    def check(self):
        (self.root / 'third_party/skills/manifest.json').write_text(json.dumps(self.manifest))
        return MODULE.check(self.root, {'source': self.upstream})

    def test_real_fixed_git_source(self):
        self.assertEqual('PASS', self.check()['status'])

    def test_updated_local_hash_does_not_forge_upstream(self):
        self.local.write_text('# Locally rewritten\n')
        self.manifest['skills'][0]['files'][0]['sha256'] = hashlib.sha256(self.local.read_bytes()).hexdigest()
        result = self.check()
        self.assertEqual('FAIL', result['status'])
        self.assertIn('not byte-identical', result['sources'][0]['issues'][0])

    def test_documented_patch_and_base_are_compared(self):
        self.local.write_text('# Original\nModified for N:SIDE: correction\n')
        patch = self.root / 'third_party/skills/patches/example.patch'
        patch.parent.mkdir()
        patch.write_text(''.join(difflib.unified_diff(self.original.decode().splitlines(keepends=True), self.local.read_text().splitlines(keepends=True), fromfile='a/SKILL.md', tofile='b/SKILL.md')))
        self.manifest['skills'][0]['files'][0]['patch'] = {'upstream_sha256': hashlib.sha256(self.original).hexdigest(), 'file': str(patch.relative_to(self.root))}
        self.assertEqual('PASS', self.check()['status'])
        self.local.write_text(self.local.read_text() + 'Unrecorded change\n')
        self.assertEqual('FAIL', self.check()['status'])

    def test_directory_coverage_and_pinned_target(self):
        self.manifest['skills'][0]['files'] = []
        self.manifest['skills'][0]['pinned_links'] = [{'url': f"https://github.com/example/source/blob/{self.manifest['sources'][0]['commit']}/missing.md"}]
        result = self.check()
        self.assertEqual('FAIL', result['status'])
        self.assertTrue(any('directory coverage' in issue for issue in result['sources'][0]['issues']))
        self.assertTrue(any('pinned reference' in issue for issue in result['sources'][0]['issues']))

    def test_unavailable_source_is_not_run(self):
        self.manifest['sources'][0]['commit'] = '0' * 40
        result = self.check()
        self.assertEqual('FAIL', result['status'])
        self.assertEqual('NOT RUN', result['sources'][0]['status'])


if __name__ == '__main__':
    unittest.main()
