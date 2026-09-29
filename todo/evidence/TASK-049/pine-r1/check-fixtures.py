"""Exercise the pine checker with real GLB color and alpha corruption."""

from pathlib import Path
import json
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
SOURCE = ROOT / "source-assets/environment-kit/vegetation/pine-street.glb"
CHECK = ROOT / "source-assets/environment-kit/vegetation/pine-check.py"
original = SOURCE.read_bytes()
json_size = struct.unpack_from("<I", original, 12)[0]
document = json.loads(original[20:20 + json_size])
leaf_material = next(i for i, m in enumerate(document["materials"]) if m["name"] == "PineNeedles")
leaf = next(p for p in document["meshes"][0]["primitives"] if p["material"] == leaf_material)
accessor = document["accessors"][leaf["attributes"]["COLOR_0"]]
view = document["bufferViews"][accessor["bufferView"]]
offset = 28 + json_size + view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
color = bytearray(original)
kind = accessor["componentType"]
code, maximum = {5126: ("f", 1.0), 5123: ("H", 65535), 5121: ("B", 255)}[kind]
components = [maximum, 0, maximum] + ([maximum] if accessor["type"] == "VEC4" else [])
struct.pack_into("<" + code * len(components), color, offset, *components)
document["materials"][leaf_material]["alphaMode"] = "BLEND"
payload = json.dumps(document, separators=(",", ":")).encode()
payload += b" " * (-len(payload) % 4)
remaining = original[20 + json_size:]
alpha = struct.pack("<5I", 0x46546C67, 2, 20 + len(payload) + len(remaining), len(payload), 0x4E4F534A) + payload + remaining
results = []
with tempfile.TemporaryDirectory(prefix="nside-pine-check-") as directory:
    for name, data, expected in [("bad-color", color, "needle palette must remain evergreen"),
                                 ("alpha", alpha, "pine must use opaque materials")]:
        path = Path(directory) / f"{name}.glb"
        path.write_bytes(data)
        result = subprocess.run(["python3", str(CHECK), str(path)], text=True, capture_output=True)
        assert result.returncode == 1 and expected in result.stderr, result
        results.append({"fixture": name, "exit_code": result.returncode, "stderr": result.stderr.strip(), "status": "PASS"})
print(json.dumps(results, indent=2))
