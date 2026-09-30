"""Check the editable V-A08 master and its actual GLB accessors in Blender."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import sys

import bmesh
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, require, values


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    master = HERE / "roof-eaves.blend"
    runtime = ROOT / "game/assets/environment/buildings/v-a08-roof-eaves.glb"
    bpy.ops.wm.open_mainfile(filepath=str(master))
    objects = list(bpy.context.scene.objects)
    require(len(objects) == 1 and objects[0].name == "V_A08_RoofEaves", "master must contain only its authored roof mesh")
    obj = objects[0]
    require(obj.type == "MESH" and not obj.modifiers, "expected editable static mesh without unapplied modifiers")
    require(tuple(obj.location) == (0, 0, 0) and tuple(obj.rotation_euler) == (0, 0, 0) and tuple(obj.scale) == (1, 1, 1), "master object must have identity transform")
    require(bpy.context.scene.unit_settings.scale_length == 1, "master must use meters")
    mesh = obj.data
    require(len(mesh.vertices) == 28 and len(mesh.polygons) == 28, "56-triangle source budget changed")
    require(all(math.isfinite(v) for vertex in mesh.vertices for v in vertex.co), "non-finite source position")
    editable = bmesh.new()
    editable.from_mesh(mesh)
    require(all(edge.is_manifold for edge in editable.edges), "source eave has an open or non-manifold edge")
    require(all(face.calc_area() > 1e-6 for face in editable.faces), "degenerate source face")
    volume = editable.calc_volume(signed=True)
    require(volume > 0, "source winding faces inward")
    editable.free()
    require(all(not face.use_smooth for face in mesh.polygons), "folded metal edges must retain flat normals")
    require([material.name for material in mesh.materials] == ["roof", "metal", "trim"], "material roles changed")
    require(mesh.uv_layers.active is not None, "missing source UVs")
    document, binary = read_glb(runtime)
    require(not document.get("skins") and not document.get("animations"), "static roof unexpectedly exports animation")
    require(not document.get("cameras") and not document.get("images"), "preview context or unexpected images exported")
    require(not document.get("extensionsUsed"), "roof should need no glTF extensions")
    require(document.get("scene") == 0 and len(document["scenes"]) == 1, "roof must load through Scene0")
    require(len(document["nodes"]) == 1 and document["scenes"][0]["nodes"] == [0], "unexpected root hierarchy")
    node = document["nodes"][0]
    require(not node.get("children") and "matrix" not in node, "unexpected transformed hierarchy")
    require(node.get("translation", [0, 0, 0]) == [0, 0, 0] and node.get("rotation", [0, 0, 0, 1]) == [0, 0, 0, 1] and node.get("scale", [1, 1, 1]) == [1, 1, 1], "exported transform must be identity")
    require(len(document["meshes"]) == 1, "expected a single eaves mesh")
    primitives = document["meshes"][0]["primitives"]
    require(len(primitives) == 3, "expected three material primitives")
    positions_all = []
    triangles = 0
    uv_area_ratio = []
    uv_edge_ratio = []
    for primitive in primitives:
        require(primitive.get("mode", 4) == 4, "expected triangles")
        attributes = primitive["attributes"]
        require({"POSITION", "NORMAL", "TEXCOORD_0"} <= attributes.keys(), "missing position/normal/UV")
        read = lambda index: values(document, binary, document["accessors"][index])
        positions = read(attributes["POSITION"])
        normals = read(attributes["NORMAL"])
        uvs = read(attributes["TEXCOORD_0"])
        indices = [row[0] for row in read(primitive["indices"])]
        require(len(indices) % 3 == 0, "triangle indices not divisible by three")
        require(all(math.isfinite(v) for rows in (positions, normals, uvs) for row in rows for v in row), "non-finite exported attributes")
        require(all(abs(Vector(normal).length - 1) < 1e-5 for normal in normals), "exported normals are not normalized")
        require(all(0 <= index < len(positions) for index in indices), "index out of bounds")
        for start in range(0, len(indices), 3):
            ia, ib, ic = indices[start:start + 3]
            cross = (Vector(positions[ib]) - Vector(positions[ia])).cross(Vector(positions[ic]) - Vector(positions[ia]))
            require(cross.length > 1e-6, "exported degenerate triangle")
            require(all(cross.normalized().dot(Vector(normals[index])) > .999 for index in (ia, ib, ic)), "winding and corner normals disagree")
            a, b, c = (Vector(uvs[index]) for index in (ia, ib, ic))
            uv_area = abs((b.x-a.x)*(c.y-a.y)-(b.y-a.y)*(c.x-a.x))
            ratio = uv_area / cross.length
            require(abs(ratio-1) < .0001, "UV coordinates do not preserve one unit per meter")
            uv_area_ratio.append(ratio)
            for first, second in ((ia, ib), (ib, ic), (ic, ia)):
                metric_length = (Vector(positions[second]) - Vector(positions[first])).length
                uv_length = (Vector(uvs[second]) - Vector(uvs[first])).length
                edge_ratio = uv_length / metric_length
                require(abs(edge_ratio - 1) < .0001, "UV edge stretches or compresses its metric length")
                uv_edge_ratio.append(edge_ratio)
        triangles += len(indices) // 3
        positions_all.extend(positions)
    require(triangles == 56, "roof triangle budget changed")
    bounds = [[min(row[axis] for row in positions_all), max(row[axis] for row in positions_all)] for axis in range(3)]
    expected = [[-9.45, 9.45], [-.35, 0], [-6.95, 6.95]]
    require(all(abs(value-goal) < 1e-5 for actual, target in zip(bounds, expected) for value, goal in zip(actual, target)), "runtime Y-up bounds differ from 18.9×13.9m / 0.35m-deep eave")
    for material in document["materials"]:
        source = mesh.materials[material["name"]].node_tree.nodes["Principled BSDF"]
        pbr = material["pbrMetallicRoughness"]
        require(all(abs(a-b) < 1e-6 for a,b in zip(pbr["baseColorFactor"], source.inputs["Base Color"].default_value)), "GLB color differs from linear source material")
        require(abs(pbr.get("roughnessFactor", 1)-source.inputs["Roughness"].default_value) < 1e-6, "roughness differs from source")
        require(abs(pbr.get("metallicFactor", 1)-source.inputs["Metallic"].default_value) < 1e-6, "metallic differs from source")
    result = {"status": "PASS", "blender_version": bpy.app.version_string, "source_sha256": digest(master), "runtime_sha256": digest(runtime), "source_vertices": len(mesh.vertices), "source_quads": len(mesh.polygons), "triangles": triangles, "material_primitives": len(primitives), "source_manifold": True, "source_volume_m3": volume, "runtime_bounds_xyz": bounds, "uv_area_ratio_range": [min(uv_area_ratio), max(uv_area_ratio)], "uv_edge_ratio_range": [min(uv_edge_ratio), max(uv_edge_ratio)], "material_names": [m["name"] for m in document["materials"]], "appearance_sha256_at_build": obj["appearance_sha256_at_build"], "collision": "No GLB collider; replacement of generated coping is an integration change", "runtime_validation": "NOT RUN by asset producer"}
    options.output.parent.mkdir(parents=True, exist_ok=True)
    options.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
