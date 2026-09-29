"""Check the actual opaque pine GLB, including the existing placement envelope."""

import hashlib
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, require, values


def check(path):
    data, binary = read_glb(path)
    require(not data.get("animations") and not data.get("skins"), "pine must be a static mesh")
    roots = data["scenes"][data.get("scene", 0)]["nodes"]
    require(len(roots) == 1, "expected one pine root")
    node = data["nodes"][roots[0]]
    require(set(node) <= {"name", "mesh"}, "pine root must be identity, Y-up and direct-mesh")
    materials = data["materials"]
    require({m["name"] for m in materials} == {"PineBark", "PineNeedles"}, "expected two pine materials")
    require(len(data.get("images", [])) == 1, "expected one embedded shared bark image")
    image = data["images"][0]
    require(image.get("mimeType") == "image/png" and "uri" not in image, "bark must be embedded PNG")
    view = data["bufferViews"][image["bufferView"]]
    offset = view.get("byteOffset", 0)
    png = binary[offset:offset + view["byteLength"]]
    require(png == Path(__file__).with_name("bark-color.png").read_bytes(), "pine must reuse unchanged street-tree bark")
    primitives = data["meshes"][node["mesh"]]["primitives"]
    require(len(primitives) == 2, "expected two draw primitives")
    points, triangles, vertices = [], 0, 0
    for primitive in primitives:
        require(primitive.get("mode", 4) == 4, "expected triangle topology")
        attrs = {key: values(data, binary, data["accessors"][value]) for key, value in primitive["attributes"].items()}
        require({"POSITION", "NORMAL", "TEXCOORD_0", "COLOR_0"} <= set(attrs), "missing pine vertex attribute")
        p = attrs["POSITION"]
        require(all(len(rows) == len(p) for rows in attrs.values()), "attribute counts differ")
        require(all(math.isfinite(x) for rows in attrs.values() for row in rows for x in row), "nonfinite vertex data")
        require(all(abs(sum(x*x for x in n)-1) < 0.002 for n in attrs["NORMAL"]), "pine normals must be normalized")
        require(all(0 <= x <= 1 for row in attrs["COLOR_0"] for x in row), "pine vertex colors out of range")
        require(all(len(row) == 3 or row[3] == 1 for row in attrs["COLOR_0"]), "unexpected vertex transparency")
        material = materials[primitive["material"]]
        pbr = material["pbrMetallicRoughness"]
        require(material.get("alphaMode", "OPAQUE") == "OPAQUE", "pine must use opaque materials")
        require(pbr.get("metallicFactor") == 0, "pine cannot be metallic")
        if material["name"] == "PineNeedles":
            require(material.get("doubleSided") and "baseColorTexture" not in pbr, "needles must be double-sided vertex-colored geometry")
            require(all(row[1] > row[0] and row[1] > row[2] for row in attrs["COLOR_0"]), "needle palette must remain evergreen")
            require(min(point[1] for point in p) > 2.8, "needle canopy violates this candidate's lower clearance")
        else:
            texture = data["textures"][pbr["baseColorTexture"]["index"]]
            require(texture["source"] == 0 and pbr["baseColorTexture"].get("texCoord", 0) == 0, "bark must use the shared image and UV0")
        indices = [row[0] for row in values(data, binary, data["accessors"][primitive["indices"]])]
        require(len(indices) % 3 == 0 and all(0 <= i < len(p) for i in indices), "invalid pine indices")
        for a, b, c in zip(indices[::3], indices[1::3], indices[2::3]):
            u, v = [p[b][i]-p[a][i] for i in range(3)], [p[c][i]-p[a][i] for i in range(3)]
            cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            require(sum(x*x for x in cross) > 1e-16, "degenerate pine triangle")
            if material["name"] == "PineBark":
                uv = attrs["TEXCOORD_0"]
                area = (uv[b][0]-uv[a][0])*(uv[c][1]-uv[a][1]) - (uv[c][0]-uv[a][0])*(uv[b][1]-uv[a][1])
                require(abs(area) > 1e-9, "pine bark UV has a degenerate triangle")
        points.extend(p)
        vertices += len(p)
        triangles += len(indices) // 3
    low = [min(p[i] for p in points) for i in range(3)]
    high = [max(p[i] for p in points) for i in range(3)]
    radius = max(math.hypot(p[0], p[2]) for p in points)
    require(abs(low[1]) < 1e-5 and abs(high[1]-6.5) < 1e-5, "expected grounded Y-up pine of height 6.5m")
    require(radius <= 1.69001, "pine exceeds the old model's 1.694578m radius envelope")
    require(1000 < triangles <= 7000, "pine exceeds this candidate's triangle ceiling")
    return {"status": "PASS", "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size,
            "triangles": triangles, "vertices": vertices, "primitives": len(primitives), "bounds": [low, high],
            "radius_m": radius, "bark_sha256": hashlib.sha256(png).hexdigest(),
            "alpha": "OPAQUE; double-sided geometric needles", "animations": "none"}


if __name__ == "__main__":
    try:
        print(json.dumps(check(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("pine-street.glb")), indent=2))
    except (ValueError, KeyError, IndexError, OSError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)
