//! Opt-in grey studies on the real controller; these are not accepted character designs
use crate::player::{self, PlayerBody, PlayerPresentation, PlayerProxy, PlayerState};
use bevy::{
    animation::{AnimatedBy, AnimationTargetId},
    app::AnimationSystems,
    asset::{LoadState, RecursiveDependencyLoadState},
    gltf::Gltf,
    material::OpaqueRendererMethod,
    mesh::skinning::SkinnedMesh,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    world_serialization::WorldInstanceReady,
};
use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterPreview {
    Yao,
    Ling,
}

impl CharacterPreview {
    pub fn parse(id: &str) -> Result<Self, String> {
        match id {
            "CHR-001" => Ok(Self::Yao),
            "CHR-002" => Ok(Self::Ling),
            _ => Err(format!(
                "unknown character preview {id:?}; use CHR-001 or CHR-002"
            )),
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Yao => "CHR-001",
            Self::Ling => "CHR-002",
        }
    }

    fn model(self) -> &'static str {
        match self {
            Self::Yao => "characters/CHR-001/yao-grey-study.glb",
            Self::Ling => "characters/CHR-002/ling-grey-study.glb",
        }
    }

    fn stride_scale(self) -> f32 {
        match self {
            Self::Yao => 1.0,
            // The shared in-place rig and translation curves were scaled with Ling's body
            Self::Ling => 1.65 / 1.744_190_6,
        }
    }
}

const SHADER: &str = "shaders/character-ink.wgsl";
const CLIPS: [&str; 3] = ["Idle", "Walk", "Run"];

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct CharacterInk {
    /// Main-light threshold, shadow cosine, lit cosine, minimum edge width
    #[uniform(100)]
    bands: Vec4,
}

