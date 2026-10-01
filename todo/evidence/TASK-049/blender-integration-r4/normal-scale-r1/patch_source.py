"""Replace only V-35's two packed normal maps; preserve editable geometry and other material values."""
import hashlib
import json
from pathlib import Path
import runpy

import bpy

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
SOURCE = ROOT / "source-assets/buildings/V-35"


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def snapshot():
    meshes = []
    for obj in sorted(bpy.context.scene.objects, key=lambda obj: obj.name):
        mesh = obj.data
        assert obj.type == "MESH", obj.name
        meshes.append({"name": obj.name, "matrix": [list(row) for row in obj.matrix_world],
                       "positions": [list(v.co) for v in mesh.vertices],
                       "normals": [list(v.vector) for v in mesh.corner_normals],
                       "faces": [(list(p.vertices), p.material_index, p.use_smooth) for p in mesh.polygons],
                       "uv": {uv.name: [list(v.uv) for v in uv.data] for uv in mesh.uv_layers},
                       "materials": [m.name for m in mesh.materials]})
    materials = {}
    for mat in bpy.data.materials:
        nodes = []
        for node in mat.node_tree.nodes:
            inputs = {}
            for socket in node.inputs:
                if not hasattr(socket, "default_value"):
                    continue
                if mat.name in {"Plaster", "Concrete"} and node.type == "NORMAL_MAP" and socket.name == "Strength":
                    continue
                value = socket.default_value
                inputs[socket.name] = list(value) if hasattr(value, "__len__") and not isinstance(value, str) else value
            image = None
            if node.type == "TEX_IMAGE" and "NormalGL" not in node.image.name:
                image = hashlib.sha256(node.image.packed_file.data).hexdigest()
            nodes.append({"name": node.name, "type": node.type, "inputs": inputs, "color_image": image})
        materials[mat.name] = {"color": list(mat.diffuse_color), "roughness": mat.roughness,
                               "metallic": mat.metallic, "nodes": nodes,
                               "links": [(l.from_node.name, l.from_socket.name, l.to_node.name, l.to_socket.name) for l in mat.node_tree.links]}
    return {"mesh_uv_normal_transform_hash": digest(meshes), "other_material_values_hash": digest(materials),
            "mesh_groups": len(meshes)}


bpy.ops.wm.open_mainfile(filepath=str(SOURCE / "facade.blend"))
before = snapshot()
changes = []
for role, stem in (("Plaster", "Plaster001"), ("Concrete", "Concrete034")):
    normal = next(n for n in bpy.data.materials[role].node_tree.nodes if n.type == "NORMAL_MAP")
    texture = normal.inputs["Color"].links[0].from_node
    original = texture.image
    old_hash = hashlib.sha256(original.packed_file.data).hexdigest()
    derived = SOURCE / f"textures/{stem}-NormalGL-scale025.png"
    texture.image = bpy.data.images.load(str(derived), check_existing=True)
    texture.image.colorspace_settings.name = "Non-Color"
    texture.image.pack()
    normal.inputs["Strength"].default_value = 1
    if original.users == 0:
        bpy.data.images.remove(original)
    changes.append({"material": role, "old_packed_normal_sha256": old_hash,
                    "derived": str(derived.relative_to(ROOT)),
                    "new_packed_normal_sha256": hashlib.sha256(texture.image.packed_file.data).hexdigest(),
                    "strength": normal.inputs["Strength"].default_value})
after = snapshot()
assert before == after, {"before": before, "after": after}
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE / "facade.blend"))
build = runpy.run_path(str(SOURCE / "build.py"), run_name="v35_export")
build["export"]()
(HERE / "source-preservation.json").write_text(json.dumps({"status": "PASS", "before": before,
                                                           "after": after, "changes": changes,
                                                           "blender": bpy.app.version_string}, indent=2) + "\n")
