"""Bake the seven shared building normal variants beside their original CC0 JPGs."""
import argparse
import hashlib
import json
import math
from pathlib import Path

MATERIALS = Path(__file__).resolve().parent
VARIANTS = (("Plaster001", .25), ("Concrete034", .25),
            ("Plaster001", .28), ("Concrete034", .28),
            ("Plaster001", .35), ("Concrete034", .35), ("WoodSiding009", .35))


def scaled(pixel, strength):
    x, y, z = (value / 127.5 - 1.0 for value in pixel)
    x *= strength
    y *= strength
    length = math.sqrt(x * x + y * y + z * z)
    return tuple(round((value / length + 1.0) * 127.5) for value in (x, y, z))


def baked_normal(source, strength):
    from PIL import Image
    with Image.open(source) as image:
        assert image.mode == "RGB", "expected RGB tangent-space source"
        result = Image.new("RGB", image.size)
        result.putdata([scaled(pixel, strength) for pixel in image.get_flattened_data()])
        return result


def main():
    from PIL import Image, __version__ as pillow_version
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    assert scaled((128, 128, 255), .25) == (128, 128, 255)
    assert scaled((255, 128, 255), .25) == (158, 128, 251)
    assert scaled((0, 128, 255), .25) == (97, 128, 251)
    assert scaled((255, 128, 255), .28) == (162, 128, 250)
    assert scaled((255, 128, 255), .35) == (170, 128, 248)
    assert scaled((255, 128, 255), 0) == (128, 128, 255)
    assert scaled((255, 128, 255), 1) == (218, 128, 218)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    reports = []
    for stem, strength in VARIANTS:
        source = MATERIALS / f"{stem}_1K-JPG_NormalGL.jpg"
        output = MATERIALS / f"{stem}-NormalGL-scale{round(strength * 100):03d}.png"
        source_hash = digest(source)
        result = baked_normal(source, strength)
        if args.check:
            with Image.open(output) as actual:
                assert actual.mode == result.mode and actual.size == result.size
                assert actual.tobytes() == result.tobytes(), f"normal bake differs: {output}"
        else:
            result.save(output)
        assert digest(source) == source_hash, "original source changed"
        reports.append({"source": source.name, "source_sha256": source_hash,
                        "output": output.name, "output_sha256": digest(output), "scale": strength,
                        "size": result.size, "mode": result.mode, "license": "CC0-1.0"})
    print(json.dumps({"status": "PASS", "gltf_scale": 1, "pillow": pillow_version,
                      "formula": "n = rgb / 127.5 - 1; n.xy *= scale; n = normalize(n); rgb = round((n + 1) * 127.5)",
                      "color_space": "Raw normal data; no sRGB transform", "files": reports}, indent=2))


if __name__ == "__main__":
    main()
