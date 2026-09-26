//! Scripted input and evidence for the real Map Viewer, using Bevy's screenshot API
//! Workflow adapted from htdt/godogen engines/bevy.md at 0b725bca053769a4727f76c332bf1f7b42e146ab
//! Copyright 2026 Alex Ermolov, MIT; retained license: third_party/skills/godogen/LICENSE.md
//! N:SIDE implementation reuses the existing Viewer and records real input/state instead of a scaffold
use super::{CaptureTarget, camera_focus};
use bevy::{
    camera_controller::free_camera::FreeCameraState,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::{TimeSystems, TimeUpdateStrategy},
};
use n_side::world::scene::SceneLoading;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

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
    pub assertions: Vec<Assertion>,
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
}

const KEYS: [(&str, KeyCode); 9] = [
    ("W", KeyCode::KeyW),
    ("A", KeyCode::KeyA),
    ("S", KeyCode::KeyS),
    ("D", KeyCode::KeyD),
    ("Q", KeyCode::KeyQ),
    ("E", KeyCode::KeyE),
    ("Shift", KeyCode::ShiftLeft),
    ("M", KeyCode::KeyM),
    ("Escape", KeyCode::Escape),
];

impl Script {
    fn validate(&self) -> Result<(), String> {
        if self.scene != "district" || self.view.is_empty() {
            return Err("scene must be district and view must name an existing Viewer view".into());
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
            {
                return Err("input spans must be ordered, non-overlapping, within frames and use known keys/finite mouse deltas".into());
            }
            previous_end = event.end;
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
                    && check.enabled.is_none())
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
            samples: vec![],
            failure: None,
            finished: false,
        })
    }

    fn finish(&mut self, loading: &SceneLoading) -> bool {
        let mut checks = vec![
            serde_json::json!({"name":"required_assets_ready", "passed":loading.ready && loading.failure.is_none()}),
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
                        && assertion.enabled.is_none_or(|v| b.enabled == v);
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
            "scope":"Existing free camera and world assets, not player collision, animation, quests or gameplay acceptance"
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
    app.insert_resource(Time::<Fixed>::from_hz(recording.script.fps as f64))
        .insert_resource(recording)
        .add_systems(First, advance_clock.before(TimeSystems))
        .add_systems(RunFixedMainLoop, drive_input.before(camera_focus))
        .add_systems(PostUpdate, record);
}

fn advance_clock(
    mut recording: ResMut<Recording>,
    loading: Res<SceneLoading>,
    mut strategy: ResMut<TimeUpdateStrategy>,
) {
    recording.tick = loading.ready
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
) {
    keys.clear();
    buttons.clear();
    motion.delta = Vec2::ZERO;
    if !recording.tick {
        return;
    }
    let frame = recording.samples.len() as u32;
    let event = recording
        .script
        .events
        .iter()
        .find(|e| e.start <= frame && frame < e.end);
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

fn record(
    mut commands: Commands,
    mut recording: ResMut<Recording>,
    loading: Res<SceneLoading>,
    target: Res<CaptureTarget>,
    camera: Query<(&Transform, &FreeCameraState), With<Camera3d>>,
    transforms: Query<(Entity, &Transform)>,
    mut exit: MessageWriter<AppExit>,
) {
    if recording.finished {
        return;
    }
    if let Some(error) = &loading.failure {
        recording.failure = Some(format!("asset loading: {error}"));
    }
    if recording.started.elapsed().as_secs() >= recording.script.timeout_seconds {
        recording.failure = Some(format!(
            "timeout after {}s: ready={}, requested={}, saved={}, pending={}",
            recording.script.timeout_seconds,
            loading.ready,
            recording.samples.len(),
            recording.saved,
            recording.pending
        ));
    }
    if recording.failure.is_some() || recording.saved == recording.script.frames {
        let passed = recording.finish(&loading);
        recording.finished = true;
        exit.write(if passed {
            AppExit::Success
        } else {
            AppExit::error()
        });
        return;
    }
    if !loading.ready {
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
    let Ok((transform, state)) = camera.single() else {
        recording.failure = Some("expected exactly one world camera".into());
        return;
    };
    let Some(target) = target.0.as_ref() else {
        recording.failure = Some("offscreen target was not initialized".into());
        return;
    };
    let frame = recording.samples.len() as u32;
    let simulation_seconds = (frame + 1) as f64 / recording.script.fps as f64;
    recording.samples.push(Sample {
        frame,
        simulation_seconds,
        position: transform.translation.to_array(),
        rotation: transform.rotation.to_array(),
        enabled: state.enabled,
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
    fn script_contract_rejects_unreproducible_inputs() {
        let script: Script =
            serde_json::from_str(include_str!("../../../capture/viewer-tour.json")).unwrap();
        assert!(script.validate().is_ok());
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
}
