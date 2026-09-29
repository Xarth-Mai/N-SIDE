"""Check the actual exported street-tree geometry and material contract."""

import hashlib
import json
import math
import struct
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, require, values


def check(path):
    data, binary = read_glb(path)
    require(not data.get("animations") and not data.get("skins"), "street tree must be static")
    require(len(data.get("images", [])) == 1, "expected one embedded bark color image")
    bark_image = data["images"][0]
    require(bark_image.get("mimeType") == "image/png" and "uri" not in bark_image, "bark image must be embedded PNG")
    view = data["bufferViews"][bark_image["bufferView"]]
    offset = view.get("byteOffset", 0)
    png = binary[offset:offset + view["byteLength"]]
    require(png[:8] == b"\x89PNG\r\n\x1a\n" and struct.unpack_from(">II", png, 16) == (256, 512), "expected 256x512 bark texture")
    roots = data["scenes"][data.get("scene", 0)]["nodes"]
    require(len(roots) == 1, "expected one direct-mesh scene root")
    node = data["nodes"][roots[0]]
    require(set(node) <= {"name", "mesh"}, "apply source transforms; direct root must be identity")
    materials = data["materials"]
    require(len(materials) == 2, "expected bark and shared vertex-colored foliage materials")
    for material in materials:
        pbr = material["pbrMetallicRoughness"]
        require(material.get("alphaMode", "OPAQUE") == "OPAQUE", "tree must use opaque materials")
        require(pbr.get("baseColorFactor", [1, 1, 1, 1])[3] == 1, "unexpected material transparency")
        require(pbr.get("metallicFactor") == 0, "tree cannot be metallic")
        if material["name"] == "Bark":
            texture = data["textures"][pbr["baseColorTexture"]["index"]]
            require(texture["source"] == 0 and pbr["baseColorTexture"].get("texCoord", 0) == 0, "bark must use the embedded texture and UV0")
        else:
            require("baseColorTexture" not in pbr, "foliage must remain vertex-colored opaque geometry")
    points, triangles, primitives = [], 0, data["meshes"][node["mesh"]]["primitives"]
    require(len(primitives) == 2, "expected two draw primitives per tree")
    for primitive in primitives:
        require(primitive.get("mode", 4) == 4, "expected triangle topology")
        attrs = {key: values(data, binary, data["accessors"][value]) for key, value in primitive["attributes"].items()}
        require({"POSITION", "NORMAL", "TEXCOORD_0", "COLOR_0"} <= set(attrs), "missing position/normal/UV/color attributes")
        p = attrs["POSITION"]
        require(all(len(rows) == len(p) for rows in attrs.values()), "attribute counts differ")
        require(all(math.isfinite(x) for rows in attrs.values() for row in rows for x in row), "nonfinite vertex data")
        require(all(abs(sum(x*x for x in normal)-1) < 0.002 for normal in attrs["NORMAL"]), "normals must be normalized")
        require(all(0 <= x <= 1 for row in attrs["COLOR_0"] for x in row), "vertex colors out of range")
        require(all(len(row) == 3 or row[3] == 1 for row in attrs["COLOR_0"]), "unexpected vertex alpha")
        indices = [row[0] for row in values(data, binary, data["accessors"][primitive["indices"]])]
        require(len(indices) % 3 == 0 and all(0 <= i < len(p) for i in indices), "invalid triangle indices")
        for a, b, c in zip(indices[::3], indices[1::3], indices[2::3]):
            u, v = [p[b][i]-p[a][i] for i in range(3)], [p[c][i]-p[a][i] for i in range(3)]
            cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            require(sum(x*x for x in cross) > 1e-16, "degenerate triangle")
            if materials[primitive["material"]]["name"] == "Bark":
                uv = attrs["TEXCOORD_0"]
                area = (uv[b][0]-uv[a][0])*(uv[c][1]-uv[a][1]) - (uv[c][0]-uv[a][0])*(uv[b][1]-uv[a][1])
                require(abs(area) > 1e-9, "bark has degenerate UV triangle")
        if materials[primitive["material"]]["name"] == "EarlyAutumnLeaves":
            require(materials[primitive["material"]].get("doubleSided"), "leaf geometry needs double-sided material")
            require(min(point[1] for point in p) > 2.2, "foliage violates street clearance")
        points.extend(p)
        triangles += len(indices) // 3
    low = [min(p[i] for p in points) for i in range(3)]
    high = [max(p[i] for p in points) for i in range(3)]
    radius = max(math.hypot(p[0], p[2]) for p in points)
    require(abs(low[1]) < 1e-5 and abs(high[1]-6) < 1e-5, "expected Y-up 6m height and grounded pivot")
    require(radius <= 3.5, "crown exceeds existing placement radius")
    require(1000 < triangles <= 12000, "street-tree triangle budget exceeded")
    return {"status": "PASS", "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size,
            "triangles": triangles, "primitives": len(primitives), "bounds": [low, high], "radius_m": radius,
            "bark_image": {"dimensions": [256, 512], "embedded_bytes": len(png), "color_space": "sRGB"},
            "alpha": "OPAQUE; geometric gaps; no cutout textures", "animations": "none"}


if __name__ == "__main__":
    try:
        print(json.dumps(check(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("street-tree.glb")), indent=2))
    except (ValueError, KeyError, IndexError, OSError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)
