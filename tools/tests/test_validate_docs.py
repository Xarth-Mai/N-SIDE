"""Examples of the document format and link checker."""
from __future__ import annotations

import importlib.util
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
        self.write('docs/design.md', '---\nid: DOC-DESIGN\n---\n# Design\n')
        self.write('todo/TASK-001.md', '---\nid: TASK-001\nstatus: ready\ndepends_on: ["DOC-DESIGN"]\n---\n# Task\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 2)

    def test_duplicate_identifier(self):
        for name in ('a', 'b'):
            self.write(f'docs/{name}.md', '---\nid: ENM-001\n---\n# Enemy\n')
        self.assertIn('ID 重复', self.messages())

    def test_missing_dependency(self):
        self.write('docs/a.md', '---\nid: ENM-001\ndepends_on: ["DOC-MISSING"]\n---\n# Enemy\n')
        self.assertIn('依赖目标缺失', self.messages())

    def test_dependency_type(self):
        self.write('docs/a.md', '---\nid: ENM-001\ndepends_on: DOC-DESIGN\n---\n# Enemy\n')
        self.assertIn('字符串数组', self.messages())

    def test_missing_heading(self):
        self.write('docs/a.md', 'A document\n')
        self.assertIn('一级标题缺失', self.messages())

    def test_missing_link(self):
        self.write('docs/a.md', '# A\n[Missing](missing.md)\n')
        self.assertIn('链接目标缺失', self.messages())

    def test_relative_reference_and_encoded_links(self):
        self.write('docs/a b.md', '# Destination\n')
        self.write('docs/index.md', '# Index\n[A](a%20b.md#title)\n[B][b]\n[b]: <a b.md>\n')
        self.assertTrue(self.result()['ok'])

    def test_fenced_and_inline_examples(self):
        self.write('docs/a.md', '# A\n```md\n[x](absent.md)\n```\n`[x](absent.md)`\n')
        self.assertTrue(self.result()['ok'])

    def test_external_and_fragment_links(self):
        self.write('README.md', '# A\n[A](https://example.com/a)\n[B](#a)\n')
        self.assertTrue(self.result()['ok'])

    def test_template_identifiers(self):
        for name in ('a', 'b'):
            self.write(f'docs/templates/{name}.md', '---\nid: ENM-000\n---\n# Template\n')
        self.assertTrue(self.result()['ok'])
        self.assertEqual(self.result()['records'], 0)

    def test_array_syntax(self):
        self.write('docs/a.md', '---\nid: ENM-001\ndepends_on: [broken]\n---\n# Enemy\n')
        self.assertIn('格式有误', self.messages())

    def test_front_matter_closing(self):
        self.write('docs/a.md', '---\nid: ENM-001\n# Enemy\n')
        self.assertIn('结束标记缺失', self.messages())

    def test_trailing_space(self):
        self.write('docs/a.md', '# A\nText \n')
        self.assertIn('行尾存在空格', self.messages())

    def test_skill_fields(self):
        self.write('.agents/skills/create/SKILL.md', '---\nname: create\ndescription: Create content.\n---\n# Create\n')
        self.assertTrue(self.result()['ok'])
        self.write('.agents/skills/create/SKILL.md', '---\nname: other\n---\n# Create\n')
        self.assertIn('目录名对应', self.messages())
        self.assertIn('description 缺失', self.messages())

    def test_state_value(self):
        self.write('todo/a.md', '---\nid: TASK-001\nstatus: approved\n---\n# Task\n')
        self.assertIn('状态值', self.messages())

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

    def test_wiki_link_stays_in_published_tree(self):
        self.write('todo/README.md', '# Work\n')
        self.write('docs/index.md', '# Wiki\n[Work](../todo/README.md)\n')
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
