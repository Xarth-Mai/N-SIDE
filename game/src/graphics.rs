//! Player-selectable controls for the renderer already used by the game
use crate::{app::EntrySettings, world::visual::DaylightSettings};
use bevy::{
    anti_alias::{
        contrast_adaptive_sharpening::ContrastAdaptiveSharpening,
        fxaa::{Fxaa, Sensitivity},
        smaa::{Smaa, SmaaPreset},
        taa::TemporalAntiAliasing,
    },
    camera::Exposure,
    core_pipeline::{
        prepass::{
            DepthPrepass, MotionVectorPrepass, NormalPrepass,
            background_motion_vectors::{
                BackgroundMotionVectorsBindGroup, BackgroundMotionVectorsPipelineId,
            },
        },
        tonemapping::{DebandDither, Tonemapping},
    },
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, ShadowFilteringMethod},
    pbr::{ContactShadows, ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel},
    post_process::bloom::Bloom,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        camera::{MipBias, TemporalJitter},
        render_resource::TextureFormat,
        renderer::{RenderAdapter, RenderDevice},
    },
    window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode},
};
use serde::{Deserialize, Serialize};

pub(crate) const LABELS: [&str; 19] = [
    "抗锯齿",
    "FXAA / SMAA 质量",
    "环境遮蔽 SSAO",
    "锐化 CAS",
    "动态阴影",
    "阴影贴图",
    "阴影距离",
    "阴影过滤",
    "接触阴影",
    "泛光 Bloom",
    "距离雾",
    "曝光补偿",
    "色调映射",
    "渐变去色带",
    "场景视距",
    "垂直视角",
    "垂直同步",
    "窗口模式",
    "窗口分辨率",
];
const AA: [&str; 7] = [
    "关闭", "MSAA 2×", "MSAA 4×", "MSAA 8×", "FXAA", "SMAA", "TAA",
];
const TONE: [Tonemapping; 9] = [
    Tonemapping::TonyMcMapface,
    Tonemapping::None,
    Tonemapping::Reinhard,
    Tonemapping::ReinhardLuminance,
    Tonemapping::AcesFitted,
    Tonemapping::AgX,
    Tonemapping::SomewhatBoringDisplayTransform,
    Tonemapping::BlenderFilmic,
    Tonemapping::KhronosPbrNeutral,
];
const TONE_LABELS: [&str; 9] = [
    "Tony McMapface",
    "关闭",
    "Reinhard",
    "Reinhard 亮度",
    "ACES",
    "AgX",
    "Boring Display",
    "Blender Filmic",
    "Khronos PBR",
];
const SHADOW_SIZE: [usize; 4] = [1024, 2048, 4096, 8192];
const SHADOW_DISTANCE: [f32; 4] = [250.0, 600.0, 1200.0, 2400.0];
const FAR: [f32; 4] = [1500.0, 3000.0, 5000.0, 7000.0];
const FOV: [f32; 5] = [45.0, 55.0, 65.0, 75.0, 90.0];
const RESOLUTIONS: [(u32, u32); 4] = [(1280, 720), (1600, 900), (1920, 1080), (2560, 1440)];

