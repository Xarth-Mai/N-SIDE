"""Check actual vertex data against the existing static environment export contract."""

import json
import math
from pathlib import Path
import struct
import unittest

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "source-assets/environment-kit"


def read_glb(path):
    data = path.read_bytes()
    magic, version, length, json_size, kind = struct.unpack_from("<5I", data)
    assert magic == 0x46546C67 and version == 2 and length == len(data), f"{path}: invalid GLB header"
    assert kind == 0x4E4F534A, f"{path}: JSON must be the first chunk"
    document = json.loads(data[20:20 + json_size])
    binary_size, binary_kind = struct.unpack_from("<2I", data, 20 + json_size)
    assert binary_kind == 0x004E4942, f"{path}: expected embedded binary chunk"
    binary = data[28 + json_size:]
    assert len(binary) == binary_size, f"{path}: truncated binary chunk"
    return document, binary


def values(document, binary, accessor):
    assert "sparse" not in accessor, "static environment fixtures do not use sparse accessors"
    view = document["bufferViews"][accessor["bufferView"]]
    assert view["buffer"] == 0, "static environment fixtures have one embedded buffer"
    count = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}[accessor["type"]]
    code = {5120: "b", 5121: "B", 5122: "h", 5123: "H", 5125: "I", 5126: "f"}[accessor["componentType"]]
    format_ = "<" + code * count
    size = struct.calcsize(format_)
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride", size)
    assert stride >= size and accessor["count"] > 0, "invalid vertex stride/count"
    end = start + stride * (accessor["count"] - 1) + size
    assert end <= view.get("byteOffset", 0) + view["byteLength"] <= len(binary), "accessor exceeds its buffer view"
    return [struct.unpack_from(format_, binary, start + i * stride) for i in range(accessor["count"])]


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
