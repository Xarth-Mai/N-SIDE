use std::{path::Path, str::FromStr};

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{Exposure, Hdr},
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, Skybox},
    pbr::{ContactShadows, ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel},
    post_process::bloom::Bloom,
    prelude::*,
};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Antialiasing {
    Msaa4,
    Taa,
    TaaSsao,
}

impl FromStr for Antialiasing {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "msaa4" => Ok(Self::Msaa4),
            "taa" => Ok(Self::Taa),
            "taa-ssao" => Ok(Self::TaaSsao),
            _ => Err(format!(
                "unknown antialiasing {value:?}; use msaa4, taa or taa-ssao"
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaylightSettings {
    pub exposure_ev100: f32,
    pub sun_illuminance: f32,
    pub sun_color: [f32; 3],
    pub sun_position: [f32; 3],
    pub sky_top: [f32; 3],
    pub sky_horizon: [f32; 3],
    pub sky_ground: [f32; 3],
    pub sky_brightness: f32,
    pub environment_intensity: f32,
    pub ambient_color: [f32; 3],
    pub ambient_brightness: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub shadow_map_size: usize,
    pub shadow_first_cascade: f32,
    pub shadow_maximum_distance: f32,
    pub contact_shadows: bool,
    pub contact_length: f32,
    pub contact_thickness: f32,
    pub bloom: bool,
}

impl DaylightSettings {
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("[visual/read] {}: {error}", path.display()))?;
        let settings: Self = serde_json::from_slice(&bytes)
            .map_err(|error| format!("[visual/json] {}: {error}", path.display()))?;
        settings
            .validate()
            .map_err(|error| format!("[visual/settings] {}: {error}", path.display()))?;
        Ok(settings)
    }

    fn validate(&self) -> Result<(), String> {
        if !self.exposure_ev100.is_finite() || !(-16.0..=32.0).contains(&self.exposure_ev100) {
            return Err("exposure_ev100 must be finite and between -16 and 32".into());
        }
        for (name, value) in [
            ("sun_illuminance", self.sun_illuminance),
            ("sky_brightness", self.sky_brightness),
            ("environment_intensity", self.environment_intensity),
            ("ambient_brightness", self.ambient_brightness),
            ("fog_start", self.fog_start),
            ("fog_end", self.fog_end),
            ("shadow_first_cascade", self.shadow_first_cascade),
            ("shadow_maximum_distance", self.shadow_maximum_distance),
            ("contact_length", self.contact_length),
            ("contact_thickness", self.contact_thickness),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("{name} must be finite and nonnegative"));
            }
        }
        for (name, color) in [
            ("sun_color", self.sun_color),
            ("sky_top", self.sky_top),
            ("sky_horizon", self.sky_horizon),
            ("sky_ground", self.sky_ground),
            ("ambient_color", self.ambient_color),
        ] {
            if color
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            {
                return Err(format!(
                    "{name} must contain three sRGB values between 0 and 1"
                ));
            }
        }
        let sun = Vec3::from_array(self.sun_position);
        if !sun.is_finite()
            || !sun.length_squared().is_finite()
            || sun.cross(Vec3::Y).length_squared() <= f32::EPSILON
        {
            return Err("sun_position must define a finite nonvertical light direction".into());
        }
        if self.fog_end <= self.fog_start {
            return Err("fog_end must exceed fog_start".into());
        }
        if self.shadow_first_cascade <= 0.0
            || self.shadow_maximum_distance <= self.shadow_first_cascade
        {
            return Err("shadow distances must satisfy 0 < first < maximum".into());
        }
        if !self.shadow_map_size.is_power_of_two() || !(512..=8192).contains(&self.shadow_map_size)
        {
            return Err("shadow_map_size must be a power of two between 512 and 8192".into());
        }
        if self.contact_shadows && (self.contact_length == 0.0 || self.contact_thickness == 0.0) {
            return Err("enabled contact shadows require positive length and thickness".into());
        }
        Ok(())
    }

    pub fn install(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_from_array(self.sky_horizon)))
            .insert_resource(GlobalAmbientLight {
                color: Color::srgb_from_array(self.ambient_color),
                brightness: self.ambient_brightness,
                ..default()
            })
            .insert_resource(DirectionalLightShadowMap {
                size: self.shadow_map_size,
            });
    }

    pub fn spawn_lighting(&self, commands: &mut Commands) {
        commands.spawn((
            DirectionalLight {
                illuminance: self.sun_illuminance,
                color: Color::srgb_from_array(self.sun_color),
                shadow_maps_enabled: true,
                contact_shadows_enabled: self.contact_shadows,
                ..default()
            },
            Transform::from_translation(Vec3::from_array(self.sun_position))
                .looking_at(Vec3::ZERO, Vec3::Y),
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: self.shadow_first_cascade,
                maximum_distance: self.shadow_maximum_distance,
                ..default()
            }
            .build(),
        ));
    }

    pub fn configure_camera(
        &self,
        commands: &mut Commands,
        camera: Entity,
        images: &mut Assets<Image>,
        mode: Antialiasing,
    ) {
        // ponytail: three sky colors omit local reflections; replace with filtered sky assets when the sample needs them
        let mut environment = EnvironmentMapLight::hemispherical_gradient(
            images,
            Color::srgb_from_array(self.sky_top),
            Color::srgb_from_array(self.sky_horizon),
            Color::srgb_from_array(self.sky_ground),
        );
        environment.intensity = self.environment_intensity;
        let skybox = Skybox {
            image: Some(environment.specular_map.clone()),
            brightness: self.sky_brightness,
            ..default()
        };
        let mut entity = commands.entity(camera);
        entity.insert((
            Hdr,
            Exposure {
                ev100: self.exposure_ev100,
            },
            Tonemapping::TonyMcMapface,
            skybox,
            environment,
            DistanceFog {
                color: Color::srgb_from_array(self.sky_horizon),
                falloff: FogFalloff::Linear {
                    start: self.fog_start,
                    end: self.fog_end,
                },
                ..default()
            },
            if mode == Antialiasing::Msaa4 {
                Msaa::Sample4
            } else {
                Msaa::Off
            },
        ));
        if self.contact_shadows {
            entity.insert(ContactShadows {
                length: self.contact_length,
                thickness: self.contact_thickness,
                ..default()
            });
        }
        if mode != Antialiasing::Msaa4 {
            entity.insert(TemporalAntiAliasing::default());
        }
        if mode == Antialiasing::TaaSsao {
            entity.insert(ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                ..default()
            });
        }
        if self.bloom {
            entity.insert(Bloom {
                intensity: 0.04,
                ..Bloom::NATURAL
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass};

    fn settings() -> DaylightSettings {
        DaylightSettings::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../source-assets/district-scene/daylight.json"),
        )
        .unwrap()
    }

    #[test]
    fn daylight_input_rejects_invalid_render_parameters() {
        let baseline = settings();
        let mut invalid = baseline.clone();
        invalid.fog_end = invalid.fog_start;
        assert!(invalid.validate().is_err());
        invalid = baseline.clone();
        invalid.sun_position = [0.0, 1.0, 0.0];
        assert!(invalid.validate().is_err());
        invalid = baseline.clone();
        invalid.shadow_map_size = 4000;
        assert!(invalid.validate().is_err());
        invalid = baseline.clone();
        invalid.shadow_maximum_distance = invalid.shadow_first_cascade;
        assert!(invalid.validate().is_err());
        invalid = baseline.clone();
        invalid.sky_top[0] = f32::NAN;
        assert!(invalid.validate().is_err());
        invalid = baseline;
        invalid.exposure_ev100 = f32::INFINITY;
        assert!(invalid.validate().is_err());
        assert!("taa+msaa4".parse::<Antialiasing>().is_err());
    }

    #[test]
    fn camera_aa_modes_use_compatible_prepasses_and_matching_sky() {
        let mut settings = settings();
        settings.contact_shadows = true;
        let mut world = World::new();
        let mut images = Assets::<Image>::default();
        for label in ["msaa4", "taa", "taa-ssao"] {
            let mode: Antialiasing = label.parse().unwrap();
            let camera = world.spawn(Camera3d::default()).id();
            settings.configure_camera(&mut world.commands(), camera, &mut images, mode);
            world.flush();
            let entity = world.entity(camera);
            assert_eq!(
                entity.get::<Msaa>().unwrap(),
                if mode == Antialiasing::Msaa4 {
                    &Msaa::Sample4
                } else {
                    &Msaa::Off
                }
            );
            assert!(entity.contains::<Hdr>());
            assert!(entity.contains::<ContactShadows>() && entity.contains::<DepthPrepass>());
            assert_eq!(
                entity.contains::<TemporalAntiAliasing>(),
                mode != Antialiasing::Msaa4
            );
            assert_eq!(
                entity.contains::<MotionVectorPrepass>(),
                mode != Antialiasing::Msaa4
            );
            assert_eq!(
                entity.contains::<ScreenSpaceAmbientOcclusion>(),
                mode == Antialiasing::TaaSsao
            );
            assert_eq!(
                entity.contains::<NormalPrepass>(),
                mode == Antialiasing::TaaSsao
            );
            let environment = entity.get::<EnvironmentMapLight>().unwrap();
            assert_eq!(
                entity.get::<Skybox>().unwrap().image.as_ref(),
                Some(&environment.specular_map)
            );
            assert_eq!(
                images
                    .get(&environment.diffuse_map)
                    .unwrap()
                    .texture_descriptor
                    .size
                    .depth_or_array_layers,
                6
            );
        }
    }
}
