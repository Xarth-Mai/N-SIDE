"""Compare the frozen V-35 GLB with the normal-strength repair."""
import copy
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, values

old_path = ROOT / "source-assets/buildings/V-35/candidate.glb"
new_path = ROOT / "game/assets/environment/buildings/v35-byte-beat-facade.glb"
old, old_bin = read_glb(old_path)
new, new_bin = read_glb(new_path)
for key in ("scenes", "nodes", "textures", "samplers"):
    assert old.get(key) == new.get(key), key
for previous, current in zip(old["meshes"][0]["primitives"], new["meshes"][0]["primitives"], strict=True):
    assert previous["material"] == current["material"]
    assert previous.get("mode", 4) == current.get("mode", 4)
    assert previous["attributes"].keys() == current["attributes"].keys()
    for key in previous["attributes"]:
        assert values(old, old_bin, old["accessors"][previous["attributes"][key]]) == values(new, new_bin, new["accessors"][current["attributes"][key]]), key
    assert values(old, old_bin, old["accessors"][previous["indices"]]) == values(new, new_bin, new["accessors"][current["indices"]]), "indices"


def color_bytes(doc, binary, material):
    texture = material["pbrMetallicRoughness"]["baseColorTexture"]["index"]
    image = doc["images"][doc["textures"][texture]["source"]]
    view = doc["bufferViews"][image["bufferView"]]
    start = view.get("byteOffset", 0)
    return binary[start:start + view["byteLength"]]


changed = []
for previous, current in zip(old["materials"], new["materials"], strict=True):
    before, after = copy.deepcopy(previous), copy.deepcopy(current)
    if previous["name"] in {"Plaster", "Concrete"}:
        assert before["normalTexture"].pop("scale", 1) == .25
        assert after["normalTexture"].pop("scale", 1) == 1
        assert color_bytes(old, old_bin, previous) == color_bytes(new, new_bin, current), "color bytes"
        changed.append({"material": previous["name"], "before_scale": .25, "after_scale": current["normalTexture"].get("scale", 1)})
    assert before == after, previous["name"]
assert len(changed) == 2
print(json.dumps({"status": "PASS", "geometry_accessors_indices_uv_normals": "exactly unchanged",
                  "nodes_scenes_texture_bindings_samplers": "unchanged", "other_material_values": "unchanged",
                  "original_color_image_bytes": "unchanged", "normal_scales": changed,
                  "before_sha256": hashlib.sha256(old_path.read_bytes()).hexdigest(),
                  "after_sha256": hashlib.sha256(new_path.read_bytes()).hexdigest()}, indent=2))
