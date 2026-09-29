#!/usr/bin/env python3
"""Check N:SIDE's dense, embedded, in-place character GLB before Bevy loading.

This checks exported data, not deformation quality, foot contact or playback
Sparse/compressed accessors and morph animation need a separate validated path
"""

import argparse
import hashlib
from io import BytesIO
import json
import math
from pathlib import Path
import struct
import sys

from PIL import Image

from glb import read_glb, require, values


def transformed(point, node):
    if "matrix" in node:
        m = node["matrix"]
        return tuple(sum(m[col * 4 + row] * value for col, value in enumerate((*point, 1))) for row in range(3))
    p = [point[i] * node.get("scale", [1, 1, 1])[i] for i in range(3)]
    x, y, z, w = node.get("rotation", [0, 0, 0, 1])
    cross = (y * p[2] - z * p[1], z * p[0] - x * p[2], x * p[1] - y * p[0])
    second = (y * cross[2] - z * cross[1], z * cross[0] - x * cross[2], x * cross[1] - y * cross[0])
    return tuple(p[i] + 2 * (w * cross[i] + second[i]) + node.get("translation", [0, 0, 0])[i] for i in range(3))


def inspect(path, *, root_name, height, height_tolerance=0.05, max_root_travel=0.001, clips=(), require_texture=False):
    document, binary = read_glb(path)
    nodes = document.get("nodes", [])
    skins = document.get("skins", [])
    require(skins, "character has no skins; static props are not animated character assets")
    require(document.get("animations"), "character has no animation clips")
    require(height > 0 and height_tolerance >= 0 and max_root_travel >= 0, "invalid scale/motion limits")
    require(all(math.isfinite(n) for n in (height, height_tolerance, max_root_travel)), "nonfinite scale/motion limits")
    accessors = document.get("accessors", [])
    rows = [values(document, binary, accessor) for accessor in accessors]
    require(all(math.isfinite(value) for data in rows for row in data for value in row), "nonfinite accessor data")
    parents = {}
    for i, node in enumerate(nodes):
        for field, count in (("translation", 3), ("rotation", 4), ("scale", 3), ("matrix", 16)):
            if field in node:
                require(len(node[field]) == count and all(math.isfinite(n) for n in node[field]), f"node {i}: invalid {field}")
        require("matrix" not in node or not any(key in node for key in ("translation", "rotation", "scale")), f"node {i}: mixed matrix and TRS")
        require(all(n > 0 for n in node.get("scale", [1, 1, 1])), f"node {i}: mirrored/zero scale needs DCC correction")
        require(abs(sum(n * n for n in node.get("rotation", [0, 0, 0, 1])) - 1) < 1e-4, f"node {i}: non-unit rotation")
        for child in node.get("children", []):
            require(0 <= child < len(nodes) and child not in parents, f"node {i}: invalid or multiply parented child {child}")
            parents[child] = i
    reachable = set()

    def visit(index, ancestors):
        require(0 <= index < len(nodes) and index not in ancestors, "invalid/cyclic scene node hierarchy")
        reachable.add(index)
        for child in nodes[index].get("children", []):
            visit(child, ancestors | {index})

    require(0 <= document.get("scene", 0) < len(document["scenes"]), "invalid default scene")
    scene = document["scenes"][document.get("scene", 0)]
    for index in scene.get("nodes", []):
        require(index not in parents, "scene root has a parent")
        visit(index, set())
    roots = [i for i in reachable if nodes[i].get("name") == root_name]
    require(len(roots) == 1, f"expected exactly one scene root-motion node named {root_name!r}")
    root = roots[0]
    motion_nodes = {root}
    ancestor = root
    while ancestor in parents:
        ancestor = parents[ancestor]
        motion_nodes.add(ancestor)

    def world_point(point, index):
        while True:
            point = transformed(point, nodes[index])
            if index not in parents:
                return point
            index = parents[index]

    joint_count = 0
    all_joints = set()
    for i, skin in enumerate(skins):
        joints = skin.get("joints", [])
        require(joints and len(joints) == len(set(joints)) and set(joints) <= reachable, f"skin {i}: invalid/duplicate/unreachable joints")
        names = [nodes[j].get("name") for j in joints]
        require(all(names) and len(names) == len(set(names)), f"skin {i}: joints need unique nonempty names")
        for joint in joints:
            ancestor = joint
            while ancestor != root and ancestor in parents:
                ancestor = parents[ancestor]
            require(ancestor == root, f"root-motion node {root_name!r} is not an ancestor of skin {i} joint {joint}")
        all_joints.update(joints)
        if "inverseBindMatrices" in skin:
            require(0 <= skin["inverseBindMatrices"] < len(accessors), f"skin {i}: invalid inverse bind accessor")
            accessor = accessors[skin["inverseBindMatrices"]]
            require(accessor["type"] == "MAT4" and accessor["componentType"] == 5126 and accessor["count"] == len(joints), f"skin {i}: inverse bind matrix count/type mismatch")
        joint_count += len(joints)
    # Include actual top-level joints if the requested root is their armature object
    for joint in all_joints:
        chain, ancestor = {joint}, joint
        while ancestor in parents:
            ancestor = parents[ancestor]
            chain.add(ancestor)
            if ancestor in all_joints:
                break
        else:
            motion_nodes.update(chain)
    require(all(nodes[index].get("scale", [1, 1, 1]) == [1, 1, 1] and "matrix" not in nodes[index] for index in motion_nodes), "root-motion hierarchy requires unit TRS scale; inspect DCC bind transforms before re-export")
    points, skinned_vertices, used_skins, used_textures = [], 0, set(), set()
    for i in reachable:
        node = nodes[i]
        if "mesh" not in node:
            continue
        require(0 <= node["mesh"] < len(document["meshes"]), f"node {i}: invalid mesh index")
        require("skin" not in node or 0 <= node["skin"] < len(skins), f"node {i}: invalid skin index")
        for primitive in document["meshes"][node["mesh"]]["primitives"]:
            require(primitive.get("mode", 4) == 4, f"node {i}: character mesh must use triangles")
            attributes = primitive["attributes"]
            require(all(0 <= index < len(accessors) for index in attributes.values()), f"node {i}: invalid attribute accessor")
            require("NORMAL" in attributes and "material" in primitive, f"node {i}: missing normal/material slot")
            require(0 <= primitive["material"] < len(document["materials"]), f"node {i}: invalid material slot")
            material = document["materials"][primitive["material"]]
            base_texture = material.get("pbrMetallicRoughness", {}).get("baseColorTexture")
            if base_texture is not None:
                require(0 <= base_texture["index"] < len(document.get("textures", [])), f"node {i}: invalid base color texture")
                require(f"TEXCOORD_{base_texture.get('texCoord', 0)}" in attributes, f"node {i}: texture has no UV accessor")
                used_textures.add(base_texture["index"])
            position_accessor = accessors[attributes["POSITION"]]
            require(position_accessor["type"] == "VEC3" and position_accessor["componentType"] == 5126, f"node {i}: position type is not float VEC3")
            positions = rows[attributes["POSITION"]]
            if "indices" in primitive:
                require(0 <= primitive["indices"] < len(accessors), f"node {i}: invalid triangle index accessor")
                index_accessor = accessors[primitive["indices"]]
                require(index_accessor["type"] == "SCALAR" and index_accessor["componentType"] in (5121, 5123, 5125) and not index_accessor.get("normalized"), f"node {i}: triangle indices must be unnormalized unsigned SCALAR")
                indices = rows[primitive["indices"]]
                require(len(indices) % 3 == 0, f"node {i}: triangle index count must be a multiple of 3")
                require(all(0 <= row[0] < len(positions) for row in indices), f"node {i}: triangle index out of vertex range")
            else:
                require(len(positions) % 3 == 0, f"node {i}: nonindexed vertex count must be a multiple of 3")
            require(all(len(rows[index]) == len(positions) for index in attributes.values()), f"node {i}: vertex attribute count mismatch")
            normal_accessor = accessors[attributes["NORMAL"]]
            require(normal_accessor["type"] == "VEC3" and normal_accessor["componentType"] == 5126, f"node {i}: normals must be float VEC3")
            require(all(abs(sum(value * value for value in normal) - 1) < 1e-3 for normal in rows[attributes["NORMAL"]]), f"node {i}: non-unit normal")
            for attribute, index in attributes.items():
                if attribute.startswith("TEXCOORD_"):
                    uv = accessors[index]
                    require(uv["type"] == "VEC2" and (uv["componentType"] == 5126 or (uv["componentType"] in (5121, 5123) and uv.get("normalized"))), f"node {i}: UVs must be float or normalized unsigned VEC2")
            points.extend(world_point(point, i) for point in positions)
            if "skin" not in node:
                continue
            used_skins.add(node["skin"])
            joints = skins[node["skin"]]["joints"]
            require("JOINTS_0" in attributes and "WEIGHTS_0" in attributes, f"node {i}: missing skin weights/joint indices")
            require(not any(name.startswith(("JOINTS_", "WEIGHTS_")) and name not in ("JOINTS_0", "WEIGHTS_0") for name in attributes), f"node {i}: more than four influences need a separately validated Bevy path")
            j_accessor, w_accessor = (accessors[attributes[key]] for key in ("JOINTS_0", "WEIGHTS_0"))
            require(j_accessor["type"] == w_accessor["type"] == "VEC4" and j_accessor["componentType"] in (5121, 5123) and not j_accessor.get("normalized"), f"node {i}: invalid joint/weight types")
            require(w_accessor["componentType"] == 5126 or (w_accessor["componentType"] in (5121, 5123) and w_accessor.get("normalized")), f"node {i}: weights must be float or normalized unsigned integers")
            indices, weights = rows[attributes["JOINTS_0"]], rows[attributes["WEIGHTS_0"]]
            for vertex, (indices_row, weights_row) in enumerate(zip(indices, weights)):
                require(all(0 <= j < len(joints) for j in indices_row), f"node {i} vertex {vertex}: joint index out of range")
                require(all(0 <= w <= 1 for w in weights_row) and abs(sum(weights_row) - 1) <= 1e-4, f"node {i} vertex {vertex}: weights do not sum to 1")
                inverse_binds = skins[node["skin"]].get("inverseBindMatrices")
                skinned = [0.0, 0.0, 0.0]
                for joint, weight in zip(indices_row, weights_row):
                    if weight == 0:
                        continue
                    point = positions[vertex]
                    if inverse_binds is not None:
                        point = transformed(point, {"matrix": rows[inverse_binds][joint]})
                    point = world_point(point, joints[joint])
                    for axis in range(3):
                        skinned[axis] += point[axis] * weight
                require(math.dist(skinned, world_point(positions[vertex], i)) < 1e-4, f"node {i} vertex {vertex}: inverse bind matrices do not preserve rest pose")
            skinned_vertices += len(positions)
    require(skinned_vertices > 0 and used_skins == set(range(len(skins))), "skin is not used by a mesh in the default scene")
    bounds = [[min(p[i] for p in points), max(p[i] for p in points)] for i in range(3)]
    measured_height = bounds[1][1] - bounds[1][0]
    require(abs(measured_height - height) <= height_tolerance, f"bind-pose height {measured_height:.4f} m differs from {height} +/- {height_tolerance} m")
    require(abs(bounds[1][0]) <= height_tolerance, f"foot pivot is {bounds[1][0]:.4f} m above/below zero")

    images = document.get("images", [])
    require(not require_texture or used_textures, "textured character requires a used base color texture and UVs")
    for i, image in enumerate(images):
        require("bufferView" in image and "uri" not in image and image.get("mimeType") in ("image/png", "image/jpeg"), f"image {i}: expected embedded PNG/JPEG")
        view = document["bufferViews"][image["bufferView"]]
        start, length = view.get("byteOffset", 0), view["byteLength"]
        require(view["buffer"] == 0 and 0 <= start < start + length <= len(binary), f"image {i}: invalid buffer view")
        signature = b"\x89PNG\r\n\x1a\n" if image["mimeType"] == "image/png" else b"\xff\xd8\xff"
        encoded = binary[start:start + length]
        require(encoded.startswith(signature), f"image {i}: invalid image signature")
        try:
            with Image.open(BytesIO(encoded)) as decoded:
                decoded.verify()
            with Image.open(BytesIO(encoded)) as decoded:
                decoded.load()
        except (OSError, ValueError, SyntaxError, Image.DecompressionBombError) as error:
            raise ValueError(f"image {i}: embedded image decoding failed: {error}") from error
    for i, texture in enumerate(document.get("textures", [])):
        require(0 <= texture.get("source", -1) < len(images), f"texture {i}: missing embedded image")

    animations = []
    for animation in document["animations"]:
        name = animation.get("name")
        require(name and name not in [a["name"] for a in animations], "animation names must be nonempty and unique")
        duration, root_travel, root_delta, root_turn, joint_channels = 0, 0, 0, 0, 0
        require(animation.get("channels"), f"{name}: no animation channels")
        targets = set()
        for channel in animation["channels"]:
            target = channel["target"]
            node, field = target["node"], target["path"]
            require(node in reachable and field in ("translation", "rotation", "scale"), f"{name}: missing node or unsupported morph animation")
            require((node, field) not in targets, f"{name}: duplicate channel target")
            targets.add((node, field))
            require(0 <= channel["sampler"] < len(animation["samplers"]), f"{name}: invalid sampler index")
            sampler = animation["samplers"][channel["sampler"]]
            require(all(0 <= sampler[key] < len(accessors) for key in ("input", "output")), f"{name}: invalid animation accessor")
            times = rows[sampler["input"]]
            require(accessors[sampler["input"]]["type"] == "SCALAR" and accessors[sampler["input"]]["componentType"] == 5126, f"{name}: time accessor must be float SCALAR")
            times = [row[0] for row in times]
            require(len(times) >= 2 and times[0] >= 0 and all(a < b for a, b in zip(times, times[1:])), f"{name}: timestamps must increase with positive duration")
            interpolation = sampler.get("interpolation", "LINEAR")
            require(interpolation in ("LINEAR", "STEP", "CUBICSPLINE"), f"{name}: invalid interpolation")
            output = accessors[sampler["output"]]
            require(output["componentType"] == 5126 and output["type"] == ("VEC4" if field == "rotation" else "VEC3"), f"{name}: output type mismatch")
            keyframes = rows[sampler["output"]]
            require(len(keyframes) == len(times) * (3 if interpolation == "CUBICSPLINE" else 1), f"{name}: keyframe count mismatch")
            if interpolation == "CUBICSPLINE":
                if node in motion_nodes:
                    require(all(value == 0 for row in keyframes[::3] + keyframes[2::3] for value in row), f"{name}: nonzero root spline tangents need a separately validated motion path")
                keyframes = keyframes[1::3]
            if field == "rotation":
                require(all(abs(sum(n * n for n in q) - 1) < 1e-4 for q in keyframes), f"{name}: non-unit rotation key")
            duration = max(duration, times[-1])
            joint_channels += int(any(node in skin["joints"] for skin in skins))
            if node in motion_nodes and field == "translation":
                rest = nodes[node].get("translation", [0, 0, 0])
                travel = max(math.dist(p, rest) for p in keyframes)
                root_travel += travel
                root_delta += math.dist(keyframes[-1], rest)
            if node in motion_nodes and field == "rotation":
                initial = nodes[node].get("rotation", [0, 0, 0, 1])
                root_turn += max(math.degrees(2 * math.acos(min(1, abs(sum(a * b for a, b in zip(q, initial))) / math.sqrt(sum(a * a for a in q) * sum(b * b for b in initial))))) for q in keyframes)
            if node in motion_nodes and field == "scale":
                require(all(all(abs(value - 1) < 1e-6 for value in key) for key in keyframes), f"{name}: animated root scale changes the meter contract")
        require(root_travel <= max_root_travel, f"{name}: root translation bound {root_travel:.6f} m exceeds in-place limit {max_root_travel} m")
        require(root_turn < 0.1, f"{name}: root rotation bound {root_turn:.4f} degrees is not in-place; controller owns world turning")
        require(joint_channels, f"{name}: no skin joint is animated")
        animations.append({"name": name, "duration_seconds": duration, "joint_channels": joint_channels, "root_translation_bound_m": root_travel, "root_end_displacement_bound_m": root_delta, "root_rotation_bound_degrees": root_turn})
    require(set(clips) <= {a["name"] for a in animations}, f"missing required clips: {sorted(set(clips) - {a['name'] for a in animations})}")
    return {"status": "PASS", "path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "skins": len(skins), "joints": joint_count, "skinned_vertices": skinned_vertices, "bind_bounds_m": bounds, "height_m": measured_height, "embedded_images": len(images), "animations": animations, "scope": "exported data only; deformation, contact, loop quality and Bevy playback require runtime review"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("file", type=Path)
    parser.add_argument("--root-node", required=True)
    parser.add_argument("--height", required=True, type=float)
    parser.add_argument("--height-tolerance", type=float, default=0.05)
    parser.add_argument("--max-root-travel", type=float, default=0.001)
    parser.add_argument("--clip", action="append", default=[])
    parser.add_argument("--require-texture", action="store_true")
    args = parser.parse_args()
    try:
        result = inspect(args.file, root_name=args.root_node, height=args.height, height_tolerance=args.height_tolerance, max_root_travel=args.max_root_travel, clips=args.clip, require_texture=args.require_texture)
    except (ValueError, KeyError, IndexError, TypeError, OSError, AttributeError, struct.error) as error:
        print(json.dumps({"status": "FAIL", "path": str(args.file), "error": str(error)}, ensure_ascii=False))
        return 1
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
