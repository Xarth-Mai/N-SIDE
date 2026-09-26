import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


TOOL = Path(__file__).resolve().parents[1] / 'validate_story_design.py'
SPEC = importlib.util.spec_from_file_location('story_design', TOOL)
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)
ROOT = TOOL.parent.parent


class StoryDesignTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.copy_file(M.DISTRICT)
        for path in M.FILES.values():
            self.copy_file(path)
        characters = json.loads((ROOT / M.FILES['characters']).read_text())['characters']
        for character in characters:
            self.copy_file(Path(character['source_file']))
        quests = json.loads((ROOT / M.FILES['quests']).read_text())['quests']
        for source in {q['source_file'] for q in quests}:
            self.copy_file(M.DOCS / source)

    def copy_file(self, relative):
        target = self.root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((ROOT / relative).read_bytes())

    def edit(self, name, change):
        path = self.root / M.FILES[name]
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data), encoding='utf-8')

    def codes(self):
        result = M.validate(self.root)
        return {issue['code'] for issue in result['issues']}

    def test_current_delivery_and_cli(self):
        result = M.validate(self.root)
        self.assertTrue(result['ok'], result)
        self.assertGreater(result['counts']['places'], 0)
        process = subprocess.run([sys.executable, '-B', str(TOOL), '--root', str(self.root), '--json'],
                                 capture_output=True, text=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(process.stdout), result)

    def test_unknown_references(self):
        cases = [('characters', 'characters', 'home_place_id', 'missing'),
                 ('characters', 'characters', 'associated_place_ids', ['missing']),
                 ('quests', 'quests', 'associated_character_ids', ['CHR-999']),
                 ('quests', 'quests', 'associated_place_ids', ['missing']),
                 ('city-story-map', 'places', 'linked_place_ids', ['missing']),
                 ('city-story-map', 'places', 'character_ids', ['CHR-999']),
                 ('city-story-map', 'places', 'quest_ids', ['QST-999']),
                 ('quests', 'quests', 'black_cases', ['BC-999'])]
        for file, collection, field, value in cases:
            with self.subTest(file=file, field=field):
                original = (self.root / M.FILES[file]).read_bytes()
                self.edit(file, lambda d: d[collection][0].update({field: value}))
                self.assertIn('reference', self.codes())
                (self.root / M.FILES[file]).write_bytes(original)

    def test_catalogue_drift(self):
        for file in ('place-catalog', 'city-story-map'):
            for field in ('name', 'block_id', 'building_id'):
                with self.subTest(file=file, field=field):
                    self.copy_file(M.FILES[file])
                    self.edit(file, lambda d: d['places'][0].update({field: 'drift'}))
                    self.assertIn('catalogue', self.codes())
            self.copy_file(M.FILES[file])
            self.edit(file, lambda d: d['places'].pop())
            self.assertIn('catalogue', self.codes())
            self.copy_file(M.FILES[file])

    def test_duplicate_ids(self):
        for file, collection in [('characters', 'characters'), ('quests', 'quests'),
                                 ('place-catalog', 'places'), ('city-story-map', 'places')]:
            with self.subTest(file=file):
                self.edit(file, lambda d: d[collection].append(copy.deepcopy(d[collection][0])))
                self.assertIn('duplicate', self.codes())
                self.copy_file(M.FILES[file])

    def test_diver_roles(self):
        for change in [{'max_simultaneous_sibling_divers': 2},
                       {'max_simultaneous_sibling_divers': True},
                       {'max_simultaneous_sibling_divers': 0},
                       {'reality_receiver': 'CHR-002'},
                       {'diver': 'CHR-003'},
                       {'diver': ['CHR-001', 'CHR-002']}]:
            with self.subTest(change=change):
                self.edit('quests', lambda d: d['quests'][1].update(change))
                self.assertIn('diver_roles', self.codes())
                self.copy_file(M.FILES['quests'])

    def test_shared_building_and_blocks(self):
        path = self.root / M.DISTRICT
        data = json.loads(path.read_text())
        next(p for p in data['places'] if p['id'] == '72')['building'] = 'V-01'
        path.write_text(json.dumps(data))
        self.assertIn('shared_building', self.codes())
        self.edit('place-catalog', lambda d: d['blocks'].update({'B01': 'changed'}))
        self.assertIn('catalogue', self.codes())

    def test_asymmetric_associations_are_valid(self):
        self.edit('quests', lambda d: d['quests'][0].update({'associated_place_ids': []}))
        self.edit('city-story-map', lambda d: d['places'][0].update({'linked_place_ids': []}))
        self.assertTrue(M.validate(self.root)['ok'])

    def test_source_missing_escape_and_stale_id(self):
        for source in ('player/story/missing.md', '../../outside.md',
                       str(self.root / M.DOCS / 'player/story/main/prologue.md')):
            with self.subTest(source=source):
                self.edit('quests', lambda d: d['quests'][0].update({'source_file': source}))
                self.assertIn('source', self.codes())
        self.copy_file(M.FILES['quests'])
        path = self.root / M.DOCS / 'player/story/main/prologue.md'
        path.write_text('No task identifiers')
        self.assertIn('reference', self.codes())

    def test_character_source_missing_or_wrong_identity(self):
        for source in ('docs/player/characters/missing.md', 'docs/player/world/history.md', '../outside.md'):
            with self.subTest(source=source):
                self.edit('characters', lambda d: d['characters'][0].update({'source_file': source}))
                self.assertIn('source', self.codes())
        self.copy_file(M.FILES['characters'])
        self.edit('characters', lambda d: d['characters'][0].update({'source_file': d['characters'][1]['source_file']}))
        self.assertIn('source', self.codes())

    def test_catalogue_keeps_character_authority_in_encyclopedia(self):
        spec = self.root / 'docs/dev/design/characters/example.md'
        spec.parent.mkdir(parents=True, exist_ok=True)
        spec.write_text('---\nsubject_id: CHR-001\n---\n# Implementation\n')
        self.assertTrue(M.validate(self.root)['ok'])
        self.edit('characters', lambda d: d['characters'][0].update({'source_file': 'docs/dev/design/characters/example.md'}))
        self.assertIn('source', self.codes())
        self.copy_file(M.FILES['characters'])
        duplicate = self.root / M.CHARACTERS / 'unregistered.md'
        duplicate.write_text('---\nsubject_id: CHR-001\n---\n# Another authority\n')
        self.assertIn('source', self.codes())

    def test_malformed_data_reports_failure(self):
        path = self.root / M.FILES['quests']
        for text in ('{bad', '{"quests":[],"quests":[]}', '{"quests":NaN}',
                     '[]', '{"quests":[null]}', '{"quests":[{"id":[]}]}'):
            with self.subTest(text=text):
                path.write_text(text)
                self.assertFalse(M.validate(self.root)['ok'])
        self.copy_file(M.FILES['quests'])
        self.edit('quests', lambda d: d['quests'][0].update({'associated_place_ids': [{}]}))
        self.assertIn('field', self.codes())


if __name__ == '__main__':
    unittest.main()
