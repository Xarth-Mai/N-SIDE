//! Two registered ground surfaces share the real terrain, lighting and prepass
use bevy::{
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

pub(super) const SHADER: &str = "shaders/terrain-slope.wgsl";
pub(super) type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainBlend>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(super) struct TerrainBlend {
    /// World-up cosines for full rock / full ground, then rock repeats per metre
    #[uniform(100)]
    slope_uv: Vec4,
    /// Linear rock tint and its roughness
    #[uniform(100)]
    color_roughness: Vec4,
    #[texture(101)]
    #[sampler(102)]
    color: Handle<Image>,
    #[texture(103)]
    #[sampler(104)]
    normal: Handle<Image>,
}

impl MaterialExtension for TerrainBlend {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

pub(super) fn material(
    ground: &StandardMaterial,
    rock: &StandardMaterial,
) -> Result<TerrainMaterial, String> {
    if ground.base_color_texture.is_none() || ground.normal_map_texture.is_none() {
        return Err("terrain color and normal textures required".into());
    }
    let tint = rock.base_color.to_linear();
    Ok(TerrainMaterial {
        base: StandardMaterial {
            opaque_render_method: OpaqueRendererMethod::Forward,
            ..ground.clone()
        },
        extension: TerrainBlend {
            // Visual trial around r3's 45-degree cutoff; no height or texture-normal threshold
            slope_uv: Vec4::new(
                55.0_f32.to_radians().cos(),
                35.0_f32.to_radians().cos(),
                rock.uv_transform.matrix2.x_axis.x,
                rock.uv_transform.matrix2.y_axis.y,
            ),
            color_roughness: Vec4::new(tint.red, tint.green, tint.blue, rock.perceptual_roughness),
            color: rock
                .base_color_texture
                .clone()
                .ok_or("terrain_rock color texture required")?,
            normal: rock
                .normal_map_texture
                .clone()
                .ok_or("terrain_rock normal texture required")?,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_uses_registered_surfaces_and_keeps_normal_prepass() {
        let ground = StandardMaterial {
            base_color_texture: Some(Handle::default()),
            normal_map_texture: Some(Handle::default()),
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(1.0 / 2.1)),
            ..default()
        };
        let rock = StandardMaterial {
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(1.0 / 1.8)),
            perceptual_roughness: 0.96,
            ..ground.clone()
        };
        let blended = material(&ground, &rock).unwrap();
        assert_eq!(blended.base.base_color_texture, ground.base_color_texture);
        assert_eq!(blended.base.normal_map_texture, ground.normal_map_texture);
        assert_eq!(blended.base.uv_transform, ground.uv_transform);
        assert_eq!(blended.extension.color_roughness.w, 0.96);
        assert_eq!(blended.extension.slope_uv.z, 1.0 / 1.8);
        assert!(TerrainBlend::enable_prepass() && TerrainBlend::enable_shadows());
        assert!(matches!(TerrainBlend::vertex_shader(), ShaderRef::Default));
        assert!(matches!(
            TerrainBlend::prepass_vertex_shader(),
            ShaderRef::Default
        ));
        assert!(material(&ground, &StandardMaterial::default()).is_err());
        assert!(
            material(&StandardMaterial::default(), &rock)
                .unwrap_err()
                .contains("terrain color")
        );
        let mut missing_normal = rock;
        missing_normal.normal_map_texture = None;
        assert!(
            material(&ground, &missing_normal)
                .unwrap_err()
                .contains("normal")
        );
    }
}