// Index values are a versioned, bounded on-disk contract; arrays retain their order
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GraphicsSettings {
    pub antialiasing: u8,
    pub post_aa_quality: u8,
    pub ssao: u8,
    pub sharpening: u8,
    pub shadows: bool,
    pub shadow_resolution: u8,
    pub shadow_distance: u8,
    pub shadow_filter: u8,
    pub contact_shadows: bool,
    pub bloom: u8,
    pub fog: bool,
    pub exposure: u8,
    pub tonemapping: u8,
    pub deband: bool,
    pub view_distance: u8,
    pub fov: u8,
    pub vsync: bool,
    pub borderless: bool,
    pub resolution: u8,
}
impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            antialiasing: 2,
            post_aa_quality: 2,
            ssao: 0,
            sharpening: 0,
            shadows: true,
            shadow_resolution: 2,
            shadow_distance: 3,
            shadow_filter: 1,
            contact_shadows: false,
            bloom: 0,
            fog: true,
            exposure: 2,
            tonemapping: 0,
            deband: false,
            view_distance: 3,
            fov: 1,
            vsync: true,
            borderless: false,
            resolution: 0,
        }
    }
}
impl GraphicsSettings {
    pub(crate) fn validate(&self) -> Result<(), String> {
        for (name, value, count) in [
            ("antialiasing", self.antialiasing, 7),
            ("post_aa_quality", self.post_aa_quality, 4),
            ("ssao", self.ssao, 5),
            ("sharpening", self.sharpening, 4),
            ("shadow_resolution", self.shadow_resolution, 4),
            ("shadow_distance", self.shadow_distance, 4),
            ("shadow_filter", self.shadow_filter, 3),
            ("bloom", self.bloom, 4),
            ("exposure", self.exposure, 5),
            ("tonemapping", self.tonemapping, 9),
            ("view_distance", self.view_distance, 4),
            ("fov", self.fov, 5),
            ("resolution", self.resolution, 4),
        ] {
            if value >= count {
                return Err(format!(
                    "graphics.{name} index {value} is outside 0..{count}"
                ));
            }
        }
        if self.ssao > 0 && (1..=3).contains(&self.antialiasing) {
            return Err("SSAO is incompatible with MSAA; select TAA, FXAA, SMAA or no AA".into());
        }
        Ok(())
    }
    pub(crate) fn value(&self, row: usize) -> String {
        let toggle = |on| if on { "开启" } else { "关闭" }.to_string();
        match row {
            0 => AA[self.antialiasing as usize].into(),
            1 => ["低", "中", "高", "极高"][self.post_aa_quality as usize].into(),
            2 => ["关闭", "低", "中", "高", "极高"][self.ssao as usize].into(),
            3 => ["关闭", "30%", "60%", "100%"][self.sharpening as usize].into(),
            4 => toggle(self.shadows),
            5 => format!("{} px", SHADOW_SIZE[self.shadow_resolution as usize]),
            6 => format!("{:.0} m", SHADOW_DISTANCE[self.shadow_distance as usize]),
            7 => ["硬件 2×2", "高斯", "时域抖动"][self.shadow_filter as usize].into(),
            8 => toggle(self.contact_shadows),
            9 => ["关闭", "柔和", "标准", "较强"][self.bloom as usize].into(),
            10 => toggle(self.fog),
            11 => format!("{:+.1} EV", (self.exposure as f32 - 2.0) * 0.5),
            12 => TONE_LABELS[self.tonemapping as usize].into(),
            13 => toggle(self.deband),
            14 => format!("{:.0} m", FAR[self.view_distance as usize]),
            15 => format!("{:.0}°", FOV[self.fov as usize]),
            16 => toggle(self.vsync),
            17 => if self.borderless {
                "无边框全屏"
            } else {
                "窗口"
            }
            .into(),
            18 => {
                let (w, h) = RESOLUTIONS[self.resolution as usize];
                format!("{w} × {h}")
            }
            _ => String::new(),
        }
    }
    pub(crate) fn adjust(&mut self, row: usize, reverse: bool) {
        let cycle =
            |value: &mut u8, count| *value = (*value + if reverse { count - 1 } else { 1 }) % count;
        match row {
            0 => {
                cycle(&mut self.antialiasing, 7);
                if (1..=3).contains(&self.antialiasing) {
                    self.ssao = 0;
                }
            }
            1 => cycle(&mut self.post_aa_quality, 4),
            2 => {
                cycle(&mut self.ssao, 5);
                if self.ssao > 0 && (1..=3).contains(&self.antialiasing) {
                    self.antialiasing = 6;
                }
            }
            3 => cycle(&mut self.sharpening, 4),
            4 => self.shadows = !self.shadows,
            5 => cycle(&mut self.shadow_resolution, 4),
            6 => cycle(&mut self.shadow_distance, 4),
            7 => cycle(&mut self.shadow_filter, 3),
            8 => self.contact_shadows = !self.contact_shadows,
            9 => cycle(&mut self.bloom, 4),
            10 => self.fog = !self.fog,
            11 => cycle(&mut self.exposure, 5),
            12 => cycle(&mut self.tonemapping, 9),
            13 => self.deband = !self.deband,
            14 => cycle(&mut self.view_distance, 4),
            15 => cycle(&mut self.fov, 5),
            16 => self.vsync = !self.vsync,
            17 => self.borderless = !self.borderless,
            18 => cycle(&mut self.resolution, 4),
            _ => {}
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct DisplayTrial {
    previous: Option<(bool, u8)>,
    timer: Timer,
}

impl DisplayTrial {
    pub(crate) fn start(&mut self, previous: GraphicsSettings) {
        self.previous
            .get_or_insert((previous.borderless, previous.resolution));
        self.timer = Timer::from_seconds(15.0, TimerMode::Once);
    }

    pub(crate) fn pending(&self) -> bool {
        self.previous.is_some()
    }

    pub(crate) fn seconds(&self) -> u32 {
        if self.pending() {
            self.timer.remaining_secs().ceil() as u32
        } else {
            0
        }
    }

    pub(crate) fn confirm(&mut self) {
        self.previous = None;
    }

    pub(crate) fn revert(&mut self, settings: &mut GraphicsSettings) {
        if let Some((borderless, resolution)) = self.previous.take() {
            settings.borderless = borderless;
            settings.resolution = resolution;
        }
    }

    pub(crate) fn persisted(&self, mut settings: EntrySettings) -> EntrySettings {
        if let Some((borderless, resolution)) = self.previous {
            settings.graphics.borderless = borderless;
            settings.graphics.resolution = resolution;
        }
        settings
    }
}

pub(crate) fn expire_display_trial(
    time: Res<Time<Real>>,
    mut trial: ResMut<DisplayTrial>,
    mut settings: ResMut<EntrySettings>,
) {
    if trial.pending() && trial.timer.tick(time.delta()).is_finished() {
        trial.revert(&mut settings.graphics);
    }
}

// Skip unavailable samples in the same navigation direction; foreign saved settings use FXAA
fn supported_antialiasing(requested: u8, previous: Option<u8>, supported: [bool; 7]) -> u8 {
    if supported[requested as usize] {
        return requested;
    }
    let Some(previous) = previous else {
        return 4;
    };
    let step = if (previous + 6) % 7 == requested {
        6
    } else {
        1
    };
    let mut candidate = (requested + step) % 7;
    while !supported[candidate as usize] {
        candidate = (candidate + step) % 7;
    }
    candidate
}

#[derive(Resource)]
struct SceneVisual(DaylightSettings);
#[derive(Resource, Default)]
pub(crate) struct GraphicsNotice(pub String);
#[derive(Resource, Clone, Default, Serialize)]
pub(crate) struct GraphicsEvidence {
    pub aa: String,
    pub ssao: String,
    pub bloom: bool,
    pub fog: bool,
    pub contact_shadows: bool,
    pub shadows: bool,
    pub shadow_map_size: usize,
    pub far: f32,
    pub fov_degrees: f32,
    pub exposure_ev100: f32,
}
pub(crate) fn install(app: &mut App, visual: DaylightSettings) {
    app.insert_resource(SceneVisual(visual))
        .init_resource::<GraphicsNotice>()
        .init_resource::<GraphicsEvidence>()
        .add_systems(
            PostUpdate,
            (apply, evidence)
                .chain()
                .before(crate::capture::CaptureRecord),
        );
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(
            Render,
            clear_unused_background_motion_vectors
                .after(RenderSystems::Prepare)
                .before(RenderSystems::Render),
        );
    }
}

// Bevy 0.19.1 prepares these caches while MotionVectorPrepass exists, but does not remove them
// Its prepass node draws any cached pipeline, even after TAA is disabled and the motion attachment is gone
fn clear_unused_background_motion_vectors(
    mut commands: Commands,
    views: Query<
        Entity,
        (
            With<BackgroundMotionVectorsPipelineId>,
            Without<MotionVectorPrepass>,
        ),
    >,
) {
    for view in &views {
        commands.entity(view).remove::<(
            BackgroundMotionVectorsPipelineId,
            BackgroundMotionVectorsBindGroup,
        )>();
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Applies the same preferences to native window, camera and lighting"
)]
fn apply(
    mut commands: Commands,
    mut settings: ResMut<EntrySettings>,
    visual: Res<SceneVisual>,
    cameras: Query<(Entity, Ref<Camera3d>, &Projection)>,
    mut lights: Query<(Entity, &mut DirectionalLight)>,
    mut shadow_map: ResMut<DirectionalLightShadowMap>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    adapter: Option<Res<RenderAdapter>>,
    device: Option<Res<RenderDevice>>,
    mut previous: Local<Option<GraphicsSettings>>,
    mut notice: ResMut<GraphicsNotice>,
) {
    let newly_added = cameras.iter().any(|(_, camera, _)| camera.is_added());
    if *previous == Some(settings.graphics) && !newly_added {
        return;
    }
    notice.0.clear();
    if let Some(adapter) = adapter {
        let supported = [1, 2, 4, 8, 1, 1, 1].map(|samples| {
            [TextureFormat::Rgba16Float, TextureFormat::Depth32Float]
                .iter()
                .all(|format| {
                    adapter
                        .get_texture_format_features(*format)
                        .flags
                        .sample_count_supported(samples)
                })
        });
        let requested = settings.graphics.antialiasing;
        settings.graphics.antialiasing =
            supported_antialiasing(requested, previous.map(|p| p.antialiasing), supported);
        if requested != settings.graphics.antialiasing {
            notice.0 = format!(
                "当前显卡不支持 {}，已使用 {}",
                AA[requested as usize],
                settings.graphics.value(0)
            );
        }
    }
    if settings.graphics.ssao > 0
        && device
            .as_ref()
            .is_some_and(|device| device.limits().max_storage_textures_per_shader_stage < 5)
    {
        settings.graphics.ssao = 0;
        notice.0 = "当前显卡不支持 SSAO，已关闭".into();
    }
    let graphics = settings.graphics;
    let daylight = &visual.0;
    for (id, _, projection) in &cameras {
        let mut camera = commands.entity(id);
        camera.remove::<(
            Fxaa,
            Smaa,
            TemporalAntiAliasing,
            TemporalJitter,
            MipBias,
            DepthPrepass,
            MotionVectorPrepass,
            NormalPrepass,
            ScreenSpaceAmbientOcclusion,
            ContactShadows,
        )>();
        camera.insert(match graphics.antialiasing {
            1 => Msaa::Sample2,
            2 => Msaa::Sample4,
            3 => Msaa::Sample8,
            _ => Msaa::Off,
        });
        match graphics.antialiasing {
            4 => {
                let sensitivity = [
                    Sensitivity::Low,
                    Sensitivity::Medium,
                    Sensitivity::High,
                    Sensitivity::Ultra,
                ][graphics.post_aa_quality as usize];
                camera.insert(Fxaa {
                    enabled: true,
                    edge_threshold: sensitivity,
                    edge_threshold_min: sensitivity,
                });
            }
            5 => {
                camera.insert(Smaa {
                    preset: [
                        SmaaPreset::Low,
                        SmaaPreset::Medium,
                        SmaaPreset::High,
                        SmaaPreset::Ultra,
                    ][graphics.post_aa_quality as usize],
                });
            }
            6 => {
                camera.insert(TemporalAntiAliasing::default());
            }
            _ => {}
        }
        if graphics.ssao > 0 {
            camera.insert(ScreenSpaceAmbientOcclusion {
                quality_level: [
                    ScreenSpaceAmbientOcclusionQualityLevel::Low,
                    ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                    ScreenSpaceAmbientOcclusionQualityLevel::High,
                    ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
                ][graphics.ssao as usize - 1],
                ..default()
            });
        }
        if graphics.contact_shadows {
            camera.insert(ContactShadows {
                length: daylight.contact_length,
                thickness: daylight.contact_thickness,
                ..default()
            });
        }
        if graphics.bloom > 0 {
            camera.insert(Bloom {
                intensity: [0.0, 0.02, 0.04, 0.08][graphics.bloom as usize],
                ..Bloom::NATURAL
            });
        } else {
            camera.remove::<Bloom>();
        }
        if graphics.fog {
            camera.insert(DistanceFog {
                color: Color::srgb_from_array(daylight.sky_horizon),
                falloff: FogFalloff::Linear {
                    start: daylight.fog_start,
                    end: daylight.fog_end,
                },
                ..default()
            });
        } else {
            camera.remove::<DistanceFog>();
        }
        if graphics.sharpening > 0 {
            camera.insert(ContrastAdaptiveSharpening {
                sharpening_strength: [0.0, 0.3, 0.6, 1.0][graphics.sharpening as usize],
                ..default()
            });
        } else {
            camera.remove::<ContrastAdaptiveSharpening>();
        }
        camera.insert((
            Exposure {
                ev100: daylight.exposure_ev100 - (graphics.exposure as f32 - 2.0) * 0.5,
            },
            TONE[graphics.tonemapping as usize],
            if graphics.deband {
                DebandDither::Enabled
            } else {
                DebandDither::Disabled
            },
            [
                ShadowFilteringMethod::Hardware2x2,
                ShadowFilteringMethod::Gaussian,
                ShadowFilteringMethod::Temporal,
            ][graphics.shadow_filter as usize],
        ));
        if let Projection::Perspective(projection) = projection {
            let mut projection = projection.clone();
            projection.far = FAR[graphics.view_distance as usize];
            projection.fov = FOV[graphics.fov as usize].to_radians();
            camera.insert(Projection::Perspective(projection));
        }
    }
    shadow_map.size = SHADOW_SIZE[graphics.shadow_resolution as usize];
    for (id, mut light) in &mut lights {
        light.shadow_maps_enabled = graphics.shadows;
        light.contact_shadows_enabled = graphics.contact_shadows;
        commands.entity(id).insert(
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: daylight
                    .shadow_first_cascade
                    .min(SHADOW_DISTANCE[graphics.shadow_distance as usize] * 0.5),
                maximum_distance: SHADOW_DISTANCE[graphics.shadow_distance as usize],
                ..default()
            }
            .build(),
        );
    }
    for mut window in &mut windows {
        window.present_mode = if graphics.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        window.mode = if graphics.borderless {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        if !graphics.borderless
            && previous
                .is_none_or(|prior| prior.borderless || prior.resolution != graphics.resolution)
        {
            let (w, h) = RESOLUTIONS[graphics.resolution as usize];
            window.resolution.set_physical_resolution(w, h);
        }
    }
    *previous = Some(graphics);
}
fn evidence(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<Camera3d>>();
    let Some(id) = query.iter(world).next() else {
        return;
    };
    let camera = world.entity(id);
    let aa = if camera.contains::<TemporalAntiAliasing>() {
        "taa".into()
    } else if camera.contains::<Fxaa>() {
        "fxaa".into()
    } else if camera.contains::<Smaa>() {
        "smaa".into()
    } else {
        match camera.get::<Msaa>().copied().unwrap_or(Msaa::Off) {
            Msaa::Off => "off",
            Msaa::Sample2 => "msaa2",
            Msaa::Sample4 => "msaa4",
            Msaa::Sample8 => "msaa8",
        }
        .into()
    };
    let ssao = camera
        .get::<ScreenSpaceAmbientOcclusion>()
        .map_or("off", |ssao| match ssao.quality_level {
            ScreenSpaceAmbientOcclusionQualityLevel::Low => "low",
            ScreenSpaceAmbientOcclusionQualityLevel::Medium => "medium",
            ScreenSpaceAmbientOcclusionQualityLevel::High => "high",
            _ => "ultra",
        })
        .into();
    let (far, fov_degrees) = match camera.get::<Projection>() {
        Some(Projection::Perspective(p)) => (p.far, p.fov.to_degrees()),
        _ => (0.0, 0.0),
    };
    let mut result = GraphicsEvidence {
        aa,
        ssao,
        bloom: camera.contains::<Bloom>(),
        fog: camera.contains::<DistanceFog>(),
        contact_shadows: camera.contains::<ContactShadows>(),
        shadows: false,
        shadow_map_size: world.resource::<DirectionalLightShadowMap>().size,
        far,
        fov_degrees,
        exposure_ev100: camera.get::<Exposure>().map_or(0.0, |e| e.ev100),
    };
    result.shadows = world
        .query::<&DirectionalLight>()
        .iter(world)
        .any(|l| l.shadow_maps_enabled);
    *world.resource_mut::<GraphicsEvidence>() = result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_trial_uses_real_time_and_only_reverts_display_choices() {
        let mut app = App::new();
        app.init_resource::<Time<Real>>()
            .init_resource::<Time<Virtual>>()
            .init_resource::<EntrySettings>()
            .init_resource::<DisplayTrial>()
            .add_systems(Update, expire_display_trial);
        app.world_mut().resource_mut::<Time<Virtual>>().pause();
        let previous = app.world().resource::<EntrySettings>().graphics;
        app.world_mut()
            .resource_mut::<DisplayTrial>()
            .start(previous);
        {
            let mut settings = app.world_mut().resource_mut::<EntrySettings>();
            settings.graphics.borderless = true;
            settings.graphics.resolution = 2;
            settings.graphics.vsync = false;
            settings.large_text = true;
        }
        assert_eq!(app.world().resource::<DisplayTrial>().seconds(), 15);
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(14_100));
        app.update();
        assert_eq!(app.world().resource::<DisplayTrial>().seconds(), 1);
        assert!(app.world().resource::<EntrySettings>().graphics.borderless);
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(900));
        app.update();
        assert!(!app.world().resource::<DisplayTrial>().pending());
        let settings = app.world().resource::<EntrySettings>();
        assert_eq!(settings.graphics.borderless, previous.borderless);
        assert_eq!(settings.graphics.resolution, previous.resolution);
        assert!(!settings.graphics.vsync && settings.large_text);
    }

