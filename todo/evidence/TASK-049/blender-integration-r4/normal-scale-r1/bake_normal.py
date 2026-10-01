#!/usr/bin/env python3
"""Bake glTF tangent-space normalTexture.scale into an RGB normal map"""

import argparse
import hashlib
import json
import math
from pathlib import Path

from PIL import Image, __version__ as pillow_version


def scaled(pixel, strength):
    x, y, z = (value / 127.5 - 1.0 for value in pixel)
    x *= strength
    y *= strength
    length = math.sqrt(x * x + y * y + z * z)
    return tuple(round((value / length + 1.0) * 127.5) for value in (x, y, z))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def self_check():
    assert scaled((128, 128, 255), .25) == (128, 128, 255)
    assert scaled((255, 128, 255), .25) == (158, 128, 251)
    assert scaled((0, 128, 255), .25) == (97, 128, 251)
    assert scaled((255, 128, 255), 0) == (128, 128, 255)
    assert scaled((255, 128, 255), 1) == (218, 128, 218)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, nargs="?")
    parser.add_argument("output", type=Path, nargs="?")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    self_check()
    if args.self_test:
        print("PASS: flat, tilted, mirrored, zero and identity normal scaling")
        return
    if args.source is None or args.output is None:
        parser.error("source and output are required")
    report = args.output.with_suffix(".json")
    if args.output.suffix.lower() != ".png" or args.output.exists() or report.exists():
        parser.error("output must be a fresh PNG path with no existing JSON report")
    source_hash = digest(args.source)
    with Image.open(args.source) as source:
        if source.mode != "RGB":
            parser.error("this experiment requires an RGB source normal map")
        result = Image.new("RGB", source.size)
        result.putdata([scaled(pixel, .25) for pixel in source.get_flattened_data()])
        args.output.parent.mkdir(parents=True, exist_ok=True)
        result.save(args.output)
        size = source.size
    assert digest(args.source) == source_hash, "source changed during bake"
    report.write_text(json.dumps({
        "source": str(args.source), "source_sha256": source_hash,
        "output": str(args.output), "output_sha256": digest(args.output),
        "size": size, "mode": "RGB", "scale": .25,
        "formula": "n = rgb / 127.5 - 1; n.xy *= scale; n = normalize(n); rgb = round((n + 1) * 127.5)",
        "color_space": "Raw normal data; no sRGB transform",
        "gltf_normalTexture_scale_after_bake": 1,
        "mips": "PNG has one level; this isolates strength and does not test mip generation",
        "pillow": pillow_version, "script_sha256": digest(Path(__file__)),
        "status": "CPU candidate only; formal source and runtime unchanged",
    }, indent=2) + "\n")
    print(report)


if __name__ == "__main__":
    main()
