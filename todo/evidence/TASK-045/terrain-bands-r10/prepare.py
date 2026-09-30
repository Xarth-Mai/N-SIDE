"""Recreate isolated flat-terrain and marked-road diagnosis roots; formal assets stay read-only"""
import json
import os
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT / 'output/assets/terrain-bands-r10'
flat = OUT / 'flat'
flat.mkdir(parents=True)
(flat / 'source-assets').symlink_to(ROOT / 'source-assets', target_is_directory=True)
(flat / 'game').mkdir()
shutil.copytree(ROOT / 'game/assets', flat / 'game/assets', copy_function=lambda s, d: os.symlink(Path(s).resolve(), d))
shader = flat / 'game/assets/shaders/terrain-slope.wgsl'
text = shader.read_text()
shader.unlink()
needle = '    var out: FragmentOutput;\n    out.color = apply_pbr_lighting(pbr);'
assert text.count(needle) == 1
shader.write_text(text.replace(needle, '    pbr.material.base_color = vec4(0.35, 0.5, 0.25, 1.0);\n    pbr.N = normalize(pbr.world_normal);\n' + needle))
marked = OUT / 'road-mark'
marked.mkdir()
(marked / 'game').symlink_to(flat / 'game', target_is_directory=True)
(marked / 'source-assets').mkdir()
for source in (ROOT / 'source-assets').iterdir():
    dest = marked / 'source-assets' / source.name
    if source.name == 'district-scene':
        shutil.copytree(source, dest, copy_function=lambda s, d: os.symlink(Path(s).resolve(), d))
    else:
        dest.symlink_to(source, target_is_directory=source.is_dir())
appearance = marked / 'source-assets/district-scene/appearance.json'
data = json.loads(appearance.read_text())
appearance.unlink()
data['materials']['paving'].update(color=[1, 0, 1, 1], color_texture=None, normal_texture=None, unlit=True)
appearance.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
print(OUT)
