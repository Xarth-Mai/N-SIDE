"""Check final exported ground props using the project's GLB reader."""
from pathlib import Path
import hashlib
import json
import math
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "tools"))
from glb import read_glb, values

rows = []
for name, height, radius, triangles in (("grass", .35, .30, 1320), ("rock", .65, .75, 320)):
    path = ROOT / f"source-assets/environment-kit/vegetation/{name}-forest.glb"
    doc, blob = read_glb(path)
    assert not any(doc.get(k) for k in ("skins", "animations", "extensionsUsed"))
    assert len(doc["meshes"]) == len(doc["materials"]) == len(doc["nodes"]) == 1
    node = doc["nodes"][0]
    assert not any(k in node for k in ("matrix", "rotation", "translation", "scale", "children"))
    primitive = doc["meshes"][0]["primitives"][0]
    attributes = {key: list(values(doc, blob, doc["accessors"][index])) for key, index in primitive["attributes"].items()}
    assert all(math.isfinite(v) for attribute in attributes.values() for point in attribute for v in point)
    assert {"POSITION", "NORMAL", "TEXCOORD_0"} <= attributes.keys()
    points = attributes["POSITION"]
    actual_height = max(p[1] for p in points)
    actual_radius = max(math.hypot(p[0], p[2]) for p in points)
    assert abs(min(p[1] for p in points)) < 1e-6
    assert abs(actual_height-height) < 1e-6 and actual_radius <= radius
    indices = [i[0] for i in values(doc, blob, doc["accessors"][primitive["indices"]])]
    assert len(indices) == triangles * 3
    minimum_area = float("inf")
    for at in range(0, len(indices), 3):
        a, b, c = [points[i] for i in indices[at:at+3]]
        u, v = [b[i]-a[i] for i in range(3)], [c[i]-a[i] for i in range(3)]
        cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
        minimum_area = min(minimum_area, math.sqrt(sum(x*x for x in cross)) / 2)
    assert minimum_area > 1e-8
    material = doc["materials"][0]
    assert material.get("alphaMode", "OPAQUE") == "OPAQUE"
    assert material["pbrMetallicRoughness"]["metallicFactor"] == 0
    assert all("bufferView" in image for image in doc["images"])
    assert "baseColorTexture" in material["pbrMetallicRoughness"]
    assert name != "rock" or "normalTexture" in material
    runtime = ROOT / f"game/assets/environment/vegetation/{name}-forest.glb"
    assert path.read_bytes() == runtime.read_bytes()
    rows.append({"file": str(path.relative_to(ROOT)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                 "triangles": triangles, "height": actual_height, "maximum_radius": actual_radius,
                 "minimum_triangle_area": minimum_area, "embedded_images": len(doc["images"]), "status": "PASS"})
print(json.dumps(rows, indent=2))
