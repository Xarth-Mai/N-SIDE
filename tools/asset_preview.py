#!/usr/bin/env python3
"""Preview a raster at display size over contrasting backgrounds, without changing it.

Uses the unmodified create-game-assets inspection helper; see
third_party/skills/awesome-gamedev-agent-skills/LICENSE and NOTICE.
Contrast-background review is adapted from Godogen asset-gen/rembg.md;
source revision and retained license are listed in docs/production/asset-tool-review.md.
"""

import argparse
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw, ImageOps

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / ".agents/skills/create-game-assets/scripts"))
from asset_report import inspect, parse_size


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--display-size", required=True, type=parse_size)
    parser.add_argument("--filter", choices=["lanczos", "nearest"], default="lanczos")
    parser.add_argument("--require-cutout", action="store_true")
    args = parser.parse_args()
    try:
        if args.input.resolve() == args.out.resolve():
            raise ValueError("preview must not overwrite the source image")
        report = inspect(args.input, None)
        if args.require_cutout and not (report["alpha_min"] < 255 and report["alpha_max"] > 0):
            raise ValueError("cutout requires both visible content and some transparency")
        with Image.open(args.input) as source:
            fitted = ImageOps.contain(
                source.convert("RGBA"), args.display_size,
                method=Image.Resampling.LANCZOS if args.filter == "lanczos" else Image.Resampling.NEAREST,
            )
        width, height = args.display_size
        sheet = Image.new("RGB", (width * 3, height + 24), "#303030")
        draw = ImageDraw.Draw(sheet)
        for index, color in enumerate(("#ffffff", "#151515", "#ff00ff")):
            tile = Image.new("RGBA", (width, height), color)
            tile.alpha_composite(fitted, ((width - fitted.width) // 2, (height - fitted.height) // 2))
            sheet.paste(tile.convert("RGB"), (index * width, 0))
            draw.text((index * width + 4, height + 4), color, fill="white")
        args.out.parent.mkdir(parents=True, exist_ok=True)
        sheet.save(args.out)
        report.update({
            "ok": True,
            "source_sha256": hashlib.sha256(args.input.read_bytes()).hexdigest(),
            "preview": str(args.out),
            "display_box": args.display_size,
            "fitted_size": fitted.size,
            "filter": args.filter,
            "visual_review": "NOT RUN: inspect the preview and the actual game separately",
        })
        print(json.dumps(report, indent=2))
        return 0
    except (OSError, ValueError) as error:
        print(json.dumps({"ok": False, "path": str(args.input), "error": str(error)}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
