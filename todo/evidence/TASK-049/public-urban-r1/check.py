"""Compare registered brick DDS pixels with originals and make a temporary tile preview."""

import hashlib
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageStat

ROOT = Path(__file__).resolve().parents[4]
SOURCE = ROOT / "source-assets/environment-kit"
RUNTIME = ROOT / "game/assets/environment"
OUTPUT = ROOT / "output/assets/public-urban-r1"
OUTPUT.mkdir(parents=True, exist_ok=True)

report = {"files": [], "runtime_scene_check": "NOT RUN"}
manifest = json.loads((SOURCE / "asset-manifest.json").read_text())
files = [f for f in manifest["files"] if f["source"].startswith("materials/public-urban/")]
assert len(files) == 2, "Expected the brick Color and NormalGL pair"
for file in files:
    source = SOURCE / file["source"]
    runtime = RUNTIME / file["output"]
    assert hashlib.sha256(source.read_bytes()).hexdigest() == file["sha256"]
    original = Image.open(source).convert("RGB")
    exported = Image.open(runtime).convert("RGBA")
    assert original.size == exported.size == (1024, 1024)
    assert exported.getchannel("A").getextrema() == (255, 255)
    difference = ImageChops.difference(original, exported.convert("RGB"))
    assert difference.getbbox() is None, f"DDS level zero changed decoded pixels: {runtime}"
    # Actual exported DDS is repeated without gutters so the wrap boundary can be reviewed
    tile = exported.convert("RGB").resize((256, 256), Image.Resampling.LANCZOS)
    preview = Image.new("RGB", (768, 768))
    for y in range(3):
        for x in range(3):
            preview.paste(tile, (x * 256, y * 256))
    path = OUTPUT / f"{runtime.stem}-tiles.png"
    preview.save(path)
    report["files"].append({
        "source": file["source"], "runtime": file["output"],
        "runtime_sha256": hashlib.sha256(runtime.read_bytes()).hexdigest(),
        "bytes": runtime.stat().st_size, "dimensions": list(exported.size),
        "dds_base_pixels_match_decoded_jpg": True, "opaque_alpha": True,
        "channel_mean": ImageStat.Stat(original).mean,
        "preview": str(path.relative_to(ROOT)),
        "preview_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    })
report["status"] = "PASS: source hash, dimensions, alpha and DDS level-zero pixels"
print(json.dumps(report, ensure_ascii=False, indent=2))