    #[test]
    fn leaving_motion_prepass_clears_only_obsolete_render_cache() {
        use bevy::render::render_resource::CachedRenderPipelineId;
        let mut app = App::new();
        app.add_systems(Update, clear_unused_background_motion_vectors);
        let camera = app
            .world_mut()
            .spawn((
                MotionVectorPrepass,
                BackgroundMotionVectorsPipelineId(CachedRenderPipelineId::INVALID),
            ))
            .id();
        app.update();
        assert!(
            app.world()
                .entity(camera)
                .contains::<BackgroundMotionVectorsPipelineId>()
        );
        app.world_mut()
            .entity_mut(camera)
            .remove::<MotionVectorPrepass>();
        app.update();
        assert!(
            !app.world()
                .entity(camera)
                .contains::<BackgroundMotionVectorsPipelineId>()
        );
        app.world_mut().entity_mut(camera).insert((
            MotionVectorPrepass,
            BackgroundMotionVectorsPipelineId(CachedRenderPipelineId::INVALID),
        ));
        app.update();
        assert!(
            app.world()
                .entity(camera)
                .contains::<BackgroundMotionVectorsPipelineId>()
        );
    }

    #[test]
    fn unsupported_msaa_does_not_trap_navigation() {
        let supported = [true, false, true, false, true, true, true];
        assert_eq!(supported_antialiasing(1, Some(0), supported), 2);
        assert_eq!(supported_antialiasing(3, Some(2), supported), 4);
        assert_eq!(supported_antialiasing(3, Some(4), supported), 2);
        assert_eq!(supported_antialiasing(1, Some(2), supported), 0);
        assert_eq!(supported_antialiasing(3, None, supported), 4);
    }

