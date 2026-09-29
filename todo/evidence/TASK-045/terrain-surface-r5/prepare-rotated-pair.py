"""Create one shader-only candidate beside the unchanged diagnostic roots."""
from pathlib import Path
import difflib
import hashlib
import json
import shutil

ROOT=Path(__file__).resolve().parents[4]
base=ROOT/'output/assets/terrain-r5-diagnostics'
project=base/'rotated-pair'
assert not project.exists(), 'Candidate already exists'
shared=base/'shared'
(project/'game/assets').mkdir(parents=True)
(project/'source-assets').symlink_to(shared/'source-assets',target_is_directory=True)
for path in (shared/'game/assets').iterdir():
    destination=project/'game/assets'/path.name
    if path.name=='shaders':
        shutil.copytree(path,destination)
    else:
        destination.symlink_to(path,target_is_directory=path.is_dir())
path=Path('game/assets/shaders/terrain-slope.wgsl')
original=(shared/path).read_text()
helper='''// Equal metric scale, one fixed rotation and phase break the source's repeated column bands
fn varied_uv(uv: vec2<f32>) -> vec2<f32> {
    return vec2(0.7986355 * uv.x - 0.601815 * uv.y,
                0.601815 * uv.x + 0.7986355 * uv.y) + vec2(0.37, 0.71);
}

fn varied_ground_normal(uv: vec2<f32>, normal: vec3<f32>, tangent: vec4<f32>,
    original_normal: vec3<f32>) -> vec3<f32> {
    let sampled = textureSampleBias(pbr_bindings::normal_map_texture,
        pbr_bindings::normal_map_sampler, varied_uv(uv), view.mip_bias).rgb * 2.0 - 1.0;
    // UVs rotate by R; tangent-space gradients rotate back by transpose(R)
    let nt = vec3(0.7986355 * sampled.x + 0.601815 * sampled.y,
                 -0.601815 * sampled.x + 0.7986355 * sampled.y, sampled.z);
    let rotated_normal = pbr_functions::apply_normal_mapping(0u,
        pbr_functions::calculate_tbn_mikktspace(normal, tangent), false, true,
        nt * 0.5 + vec3(0.5));
    return normalize(mix(original_normal, rotated_normal, 0.5));
}

'''
s=original.replace('fn rock_weight(',helper+'fn rock_weight(',1)
s=s.replace('''    let normal = blend_normal(in.uv, in.world_normal, in.world_tangent,
        ground_n, rock_weight(in.world_normal));''','''    let varied_n = varied_ground_normal(uv, in.world_normal, in.world_tangent, ground_n);
    let normal = blend_normal(in.uv, in.world_normal, in.world_tangent,
        varied_n, rock_weight(in.world_normal));''')
s=s.replace('''    pbr.material.base_color = vec4(mix(pbr.material.base_color.rgb, color, weight), 1.0);''','''    let ground_uv = (pbr_bindings::material.uv_transform * vec3(in.uv, 1.0)).xy;
    var varied_color = textureSampleBias(pbr_bindings::base_color_texture,
        pbr_bindings::base_color_sampler, varied_uv(ground_uv), view.mip_bias).rgb
        * pbr_bindings::material.base_color.rgb;
#ifdef VERTEX_COLORS
    varied_color *= in.color.rgb;
#endif
    let ground_color = mix(pbr.material.base_color.rgb, varied_color, 0.5);
    pbr.material.base_color = vec4(mix(ground_color, color, weight), 1.0);''')
s=s.replace('''    pbr.N = blend_normal(in.uv, pbr.world_normal, in.world_tangent, pbr.N, weight);''','''    pbr.N = varied_ground_normal(ground_uv, pbr.world_normal, in.world_tangent, pbr.N);
    pbr.N = blend_normal(in.uv, pbr.world_normal, in.world_tangent, pbr.N, weight);''')
assert s!=original and s.count('varied_ground_normal(')==3
(project/path).write_text(s)
evidence=ROOT/'todo/evidence/TASK-045/terrain-surface-r5'
(evidence/'rotated-pair.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True),s.splitlines(True),fromfile='a/'+str(path),tofile='b/'+str(path))))
(evidence/'rotated-pair.json').write_text(json.dumps({'status':'candidate, GPU NOT RUN','root':str(project.relative_to(ROOT)),'base_shader_sha256':hashlib.sha256(original.encode()).hexdigest(),'candidate_shader_sha256':hashlib.sha256(s.encode()).hexdigest(),'scale':1.0,'rotation_degrees':37,'phase':[.37,.71],'mix':.5,'sample_increase':'one ground color and one ground normal per visible surface; SSAO forward reuses the prepass normal'},indent=2)+'\n')
print(project.relative_to(ROOT))