impl MaterialExtension for CharacterInk {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

type InkMaterial = ExtendedMaterial<StandardMaterial, CharacterInk>;

fn ink_material(base: &StandardMaterial) -> InkMaterial {
    InkMaterial {
        base: StandardMaterial {
            // This fragment extension requires forward lighting; the standard shadow/prepass stays intact
            opaque_render_method: OpaqueRendererMethod::Forward,
            ..base.clone()
        },
        extension: CharacterInk {
            bands: Vec4::new(0.2, 0.2, 0.82, 0.008),
        },
    }
}

/// Capture reads actual asset/animation state, without influencing playback
#[derive(Resource, Default, Clone, Serialize)]
pub struct CharacterStatus {
    pub id: Option<&'static str>,
    pub ready: bool,
    pub error: Option<String>,
    pub clip: Option<&'static str>,
    pub clip_time: f32,
    pub clip_elapsed: f32,
    pub transitions: u32,
    pub paused: bool,
    pub skinned_meshes: usize,
    /// Scene meshes assigned the ink material; GPU pipeline success requires runtime evidence
    pub shaded_meshes: usize,
    pub animated_targets: usize,
}

#[derive(Resource)]
struct CharacterLoad {
    gltf: Handle<Gltf>,
    shader: Handle<Shader>,
    started: Instant,
    spawned: bool,
}

#[derive(Component)]
struct CharacterScene {
    graph: Handle<AnimationGraph>,
    nodes: [AnimationNodeIndex; 3],
    clips: [Handle<AnimationClip>; 3],
}

#[derive(Component)]
struct CharacterAnimation {
    nodes: [AnimationNodeIndex; 3],
    current: usize,
}

pub fn install(app: &mut App, preview: Option<CharacterPreview>) {
    let Some(preview) = preview else { return };
    app.add_plugins(MaterialPlugin::<InkMaterial>::default())
        .insert_resource(preview)
        .init_resource::<CharacterStatus>()
        .add_systems(
            Update,
            (
                begin_loading,
                poll_loading,
                player::gameplay_active.pipe(animate),
            )
                .chain()
                .after(PlayerPresentation),
        )
        .add_systems(
            PostUpdate,
            record_animation
                .after(AnimationSystems)
                .before(crate::capture::CaptureRecord),
        );
}

fn begin_loading(
    mut commands: Commands,
    assets: Res<AssetServer>,
    bodies: Query<(), With<PlayerBody>>,
    loading: Option<Res<CharacterLoad>>,
    mut status: ResMut<CharacterStatus>,
    preview: Res<CharacterPreview>,
) {
    if bodies.is_empty() {
        if loading.is_some() {
            commands.remove_resource::<CharacterLoad>();
            *status = CharacterStatus::default();
        }
    } else if loading.is_none() {
        *status = CharacterStatus::default();
        status.id = Some(preview.id());
        commands.insert_resource(CharacterLoad {
            gltf: assets.load(preview.model()),
            shader: assets.load(SHADER),
            started: Instant::now(),
            spawned: false,
        });
        info!("[character/load] {}", preview.model());
    }
}

fn fail(status: &mut CharacterStatus, message: impl Into<String>) {
    let message = message.into();
    error!(
        "[character/load] {:?}: {message}; retaining neutral proxy",
        status.id
    );
    status.ready = false;
    status.error = Some(message);
}

fn poll_loading(
    mut commands: Commands,
    assets: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    loading: Option<ResMut<CharacterLoad>>,
    bodies: Query<Entity, With<PlayerBody>>,
    mut status: ResMut<CharacterStatus>,
) {
    let (Some(mut loading), Ok(body)) = (loading, bodies.single()) else {
        return;
    };
    if status.ready || status.error.is_some() {
        return;
    }
    if let LoadState::Failed(error) = assets.load_state(&loading.gltf) {
        fail(&mut status, error.to_string());
        return;
    }
    if let LoadState::Failed(error) = assets.load_state(&loading.shader) {
        fail(&mut status, format!("character shader failed: {error}"));
        return;
    }
    if let RecursiveDependencyLoadState::Failed(error) =
        assets.recursive_dependency_load_state(&loading.gltf)
    {
        fail(&mut status, format!("dependency failed: {error}"));
        return;
    }
    if loading.started.elapsed() > Duration::from_secs(30) {
        fail(
            &mut status,
            "30s deadline exceeded while loading dependencies or spawning the scene",
        );
        return;
    }
    if loading.spawned
        || !assets.is_loaded_with_dependencies(&loading.gltf)
        || !assets.is_loaded_with_dependencies(&loading.shader)
    {
        return;
    }
    let Some(gltf) = gltfs.get(&loading.gltf) else {
        return;
    };
    let Some(scene) = gltf.named_scenes.get("Scene") else {
        fail(&mut status, "required named scene 'Scene' is missing");
        return;
    };
    let clips: Result<Vec<_>, _> = CLIPS
        .iter()
        .map(|name| {
            gltf.named_animations
                .get(*name)
                .cloned()
                .ok_or_else(|| format!("required named clip '{name}' is missing"))
        })
        .collect();
    let clips: [Handle<AnimationClip>; 3] = match clips {
        Ok(clips) => clips.try_into().expect("three required clip names"),
        Err(error) => {
            fail(&mut status, error);
            return;
        }
    };
    let (graph, nodes) = AnimationGraph::from_clips(clips.iter().cloned());
    commands
        .spawn((
            Name::new(format!("{} grey study (unaccepted)", status.id.unwrap())),
            WorldAssetRoot(scene.clone()),
            Transform::default(),
            Visibility::Hidden,
            ChildOf(body),
            CharacterScene {
                graph: graphs.add(graph),
                nodes: nodes.try_into().expect("one graph node per required clip"),
                clips,
            },
        ))
        .observe(scene_ready);
    loading.spawned = true;
}

#[expect(
    clippy::too_many_arguments,
    reason = "Validate the actual glTF scene before hiding its fallback"
)]
fn scene_ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    scenes: Query<&CharacterScene>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    skins: Query<&SkinnedMesh>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    materials: Res<Assets<StandardMaterial>>,
    mut ink_materials: ResMut<Assets<InkMaterial>>,
    targets: Query<(&AnimationTargetId, &AnimatedBy)>,
    clips: Res<Assets<AnimationClip>>,
    proxies: Query<Entity, With<PlayerProxy>>,
    mut status: ResMut<CharacterStatus>,
) {
    let Ok(scene) = scenes.get(event.entity) else {
        return;
    };
    if status.error.is_some() {
        return;
    }
    let descendants: Vec<_> = children.iter_descendants(event.entity).collect();
    let animation_players: Vec<_> = descendants
        .iter()
        .copied()
        .filter(|entity| players.contains(*entity))
        .collect();
    let skin_count = descendants
        .iter()
        .filter(|entity| skins.contains(**entity))
        .count();
    if animation_players.len() != 1 || skin_count == 0 {
        fail(
            &mut status,
            format!(
                "expected one animation player and a skinned mesh; got {} players / {skin_count} skins",
                animation_players.len()
            ),
        );
        return;
    }
    let actor = animation_players[0];
    let target_ids: std::collections::HashSet<_> = descendants
        .iter()
        .filter_map(|entity| targets.get(*entity).ok())
        .filter(|(_, owner)| owner.0 == actor)
        .map(|(target, _)| *target)
        .collect();
    for (name, handle) in CLIPS.iter().zip(&scene.clips) {
        let Some(clip) = clips.get(handle) else {
            fail(
                &mut status,
                format!("clip '{name}' unavailable after dependency load"),
            );
            return;
        };
        if clip.duration() <= 0.0
            || clip.curves().is_empty()
            || clip.curves().keys().any(|id| !target_ids.contains(id))
        {
            fail(
                &mut status,
                format!("clip '{name}' has no duration/curves or unresolved scene targets"),
            );
            return;
        }
    }
    let mut shaded = Vec::new();
    for entity in descendants.iter().copied().filter(|e| skins.contains(*e)) {
        let Some(base) = mesh_materials
            .get(entity)
            .ok()
            .and_then(|handle| materials.get(&handle.0))
        else {
            fail(&mut status, "skinned mesh has no loaded standard material");
            return;
        };
        shaded.push((entity, ink_material(base)));
    }
    for (entity, material) in shaded {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(ink_materials.add(material)));
    }
    players
        .get_mut(actor)
        .unwrap()
        .play(scene.nodes[0])
        .repeat();
    commands.entity(actor).insert((
        AnimationGraphHandle(scene.graph.clone()),
        CharacterAnimation {
            nodes: scene.nodes,
            current: 0,
        },
    ));
    commands.entity(event.entity).insert(Visibility::Inherited);
    for proxy in &proxies {
        commands.entity(proxy).insert(Visibility::Hidden);
    }
    status.ready = true;
    status.clip = Some(CLIPS[0]);
    status.skinned_meshes = skin_count;
    status.shaded_meshes = skin_count;
    status.animated_targets = target_ids.len();
    info!(
        "[character/ready] {:?}: {skin_count} skins with ink material, {} targets, clips {:?}",
        status.id,
        target_ids.len(),
        CLIPS
    );
}

