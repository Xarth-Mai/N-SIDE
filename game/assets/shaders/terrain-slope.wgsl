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
    let nt = textureSampleBias(pbr_bindings::normal_map_texture,
        pbr_bindings::normal_map_sampler, uv, view.mip_bias).rgb;
    let ground_n = pbr_functions::apply_normal_mapping(pbr_bindings::material.flags,
        pbr_functions::calculate_tbn_mikktspace(in.world_normal, in.world_tangent), false, is_front, nt);
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
    pbr.material.base_color = vec4(mix(pbr.material.base_color.rgb, color, weight), 1.0);
    pbr.material.perceptual_roughness = mix(pbr.material.perceptual_roughness,
        terrain.color_roughness.w, weight);
#ifndef LOAD_PREPASS_NORMALS
    pbr.N = blend_normal(in.uv, pbr.world_normal, in.world_tangent, pbr.N, weight);
#endif
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
}
#endif
