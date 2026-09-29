"""Independent CLI mutations: no imports from the validator or its test helpers."""

import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIXTURE = ROOT / "tools/tests/fixtures/character.glb"
raw = FIXTURE.read_bytes()
json_length = struct.unpack_from("<I", raw, 12)[0]
source = json.loads(raw[20:20 + json_length])
results = []
cases = ("valid", "index_range", "index_type", "index_count", "broken_png", "root_first_key", "fake_root", "normal_type", "normal_zero", "uv_count", "uv_type")
with tempfile.TemporaryDirectory(prefix="nside-character-qa-") as directory:
    for case in cases:
        document = json.loads(json.dumps(source))
        binary = bytearray(raw[28 + json_length:28 + json_length + source["buffers"][0]["byteLength"]])
        accessors, views = document["accessors"], document["bufferViews"]
        primitive = document["meshes"][0]["primitives"][0]

        def position(index):
            accessor = accessors[index]
            return views[accessor["bufferView"]].get("byteOffset", 0) + accessor.get("byteOffset", 0)

        if case == "index_range":
            struct.pack_into("<H", binary, position(primitive["indices"]), 24)
        elif case == "index_type":
            accessors[primitive["indices"]]["componentType"] = 5122
        elif case == "index_count":
            accessors[primitive["indices"]]["count"] -= 1
        elif case == "broken_png":
            image_view = views[document["images"][0]["bufferView"]]
            binary[image_view["byteOffset"] + 8:image_view["byteOffset"] + image_view["byteLength"]] = bytes(image_view["byteLength"] - 8)
        elif case == "root_first_key":
            animation = document["animations"][0]
            channel = next(c for c in animation["channels"] if c["target"]["path"] == "translation" and document["nodes"][c["target"]["node"]]["name"] == "Root")
            struct.pack_into("<f", binary, position(animation["samplers"][channel["sampler"]]["output"]), 0.1)
        elif case == "fake_root":
            next(n for n in document["nodes"] if n["name"] == "Root")["name"] = "ActualRoot"
            document["scenes"][0]["nodes"].append(len(document["nodes"]))
            document["nodes"].append({"name": "Root"})
        elif case == "normal_type":
            accessors[primitive["attributes"]["NORMAL"]]["type"] = "VEC2"
        elif case == "normal_zero":
            struct.pack_into("<3f", binary, position(primitive["attributes"]["NORMAL"]), 0, 0, 0)
        elif case == "uv_count":
            accessors[primitive["attributes"]["TEXCOORD_0"]]["count"] -= 1
        elif case == "uv_type":
            accessors[primitive["attributes"]["TEXCOORD_0"]]["type"] = "SCALAR"
        encoded = json.dumps(document).encode()
        encoded += b" " * (-len(encoded) % 4)
        binary += bytes(-len(binary) % 4)
        path = Path(directory) / f"{case}.glb"
        path.write_bytes(struct.pack("<5I", 0x46546C67, 2, 28 + len(encoded) + len(binary), len(encoded), 0x4E4F534A) + encoded + struct.pack("<2I", len(binary), 0x004E4942) + binary)
        command = [sys.executable, "-B", str(ROOT / "tools/validate_character.py"), str(path), "--root-node", "Root", "--height", "1.7", "--clip", "Idle", "--clip", "Sway", "--require-texture"]
        run = subprocess.run(command, capture_output=True, text=True)
        report = json.loads(run.stdout)
        expected_exit = 0 if case == "valid" else 1
        results.append({"case": case, "expected_exit": expected_exit, "exit_code": run.returncode, "status": report["status"], "error": report.get("error"), "passed": run.returncode == expected_exit})
print(json.dumps({"fixture_sha256": hashlib.sha256(raw).hexdigest(), "command": "python3 -B tools/validate_character.py <mutated-file> --root-node Root --height 1.7 --clip Idle --clip Sway --require-texture", "results": results, "temporary_files_removed": True}, indent=2))
sys.exit(0 if all(r["passed"] for r in results) else 1)