    #[test]
    fn choices_wrap_and_ssao_msaa_never_coexist() {
        for row in 0..LABELS.len() {
            let mut settings = GraphicsSettings::default();
            for _ in 0..20 {
                settings.adjust(row, false);
                settings.validate().unwrap();
            }
            for _ in 0..20 {
                settings.adjust(row, true);
                settings.validate().unwrap();
            }
        }
        let mut settings = GraphicsSettings::default();
        settings.adjust(2, false);
        assert_eq!(settings.antialiasing, 6);
        assert_eq!(settings.ssao, 1);
        settings.antialiasing = 1;
        assert!(settings.validate().is_err());
        settings.adjust(0, false);
        assert_eq!(settings.ssao, 0);
        settings.antialiasing = 255;
        assert!(settings.validate().is_err());
    }
    #[test]
    fn actual_camera_switch_removes_temporal_state_and_restores_shared_prepass() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<EntrySettings>()
            .insert_resource(DirectionalLightShadowMap::default());
        install(
            &mut app,
            DaylightSettings::load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../source-assets/district-scene/daylight.json"),
            )
            .unwrap(),
        );
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Projection::default()))
            .id();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.world_mut().spawn(DirectionalLight::default());
        {
            let mut settings = app.world_mut().resource_mut::<EntrySettings>();
            settings.graphics.adjust(2, false);
            settings.graphics.contact_shadows = true;
            settings.graphics.bloom = 1;
            settings.graphics.vsync = false;
            settings.graphics.resolution = 2;
        }
        app.update();
        let display = app.world().get::<Window>(window).unwrap();
        assert_eq!(display.present_mode, PresentMode::AutoNoVsync);
        assert_eq!(display.resolution.physical_size(), UVec2::new(1920, 1080));
        assert!(app.world().entity(camera).contains::<TemporalJitter>());
        assert!(app.world().entity(camera).contains::<NormalPrepass>());
        assert_eq!(app.world().resource::<GraphicsEvidence>().aa, "taa");
        app.world_mut()
            .resource_mut::<EntrySettings>()
            .graphics
            .antialiasing = 0;
        app.update();
        assert_eq!(app.world().resource::<GraphicsEvidence>().ssao, "low");
        assert!(app.world().entity(camera).contains::<NormalPrepass>());
        assert!(!app.world().entity(camera).contains::<MotionVectorPrepass>());
        assert!(!app.world().entity(camera).contains::<TemporalJitter>());
        app.world_mut().resource_mut::<EntrySettings>().graphics = GraphicsSettings {
            contact_shadows: true,
            ..default()
        };
        app.update();
        let camera = app.world().entity(camera);
        assert!(!camera.contains::<TemporalJitter>());
        assert!(!camera.contains::<NormalPrepass>());
        assert!(!camera.contains::<Bloom>());
        assert!(camera.contains::<DepthPrepass>());
        assert_eq!(app.world().resource::<GraphicsEvidence>().aa, "msaa4");
        let display = app.world().get::<Window>(window).unwrap();
        assert_eq!(display.present_mode, PresentMode::AutoVsync);
        assert_eq!(display.resolution.physical_size(), UVec2::new(1280, 720));
    }
}
