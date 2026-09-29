// CHR-001 grey-study lighting experiment; geometry, skinning and shadow passes stay Bevy's
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::lights,
}

struct CharacterInk {
    bands: vec4<f32>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> ink: CharacterInk;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr = pbr_input_from_standard_material(in, is_front);
    pbr.material.base_color = alpha_discard(pbr.material, pbr.material.base_color);

    // ponytail: the current city has one sun; select its strongest directional light
    // Point/spot-only scenes retain PBR, and multiple key lights need a per-light model
    var strongest = 0.0;
    var light_direction = vec3<f32>(0.0, 1.0, 0.0);
    for (var i = 0u; i < lights.n_directional_lights; i += 1u) {
        let light = lights.directional_lights[i];
        let strength = dot(light.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        if strength > strongest {
            strongest = strength;
            light_direction = light.direction_to_light;
        }
    }
    if strongest > 0.0 {
        let ndotl = dot(pbr.N, light_direction);
        let edge = max(fwidth(ndotl), ink.bands.w);
        let band = mix(ink.bands.y, ink.bands.z,
            smoothstep(ink.bands.x - edge, ink.bands.x + edge, ndotl));
        var tangent = pbr.N - light_direction * ndotl;
        if dot(tangent, tangent) < 0.00001 {
            // A stable perpendicular at the pole avoids normalize(0)
            let axis = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
                abs(light_direction.y) > 0.9);
            tangent = cross(light_direction, axis);
        }
        pbr.N = normalize(tangent) * sqrt(1.0 - band * band) + light_direction * band;
    }

    // world_normal is unchanged: real occluders and skin deformation still drive shadow maps
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr);
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
}
