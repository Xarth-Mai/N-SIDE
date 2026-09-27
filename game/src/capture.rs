//! Scripted input and evidence for the real game and Map Viewer, using Bevy's screenshot API
//! Workflow adapted from htdt/godogen engines/bevy.md at 0b725bca053769a4727f76c332bf1f7b42e146ab
//! Copyright 2026 Alex Ermolov, MIT; retained license: third_party/skills/godogen/LICENSE.md
//! N:SIDE implementation records real application input, state and rendered frames
use crate::{
    app::{GameLoadError, GamePhase},
    ui::{SignalUi, UiFont, UiInput},
    world::scene::{MapSource, SceneLoading},
};
#[cfg(feature = "viewer")]
use bevy::camera_controller::free_camera::FreeCameraState;
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::{TimeSystems, TimeUpdateStrategy},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Resource, Default)]
pub struct CaptureTarget(pub Option<Handle<Image>>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaptureInput;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Script {
    pub scene: String,
    pub view: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frames: u32,
    pub seed: u64,
    pub timeout_seconds: u64,
    pub keyframes: Vec<u32>,
    pub events: Vec<InputSpan>,
    #[serde(default)]
    pub waits: Vec<Wait>,
    pub assertions: Vec<Assertion>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Wait {
    frame: u32,
    game_page: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputSpan {
    start: u32,
    end: u32,
    #[serde(default)]
    keys: Vec<String>,
    #[serde(default)]
    right_mouse: bool,
    #[serde(default)]
    gamepad: Vec<String>,
    #[serde(default)]
    look: [f32; 2],
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    name: String,
    from: u32,
    to: u32,
    min_distance: Option<f32>,
    max_distance: Option<f32>,
    min_rotation: Option<f32>,
    enabled: Option<bool>,
    ui_page: Option<String>,
    ui_focus: Option<usize>,
    ui_scale: Option<f32>,
    ui_device: Option<String>,
    ui_reduced_motion: Option<bool>,
    ui_record: Option<usize>,
    min_ui_scroll: Option<f32>,
    max_ui_scroll: Option<f32>,
    game_page: Option<String>,
    world_ready: Option<bool>,
    min_world_entities: Option<usize>,
    max_world_entities: Option<usize>,
    same_world_entities: Option<bool>,
}

const GAME_PAGES: [&str; 4] = ["title", "loading", "world", "failed"];

const KEYS: [(&str, KeyCode); 15] = [
    ("W", KeyCode::KeyW),
    ("A", KeyCode::KeyA),
    ("S", KeyCode::KeyS),
    ("D", KeyCode::KeyD),
    ("Q", KeyCode::KeyQ),
    ("E", KeyCode::KeyE),
    ("Shift", KeyCode::ShiftLeft),
    ("M", KeyCode::KeyM),
    ("Escape", KeyCode::Escape),
    ("Tab", KeyCode::Tab),
    ("Enter", KeyCode::Enter),
    ("Down", KeyCode::ArrowDown),
    ("Up", KeyCode::ArrowUp),
    ("PageDown", KeyCode::PageDown),
    ("PageUp", KeyCode::PageUp),
];

const PAD: [(&str, GamepadButton); 7] = [
    ("Down", GamepadButton::DPadDown),
    ("Up", GamepadButton::DPadUp),
    ("Confirm", GamepadButton::South),
    ("Back", GamepadButton::East),
    ("Menu", GamepadButton::Start),
    ("ScrollDown", GamepadButton::RightTrigger),
    ("ScrollUp", GamepadButton::LeftTrigger),
];
#[derive(Component)]
struct ScriptGamepad;

impl Script {
    fn validate(&self) -> Result<(), String> {
        if !matches!(self.scene.as_str(), "district" | "ui-signal" | "game-entry")
            || self.view.is_empty()
            || (self.scene == "game-entry" && self.view != "shop")
        {
            return Err(
                "scene must be district/ui-signal with an existing Viewer view, or game-entry with view shop".into(),
            );
        }
        if !(64..=4096).contains(&self.width)
            || !(64..=4096).contains(&self.height)
            || !self.width.is_multiple_of(2)
            || !self.height.is_multiple_of(2)
            || !(1..=120).contains(&self.fps)
            || !(2..=36000).contains(&self.frames)
            || !(1..=3600).contains(&self.timeout_seconds)
        {
            return Err("invalid dimensions (even, 64..4096), fps (1..120), frames (2..36000), or timeout (1..3600)".into());
        }
        let mut previous_end = 0;
        for event in &self.events {
            if event.start < previous_end
                || event.start >= event.end
                || event.end > self.frames
                || event.look.iter().any(|v| !v.is_finite())
                || event
                    .keys
                    .iter()
                    .any(|key| !KEYS.iter().any(|(name, _)| name == key))
                || event
                    .gamepad
                    .iter()
                    .any(|button| !PAD.iter().any(|(name, _)| name == button))
            {
                return Err("input spans must be ordered, non-overlapping, within frames and use known keys/finite mouse deltas".into());
            }
            previous_end = event.end;
        }
        let mut wait_frames = std::collections::BTreeSet::new();
        for wait in &self.waits {
            if self.scene != "game-entry"
                || wait.frame >= self.frames
                || !wait_frames.insert(wait.frame)
                || !GAME_PAGES.contains(&wait.game_page.as_str())
            {
                return Err(
                    "waits require game-entry, unique frames within frames and a known game_page"
                        .into(),
                );
            }
        }
        if self.assertions.is_empty() || self.keyframes.iter().any(|f| *f >= self.frames) {
            return Err(
                "at least one assertion is required; keyframes must be within frames".into(),
            );
        }
        for check in &self.assertions {
            if check.name.trim().is_empty()
                || check.from > check.to
                || check.to >= self.frames
                || [check.min_distance, check.max_distance, check.min_rotation]
                    .into_iter()
                    .flatten()
                    .any(|v| !v.is_finite() || v < 0.0)
                || (check.min_distance.is_none()
                    && check.max_distance.is_none()
                    && check.min_rotation.is_none()
                    && check.enabled.is_none()
                    && check.ui_page.is_none()
                    && check.ui_focus.is_none()
                    && check.ui_scale.is_none()
                    && check.ui_device.is_none()
                    && check.ui_reduced_motion.is_none()
                    && check.ui_record.is_none()
                    && check.min_ui_scroll.is_none()
                    && check.max_ui_scroll.is_none()
                    && check.game_page.is_none()
                    && check.world_ready.is_none()
                    && check.min_world_entities.is_none()
                    && check.max_world_entities.is_none()
                    && check.same_world_entities.is_none())
                || check.game_page.as_ref().is_some_and(|page| {
                    self.scene != "game-entry" || !GAME_PAGES.contains(&page.as_str())
                })
                || check
                    .min_world_entities
                    .zip(check.max_world_entities)
                    .is_some_and(|(min, max)| min > max)
                || check.ui_page.as_ref().is_some_and(|page| {
                    !["world", "menu", "records", "settings", "district"].contains(&page.as_str())
                })
                || check
                    .ui_device
                    .as_ref()
                    .is_some_and(|device| !["keyboard_mouse", "gamepad"].contains(&device.as_str()))
                || check
                    .ui_scale
                    .is_some_and(|scale| ![1.0, 1.25].contains(&scale))
                || [check.min_ui_scroll, check.max_ui_scroll]
                    .into_iter()
                    .flatten()
                    .any(|scroll| !scroll.is_finite() || scroll < 0.0)
            {
                return Err("assertions require a name, valid frame interval and finite nonnegative limits or enabled expectation".into());
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct Sample {
    frame: u32,
    simulation_seconds: f64,
    position: [f32; 3],
    rotation: [f32; 4],
    enabled: bool,
    game_page: Option<String>,
    world_ready: bool,
    world_entities: usize,
    ui: Option<serde_json::Value>,
}

#[derive(Resource)]
pub struct Recording {
    pub script: Script,
    output: PathBuf,
    started: Instant,
    warmup: u32,
    tick: bool,
    pending: bool,
    saved: u32,
    transforms_checked: u32,
    world_ready_seen: bool,
    samples: Vec<Sample>,
    failure: Option<String>,
    finished: bool,
}

impl Recording {
    pub fn load(path: &Path, output: PathBuf) -> Result<Self, String> {
        let bytes =
            fs::read(path).map_err(|e| format!("[capture/script] {}: {e}", path.display()))?;
        let script: Script = serde_json::from_slice(&bytes)
            .map_err(|e| format!("[capture/script] {}: {e}", path.display()))?;
        script
            .validate()
            .map_err(|e| format!("[capture/script] {e}"))?;
        if output.join("frames").exists() || output.join("state.json").exists() {
            return Err(
                "[capture/output] use a fresh directory; frames/state.json already exist".into(),
            );
        }
        fs::create_dir_all(output.join("frames")).map_err(|e| format!("[capture/output] {e}"))?;
        fs::create_dir_all(output.join("keyframes"))
            .map_err(|e| format!("[capture/output] {e}"))?;
        Ok(Self {
            script,
            output,
            started: Instant::now(),
            warmup: 30,
            tick: false,
            pending: false,
            saved: 0,
            transforms_checked: 0,
            world_ready_seen: false,
            samples: vec![],
            failure: None,
            finished: false,
        })
    }

    fn finish(&mut self, loading: Option<&SceneLoading>) -> bool {
        let assets_ready = if self.script.scene == "game-entry" {
            self.world_ready_seen
        } else {
            loading.is_some_and(|loading| loading.ready && loading.failure.is_none())
        };
        let mut checks = vec![
            serde_json::json!({"name":"required_assets_ready", "passed":assets_ready}),
            serde_json::json!({"name":"all_frames_saved", "passed":self.saved == self.script.frames, "saved":self.saved, "expected":self.script.frames}),
            serde_json::json!({"name":"finite_transforms", "passed":self.transforms_checked == self.script.frames, "checked_frames":self.transforms_checked}),
        ];
        for assertion in &self.script.assertions {
            let measured = self
                .samples
                .get(assertion.from as usize)
                .zip(self.samples.get(assertion.to as usize));
            let (distance, rotation, enabled, passed) =
                measured.map_or((None, None, None, false), |(a, b)| {
                    let distance =
                        Vec3::from_array(a.position).distance(Vec3::from_array(b.position));
                    let rotation =
                        Quat::from_array(a.rotation).angle_between(Quat::from_array(b.rotation));
                    let peak_distance = self.samples
                        [assertion.from as usize..=assertion.to as usize]
                        .iter()
                        .map(|sample| {
                            Vec3::from_array(a.position).distance(Vec3::from_array(sample.position))
                        })
                        .fold(0.0_f32, f32::max);
                    let passed = assertion.min_distance.is_none_or(|v| distance >= v)
                        && assertion.max_distance.is_none_or(|v| peak_distance <= v)
                        && assertion.min_rotation.is_none_or(|v| rotation >= v)
                        && assertion.enabled.is_none_or(|v| b.enabled == v)
                        && assertion
                            .game_page
                            .as_ref()
                            .is_none_or(|v| b.game_page.as_ref() == Some(v))
                        && assertion.world_ready.is_none_or(|v| b.world_ready == v)
                        && assertion
                            .min_world_entities
                            .is_none_or(|v| b.world_entities >= v)
                        && assertion
                            .max_world_entities
                            .is_none_or(|v| b.world_entities <= v)
                        && assertion
                            .same_world_entities
                            .is_none_or(|v| (a.world_entities == b.world_entities) == v)
                        && assertion
                            .ui_page
                            .as_ref()
                            .is_none_or(|v| b.ui.as_ref().is_some_and(|ui| ui["page"] == *v))
                        && assertion
                            .ui_focus
                            .is_none_or(|v| b.ui.as_ref().is_some_and(|ui| ui["focus"] == v))
                        && assertion
                            .ui_scale
                            .is_none_or(|v| b.ui.as_ref().is_some_and(|ui| ui["scale"] == v))
                        && assertion
                            .ui_device
                            .as_ref()
                            .is_none_or(|v| b.ui.as_ref().is_some_and(|ui| ui["device"] == *v))
                        && assertion.ui_reduced_motion.is_none_or(|v| {
                            b.ui.as_ref().is_some_and(|ui| ui["reduced_motion"] == v)
                        })
                        && assertion.ui_record.is_none_or(|v| {
                            b.ui.as_ref().is_some_and(|ui| ui["selected_record"] == v)
                        })
                        && assertion.min_ui_scroll.is_none_or(|v| {
                            b.ui.as_ref().is_some_and(|ui| {
                                ui["scroll"]
                                    .as_f64()
                                    .is_some_and(|scroll| scroll >= v as f64)
                            })
                        })
                        && assertion.max_ui_scroll.is_none_or(|v| {
                            b.ui.as_ref().is_some_and(|ui| {
                                ui["scroll"]
                                    .as_f64()
                                    .is_some_and(|scroll| scroll <= v as f64)
                            })
                        });
                    (Some(distance), Some(rotation), Some(b.enabled), passed)
                });
            checks.push(serde_json::json!({"name":assertion.name, "passed":passed, "expected":assertion, "distance":distance, "rotation_radians":rotation, "enabled":enabled}));
        }
        let passed = self.failure.is_none() && checks.iter().all(|c| c["passed"] == true);
        let report = serde_json::json!({
            "status":if passed {"PASS"} else {"FAIL"}, "script":self.script,
            "elapsed_wall_seconds":self.started.elapsed().as_secs_f64(), "error":self.failure,
            "checks":checks, "samples":self.samples,
            "determinism":"Fixed simulated dt and explicit input sequence; scene has no randomized behavior. Seed is recorded, not consumed. GPU pixels and wall time are not cross-platform deterministic.",
            "visual_review":"NOT RUN: inspect frames/video separately; assertions cannot establish visual quality",
            "scope":"Real game entry, world assets, Viewer camera and opt-in UI experiment; sample records are not quest state. Script gamepad injection verifies software routing, not physical gamepad hardware. No player collision, animation or gameplay acceptance"
        });
        let written = serde_json::to_vec_pretty(&report)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                fs::write(self.output.join("state.json"), bytes).map_err(|e| e.to_string())
            });
        if let Err(error) = written {
            error!("[capture/report] {error}");
            return false;
        }
        for check in &checks {
            info!("[capture/check] {check}");
        }
        if let Some(error) = &self.failure {
            error!("[capture/failed] {error}");
        }
        info!(
            "[capture/finished] status={} saved={} report={}",
            if passed { "PASS" } else { "FAIL" },
            self.saved,
            self.output.join("state.json").display()
        );
        passed
    }
}

pub fn install(app: &mut App, recording: Recording) {
    // The script permits 1 fps; do not clamp that fixed step to Bevy's default 250 ms
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(1));
    app.world_mut().spawn((ScriptGamepad, Gamepad::default()));
    app.insert_resource(Time::<Fixed>::from_hz(recording.script.fps as f64))
        .insert_resource(recording)
        .add_systems(First, advance_clock.before(TimeSystems))
        .add_systems(
            RunFixedMainLoop,
            drive_input.in_set(CaptureInput).before(UiInput),
        )
        .add_systems(PostUpdate, record);
}

fn advance_clock(
    mut recording: ResMut<Recording>,
    loading: Option<Res<SceneLoading>>,
    phase: Option<Res<State<GamePhase>>>,
    mut strategy: ResMut<TimeUpdateStrategy>,
    ui_font: Option<Res<UiFont>>,
    assets: Res<AssetServer>,
) {
    recording.tick = ui_font
        .as_ref()
        .is_none_or(|font| assets.is_loaded_with_dependencies(font.0.id()))
        && (recording.script.scene == "game-entry"
            || loading.as_ref().is_some_and(|loading| loading.ready))
        && recording.script.waits.iter().all(|wait| {
            wait.frame != recording.samples.len() as u32
                || phase
                    .as_ref()
                    .is_some_and(|phase| phase.get().as_str() == wait.game_page)
        })
        && recording.warmup == 0
        && !recording.pending
        && recording.samples.len() < recording.script.frames as usize;
    // One simulation step per saved frame; readback waits never advance the action
    *strategy = TimeUpdateStrategy::ManualDuration(if recording.tick {
        Duration::from_secs_f64(1.0 / recording.script.fps as f64)
    } else {
        Duration::ZERO
    });
}

fn drive_input(
    recording: Res<Recording>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut pad: Query<&mut Gamepad, With<ScriptGamepad>>,
) {
    keys.clear();
    buttons.clear();
    motion.delta = Vec2::ZERO;
    for mut gamepad in &mut pad {
        gamepad.digital_mut().clear();
    }
    if !recording.tick {
        return;
    }
    let frame = recording.samples.len() as u32;
    let event = recording
        .script
        .events
        .iter()
        .find(|e| e.start <= frame && frame < e.end);
    for mut gamepad in &mut pad {
        for (name, button) in PAD {
            if event.is_some_and(|e| e.gamepad.iter().any(|b| b == name)) {
                gamepad.digital_mut().press(button);
            } else {
                gamepad.digital_mut().release(button);
            }
        }
    }
    for (name, key) in KEYS {
        if event.is_some_and(|e| e.keys.iter().any(|k| k == name)) {
            keys.press(key);
        } else {
            keys.release(key);
        }
    }
    if event.is_some_and(|e| e.right_mouse) {
        buttons.press(MouseButton::Right);
    } else {
        buttons.release(MouseButton::Right);
    }
    if let Some(event) = event {
        motion.delta = Vec2::from_array(event.look);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects recorder, world, UI and asset state for evidence"
)]
fn record(
    mut commands: Commands,
    mut recording: ResMut<Recording>,
    loading: Option<Res<SceneLoading>>,
    phase: Option<Res<State<GamePhase>>>,
    load_error: Option<Res<GameLoadError>>,
    target: Res<CaptureTarget>,
    camera: Query<&Transform, With<Camera3d>>,
    #[cfg(feature = "viewer")] controller: Query<&FreeCameraState, With<Camera3d>>,
    world_entities: Query<(), With<MapSource>>,
    transforms: Query<(Entity, &Transform)>,
    mut exit: MessageWriter<AppExit>,
    ui: Option<Res<SignalUi>>,
    ui_font: Option<Res<UiFont>>,
    assets: Res<AssetServer>,
) {
    if recording.finished {
        return;
    }
    if let Some(font) = &ui_font
        && let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(font.0.id())
    {
        recording.failure = Some(format!("UI font loading failed: {error}"));
    }
    let world_ready = loading.as_ref().is_some_and(|loading| loading.ready);
    recording.world_ready_seen |= world_ready;
    if let Some(error) = load_error {
        recording.failure = Some(format!("game loading: {}", error.0));
    }
    if let Some(error) = loading
        .as_ref()
        .and_then(|loading| loading.failure.as_ref())
    {
        recording.failure = Some(format!("asset loading: {error}"));
    }
    if recording.failure.is_none()
        && recording.started.elapsed().as_secs() >= recording.script.timeout_seconds
    {
        recording.failure = Some(format!(
            "timeout after {}s: game_page={}, ready={}, requested={}, saved={}, pending={}",
            recording.script.timeout_seconds,
            phase
                .as_ref()
                .map_or("viewer", |phase| phase.get().as_str()),
            world_ready,
            recording.samples.len(),
            recording.saved,
            recording.pending
        ));
    }
    if recording.failure.is_some() || recording.saved == recording.script.frames {
        let passed = recording.finish(loading.as_deref());
        recording.finished = true;
        exit.write(if passed {
            AppExit::Success
        } else {
            AppExit::error()
        });
        return;
    }
    if recording.script.scene != "game-entry" && !world_ready {
        return;
    }
    if recording.warmup > 0 {
        recording.warmup -= 1;
        return;
    }
    if !recording.tick {
        return;
    }
    for (entity, transform) in &transforms {
        if !transform.translation.is_finite()
            || !transform.rotation.is_finite()
            || !transform.scale.is_finite()
        {
            recording.failure = Some(format!("non-finite Transform on entity {entity}"));
            return;
        }
    }
    recording.transforms_checked += 1;
    let Ok(transform) = camera.single() else {
        recording.failure = Some("expected exactly one world camera".into());
        return;
    };
    let Some(target) = target.0.as_ref() else {
        recording.failure = Some("offscreen target was not initialized".into());
        return;
    };
    let frame = recording.samples.len() as u32;
    let simulation_seconds = (frame + 1) as f64 / recording.script.fps as f64;
    #[cfg(feature = "viewer")]
    let enabled = controller.single().is_ok_and(|state| state.enabled);
    #[cfg(not(feature = "viewer"))]
    let enabled = false;
    recording.samples.push(Sample {
        frame,
        simulation_seconds,
        position: transform.translation.to_array(),
        rotation: transform.rotation.to_array(),
        enabled,
        game_page: phase.as_ref().map(|phase| phase.get().as_str().to_owned()),
        world_ready,
        world_entities: world_entities.iter().count(),
        ui: ui
            .as_ref()
            .map(|ui| serde_json::to_value(&**ui).expect("finite UI state")),
    });
    recording.pending = true;
    commands.spawn(Screenshot::image(target.clone())).observe(
        move |event: On<ScreenshotCaptured>, mut recording: ResMut<Recording>| {
            let file = recording.output.join(format!("frames/frame{frame:05}.png"));
            let result = (|| -> Result<(), String> {
                let size = event.image.size();
                if size != UVec2::new(recording.script.width, recording.script.height) {
                    return Err(format!("frame {frame}: unexpected image size {size:?}"));
                }
                let image = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| e.to_string())?
                    .to_rgb8();
                image
                    .save(&file)
                    .map_err(|e| format!("{}: {e}", file.display()))?;
                if recording.script.keyframes.contains(&frame) {
                    fs::copy(
                        &file,
                        recording
                            .output
                            .join(format!("keyframes/frame{frame:05}.png")),
                    )
                    .map_err(|e| e.to_string())?;
                }
                Ok(())
            })();
            match result {
                Ok(()) => recording.saved += 1,
                Err(error) => recording.failure = Some(format!("screenshot save: {error}")),
            }
            recording.pending = false;
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_waits_hold_the_clock_until_the_real_phase_matches() {
        let mut script: Script =
            serde_json::from_str(include_str!("../capture/game-entry.json")).unwrap();
        script.waits[0].frame = 0;
        let recording = Recording {
            script,
            output: PathBuf::new(),
            started: Instant::now(),
            warmup: 0,
            tick: false,
            pending: false,
            saved: 0,
            transforms_checked: 0,
            world_ready_seen: false,
            samples: vec![],
            failure: None,
            finished: false,
        };
        let mut app = App::new();
        app.add_plugins((bevy::app::TaskPoolPlugin::default(), AssetPlugin::default()))
            .insert_resource(recording)
            .insert_resource(State::new(GamePhase::Title))
            .init_resource::<TimeUpdateStrategy>()
            .add_systems(Update, advance_clock);
        app.update();
        assert!(!app.world().resource::<Recording>().tick);
        assert!(
            matches!(app.world().resource::<TimeUpdateStrategy>(), TimeUpdateStrategy::ManualDuration(dt) if dt.is_zero())
        );
        app.world_mut()
            .insert_resource(State::new(GamePhase::World));
        app.update();
        assert!(app.world().resource::<Recording>().tick);
        assert!(
            matches!(app.world().resource::<TimeUpdateStrategy>(), TimeUpdateStrategy::ManualDuration(dt) if *dt == Duration::from_secs_f64(1.0 / 30.0))
        );
        app.world_mut().resource_mut::<Recording>().pending = true;
        app.update();
        assert!(!app.world().resource::<Recording>().tick);
        assert!(
            matches!(app.world().resource::<TimeUpdateStrategy>(), TimeUpdateStrategy::ManualDuration(dt) if dt.is_zero())
        );
    }

    #[test]
    fn script_contract_rejects_unreproducible_inputs() {
        let script: Script =
            serde_json::from_str(include_str!("../capture/viewer-tour.json")).unwrap();
        assert!(script.validate().is_ok());
        let ui_script: Script =
            serde_json::from_str(include_str!("../capture/ui-signal.json")).unwrap();
        assert!(ui_script.validate().is_ok());
        let mut invalid_ui = ui_script.clone();
        invalid_ui.events[0].gamepad.push("typo".into());
        assert!(invalid_ui.validate().is_err());
        let mut invalid_ui = ui_script;
        invalid_ui.assertions[0].ui_page = Some("fake_quest".into());
        assert!(invalid_ui.validate().is_err());
        let mut invalid = script.clone();
        invalid.events[0].keys.push("typo".into());
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.events[1].start = 0;
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[0].to = invalid.frames;
        assert!(invalid.validate().is_err());
        let mut invalid = script;
        invalid.width = 0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn script_contract_validates_game_waits_and_world_assertions() {
        let script: Script =
            serde_json::from_str(include_str!("../capture/game-entry.json")).unwrap();
        assert!(script.validate().is_ok());
        let mut invalid = script.clone();
        invalid.scene = "district".into();
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.waits.push(invalid.waits[0].clone());
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.waits[0].frame = invalid.frames;
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.waits[0].game_page = "fake_world".into();
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[0].game_page = Some("fake_world".into());
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[1].max_world_entities = Some(0);
        assert!(invalid.validate().is_err());
        let mut invalid = script;
        invalid.view = "overview".into();
        assert!(invalid.validate().is_err());
    }
}
