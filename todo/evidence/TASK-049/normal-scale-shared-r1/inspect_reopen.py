"""Identify exact saved-master differences without writing any source asset."""
import json
from pathlib import Path
import bpy

HERE = Path(__file__).resolve().parent
script = HERE / "candidates/patch_source.py"
code = script.read_text().replace('return {"objects_mesh_uv_normals_transforms_fonts": digest(objects), "other_material_values_images_links": digest(materials)}', 'return {"objects": objects, "materials": materials}')
scope = {"__file__": str(script), "__name__": "inspect_only"}
exec(compile(code, str(script), "exec"), scope)
root = scope["ROOT"]
reports = {}
for asset, (filename, strength, prefix, stems) in scope["ASSETS"].items():
    states = []
    for master in [root / "output/assets/normal-scale-shared-r1" / asset / filename,
                   root / "source-assets/buildings" / asset / filename]:
        bpy.ops.wm.open_mainfile(filepath=str(master))
        targets = {}
        for stem in stems:
            role = prefix + stem if prefix else {"Plaster001": "Plaster", "Concrete034": "Concrete"}[stem]
            normal = bpy.data.materials[role].node_tree.nodes["Principled BSDF"].inputs["Normal"].links[0].from_node
            targets[role] = (normal, normal.inputs["Color"].links[0].from_node, stem)
        states.append(scope["snapshot"](targets))
    before, after = states
    assert before["objects"] == after["objects"]
    reports[asset] = {name: {"before": before["materials"].get(name), "after": after["materials"].get(name)}
                      for name in before["materials"].keys() | after["materials"].keys()
                      if before["materials"].get(name) != after["materials"].get(name)}
print(json.dumps(reports, indent=2))
