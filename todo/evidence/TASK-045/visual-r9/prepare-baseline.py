"""Recreate the immutable r9 comparison inputs; run before building the baseline Viewer."""
from pathlib import Path
import hashlib
import io
import json
import subprocess
import tarfile

root = Path(__file__).resolve().parents[4]
target = root / "output/visual-r9/baseline"
revision = "13397c5626c8952511ceda52d3c4a8877b901acc"
paths = ["game/assets", "game/src", "game/Cargo.toml", "game/Cargo.lock",
         "source-assets/district-map/district.json", "source-assets/ui-kit/tokens.json",
         "source-assets/district-scene/appearance.json", "source-assets/district-scene/daylight.json"]
target.mkdir(parents=True, exist_ok=False)
archive = subprocess.check_output(["git", "archive", revision, *paths], cwd=root)
with tarfile.open(fileobj=io.BytesIO(archive)) as files:
    files.extractall(target, filter="data")
# Add only the inspection camera to the old implementation, leaving old geometry intact
viewer = target / "game/src/bin/map_viewer.rs"
source = viewer.read_text()
needle = '    views.push(("inspect-shop-retaining", view(eye, target)));\n'
assert source.count(needle) == 1
source = source.replace(needle, needle + '''    // The public tree_entry keeps the north retaining face clear of trees[45] on the left
    let mut eye = map.nodes["tree_entry"];
    eye[2] += 1.7;
    let target = [60., 223., eye[2]];
    println!(
        "[visual/view] name=inspect-tree-court-retaining source=surfaces[2] eye={eye:?} target={target:?} fov=55"
    );
    views.push(("inspect-tree-court-retaining", view(eye, target)));
''')
viewer.write_text(source)
print(json.dumps({"revision": revision, "project_root": str(target),
                  "camera_only_viewer_sha256": hashlib.sha256(viewer.read_bytes()).hexdigest()}))
