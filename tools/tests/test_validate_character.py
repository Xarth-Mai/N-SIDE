"""Regress the actual Blender export and representative broken export contracts."""

import json
from io import BytesIO
import math
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, values
from validate_character import inspect

FIXTURE = Path(__file__).parent / "fixtures/character.glb"


def write_glb(path, document, binary):
    encoded = json.dumps(document).encode()
    encoded += b" " * (-len(encoded) % 4)
    binary = bytes(binary) + b"\0" * (-len(binary) % 4)
    path.write_bytes(struct.pack("<5I", 0x46546C67, 2, 28 + len(encoded) + len(binary), len(encoded), 0x4E4F534A) + encoded + struct.pack("<2I", len(binary), 0x004E4942) + binary)


def change_first(document, binary, index, format_, value, row=0, component=0):
    accessor = document["accessors"][index]
    view = document["bufferViews"][accessor["bufferView"]]
    width = {"SCALAR": 1, "VEC3": 3, "VEC4": 4, "MAT4": 16}[accessor["type"]]
    stride = view.get("byteStride", width * struct.calcsize(format_))
    offset = view.get("byteOffset", 0) + accessor.get("byteOffset", 0) + row * stride + component * struct.calcsize(format_)
    struct.pack_into("<" + format_, binary, offset, value)


