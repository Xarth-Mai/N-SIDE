"""Examples of the document format and link checker."""
from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[1] / 'validate_docs.py'
SPEC = importlib.util.spec_from_file_location('validate_docs', TOOL)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class DocumentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.write('README.md', '# Project\n')

    def write(self, name: str, text: str) -> None:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')

    def result(self):
        return MODULE.validate(self.root)

    def messages(self):
        return '\n'.join(x['message'] for x in self.result()['issues'])

    def test_valid_minimal(self):
        self.assertTrue(self.result()['ok'])

    def test_valid_record_and_dependency(self):
        self.write('docs/dev/design.md', '---\ndocument_id: DOC-DESIGN\n---\n# Design\n')
        self.write('docs/dev/other.md', '---\ndocument_id: DOC-OTHER\nstatus: accepted\ndepends_on: ["DOC-DESIGN"]\n---\n# Other\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 2)

    def test_duplicate_identifier(self):
        for name in ('a', 'b'):
            self.write(f'docs/{name}.md', '---\ndocument_id: DOC-ENEMY\n---\n# Enemy\n')
        self.assertIn('ID 重复', self.messages())

    def test_missing_dependency(self):
        self.write('docs/a.md', '---\nsubject_id: ENM-001\ndepends_on: ["DOC-MISSING"]\n---\n# Enemy\n')
        self.assertIn('依赖目标缺失', self.messages())

    def test_dependency_type(self):
        self.write('docs/a.md', '---\nsubject_id: ENM-001\ndepends_on: DOC-DESIGN\n---\n# Enemy\n')
        self.assertIn('字符串数组', self.messages())

    def test_missing_heading(self):
        self.write('docs/a.md', 'A document\n')
        self.assertIn('一级标题缺失', self.messages())

    def test_missing_link(self):
        self.write('docs/a.md', '# A\n[Missing](missing.md)\n')
        self.assertIn('链接目标缺失', self.messages())

    def test_relative_reference_and_encoded_links(self):
        self.write('docs/a b.md', '# Destination\n')
        self.write('docs/index.md', '# Index\n[A](a%20b.md#destination)\n[B][b]\n[b]: <a b.md>\n')
        self.assertTrue(self.result()['ok'])

    def test_fenced_and_inline_examples(self):
        self.write('docs/a.md', '# A\n```md\n[x](absent.md)\n```\n`[x](absent.md)`\n')
        self.assertTrue(self.result()['ok'])

    def test_external_and_fragment_links(self):
        self.write('README.md', '# A\n[A](https://example.com/a)\n[B](#a)\n')
        self.assertTrue(self.result()['ok'])

    def test_template_identifiers(self):
        for name in ('a', 'b'):
            self.write(f'docs/dev/handbook/templates/{name}.md', '---\nsubject_id: ENM-000\n---\n# Template\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 0)

    def test_array_syntax(self):
        self.write('docs/a.md', '---\nsubject_id: ENM-001\ndepends_on: [broken]\n---\n# Enemy\n')
        self.assertIn('格式有误', self.messages())

    def test_front_matter_closing(self):
        self.write('docs/a.md', '---\nsubject_id: ENM-001\n# Enemy\n')
        self.assertIn('结束标记缺失', self.messages())

    def test_trailing_space(self):
        self.write('docs/a.md', '# A\nText \n')
        self.assertIn('行尾存在空格', self.messages())

    def test_task_and_skill_yaml_owned_by_bun_checks(self):
        self.write('.agents/skills/create/SKILL.md', '---\nname: create\ndescription: >-\n  Create content\n---\n# Create\n')
        self.write('todo/tasks/TASK-020-example.md', '---\nid: TASK-020\nacceptance:\n  role: codex\n---\n# Task\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 0)
        self.write('todo/tasks/TASK-020-example.md', '# Task\n[Missing](missing.md)\n')
        self.assertIn('链接目标缺失', self.messages())

    def test_upstream_style_is_checked_by_skill_validator(self):
        self.write('.agents/skills/original/SKILL.md', '---\nname: original\ndescription: >-\n  Original description\n---\nOriginal text  \n')
        self.write('third_party/skills/manifest.json', json.dumps({'skills': [{
            'local_path': '.agents/skills/original', 'mode': 'verbatim',
            'files': [{'path': 'SKILL.md'}]}]}))
        self.assertTrue(self.result()['ok'])
        self.write('.agents/skills/original/UPSTREAM.md', '# Source\n[Missing](absent.md)\n')
        self.assertIn('链接目标缺失', self.messages())

    def test_state_value(self):
        self.write('docs/dev/a.md', '---\ndocument_id: DOC-ONE\nstatus: approved\n---\n# Design\n')
        self.assertIn('状态值', self.messages())

    def test_subject_references_do_not_duplicate_document_identity(self):
        self.write('docs/player/encyclopedia/characters/a.md', '---\nsubject_id: CHR-001\n---\n# A\n')
        self.write('docs/dev/design/characters/a.md', '---\ndocument_id: DOC-A-SPEC\nsubject_id: CHR-001\ndepends_on: ["CHR-001"]\n---\n# A spec\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 2)
        self.write('docs/player/encyclopedia/characters/b.md', '---\nsubject_id: CHR-001\n---\n# Duplicate\n')
        self.assertIn('对象权威页重复', self.messages())

    def test_object_references_use_the_existing_catalogue(self):
        self.write('docs/dev/design/catalogs/quests.json', json.dumps({'quests': [{'id': 'QST-002'}]}))
        self.write('docs/dev/spec.md', '---\ndocument_id: DOC-SPEC\nsubject_id: QST-002\ndepends_on: ["QST-002"]\n---\n# Spec\n')
        self.assertTrue(self.result()['ok'])
        self.write('docs/dev/spec.md', '---\nsubject_id: QST-999\n---\n# Missing\n')
        self.assertIn('对象未登记', self.messages())

    def test_document_and_subject_fields_are_distinct(self):
        self.write('docs/dev/a.md', '---\ndocument_id: CHR-001\nsubject_id: DOC-A\n---\n# A\n')
        self.assertIn('document_id 格式', self.messages())
        self.assertIn('subject_id 格式', self.messages())
        self.write('docs/dev/a.md', '---\nid: DOC-A\n---\n# A\n')
        self.assertIn('使用 document_id', self.messages())

    def test_legacy_metadata_is_not_an_active_task_and_links_still_checked(self):
        self.write('todo/archive/legacy/a.md', '---\nid: TASK-001\nstatus: in_progress\n---\n# Old  \n')
        self.write('todo/tasks/TASK-001-example.md', '---\nid: TASK-001\nstatus: active\n---\n# Current\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 0)
        self.write('todo/archive/legacy/a.md', '# Old\n[Missing](absent.md)\n')
        self.assertIn('链接目标缺失', self.messages())

    def test_anchors_include_unicode_explicit_repeated_and_inline_code(self):
        self.write('docs/dev/a.md', '# 规则\n## 动作 `start`\n## 标题\n## 标题\n## 显式 {#custom}\n<a id="marker"></a>\n[A](#动作-start)\n[B](#标题-1)\n[C](#custom)\n[D](#marker)\n')
        self.assertTrue(self.result()['ok'], self.messages())
        self.write('docs/dev/b.md', '# B\n[Missing](a.md#不存在)\n')
        self.assertIn('链接锚点缺失', self.messages())

    def test_wiki_slug_matches_numeric_nfkd_and_punctuation_rules(self):
        self.write('docs/dev/a.md', '# 1. Échelle / 尺度（m）\n## 动作 `start_stop`\n[A](#_1-echelle-尺度-m)\n[B](#动作-start-stop)\n')
        self.assertTrue(self.result()['ok'], self.messages())
        self.write('docs/dev/a.md', '# 1. Title\n[Bad](#1-title)\n')
        self.assertIn('链接锚点缺失', self.messages())
        self.write('README.md', '# 1. Title\n[OK](#1-title)\n')
        self.write('docs/dev/a.md', '# A\n')
        self.assertTrue(self.result()['ok'])

    def test_utf8(self):
        path = self.root / 'docs/a.md'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b'\xff')
        self.assertIn('UTF-8 读取失败', self.messages())

    def test_line_endings(self):
        (self.root / 'README.md').write_bytes(b'# Project\r\n')
        self.assertIn('行尾格式使用 LF', self.messages())


    def test_wiki_public_image(self):
        self.write('docs/public/images/example.svg', '<svg/>')
        self.write('docs/index.md', '# Wiki\n![Image](/images/example.svg)\n')
        self.assertTrue(self.result()['ok'])

    def test_wiki_routes(self):
        self.write('docs/vision.md', '# Vision\n')
        self.write('docs/quests/index.md', '# Quests\n')
        self.write('docs/index.md', '# Wiki\n[A](/vision)\n[B](/quests/)\n[C](/vision.html)\n')
        self.assertTrue(self.result()['ok'])

    def test_directory_route_requires_index_and_checks_its_anchor(self):
        self.write('docs/dev/section/index.md', '# Landing\n')
        self.write('docs/dev/index.md', '# Dev\n[Section](section/#landing)\n')
        self.assertTrue(self.result()['ok'])
        self.write('docs/dev/index.md', '# Dev\n[Section](section/#missing)\n')
        self.assertIn('链接锚点缺失', self.messages())
        (self.root / 'docs/dev/section/index.md').unlink()
        self.assertIn('链接目标缺失', self.messages())

    def test_wiki_link_stays_in_published_tree(self):
        self.write('todo/README.md', '# Work\n')
        self.write('docs/index.md', '# Wiki\n[Work](../todo/README.md)\n')
        self.assertIn('发布路径', self.messages())

    def test_dev_repo_source_and_player_audience_boundary(self):
        self.write('source-assets/LICENSE', 'License')
        self.write('docs/dev/page.md', '# Dev\n[License](../../source-assets/LICENSE)\n')
        self.write('docs/player/page.md', '# Player\n')
        self.assertTrue(self.result()['ok'])
        self.write('docs/player/page.md', '# Player\n[Dev](../dev/page.md)\n')
        self.assertIn('越出发布受众', self.messages())
        self.write('docs/dev/page.md', '# Dev\n[Outside](../../../outside.md)\n')
        self.assertIn('发布路径', self.messages())

    def test_repository_entry_can_link_work(self):
        self.write('todo/README.md', '# Work\n')
        self.write('README.md', '# Project\n[Work](todo/README.md)\n')
        self.assertTrue(self.result()['ok'])

    def test_renderer_and_exported_files_are_separate(self):
        self.write('docs/index.md', '# Wiki\n')
        self.write('docs/.vitepress/cache/generated.md', 'generated')
        self.write('docs/public/export.md', 'export')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['files'], 2)

    def test_missing_wiki_image(self):
        self.write('docs/index.md', '# Wiki\n![Image](/images/missing.png)\n')
        self.assertIn('链接目标缺失', self.messages())


if __name__ == '__main__':
    unittest.main()
