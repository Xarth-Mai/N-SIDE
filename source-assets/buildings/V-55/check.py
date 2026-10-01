"""Check the candidate GLB and source map contract; runtime paths stay with the integrator."""
import hashlib
import json
import math
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, require, values


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def sub(a, b): return tuple(x-y for x, y in zip(a, b))
def dot(a, b): return sum(x*y for x, y in zip(a, b))
def length(v): return math.sqrt(dot(v, v))
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def main():
    candidate = ROOT / "game/assets/environment/buildings/v55-workshop-facade.glb"
    doc, binary = read_glb(candidate)
    require(doc.get("scene", 0) == 0 and len(doc["scenes"]) == len(doc["nodes"]) == len(doc["meshes"]) == 1, "expected one Scene0, root and mesh")
    require(set(doc["nodes"][0]) <= {"name", "mesh"}, "single source anchor needs identity root transform")
    require(not any(doc.get(key) for key in ("skins", "animations", "cameras", "extensionsUsed")), "unexpected animation or material extension")
    roles = {m["name"]: m for m in doc["materials"]}
    require(set(roles) == {"Plaster", "Concrete", "Graphite", "Indigo", "Glass", "Paper"}, "six authored material roles changed")
    require(all(m.get("alphaMode", "OPAQUE") == "OPAQUE" for m in roles.values()), "facade uses opaque backing, not unverified transparent interior")
    require(len(doc["images"]) == 4 and all("bufferView" in image and "uri" not in image for image in doc["images"]), "four PBR sources must be embedded")
    records = json.loads((ROOT / "source-assets/environment-kit/asset-manifest.json").read_text())["files"]
    textures = {}
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
            require(hashlib.sha256(embedded).hexdigest() == digest, "GLB changed reviewed PBR bytes")
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
    require(0 < len(triangles) < 8000, f"two-face asset triangle budget exceeded: {len(triangles)}")
    bounds = [[min(p[i] for p in positions), max(p[i] for p in positions)] for i in range(3)]
    require(all(abs(v-e) < 1e-4 for row, expected in zip(bounds, ((-6.75, 6), (-.55, 9), (-8, 8.25))) for v, e in zip(row, expected)), f"unexpected bounds {bounds}")
    district = json.loads((ROOT / "source-assets/district-map/district.json").read_text())
    building = next(b for b in district["buildings"] if b["id"] == "V-55")
    require(building["polygon"] == [[148,198],[160,198],[160,214],[148,214]] and building["elevation"] == 24 and building["height"] == 9, "source shell changed")
    require([f["z"] for f in building["design"]["floors"]] == [24,27,30], "workshop floor levels changed")
    require([district["nodes"][e["node"]] for e in building["design"]["entries"]] == [[148,202,24],[148,210,24]], "source entries changed")
    approaches = []
    for start_name, door_name in (("fw_f_junction20", "fw_f_v_55_public"), ("fw_f_junction21", "fw_f_v_55_service")):
        start, end = (district["nodes"][name] for name in (start_name, door_name))
        distance = math.hypot(end[0]-start[0], end[1]-start[1]); stop = 1-.55/distance
        for sample in range(41):
            t = stop * sample / 40
            x, north, elevation = (a+(b-a)*t for a, b in zip(start, end))
            center = [x-154, elevation-24+.025, 206-north]
            lo = [center[0]-.32, center[1]+.02, center[2]-.32]
            hi = [center[0]+.32, center[1]+1.72, center[2]+.32]
            hits = [tri for tri in triangles if all(max(p[k] for p in tri) > lo[k]+1e-6 and min(p[k] for p in tri) < hi[k]-1e-6 for k in range(3))]
            require(not hits, f"{door_name} slanted approach AABB overlaps {len(hits)} triangles at {t:.3f}")
        approaches.append({"start": start_name, "door": door_name, "samples": 41, "stop_before_closed_door_m": .55, "half_width_m": .32, "height_m": 1.7})
    report = {"status": "PASS", "file": str(candidate.relative_to(ROOT)), "sha256": sha(candidate), "source_sha256": sha(HERE/"facade.blend"), "bytes": candidate.stat().st_size,
              "triangles": len(triangles), "vertices": count_vertices, "materials": len(roles), "images": 4, "runtime_bounds_xyz": bounds,
              "anchor_map_xyz": [154,206,24], "textured_uv_metric_ratio": [min(ratios),max(ratios)], "texture_sources": textures,
              "approaches": approaches, "approach_scope": "Conservative triangle-AABB broad check with source linear road elevations; actual Rust Ground/capsule and runtime NOT RUN"}
    if "--source" in sys.argv:
        import bpy
        bpy.ops.wm.open_mainfile(filepath=str(HERE/"facade.blend"))
        collection = bpy.data.collections["V55_EXPORT_Workshop"]
        objects = list(collection.objects)
        require(len(objects) == len(bpy.context.scene.objects), "review-only context leaked into source")
        require(bpy.context.scene["owner_id"] == "V-55" and list(bpy.context.scene["pivot_map_xyz"]) == [154,206,24], "source placement metadata changed")
        require(json.loads(bpy.context.scene["source_building"]) == building, "master uses another building source")
        require(json.loads(bpy.context.scene["door_nodes"]) == {e["node"]: district["nodes"][e["node"]] for e in building["design"]["entries"]}, "master doors differ from source")
        labels = {obj.name: obj.data.body for obj in objects if obj.type == "FONT"}
        require(labels == {"V55_PurposeTitle":"包装 / 修补","V55_ServiceLabel":"收件","V55_SampleLabel":"样品"}, "editable purpose labels changed")
        require(all(obj.type in {"MESH","FONT"} and not obj.modifiers and not obj.constraints and not obj.animation_data for obj in objects), "unexpected dynamic source object")
        images = [i for i in bpy.data.images if i.type == "IMAGE" and i.size[0]>0]
        require(len(images) == 4 and all(i.packed_file for i in images), "four texture masters must remain packed")
        require(all(i.colorspace_settings.name == ("Non-Color" if "NormalGL" in i.name else "sRGB") for i in images), "source texture color spaces changed")
        require(all(obj.data.font.packed_file for obj in objects if obj.type == "FONT"), "editable label font must remain packed")
        report["blender_source"] = {"status":"PASS", "version":bpy.app.version_string, "objects":len(objects), "mesh_groups":len(objects)-len(labels), "editable_labels":labels, "windows":bpy.context.scene["window_count"], "packed_images":4}
    print(json.dumps(report, indent=2, ensure_ascii=False))


if __name__ == "__main__": main()