class CharacterChecks(unittest.TestCase):
    def test_real_blender_fixture(self):
        report = inspect(FIXTURE, root_name="Root", height=1.7, clips=["Idle", "Sway"], require_texture=True)
        self.assertEqual(report["status"], "PASS")
        self.assertEqual((report["skins"], report["joints"], report["embedded_images"]), (1, 2, 1))
        self.assertAlmostEqual(report["height_m"], 1.7, places=6)
        self.assertEqual(len(report["animations"]), 2)
        self.assertEqual(report["animations"][0]["duration_seconds"], 1)

    def test_static_real_model_rejected_with_cli_exit(self):
        result = subprocess.run([sys.executable, "-B", str(ROOT / "tools/validate_character.py"), str(ROOT / "source-assets/environment-kit/nature/tree_oak.glb"), "--root-node", "Root", "--height", "1.7"], text=True, capture_output=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("no skins", json.loads(result.stdout)["error"])

    def test_declared_scale_clip_and_root_contracts(self):
        for overrides, message in [({"height": 170}, "height"), ({"root_name": "Missing"}, "root-motion"), ({"clips": ["Run"]}, "missing required clips")]:
            with self.subTest(overrides=overrides), self.assertRaisesRegex(ValueError, message):
                inspect(FIXTURE, **({"root_name": "Root", "height": 1.7} | overrides))

    def test_corrupt_weights_joints_bind_pose_times_root_motion_texture_and_sparse(self):
        for issue in ("weights", "joint", "bind", "time", "root", "texture", "sparse"):
            with self.subTest(issue=issue), tempfile.TemporaryDirectory() as directory:
                document, binary = read_glb(FIXTURE)
                binary = bytearray(binary)
                primitive = document["meshes"][0]["primitives"][0]
                if issue == "weights":
                    change_first(document, binary, primitive["attributes"]["WEIGHTS_0"], "f", 0.3)
                    message = "weights do not sum"
                elif issue == "joint":
                    change_first(document, binary, primitive["attributes"]["JOINTS_0"], "B", 99)
                    message = "joint index out of range"
                elif issue == "bind":
                    change_first(document, binary, document["skins"][0]["inverseBindMatrices"], "f", 0)
                    message = "inverse bind matrices"
                elif issue == "time":
                    change_first(document, binary, document["animations"][0]["samplers"][0]["input"], "f", 0, row=1)
                    message = "timestamps"
                elif issue == "root":
                    animation = document["animations"][0]
                    channel = next(c for c in animation["channels"] if c["target"]["path"] == "translation" and document["nodes"][c["target"]["node"]]["name"] == "Root")
                    change_first(document, binary, animation["samplers"][channel["sampler"]]["output"], "f", 1, row=1)
                    message = "root translation"
                elif issue == "texture":
                    document["images"][0]["uri"] = "external.png"
                    message = "embedded PNG"
                else:
                    document["accessors"][0]["sparse"] = {}
                    message = "sparse accessors"
                file = Path(directory) / "broken.glb"
                write_glb(file, document, binary)
                with self.assertRaisesRegex(ValueError, message):
                    inspect(file, root_name="Root", height=1.7)

    def test_strided_normalized_accessor_and_malformed_cli(self):
        document = {"bufferViews": [{"buffer": 0, "byteOffset": 1, "byteLength": 10, "byteStride": 6}]}
        accessor = {"bufferView": 0, "componentType": 5121, "type": "VEC4", "count": 2, "normalized": True}
        self.assertEqual(values(document, bytes([0, 255, 0, 0, 0, 0, 0, 0, 255, 0, 0]), accessor), [(1, 0, 0, 0), (0, 1, 0, 0)])
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "broken.glb"
            path.write_bytes(b"glTF")
            result = subprocess.run([sys.executable, "-B", str(ROOT / "tools/validate_character.py"), str(path), "--root-node", "Root", "--height", "1.7"], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(json.loads(result.stdout)["status"], "FAIL")

    def test_audited_false_passes_fail_through_cli(self):
        issues = ("index_range", "index_type", "index_count", "short_png", "short_jpeg", "shifted_root", "turned_root", "fake_root", "armature_root_shift")
        for issue in issues:
            with self.subTest(issue=issue), tempfile.TemporaryDirectory() as directory:
                document, binary = read_glb(FIXTURE)
                binary = bytearray(binary)
                primitive = document["meshes"][0]["primitives"][0]
                root_name = "Root"
                if issue.startswith("index_"):
                    index = primitive["indices"]
                    if issue == "index_range":
                        change_first(document, binary, index, "H", 999)
                        message = "triangle index out of vertex range"
                    elif issue == "index_type":
                        document["accessors"][index]["componentType"] = 5122
                        message = "unnormalized unsigned SCALAR"
                    else:
                        document["accessors"][index]["count"] = 35
                        message = "multiple of 3"
                elif issue.startswith("short_"):
                    image = document["images"][0]
                    view = document["bufferViews"][image["bufferView"]]
                    view["byteLength"] = 8
                    if issue == "short_jpeg":
                        image["mimeType"] = "image/jpeg"
                        binary[view["byteOffset"]:view["byteOffset"] + 8] = b"\xff\xd8\xff\xe0\x00\x10JF"
                    message = "embedded image decoding failed"
                else:
                    animation = document["animations"][0]
                    field = "rotation" if issue == "turned_root" else "translation"
                    channel = next(c for c in animation["channels"] if c["target"]["path"] == field and document["nodes"][c["target"]["node"]]["name"] == "Root")
                    index = animation["samplers"][channel["sampler"]]["output"]
                    for row in range(document["accessors"][index]["count"]):
                        if field == "rotation":
                            change_first(document, binary, index, "f", math.sin(math.pi / 8), row=row, component=1)
                            change_first(document, binary, index, "f", math.cos(math.pi / 8), row=row, component=3)
                        else:
                            change_first(document, binary, index, "f", 1, row=row)
                    message = f"root {field} bound"
                    if issue == "fake_root":
                        document["nodes"][channel["target"]["node"]]["name"] = "RealRoot"
                        document["scenes"][0]["nodes"].append(len(document["nodes"]))
                        document["nodes"].append({"name": "Root"})
                        message = "not an ancestor"
                    elif issue == "armature_root_shift":
                        root_name = "FixtureRig"
                file = Path(directory) / "broken.glb"
                write_glb(file, document, binary)
                result = subprocess.run([sys.executable, "-B", str(ROOT / "tools/validate_character.py"), str(file), "--root-node", root_name, "--height", "1.7"], capture_output=True, text=True)
                self.assertEqual(result.returncode, 1, result.stderr)
                report = json.loads(result.stdout)
                self.assertEqual(report["status"], "FAIL")
                self.assertIn(message, report["error"])

    def test_real_embedded_jpeg_and_armature_root(self):
        document, binary = read_glb(FIXTURE)
        encoded = BytesIO()
        Image.new("RGB", (4, 4), (20, 80, 140)).save(encoded, format="JPEG")
        binary += b"\0" * (-len(binary) % 4)
        image = document["images"][0]
        image["mimeType"] = "image/jpeg"
        view = document["bufferViews"][image["bufferView"]]
        view["byteOffset"], view["byteLength"] = len(binary), len(encoded.getvalue())
        binary += encoded.getvalue()
        document["buffers"][0]["byteLength"] = len(binary)
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / "jpeg.glb"
            write_glb(file, document, binary)
            self.assertEqual(inspect(file, root_name="FixtureRig", height=1.7, require_texture=True)["status"], "PASS")

    def test_invalid_normal_and_uv_attributes(self):
        for issue, message in (("normal_type", "normals must"), ("normal_zero", "non-unit normal"), ("uv_count", "attribute count mismatch"), ("uv_type", "UVs must"), ("uv_integer", "UVs must")):
            with self.subTest(issue=issue), tempfile.TemporaryDirectory() as directory:
                document, binary = read_glb(FIXTURE)
                binary = bytearray(binary)
                attributes = document["meshes"][0]["primitives"][0]["attributes"]
                normal = document["accessors"][attributes["NORMAL"]]
                uv = document["accessors"][attributes["TEXCOORD_0"]]
                if issue == "normal_type":
                    normal["type"] = "VEC2"
                elif issue == "normal_zero":
                    for component in range(3):
                        change_first(document, binary, attributes["NORMAL"], "f", 0, component=component)
                elif issue == "uv_count":
                    uv["count"] -= 1
                elif issue == "uv_type":
                    uv["type"] = "SCALAR"
                else:
                    uv["componentType"] = 5123
                file = Path(directory) / "broken.glb"
                write_glb(file, document, binary)
                with self.assertRaisesRegex(ValueError, message):
                    inspect(file, root_name="Root", height=1.7, require_texture=True)

    def test_normalized_integer_uvs(self):
        document, binary = read_glb(FIXTURE)
        binary = bytearray(binary)
        index = document["meshes"][0]["primitives"][0]["attributes"]["TEXCOORD_0"]
        accessor = document["accessors"][index]
        uvs = values(document, binary, accessor)
        view = document["bufferViews"][accessor["bufferView"]]
        for row, uv in enumerate(uvs):
            struct.pack_into("<2H", binary, view["byteOffset"] + row * 4, *(round(value * 65535) for value in uv))
        accessor.update(componentType=5123, normalized=True)
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / "integer-uv.glb"
            write_glb(file, document, binary)
            self.assertEqual(inspect(file, root_name="Root", height=1.7, require_texture=True)["status"], "PASS")


if __name__ == "__main__":
    unittest.main()
