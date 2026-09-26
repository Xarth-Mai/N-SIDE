"""Small end-to-end checks for the imported raster tools and project adapter."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / ".agents/skills/create-game-assets/scripts"


class AssetPreviewTests(unittest.TestCase):
    def run_tool(self, path, *args):
        return subprocess.run([sys.executable, "-B", str(path), *map(str, args)], capture_output=True, text=True)

    def test_color_budget_boundaries(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "colors.png"
            # N=3 exercises exact N+1 as well as Pillow's over-limit sentinel
            for count in (2, 3, 4, 5):
                with self.subTest(count=count):
                    fixture = Image.new("RGBA", (count, 1))
                    fixture.putdata([(value, 0, 0, 255) for value in range(count)])
                    fixture.save(source)
                    before = source.read_bytes()
                    result = self.run_tool(SCRIPTS / "asset_report.py", source,
                                           "--max-colors", "3", "--json")
                    report = json.loads(result.stdout)
                    self.assertEqual(result.returncode, int(count > 3), result.stderr)
                    self.assertEqual(report["ok"], count <= 3)
                    self.assertEqual(bool(report["assets"][0]["problems"]), count > 3)
                    self.assertEqual(source.read_bytes(), before)

    def test_real_logo_report_and_wrong_size_exit(self):
        logo = ROOT / "game/assets/branding/n-logo.png"
        passed = self.run_tool(SCRIPTS / "asset_report.py", logo, "--expect-size", "512x512", "--require-alpha", "--json")
        self.assertEqual(passed.returncode, 0, passed.stderr)
        self.assertTrue(json.loads(passed.stdout)["ok"])
        failed = self.run_tool(SCRIPTS / "asset_report.py", logo, "--expect-size", "64x64", "--json")
        self.assertEqual(failed.returncode, 1, failed.stderr)
        self.assertIn("expected 64x64", json.loads(failed.stdout)["assets"][0]["problems"][0])

    def test_preview_alpha_composite_and_failures(self):
        with tempfile.TemporaryDirectory() as directory:
            source, output = Path(directory) / "source.png", Path(directory) / "preview.png"
            cutout = Image.new("RGBA", (4, 2), (0, 0, 0, 0))
            cutout.putpixel((0, 0), (255, 0, 0, 255))
            cutout.save(source)
            args = ("--out", output, "--display-size", "4x4", "--require-cutout")
            result = self.run_tool(ROOT / "tools/asset_preview.py", source, *args)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(json.loads(result.stdout)["fitted_size"], [4, 2])
            with Image.open(output) as preview:
                self.assertEqual(preview.getpixel((3, 1)), (255, 255, 255))
                self.assertEqual(preview.getpixel((7, 1)), (21, 21, 21))
                self.assertEqual(preview.getpixel((11, 1)), (255, 0, 255))
                self.assertEqual(preview.getpixel((0, 1)), (255, 0, 0))
            for color in ((0, 0, 0, 0), (255, 255, 255, 255)):
                Image.new("RGBA", (4, 2), color).save(source)
                failed = self.run_tool(ROOT / "tools/asset_preview.py", source, *args)
                self.assertEqual(failed.returncode, 1)
                self.assertIn("visible content", json.loads(failed.stdout)["error"])
            before = source.read_bytes()
            failed = self.run_tool(ROOT / "tools/asset_preview.py", source, "--out", source, "--display-size", "4x4")
            self.assertEqual(failed.returncode, 1)
            self.assertEqual(source.read_bytes(), before)

    def test_upstream_contact_sheet(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "sheet.png"
            result = self.run_tool(SCRIPTS / "build_preview_sheet.py", ROOT / "game/assets/branding/n-logo.png", "--out", output, "--columns", 1, "--cell-size", 128)
            self.assertEqual(result.returncode, 0, result.stderr)
            with Image.open(output) as preview:
                self.assertEqual(preview.size, (152, 176))


if __name__ == "__main__":
    unittest.main()
