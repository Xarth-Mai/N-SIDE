"""Read back the four saved masters and compare the actual pre-save fingerprints."""
import hashlib
import json
from pathlib import Path
import runpy

import bpy

HERE = Path(__file__).resolve().parent
patch = runpy.run_path(str(HERE / "candidates/patch_source.py"))
ROOT = patch["ROOT"]
reports = {}


def targets_for(prefix, stems):
    targets = {}
    for stem in stems:
        role = prefix + stem if prefix else {"Plaster001": "Plaster", "Concrete034": "Concrete"}[stem]
        shader = bpy.data.materials[role].node_tree.nodes["Principled BSDF"]
        normal = shader.inputs["Normal"].links[0].from_node
        targets[role] = (normal, normal.inputs["Color"].links[0].from_node, stem)
    return targets


for asset, (filename, strength, prefix, stems) in patch["ASSETS"].items():
    prior = json.loads((HERE / f"{asset}-source-repair.json").read_text())
    master = ROOT / prior["source"]
    assert hashlib.sha256(master.read_bytes()).hexdigest() == prior["after_sha256"]
    expected = prior["before"]
    removed = []
    if asset in {"V-A08", "V-55"}:
        # Blender drops this observed zero-user default on save; compare every used material
        backup = ROOT / "output/assets/normal-scale-shared-r1" / asset / filename
        assert hashlib.sha256(backup.read_bytes()).hexdigest() == prior["before_sha256"]
        bpy.ops.wm.open_mainfile(filepath=str(backup))
        unused = bpy.data.materials["Material"]
        assert unused.users == 0 and not unused.use_fake_user
        assert patch["snapshot"](targets_for(prefix, stems)) == prior["before"]
        bpy.data.materials.remove(unused)
        expected = patch["snapshot"](targets_for(prefix, stems))
        removed = ["Material: zero users, no fake user; automatically removed on save"]
    bpy.ops.wm.open_mainfile(filepath=str(master))
    targets = targets_for(prefix, stems)
    for role, (normal, texture, stem) in targets.items():
        path = ROOT / "source-assets/environment-kit/materials" / f"{stem}-NormalGL-scale{round(strength * 100):03d}.png"
        assert normal.inputs["Strength"].default_value == 1
        assert texture.image.colorspace_settings.name == "Non-Color"
        assert Path(bpy.path.abspath(texture.image.filepath)).resolve() == path
        assert texture.image.packed_file.data == path.read_bytes()
    actual = patch["snapshot"](targets)
    assert prior["before"] == prior["after"]
    assert actual == expected, (asset, actual, expected)
    assert actual["objects_mesh_uv_normals_transforms_fonts"] == prior["before"]["objects_mesh_uv_normals_transforms_fonts"]
    reports[asset] = {"status": "PASS", "sha256": prior["after_sha256"], "protected": actual,
                      "packed_normal_bytes_paths_strength": "PASS", "unused_datablocks_removed_on_save": removed}
print(json.dumps({"status": "PASS", "blender": bpy.app.version_string, "masters": reports}, indent=2))
