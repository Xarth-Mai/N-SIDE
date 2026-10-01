"""Patch one frozen Blender master to shared normal PNGs; save only, never export."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import bpy

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
ASSETS = {
    "V-15": ("mirror-hall-facade.blend", .35, "V15_ambientCG_", ("Plaster001", "Concrete034", "WoodSiding009")),
    "V-A08": ("facade.blend", .28, "", ("Plaster001", "Concrete034")),
    "V-55": ("facade.blend", .25, "", ("Plaster001", "Concrete034")),
    "V-35": ("facade.blend", .25, "", ("Plaster001", "Concrete034")),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def digest(value):
    return sha(json.dumps(value, sort_keys=True).encode())


def scalar_or_array(value):
    return list(value) if hasattr(value, "__len__") and not isinstance(value, str) else value


def snapshot(targets):
    objects = []
    for obj in sorted(bpy.context.scene.objects, key=lambda item: item.name):
        data = obj.data
        item = {"name": obj.name, "type": obj.type, "data": data.name if data else None,
                "world": [list(row) for row in obj.matrix_world], "local": [list(row) for row in obj.matrix_basis],
                "parent": obj.parent.name if obj.parent else None,
                "modifiers": [{"name": mod.name, "type": mod.type,
                               **{key: scalar_or_array(getattr(mod, key)) for key in
                                  ("show_viewport", "show_render", "width", "segments", "limit_method", "angle_limit", "affect", "harden_normals")
                                  if hasattr(mod, key)}} for mod in obj.modifiers]}
        if obj.type in {"MESH", "FONT"}:
            item["materials"] = [mat.name if mat else None for mat in data.materials]
        if obj.type == "MESH":
            item.update(positions=[list(vertex.co) for vertex in data.vertices],
                        normals=[list(normal.vector) for normal in data.corner_normals],
                        edges=[list(edge.vertices) for edge in data.edges],
                        faces=[(list(poly.vertices), poly.material_index, poly.use_smooth) for poly in data.polygons],
                        uv={uv.name: [list(loop.uv) for loop in uv.data] for uv in data.uv_layers})
        elif obj.type == "FONT":
            item["curve"] = {key: getattr(data, key) for key in
                             ("body", "size", "shear", "offset", "offset_x", "offset_y", "extrude", "bevel_depth", "bevel_resolution",
                              "resolution_u", "render_resolution_u", "dimensions", "fill_mode", "align_x", "align_y",
                              "space_character", "space_word", "space_line", "small_caps_scale", "underline_position", "underline_height")}
            item["fonts"] = {key: {"name": font.name, "path": font.filepath,
                                    "packed": sha(font.packed_file.data) if font.packed_file else None}
                             for key in ("font", "font_bold", "font_italic", "font_bold_italic")
                             if (font := getattr(data, key)) is not None}
            item["text_boxes"] = [[box.x, box.y, box.width, box.height] for box in data.text_boxes]
            item["format"] = [[getattr(char, key) for key in
                               ("use_bold", "use_italic", "use_underline", "use_small_caps", "material_index")]
                              for char in data.body_format]
        objects.append(item)
    materials = {}
    for mat in bpy.data.materials:
        target = targets.get(mat.name)
        nodes = []
        for node in mat.node_tree.nodes if mat.node_tree else ():
            inputs = [(socket.identifier, socket.name, scalar_or_array(socket.default_value)) for socket in node.inputs
                      if hasattr(socket, "default_value") and not (target and node == target[0] and socket.name == "Strength")]
            item = {"name": node.name, "type": node.type, "mute": node.mute, "inputs": inputs,
                    "settings": {key: getattr(node, key) for key in
                                 ("space", "uv_map", "interpolation", "projection", "projection_blend", "extension") if hasattr(node, key)}}
            if node.type == "TEX_IMAGE":
                image = node.image
                assert image and image.packed_file, (mat.name, node.name, "expected packed image")
                item["image"] = {"size": list(image.size), "colorspace": image.colorspace_settings.name,
                                 "alpha_mode": image.alpha_mode, "source": image.source}
                if not (target and node == target[1]):
                    item["image"].update(name=image.name, path=image.filepath, packed=sha(image.packed_file.data))
            nodes.append(item)
        materials[mat.name] = {"color": list(mat.diffuse_color), "roughness": mat.roughness, "metallic": mat.metallic,
                               "settings": {key: getattr(mat, key) for key in
                                            ("use_nodes", "use_backface_culling", "surface_render_method", "use_transparency_overlap")
                                            if hasattr(mat, key)}, "nodes": nodes,
                               "links": sorted((link.from_node.name, link.from_socket.identifier, link.to_node.name, link.to_socket.identifier)
                                               for link in mat.node_tree.links) if mat.node_tree else []}
    return {"objects_mesh_uv_normals_transforms_fonts": digest(objects), "other_material_values_images_links": digest(materials)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset", choices=ASSETS, required=True)
    parser.add_argument("--baseline", type=Path, default=HERE.parent / "before-semantics.json")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    filename, strength, prefix, stems = ASSETS[args.asset]
    master = ROOT / "source-assets/buildings" / args.asset / filename
    baseline = json.loads(args.baseline.read_text())["assets"][args.asset]
    assert baseline["source"] == str(master.relative_to(ROOT)), "baseline selects another master"
    assert sha(master.read_bytes()) == baseline["source_sha256"], "master drifted from before-semantics.json"
    bpy.ops.wm.open_mainfile(filepath=str(master))
    targets = {}
    for stem in stems:
        role = prefix + stem if prefix else {"Plaster001": "Plaster", "Concrete034": "Concrete"}[stem]
        shader = bpy.data.materials[role].node_tree.nodes["Principled BSDF"]
        assert len(shader.inputs["Normal"].links) == 1
        normal = shader.inputs["Normal"].links[0].from_node
        assert normal.type == "NORMAL_MAP" and not normal.inputs["Strength"].is_linked
        assert len(normal.inputs["Color"].links) == 1
        texture = normal.inputs["Color"].links[0].from_node
        assert texture.type == "TEX_IMAGE" and texture.image.packed_file
        expected = baseline["normals"][role]
        assert abs(normal.inputs["Strength"].default_value - expected["scale"]) < 1e-7, "source strength differs from frozen export"
        assert abs(expected["scale"] - (1 if args.asset == "V-35" else strength)) < 1e-7
        assert sha(texture.image.packed_file.data) == expected["image_sha256"], "packed normal differs from frozen export"
        assert texture.image.colorspace_settings.name == "Non-Color"
        targets[role] = (normal, texture, stem)
    assert set(targets) == set(baseline["normals"]), "target role set differs from baseline"
    before = snapshot(targets)
    changes = []
    for role, (normal, texture, stem) in targets.items():
        original = texture.image
        old_hash, old_path = sha(original.packed_file.data), original.filepath
        derived = ROOT / "source-assets/environment-kit/materials" / f"{stem}-NormalGL-scale{round(strength * 100):03d}.png"
        expected_hash = sha(derived.read_bytes())
        if args.asset == "V-35":
            assert old_hash == expected_hash, "V-35 migration must preserve packed PNG bytes"
            original.filepath_raw = str(derived)
        else:
            texture.image = bpy.data.images.load(str(derived), check_existing=True)
            texture.image.colorspace_settings.name = "Non-Color"
            texture.image.pack()
            normal.inputs["Strength"].default_value = 1
            if original.users == 0:
                bpy.data.images.remove(original)
        assert normal.inputs["Strength"].default_value == 1
        assert sha(texture.image.packed_file.data) == expected_hash
        changes.append({"material": role, "before_image_sha256": old_hash, "before_path": old_path,
                        "after_image_sha256": expected_hash, "after_path": str(derived.relative_to(ROOT)), "strength": 1})
    after = snapshot(targets)
    assert before == after, {"before": before, "after": after}
    assert sha(master.read_bytes()) == baseline["source_sha256"], "master changed on disk before save"
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=str(master))
    print(json.dumps({"status": "PASS", "asset": args.asset, "source": str(master.relative_to(ROOT)),
                      "before_sha256": baseline["source_sha256"], "after_sha256": sha(master.read_bytes()),
                      "before": before, "after": after, "changes": changes,
                      "blender": bpy.app.version_string, "runtime_export": "NOT RUN; invoke the existing build entry separately"}, indent=2))


if __name__ == "__main__":
    main()
