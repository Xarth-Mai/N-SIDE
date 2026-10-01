"""Check the runtime GLB and editable source map contract; engine acceptance is separate."""
import hashlib
import json
import math
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, require, values
MATERIALS = ROOT / "source-assets/environment-kit/materials"
sys.path.insert(0, str(MATERIALS))


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def sub(a, b): return tuple(x-y for x, y in zip(a, b))
def dot(a, b): return sum(x*y for x, y in zip(a, b))
def length(v): return math.sqrt(dot(v, v))
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def main():
    runtime = ROOT / "game/assets/environment/buildings/v35-byte-beat-facade.glb"
    doc, binary = read_glb(runtime)
    require(doc.get("scene", 0) == 0 and len(doc["scenes"]) == len(doc["nodes"]) == len(doc["meshes"]) == 1, "expected one Scene0, root and mesh")
    require(set(doc["nodes"][0]) <= {"name", "mesh"}, "single source anchor needs identity root transform")
    require(not any(doc.get(key) for key in ("skins", "animations", "cameras", "extensionsUsed")), "unexpected animation or material extension")
    roles = {m["name"]: m for m in doc["materials"]}
    require(set(roles) == {"Plaster", "Concrete", "Graphite", "Mauve", "Lime", "Glass", "Paper"}, "seven authored material roles changed")
    require(all(m.get("alphaMode", "OPAQUE") == "OPAQUE" for m in roles.values()), "facade uses opaque backing, not unverified transparent interior")
    require(len(doc["images"]) == 4 and all("bufferView" in image and "uri" not in image for image in doc["images"]), "two original colors and two baked normals must be embedded")
    records = json.loads((ROOT / "source-assets/environment-kit/asset-manifest.json").read_text())["files"]
    textures, derived_normals = {}, {}
    for role, stem in (("Plaster", "Plaster001"), ("Concrete", "Concrete034")):
        material = roles[role]
        for suffix, index in (("Color", material["pbrMetallicRoughness"]["baseColorTexture"]["index"]), ("NormalGL", material["normalTexture"]["index"])):
            image = doc["images"][doc["textures"][index]["source"]]
            view = doc["bufferViews"][image["bufferView"]]; start = view.get("byteOffset", 0)
            embedded = binary[start:start+view["byteLength"]]
            source = ROOT / f"source-assets/environment-kit/materials/{stem}_1K-JPG_{suffix}.jpg"
            digest = sha(source)
            record = next((r for r in records if r["source"] == f"materials/{source.name}"), None)
            require(record and record["license"] == "CC0-1.0" and record["sha256"] == digest, "PBR source license/hash differs from AST-003")
            if suffix == "Color":
                require(hashlib.sha256(embedded).hexdigest() == digest, "GLB changed original color bytes")
            else:
                derived = MATERIALS / f"{stem}-NormalGL-scale025.png"
                require(image["mimeType"] == "image/png" and hashlib.sha256(embedded).hexdigest() == sha(derived), "GLB normal differs from lossless derived PNG")
                require(material["normalTexture"].get("scale", 1) == 1, "baked normal must use glTF scale 1")
                if "--source" not in sys.argv:
                    from PIL import Image
                    from bake_normals import baked_normal
                    expected = baked_normal(source, .25)
                    with Image.open(derived) as actual:
                        require(actual.mode == expected.mode and actual.size == expected.size and actual.tobytes() == expected.tobytes(), "derived normal does not match glTF xy*.25, z unchanged, normalize")
                derived_normals[str(derived.relative_to(ROOT))] = {"sha256": sha(derived), "source": str(source.relative_to(ROOT)), "source_sha256": digest, "baked_scale": .25, "gltf_scale": 1}
            textures[str(source.relative_to(ROOT))] = digest
    triangles, positions, ratios = [], [], []
    count_vertices = 0
    for primitive in doc["meshes"][0]["primitives"]:
        require(primitive.get("mode", 4) == 4, "non-triangle primitive")
        attrs = primitive["attributes"]
        require({"POSITION", "NORMAL", "TEXCOORD_0"} <= attrs.keys(), "missing positions/normals/UVs")
        read = lambda index: values(doc, binary, doc["accessors"][index])
        ps, ns, uv = (read(attrs[key]) for key in ("POSITION", "NORMAL", "TEXCOORD_0"))
        ids = [r[0] for r in read(primitive["indices"])]
        require(len(ids) % 3 == 0 and all(0 <= i < len(ps) for i in ids), "invalid indices")
        require(all(math.isfinite(v) for rows in (ps, ns, uv) for row in rows for v in row), "nonfinite mesh")
        require(all(abs(length(n)-1) < 1e-4 for n in ns), "non-unit normals")
        role = doc["materials"][primitive["material"]]["name"]
        repeat = {"Plaster": (2, 2), "Concrete": (1.1, .55)}.get(role)
        for start in range(0, len(ids), 3):
            a, b, c = ids[start:start+3]
            normal = cross(sub(ps[b], ps[a]), sub(ps[c], ps[a])); area = length(normal)
            require(area > 1e-9, f"degenerate triangle in {role}")
            require(all(dot(normal, ns[i])/area > .99 for i in (a, b, c)), f"normal/winding mismatch in {role}")
            if repeat:
                for i, j in ((a, b), (b, c), (c, a)):
                    ratio = length([(uv[j][k]-uv[i][k])*repeat[k] for k in range(2)]) / length(sub(ps[j], ps[i]))
                    require(abs(ratio-1) < .002, "PBR UV metric stretch")
                    ratios.append(ratio)
            triangles.append([ps[a], ps[b], ps[c]])
        positions.extend(ps); count_vertices += len(ps)
    require(0 < len(triangles) < 8000, f"north-facade triangle budget exceeded: {len(triangles)}")
    bounds = [[min(p[i] for p in positions), max(p[i] for p in positions)] for i in range(3)]
    require(all(abs(v-e) < 1e-4 for row, expected in zip(bounds, ((-16, 16), (-.18, 11), (-1.5, -.02))) for v, e in zip(row, expected)), f"unexpected bounds {bounds}")
    district = json.loads((ROOT / "source-assets/district-map/district.json").read_text())
    building = next(b for b in district["buildings"] if b["id"] == "V-35")
    require(building["polygon"] == [[-402,113],[-370,113],[-370,153],[-402,153]] and building["elevation"] == 12 and building["height"] == 11, "source shell changed")
    require([f["z"] for f in building["design"]["floors"]] == [12,18], "arcade floor levels changed")
    require([district["nodes"][e["node"]] for e in building["design"]["entries"]] == [[-386,153,12],[-386,113,12]], "source entries changed")
    # The existing sign/frame must remain visible without coplanar overlays
    for tri in triangles:
        lo, hi = [(-3.08,3.17,-.366),(3.08,4.83,-.1201)]
        require(not all(max(p[k] for p in tri) > lo[k] and min(p[k] for p in tri) < hi[k] for k in range(3)), "facade overlaps retained BYTE BEAT sign")
    require(building["design"]["canopy"] == 1.5, "source canopy depth changed")
    approaches = []
    for start_name, door_name in (("game_front_court", "game_entry"),):
        start, end = (district["nodes"][name] for name in (start_name, door_name))
        distance = math.hypot(end[0]-start[0], end[1]-start[1]); stop = 1-.55/distance
        for sample in range(41):
            t = stop * sample / 40
            x, north, elevation = (a+(b-a)*t for a, b in zip(start, end))
            center = [x+386, elevation-12+.025, 153-north]
            lo = [center[0]-.32, center[1]+.02, center[2]-.32]
            hi = [center[0]+.32, center[1]+1.72, center[2]+.32]
            hits = [tri for tri in triangles if all(max(p[k] for p in tri) > lo[k]+1e-6 and min(p[k] for p in tri) < hi[k]-1e-6 for k in range(3))]
            require(not hits, f"{door_name} public approach AABB overlaps {len(hits)} triangles at {t:.3f}")
        approaches.append({"start": start_name, "door": door_name, "samples": 41, "stop_before_closed_door_m": .55, "half_width_m": .32, "height_m": 1.7})
    report = {"status": "PASS", "file": str(runtime.relative_to(ROOT)), "sha256": sha(runtime), "source_sha256": sha(HERE/"facade.blend"), "bytes": runtime.stat().st_size,
              "triangles": len(triangles), "vertices": count_vertices, "materials": len(roles), "images": 4, "runtime_bounds_xyz": bounds,
              "anchor_map_xyz": [-386,153,12], "textured_uv_metric_ratio": [min(ratios),max(ratios)], "texture_sources": textures, "derived_normals": derived_normals,
              "normal_pixel_check": "NOT RUN in --source mode; run system Python for pixels" if "--source" in sys.argv else "PASS",
              "retained_sign_source_sha256": sha(ROOT/"source-assets/district-scene/byte-beat.svg"), "approaches": approaches, "approach_scope": "Conservative triangle-AABB broad check with source linear road elevations; actual Rust Ground/capsule and runtime NOT RUN"}
    if "--source" in sys.argv:
        import bpy
        bpy.ops.wm.open_mainfile(filepath=str(HERE/"facade.blend"))
        collection = bpy.data.collections["V35_EXPORT_North"]
        objects = list(collection.objects)
        require(len(objects) == len(bpy.context.scene.objects), "review-only context leaked into source")
        require(bpy.context.scene["owner_id"] == "V-35" and list(bpy.context.scene["pivot_map_xyz"]) == [-386,153,12], "source placement metadata changed")
        require(json.loads(bpy.context.scene["source_building"]) == building, "master uses another building source")
        require(json.loads(bpy.context.scene["door_nodes"]) == {e["node"]: district["nodes"][e["node"]] for e in building["design"]["entries"]}, "master doors differ from source")
        labels = {obj.name: obj.data.body for obj in objects if obj.type == "FONT"}
        require(not labels, "existing sign is retained independently, not reauthored as text")
        require(all(obj.type in {"MESH","FONT"} and not obj.modifiers and not obj.constraints and not obj.animation_data for obj in objects), "unexpected dynamic source object")
        images = [i for i in bpy.data.images if i.type == "IMAGE" and i.size[0]>0]
        require(len(images) == 4 and all(i.packed_file for i in images), "four texture masters must remain packed")
        require(all(i.colorspace_settings.name == ("Non-Color" if "NormalGL" in i.name else "sRGB") for i in images), "source texture color spaces changed")
        for role, stem in (("Plaster", "Plaster001"), ("Concrete", "Concrete034")):
            shader = bpy.data.materials[role].node_tree.nodes["Principled BSDF"]
            require(len(shader.inputs["Normal"].links) == 1, "source normal must be bound to shader")
            normal = shader.inputs["Normal"].links[0].from_node
            require(normal.type == "NORMAL_MAP" and not normal.inputs["Strength"].is_linked and normal.inputs["Strength"].default_value == 1, "source normal must use Strength 1")
            require(len(normal.inputs["Color"].links) == 1, "source normal must have one image input")
            texture = normal.inputs["Color"].links[0].from_node
            require(texture.type == "TEX_IMAGE" and texture.image and texture.image.packed_file, "source normal must use a packed image")
            image = texture.image; derived = MATERIALS / f"{stem}-NormalGL-scale025.png"
            require(image.colorspace_settings.name == "Non-Color" and Path(bpy.path.abspath(image.filepath)).resolve() == derived.resolve(), "source normal must reference shared Non-Color PNG")
            require(hashlib.sha256(image.packed_file.data).hexdigest() == sha(derived), "source packed normal differs from shared PNG")
        require(len([o for o in objects if o.name.startswith("V35_Showcase_Arcade_")]) == 6, "six original showcase machines must remain editable")
        report["blender_source"] = {"status":"PASS", "version":bpy.app.version_string, "objects":len(objects), "mesh_groups":len(objects)-len(labels), "display_machines":bpy.context.scene["display_machine_count"], "windows":bpy.context.scene["window_count"], "packed_images":4}
    print(json.dumps(report, indent=2, ensure_ascii=False))


if __name__ == "__main__": main()
