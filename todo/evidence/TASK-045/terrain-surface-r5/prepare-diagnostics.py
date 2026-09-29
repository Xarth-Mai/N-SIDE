"""Freeze the recorded Git baseline and alter only the terrain shader per A/B case."""
from pathlib import Path
import hashlib
import json
import shutil
import io
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT / 'output/assets/terrain-r5-diagnostics'
assert not OUT.exists(), 'Use a fresh diagnostic destination'
shared = OUT / 'shared'
# Replays remain usable after the selected shader is promoted to the live tree
revision = 'b948cabe717c1c6081055267a4b0c063b7de855d'
paths = ['game/assets', 'source-assets/district-map/district.json',
         'source-assets/district-scene/appearance.json', 'source-assets/district-scene/daylight.json']
archive = subprocess.run(['git', 'archive', revision, *paths], cwd=ROOT, check=True, capture_output=True).stdout
with tarfile.open(fileobj=io.BytesIO(archive)) as files:
    files.extractall(shared, filter='data')
shader_path = Path('game/assets/shaders/terrain-slope.wgsl')
original = (shared / shader_path).read_text()
plain_normal = original.replace('    out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);',
    '    out.normal = vec4(normalize(in.world_normal) * 0.5 + vec3(0.5), 1.0);')
plain_normal = plain_normal.replace('    out.color = apply_pbr_lighting(pbr);',
    '    pbr.N = normalize(pbr.world_normal);\n    out.color = apply_pbr_lighting(pbr);')
plain_color = original.replace('    pbr.material.base_color = vec4(mix(pbr.material.base_color.rgb, color, weight), 1.0);',
    '    pbr.material.base_color = vec4(0.28, 0.33, 0.18, 1.0);')
assert plain_normal != original and plain_color != original
records = {}
for name, shader in [('baseline', original), ('no-normal', plain_normal), ('solid-color', plain_color)]:
    project = OUT / name
    (project / 'game/assets').mkdir(parents=True)
    (project / 'source-assets').symlink_to(shared / 'source-assets', target_is_directory=True)
    for path in (shared / 'game/assets').iterdir():
        destination = project / 'game/assets' / path.name
        if path.name == 'shaders':
            shutil.copytree(path, destination)
        else:
            destination.symlink_to(path, target_is_directory=path.is_dir())
    (project / shader_path).write_text(shader)
    records[name] = {'project_root':str(project.relative_to(ROOT)), 'shader_sha256':hashlib.sha256(shader.encode()).hexdigest()}
files = [{'path':str(path.relative_to(shared)), 'sha256':hashlib.sha256(path.read_bytes()).hexdigest()} for path in sorted(shared.rglob('*')) if path.is_file()]
record = {'baseline_revision':revision, 'scope':'Independent diagnostic inputs; runtime/GPU NOT RUN by this script', 'variants':records, 'frozen_files':files}
(ROOT / 'todo/evidence/TASK-045/terrain-surface-r5/diagnostic-inputs.json').write_text(json.dumps(record, ensure_ascii=False, indent=2)+'\n')
print(json.dumps(records, indent=2))
