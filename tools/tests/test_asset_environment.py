"""Check actual vertex data against the existing static environment export contract."""

import json
import math
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "source-assets/environment-kit"
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, values


class AssetEnvironmentTests(unittest.TestCase):
    def test_source_and_export_vertex_bounds_ground_pivot_and_scale(self):
        manifest = json.loads((SOURCE / "asset-manifest.json").read_text())
        for file in manifest["files"]:
            if "transform" not in file:
                continue
            for exported in (False, True):
                path = ROOT / "game/assets/environment" / file["output"] if exported else SOURCE / file["source"]
                with self.subTest(path=path):
                    document, binary = read_glb(path)
                    self.assertFalse(document.get("skins"), "static environment unexpectedly has a skin")
                    self.assertFalse(document.get("animations"), "static environment unexpectedly has animation")
                    for node in document["nodes"]:
                        for field in ("translation", "rotation", "scale", "matrix"):
                            self.assertTrue(all(math.isfinite(x) for x in node.get(field, [])), f"nonfinite node {field}")
                    for accessor in document["accessors"]:
                        self.assertTrue(all(math.isfinite(x) for row in values(document, binary, accessor) for x in row), "nonfinite accessor data")
                    scene = document["scenes"][document.get("scene", 0)]
                    self.assertEqual(len(scene["nodes"]), 1)
                    node = document["nodes"][scene["nodes"][0]]
                    self.assertFalse(node.get("children"))
                    self.assertNotIn("matrix", node)
                    self.assertEqual(node.get("rotation", [0, 0, 0, 1]), [0, 0, 0, 1])
                    points = []
                    for primitive in document["meshes"][node["mesh"]]["primitives"]:
                        accessor = document["accessors"][primitive["attributes"]["POSITION"]]
                        positions = values(document, binary, accessor)
                        for axis in range(3):
                            self.assertAlmostEqual(min(p[axis] for p in positions), accessor["min"][axis], places=6)
                            self.assertAlmostEqual(max(p[axis] for p in positions), accessor["max"][axis], places=6)
                        points.extend(positions)
                    height = max(p[1] for p in points) - min(p[1] for p in points)
                    settings = file["transform"]
                    self.assertAlmostEqual(height, settings["source_height"], places=6)
                    scale = node.get("scale", [1, 1, 1])
                    self.assertEqual(scale, [scale[0]] * 3)
                    self.assertGreater(scale[0], 0)
                    if exported:
                        self.assertAlmostEqual(height * scale[1], settings["height_m"], places=6)
                        self.assertAlmostEqual(min(p[1] for p in points) * scale[1] + node.get("translation", [0, 0, 0])[1], 0, places=6)


if __name__ == "__main__":
    unittest.main()
