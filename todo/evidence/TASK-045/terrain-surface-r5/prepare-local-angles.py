"""Bounded variation on the local-patches trial: 2m coverage and continuous fixed angles."""
from pathlib import Path
import hashlib
import json
import shutil
import difflib

ROOT = Path(__file__).resolve().parents[4]
base = ROOT / 'output/assets/terrain-r5-diagnostics'
project = base / 'local-angles'
assert not project.exists(), 'Candidate already exists'
shutil.copytree(base / 'local-patches', project, symlinks=True)
path = Path('game/assets/shaders/terrain-slope.wgsl')
s = (project / path).read_text()
s = s.replace('every six metres', 'every two metres').replace('metres / 6.0', 'metres / 2.0')
start = s.index('fn quarter_turn(')
end = s.index('fn ground_phase(', start)
s = s[:start] + '''fn ground_angle(hash: u32) -> f32 {
    return f32(hash & 65535u) * (6.28318530718 / 65536.0);
}
fn rotate_ground(value: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2(c * value.x - s * value.y, s * value.x + c * value.y);
}
''' + s[end:]
s = s.replace('quarter_turn(uv, hash)', 'rotate_ground(uv, ground_angle(hash))')
s = s.replace('quarter_turn(dx, hash)', 'rotate_ground(dx, ground_angle(hash))')
s = s.replace('quarter_turn(dy, hash)', 'rotate_ground(dy, ground_angle(hash))')
s = s.replace('quarter_turn(nt.xy, 4u - (hash & 3u))', 'rotate_ground(nt.xy, -ground_angle(hash))')
(project / path).write_text(s)
evidence = ROOT / 'todo/evidence/TASK-045/terrain-surface-r5'
original = (base / 'shared' / path).read_text()
(evidence / 'local-angles.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True), s.splitlines(True), fromfile='a/' + str(path), tofile='b/' + str(path))))
(evidence / 'local-angles.json').write_text(json.dumps({'status': 'candidate, GPU NOT RUN', 'root': str(project.relative_to(ROOT)), 'candidate_shader_sha256': hashlib.sha256(s.encode()).hexdigest(), 'patch_metres': 2, 'texture_scale': 1, 'rotation': 'continuous, fixed angle per lattice vertex', 'neighbours': 3}, indent=2) + '\n')
print(project.relative_to(ROOT))