fn desired_clip(speed: f32, grounded: bool) -> usize {
    if !grounded || speed < 0.04 {
        0
    } else if speed > 4.2 {
        2
    } else {
        1
    }
}

fn animate(
    In(active): In<bool>,
    state: Option<Res<PlayerState>>,
    preview: Res<CharacterPreview>,
    mut players: Query<(&mut AnimationPlayer, &mut CharacterAnimation)>,
    mut status: ResMut<CharacterStatus>,
) {
    let Some(state) = state else { return };
    for (mut player, mut animation) in &mut players {
        status.paused = !active;
        if !active {
            player.pause_all();
            continue;
        }
        player.resume_all();
        let speed = state.velocity.xz().length();
        let desired = desired_clip(speed, state.grounded);
        if desired != animation.current {
            // ponytail: hard clip switches for the grey study; blend once authored transitions exist
            player.stop_all().play(animation.nodes[desired]).repeat();
            animation.current = desired;
            status.transitions += 1;
        }
        if let Some(playing) = player.animation_mut(animation.nodes[desired]) {
            playing.set_speed(match desired {
                1 => speed / (3.2 * preview.stride_scale()),
                2 => speed / (5.6 * preview.stride_scale()),
                _ => 1.0,
            });
        }
    }
}

fn record_animation(
    players: Query<(&AnimationPlayer, &CharacterAnimation)>,
    mut status: ResMut<CharacterStatus>,
) {
    for (player, animation) in &players {
        if let Some(playing) = player.animation(animation.nodes[animation.current]) {
            status.clip = Some(CLIPS[animation.current]);
            status.clip_time = playing.seek_time();
            status.clip_elapsed = playing.elapsed();
            status.paused = playing.is_paused();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ink_keeps_gltf_texture_and_geometry_contract() {
        let mut images = Assets::<Image>::default();
        let texture = images.add(Image::default());
        let base = StandardMaterial {
            base_color_texture: Some(texture.clone()),
            metallic_roughness_texture: Some(texture.clone()),
            normal_map_texture: Some(texture.clone()),
            perceptual_roughness: 0.73,
            reflectance: 0.21,
            base_color: Color::srgb(0.6, 0.6, 0.6),
            double_sided: true,
            ..default()
        };
        let ink = ink_material(&base);
        assert_eq!(ink.base.base_color_texture, Some(texture.clone()));
        assert_eq!(ink.base.metallic_roughness_texture, Some(texture.clone()));
        assert_eq!(ink.base.normal_map_texture, Some(texture));
        assert_eq!(ink.base.perceptual_roughness, base.perceptual_roughness);
        assert_eq!(ink.base.reflectance, base.reflectance);
        assert_eq!(ink.base.base_color, base.base_color);
        assert_eq!(ink.base.alpha_mode, base.alpha_mode);
        assert_eq!(ink.base.double_sided, base.double_sided);
        assert!(!ink.base.unlit);
        assert_eq!(ink.base.opaque_render_method, OpaqueRendererMethod::Forward);
        assert!(matches!(CharacterInk::vertex_shader(), ShaderRef::Default));
        assert!(CharacterInk::enable_shadows());
        assert!(CharacterInk::enable_prepass());
        assert!(matches!(
            CharacterInk::prepass_vertex_shader(),
            ShaderRef::Default
        ));
    }

    #[test]
    fn character_selection_rejects_unknown_ids_and_adjusts_smaller_stride() {
        assert_eq!(
            CharacterPreview::parse("CHR-001"),
            Ok(CharacterPreview::Yao)
        );
        assert_eq!(
            CharacterPreview::parse("CHR-002"),
            Ok(CharacterPreview::Ling)
        );
        assert!(CharacterPreview::parse("../other.glb").is_err());
        assert!(CharacterPreview::parse("").is_err());
        assert_eq!(CharacterPreview::Yao.stride_scale(), 1.0);
        assert!((CharacterPreview::Ling.stride_scale() - 0.946).abs() < 0.001);
    }

    #[test]
    fn resolved_motion_selects_clips_and_airborne_does_not_fake_a_jump_clip() {
        assert_eq!(desired_clip(0.0, true), 0);
        assert_eq!(desired_clip(0.039, true), 0);
        assert_eq!(desired_clip(1.0, true), 1);
        assert_eq!(desired_clip(3.2, true), 1);
        assert_eq!(desired_clip(5.6, true), 2);
        assert_eq!(desired_clip(5.6, false), 0);
    }
}
