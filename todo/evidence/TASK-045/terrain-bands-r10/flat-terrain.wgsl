// TASK-045: continuous coverage from the unperturbed terrain normal
// Geometry, shadow depth and motion vectors keep Bevy 0.19's real world transforms
#ifdef PREPASS_PIPELINE
#import bevy_pbr::{prepass_io::VertexOutput, pbr_prepass_functions}
#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::FragmentOutput
#endif
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif
#import bevy_pbr::{pbr_bindings, pbr_functions, mesh_view_bindings::view}

struct TerrainBlend {
    slope_uv: vec4<f32>,
    color_roughness: vec4<f32>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> terrain: TerrainBlend;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var rock_color: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var rock_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var rock_normal: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var rock_normal_sampler: sampler;

// Shared lattice vertices keep the same phase across adjacent triangles
// Coverage changes every two metres; each texture still uses its original metre scale
struct GroundPatches {
    cells: array<vec2<i32>, 3>,
    weights: vec3<f32>,
}
fn ground_patches(metres: vec2<f32>) -> GroundPatches {
    let position = metres / 2.0;
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
fn ground_angle(hash: u32) -> f32 {
    return f32(hash & 65535u) * (6.28318530718 / 65536.0);
}
fn rotate_ground(value: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2(c * value.x - s * value.y, s * value.x + c * value.y);
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
            pbr_bindings::base_color_sampler, rotate_ground(uv, ground_angle(hash)) + ground_phase(hash),
            rotate_ground(dx, ground_angle(hash)), rotate_ground(dy, ground_angle(hash))).rgb;
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
            pbr_bindings::normal_map_sampler, rotate_ground(uv, ground_angle(hash)) + ground_phase(hash),
            rotate_ground(dx, ground_angle(hash)), rotate_ground(dy, ground_angle(hash))).rgb * 2.0 - 1.0;
        // NormalGL tangent gradients use the inverse of the UV rotation
        tangent_normal += patches.weights[i] * vec3(rotate_ground(nt.xy, -ground_angle(hash)), nt.z);
    }
    return pbr_functions::apply_normal_mapping(0u,
        pbr_functions::calculate_tbn_mikktspace(normal, tangent), false, true,
        normalize(tangent_normal) * 0.5 + vec3(0.5));
}

fn rock_weight(world_normal: vec3<f32>) -> f32 {
    return 1.0 - smoothstep(terrain.slope_uv.x, terrain.slope_uv.y,
        abs(normalize(world_normal).y));
}

fn blend_normal(uv: vec2<f32>, normal: vec3<f32>, tangent: vec4<f32>,
    ground_normal: vec3<f32>, weight: f32) -> vec3<f32> {
    let rock_nt = textureSampleBias(rock_normal, rock_normal_sampler,
        uv * terrain.slope_uv.zw, view.mip_bias).rgb;
    // Both registered maps are three-channel NormalGL, sampled as linear data
    let rock_n = pbr_functions::apply_normal_mapping(0u,
        pbr_functions::calculate_tbn_mikktspace(normal, tangent), false, true, rock_nt);
    return normalize(mix(ground_normal, rock_n, weight));
}

#ifdef PREPASS_PIPELINE
#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var out: FragmentOutput;
#ifdef NORMAL_PREPASS
    let uv = (pbr_bindings::material.uv_transform * vec3(in.uv, 1.0)).xy;
    let ground_n = ground_normal(uv, ground_patches(in.uv), in.world_normal, in.world_tangent);
    let normal = blend_normal(in.uv, in.world_normal, in.world_tangent,
        ground_n, rock_weight(in.world_normal));
    out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);
#endif
#ifdef MOTION_VECTOR_PREPASS
    out.motion_vector = pbr_prepass_functions::calculate_motion_vector(
        in.world_position, in.previous_world_position);
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif
    return out;
}
#else
@fragment
fn fragment(in: VertexOutput) {}
#endif
#else
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in, is_front);
    let weight = rock_weight(in.world_normal);
    let color = textureSampleBias(rock_color, rock_color_sampler,
        in.uv * terrain.slope_uv.zw, view.mip_bias).rgb * terrain.color_roughness.rgb;
    let uv = (pbr_bindings::material.uv_transform * vec3(in.uv, 1.0)).xy;
    let patches = ground_patches(in.uv);
    var ground = ground_color(uv, patches) * pbr_bindings::material.base_color.rgb;
#ifdef VERTEX_COLORS
    ground *= in.color.rgb;
#endif
    pbr.material.base_color = vec4(mix(ground, color, weight), 1.0);
    pbr.material.perceptual_roughness = mix(pbr.material.perceptual_roughness,
        terrain.color_roughness.w, weight);
#ifndef LOAD_PREPASS_NORMALS
    pbr.N = ground_normal(uv, patches, pbr.world_normal, in.world_tangent);
    pbr.N = blend_normal(in.uv, pbr.world_normal, in.world_tangent, pbr.N, weight);
#endif
    pbr.material.base_color = vec4(0.35, 0.5, 0.25, 1.0);
    pbr.N = normalize(pbr.world_normal);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
}
#endif
