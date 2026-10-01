"""Read-only integrity and geometry check for the selected aircon source package."""
import hashlib
import io
import json
import math
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT.parents[2] / "tools"))
from glb import read_glb, require, values

provenance = json.loads((ROOT / "provenance.json").read_text())
api = json.loads((ROOT / "aircon-files.json").read_text())
original_paths = [entry["path"] for entry in provenance["originals"]]
require(len(original_paths) == 9 and len(set(original_paths)) == 9, "expected nine distinct downloaded originals")
require(set(original_paths) == {p.relative_to(ROOT).as_posix() for p in (ROOT / "original").rglob("*") if p.is_file()}, "original-file coverage differs from provenance")
require({entry["path"] for entry in provenance["candidate_files"]} == {"aircon-candidate.blend", "aircon-candidate.glb"} and len(provenance["candidate_files"]) == 2, "expected both candidate source and export")
require("CC0 1.0 Universal" in (ROOT / provenance["license_file"]).read_text(), "retained license missing")
official = api["gltf"]["1k"]["gltf"]
official_files = {"exterior_aircon_unit_1k.gltf": official, **official["include"]}
official_files["textures/exterior_aircon_unit_02_diff_1k.jpg"] = api["02_diff"]["1k"]["jpg"]
for entry in provenance["originals"]:
    path = ROOT / entry["path"]
    source = official_files[path.relative_to(ROOT / "original").as_posix()]
    raw = path.read_bytes()
    require(entry["url"] == source["url"], f"{path}: source URL mismatch")
    require(len(raw) == entry["official_size"] == source["size"], f"{path}: size mismatch")
    require(hashlib.md5(raw).hexdigest() == entry["official_md5"] == source["md5"], f"{path}: official MD5 mismatch")
    require(hashlib.sha256(raw).hexdigest() == entry["sha256"], f"{path}: SHA256 mismatch")
for entry in provenance["candidate_files"]:
    raw = (ROOT / entry["path"]).read_bytes()
    require(len(raw) == entry["bytes"] and hashlib.sha256(raw).hexdigest() == entry["sha256"], f"{entry['path']}: candidate differs from recorded export")

document, binary = read_glb(ROOT / "aircon-candidate.glb")
require(not document.get("extensionsRequired") and not document.get("skins") and not document.get("animations"), "expected static uncompressed model")
require(len(document["nodes"]) == 1 and document["scenes"][document["scene"]]["nodes"] == [0], "expected a single root")
require(not any(k in document["nodes"][0] for k in ("matrix", "translation", "rotation", "scale")), "expected baked identity root")
positions, triangles = [], 0
for mesh in document["meshes"]:
    for primitive in mesh["primitives"]:
        attributes = {key: values(document, binary, document["accessors"][index]) for key, index in primitive["attributes"].items()}
        require({"POSITION", "NORMAL", "TEXCOORD_0", "TANGENT"} <= attributes.keys(), "missing required vertex attribute")
        require(all(math.isfinite(x) for rows in attributes.values() for row in rows for x in row), "nonfinite vertex data")
        require(all(abs(sum(x*x for x in row)-1) < 1e-4 for row in attributes["NORMAL"]), "nonunit normals")
        vertices = attributes["POSITION"]
        positions.extend(vertices)
        indices = [row[0] for row in values(document, binary, document["accessors"][primitive["indices"]])]
        require(primitive.get("mode", 4) == 4 and len(indices) % 3 == 0 and all(0 <= i < len(vertices) for i in indices), "invalid triangle indices")
        triangles += len(indices) // 3
        for a, b, c in zip(indices[::3], indices[1::3], indices[2::3]):
            u = [vertices[b][i] - vertices[a][i] for i in range(3)]
            v = [vertices[c][i] - vertices[a][i] for i in range(3)]
            cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            require(sum(x*x for x in cross) > 4e-24, "degenerate triangle")
low = [min(p[i] for p in positions) for i in range(3)]
high = [max(p[i] for p in positions) for i in range(3)]
require(triangles == 9493 and abs(low[1]) < 1e-6, "unexpected geometry or lower pivot")
require(all(abs(high[i]-low[i]-expected) < 1e-5 for i, expected in enumerate((.799971312, .927897096, .374159217))), "unexpected scale")
materials = document["materials"]
require(len(materials) == 2, "expected two materials")
grille = next(m for m in materials if m["name"] == "exterior_aircon_unit_02")
require(grille.get("alphaMode") == "MASK" and grille.get("alphaCutoff", .5) == .5, "grille alpha mask missing")
image_index = document["textures"][grille["pbrMetallicRoughness"]["baseColorTexture"]["index"]]["source"]
for index, image in enumerate(document["images"]):
    require("uri" not in image, "external runtime texture")
    view = document["bufferViews"][image["bufferView"]]
    offset = view.get("byteOffset", 0)
    with Image.open(io.BytesIO(binary[offset:offset+view["byteLength"]])) as decoded:
        decoded.load()
        require(decoded.size == (1024, 1024), "unexpected texture size")
        if index == image_index:
            require(decoded.mode == "RGBA" and decoded.getchannel("A").getextrema() == (0, 255), "grille texture lacks useful alpha")
print(json.dumps({"status": "PASS", "original_files": len(provenance["originals"]), "triangles": triangles, "bounds_gltf": [low, high], "materials": len(materials), "images": len(document["images"]), "runtime": "NOT RUN", "original_candidate_rebuild_after_migration": "NOT RUN"}, indent=2))
