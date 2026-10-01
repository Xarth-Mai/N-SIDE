"""Read the four known building exports; emit semantic fingerprints without opening Blender."""
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, values

ASSETS = (
    ("V-15", "mirror-hall-facade.blend", "v15-mirror-hall-facade.glb",
     ("V15_ambientCG_Plaster001", "V15_ambientCG_Concrete034", "V15_ambientCG_WoodSiding009")),
    ("V-A08", "facade.blend", "v-a08-facade.glb", ("Plaster", "Concrete")),
    ("V-55", "facade.blend", "v55-workshop-facade.glb", ("Plaster", "Concrete")),
    ("V-35", "facade.blend", "v35-byte-beat-facade.glb", ("Plaster", "Concrete")),
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def semantic_digest(value):
    return digest(json.dumps(value, sort_keys=True).encode())


def snapshot():
    report = {}
    for owner, master, filename, roles in ASSETS:
        source = ROOT / "source-assets/buildings" / owner / master
        runtime = ROOT / "game/assets/environment/buildings" / filename
        doc, binary = read_glb(runtime)
        protected_materials = copy.deepcopy(doc["materials"])
        normals, normal_images = {}, set()
        for material in protected_materials:
            if material["name"] not in roles:
                continue
            normal = material["normalTexture"]
            image_index = doc["textures"][normal["index"]]["source"]
            image = doc["images"][image_index]
            view = doc["bufferViews"][image["bufferView"]]
            start = view.get("byteOffset", 0)
            normals[material["name"]] = {"scale": normal.pop("scale", 1), "image": image_index,
                                         "image_sha256": digest(binary[start:start + view["byteLength"]])}
            normal_images.add(image_index)
        assert set(normals) == set(roles)
        remaining_images = {}
        for index, image in enumerate(doc["images"]):
            if index not in normal_images:
                view = doc["bufferViews"][image["bufferView"]]
                start = view.get("byteOffset", 0)
                remaining_images[index] = digest(binary[start:start + view["byteLength"]])
        meshes = []
        for mesh in doc["meshes"]:
            primitives = []
            for primitive in mesh["primitives"]:
                assert not primitive.get("targets"), "expected static building geometry"
                attributes = {}
                for key, index in {**primitive["attributes"], "indices": primitive["indices"]}.items():
                    accessor = doc["accessors"][index]
                    attributes[key] = {"values": values(doc, binary, accessor),
                                       "metadata": {k: accessor[k] for k in ("componentType", "type", "normalized", "min", "max") if k in accessor}}
                primitives.append({"material": primitive["material"], "mode": primitive.get("mode", 4), "attributes": attributes})
            meshes.append({"name": mesh.get("name"), "primitives": primitives})
        protected = {"geometry_accessors_uv_normals_indices": semantic_digest(meshes),
                     "materials_except_target_normal_scale": semantic_digest(protected_materials),
                     "non_target_image_bytes": remaining_images,
                     "nodes_scenes_samplers_texture_bindings": semantic_digest({k: doc.get(k) for k in ("nodes", "scenes", "scene", "samplers", "textures")})}
        report[owner] = {"source": str(source.relative_to(ROOT)), "source_sha256": digest(source.read_bytes()),
                         "runtime": str(runtime.relative_to(ROOT)), "runtime_sha256": digest(runtime.read_bytes()),
                         "normals": normals, "protected": protected}
    return report


if __name__ == "__main__":
    print(json.dumps({"scope": "four frozen building exports; editable geometry not opened in Blender", "assets": snapshot()}, indent=2))
