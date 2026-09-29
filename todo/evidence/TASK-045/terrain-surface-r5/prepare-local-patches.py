"""Isolated three-neighbour terrain candidate; native metre UVs remain unchanged."""
from pathlib import Path
import difflib
import hashlib
import json
import shutil

ROOT = Path(__file__).resolve().parents[4]
base = ROOT / 'output/assets/terrain-r5-diagnostics'
project = base / 'local-patches'
assert not project.exists(), 'Candidate already exists'
shared = base / 'shared'
(project / 'game/assets').mkdir(parents=True)
(project / 'source-assets').symlink_to(shared / 'source-assets', target_is_directory=True)
for path in (shared / 'game/assets').iterdir():
    destination = project / 'game/assets' / path.name
    if path.name == 'shaders':
        shutil.copytree(path, destination)
    else:
        destination.symlink_to(path, target_is_directory=path.is_dir())
path = Path('game/assets/shaders/terrain-slope.wgsl')
original = (shared / path).read_text()
helper = '''// Shared lattice vertices keep the same phase across adjacent triangles
// Coverage changes every six metres; each texture still uses its original metre scale
struct GroundPatches {
    cells: array<vec2<i32>, 3>,
    weights: vec3<f32>,
}
fn ground_patches(metres: vec2<f32>) -> GroundPatches {
    let position = metres / 6.0;
    let cell = vec2<i32>(floor(position));
    let f = fract(position);
    var patches: GroundPatches;
    if f.x + f.y < 1.0 {
        patches.cells = array(cell, cell + vec2(1, 0), cell + vec2(0, 1));
        patches.weights = vec3(1.0 - f.x - f.y, f.x, f.y);
    } else {
        patches.cells = array(cell + vec2(1, 1), cell + vec2(0, 1), cell + vec2(1, 0));
        patches.weights = vec3(f.x + f.y - 1.0, 1.0 - f.x, 1.0 - f.y);
    }
    patches.weights = patches.weights * patches.weights * (3.0 - 2.0 * patches.weights);
    patches.weights /= dot(patches.weights, vec3(1.0));
    return patches;
}
fn ground_hash(cell: vec2<i32>) -> u32 {
    var value = bitcast<u32>(cell.x) * 1664525u + bitcast<u32>(cell.y) * 1013904223u;
    value = (value ^ (value >> 16u)) * 2246822519u;
    return value ^ (value >> 13u);
}
fn quarter_turn(value: vec2<f32>, turns: u32) -> vec2<f32> {
    switch turns & 3u {
        case 1u: { return vec2(-value.y, value.x); }
        case 2u: { return -value; }
        case 3u: { return vec2(value.y, -value.x); }
        default: { return value; }
    }
}
fn ground_phase(hash: u32) -> vec2<f32> {
    return vec2(f32((hash >> 2u) & 32767u), f32((hash >> 17u) & 32767u)) / 32768.0;
}
fn ground_color(uv: vec2<f32>, patches: GroundPatches) -> vec3<f32> {
    // Derivatives are taken before the discrete phase choice, avoiding false coarse mip selection
    let dx = dpdx(uv) * exp2(view.mip_bias);
    let dy = dpdy(uv) * exp2(view.mip_bias);
    var color = vec3(0.0);
    for (var i = 0u; i < 3u; i++) {
        let hash = ground_hash(patches.cells[i]);
        color += patches.weights[i] * textureSampleGrad(pbr_bindings::base_color_texture,
            pbr_bindings::base_color_sampler, quarter_turn(uv, hash) + ground_phase(hash),
            quarter_turn(dx, hash), quarter_turn(dy, hash)).rgb;
    }
    return color;
}
fn ground_normal(uv: vec2<f32>, patches: GroundPatches,
    normal: vec3<f32>, tangent: vec4<f32>) -> vec3<f32> {
    let dx = dpdx(uv) * exp2(view.mip_bias);
    let dy = dpdy(uv) * exp2(view.mip_bias);
    var tangent_normal = vec3(0.0);
    for (var i = 0u; i < 3u; i++) {
        let hash = ground_hash(patches.cells[i]);
        let nt = textureSampleGrad(pbr_bindings::normal_map_texture,
            pbr_bindings::normal_map_sampler, quarter_turn(uv, hash) + ground_phase(hash),
            quarter_turn(dx, hash), quarter_turn(dy, hash)).rgb * 2.0 - 1.0;
        // NormalGL tangent gradients use the inverse of the UV rotation
        tangent_normal += patches.weights[i] * vec3(quarter_turn(nt.xy, 4u - (hash & 3u)), nt.z);
    }
    return pbr_functions::apply_normal_mapping(0u,
        pbr_functions::calculate_tbn_mikktspace(normal, tangent), false, true,
        normalize(tangent_normal) * 0.5 + vec3(0.5));
}

'''
s = original.replace('fn rock_weight(', helper + 'fn rock_weight(', 1)
s = s.replace('''    let nt = textureSampleBias(pbr_bindings::normal_map_texture,
        pbr_bindings::normal_map_sampler, uv, view.mip_bias).rgb;
    let ground_n = pbr_functions::apply_normal_mapping(pbr_bindings::material.flags,
        pbr_functions::calculate_tbn_mikktspace(in.world_normal, in.world_tangent), false, is_front, nt);''', '''    let ground_n = ground_normal(uv, ground_patches(in.uv), in.world_normal, in.world_tangent);''')
s = s.replace('''    pbr.material.base_color = vec4(mix(pbr.material.base_color.rgb, color, weight), 1.0);''', '''    let uv = (pbr_bindings::material.uv_transform * vec3(in.uv, 1.0)).xy;
    let patches = ground_patches(in.uv);
    var ground = ground_color(uv, patches) * pbr_bindings::material.base_color.rgb;
#ifdef VERTEX_COLORS
    ground *= in.color.rgb;
#endif
    pbr.material.base_color = vec4(mix(ground, color, weight), 1.0);''')
s = s.replace('''    pbr.N = blend_normal(in.uv, pbr.world_normal, in.world_tangent, pbr.N, weight);''', '''    pbr.N = ground_normal(uv, patches, pbr.world_normal, in.world_tangent);
    pbr.N = blend_normal(in.uv, pbr.world_normal, in.world_tangent, pbr.N, weight);''')
assert s != original and s.count('ground_normal(') == 3
(project / path).write_text(s)
evidence = ROOT / 'todo/evidence/TASK-045/terrain-surface-r5'
(evidence / 'local-patches.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True), s.splitlines(True), fromfile='a/' + str(path), tofile='b/' + str(path))))
(evidence / 'local-patches.json').write_text(json.dumps({'status': 'candidate, GPU NOT RUN', 'root': str(project.relative_to(ROOT)), 'base_shader_sha256': hashlib.sha256(original.encode()).hexdigest(), 'candidate_shader_sha256': hashlib.sha256(s.encode()).hexdigest(), 'patch_metres': 6, 'texture_scale': 1, 'rotations_degrees': [0, 90, 180, 270], 'neighbours': 3, 'seed': 'fixed integer world-UV lattice hash'}, indent=2) + '\n')
print(project.relative_to(ROOT))
