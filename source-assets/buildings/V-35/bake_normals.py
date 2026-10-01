"""Bake V-35's glTF normal scale into the two existing CC0 normal maps."""
import argparse
import hashlib
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
STEMS = ("Plaster001", "Concrete034")


# Same glTF xy scaling as the R4 normal-scale-r1 diagnostic
def scaled(pixel, strength):
    x, y, z = (value / 127.5 - 1.0 for value in pixel)
    x *= strength
    y *= strength
    length = math.sqrt(x * x + y * y + z * z)
    return tuple(round((value / length + 1.0) * 127.5) for value in (x, y, z))


def baked_normal(source):
    from PIL import Image
    with Image.open(source) as image:
        assert image.mode == "RGB", "expected RGB tangent-space source"
        result = Image.new("RGB", image.size)
        result.putdata([scaled(pixel, .25) for pixel in image.get_flattened_data()])
        return result


def main():
    from PIL import Image, __version__ as pillow_version
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    assert scaled((128, 128, 255), .25) == (128, 128, 255)
    assert scaled((255, 128, 255), .25) == (158, 128, 251)
    assert scaled((0, 128, 255), .25) == (97, 128, 251)
    assert scaled((255, 128, 255), 0) == (128, 128, 255)
    assert scaled((255, 128, 255), 1) == (218, 128, 218)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reports = []
    for stem in STEMS:
        source = ROOT / f"source-assets/environment-kit/materials/{stem}_1K-JPG_NormalGL.jpg"
        output = HERE / f"textures/{stem}-NormalGL-scale025.png"
        source_hash = digest(source)
        result = baked_normal(source)
        if args.check:
            with Image.open(output) as actual:
                assert actual.mode == result.mode and actual.size == result.size
                assert actual.tobytes() == result.tobytes(), f"normal bake differs: {output}"
        else:
            output.parent.mkdir(exist_ok=True)
            result.save(output)
        assert digest(source) == source_hash, "original source changed"
        reports.append({"source": str(source.relative_to(ROOT)), "source_sha256": source_hash,
                        "output": str(output.relative_to(ROOT)), "output_sha256": digest(output),
                        "size": result.size, "mode": result.mode, "license": "CC0-1.0"})
    print(json.dumps({"status": "PASS", "scale": .25, "gltf_scale": 1,
                      "formula": "n = rgb / 127.5 - 1; n.xy *= .25; n = normalize(n); rgb = round((n + 1) * 127.5)",
                      "color_space": "Raw normal data; no sRGB transform", "pillow": pillow_version,
                      "files": reports}, indent=2))


if __name__ == "__main__":
    main()
