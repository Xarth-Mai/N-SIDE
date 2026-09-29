//! Scripted input and evidence for the real game and Map Viewer, using Bevy's screenshot API
//! Workflow adapted from htdt/godogen engines/bevy.md at 0b725bca053769a4727f76c332bf1f7b42e146ab
//! Copyright 2026 Alex Ermolov, MIT; retained license: third_party/skills/godogen/LICENSE.md
//! N:SIDE implementation records real application input, state and rendered frames
use crate::{
    app::{EntrySettings, EntryUi, GameLoadError, GamePhase},
    character::CharacterStatus,
    graphics::GraphicsEvidence,
    places::{Observation, PlaceHud},
    player::{PlayerState, PointerLock},
    story::ShopHandoff,
    ui::{SignalUi, UiFont, UiInput},
    world::{
        collision::CollisionWorld,
        scene::{MapSource, SceneLoading},
    },
};
mod route;
#[cfg(feature = "viewer")]
use bevy::camera_controller::free_camera::FreeCameraState;
use bevy::{
    app::RunFixedMainLoopSystems,
    camera::RenderTarget,
    input::{
        InputSystems,
        gamepad::{GamepadConnection, GamepadConnectionEvent},
        mouse::AccumulatedMouseMotion,
    },
    picking::{
        PickingSystems,
        pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput},
    },
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::{TimeSystems, TimeUpdateStrategy},
    window::{PrimaryWindow, WindowFocused},
};
use route::{RouteDriver, RouteScript};
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CaptureRecord;

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
    #[serde(default)]
    pub route: Option<RouteScript>,
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
    pointer: Option<[f32; 2]>,
    #[serde(default)]
    left_mouse: bool,
    #[serde(default)]
    gamepad: Vec<String>,
    #[serde(default)]
    look: [f32; 2],
    #[serde(default)]
    move_axis: [f32; 2],
    #[serde(default)]
    look_axis: [f32; 2],
    #[serde(default)]
    focused: Option<bool>,
    #[serde(default)]
    gamepad_connected: Option<bool>,
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
    max_rotation: Option<f32>,
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
    min_player_distance: Option<f32>,
    max_player_distance: Option<f32>,
    min_player_height: Option<f32>,
    max_player_height: Option<f32>,
    min_player_camera_distance: Option<f32>,
    max_player_camera_distance: Option<f32>,
    player_grounded: Option<bool>,
    min_player_jumps: Option<u32>,
    max_player_jumps: Option<u32>,
    player_sprinting: Option<bool>,
    pointer_locked: Option<bool>,
    max_player_resets: Option<u32>,
    player_blocked: Option<String>,
    gamepad_connected: Option<bool>,
    place_id: Option<String>,
    place_room: Option<String>,
    inside_room: Option<bool>,
    place_name: Option<String>,
    place_visible: Option<bool>,
    place_gamepad: Option<bool>,
    place_known: Option<bool>,
    observation_target: Option<String>,
    observation_selected: Option<String>,
    observation_open: Option<bool>,
    observation_visible: Option<bool>,
    handoff: Option<HandoffCheck>,
    character: Option<CharacterCheck>,
    settings_open: Option<bool>,
    graphics_tab: Option<bool>,
    graphics_page: Option<usize>,
    display_pending: Option<bool>,
    configured_resolution: Option<u8>,
    configured_borderless: Option<bool>,
    graphics: Option<GraphicsCheck>,
    entry_focus: Option<usize>,
    text_scale: Option<f32>,
    camera_sensitivity: Option<f32>,
    entry_device: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CharacterCheck {
    ready: bool,
    clip: Option<String>,
    min_clip_elapsed: Option<f32>,
    min_advance: Option<f32>,
    max_drift: Option<f32>,
    paused: Option<bool>,
    min_transitions: Option<u32>,
}

impl CharacterCheck {
    fn valid(&self) -> bool {
        self.clip
            .as_deref()
            .is_none_or(|v| ["Idle", "Walk", "Run"].contains(&v))
            && [self.min_clip_elapsed, self.min_advance, self.max_drift]
                .into_iter()
                .flatten()
                .all(|v| v.is_finite() && v >= 0.0)
    }

    fn matches_range(&self, samples: &[Sample]) -> bool {
        if !samples.iter().all(|s| self.matches(s.character.as_ref())) {
            return false;
        }
        if self.min_advance.is_none() && self.max_drift.is_none() {
            return true;
        }
        let Some(first) = samples.first().and_then(|s| s.character.as_ref()) else {
            return false;
        };
        let last = samples.last().unwrap().character.as_ref().unwrap();
        self.min_advance.is_none_or(|v| {
            last.clip_elapsed - first.clip_elapsed >= v
                && (v == 0.0
                    || samples.iter().any(|s| {
                        (s.character.as_ref().unwrap().clip_time - first.clip_time).abs() > 0.00001
                    }))
        }) && samples.iter().all(|s| {
            let actual = s.character.as_ref().unwrap();
            actual.clip == first.clip
                && actual.transitions == first.transitions
                && self.max_drift.is_none_or(|v| {
                    (actual.clip_elapsed - first.clip_elapsed).abs() <= v
                        && (actual.clip_time - first.clip_time).abs() <= v
                })
        })
    }

    fn matches(&self, actual: Option<&CharacterStatus>) -> bool {
        let Some(actual) = actual else { return false };
        actual.error.is_none()
            && actual.clip_elapsed.is_finite()
            && actual.clip_time.is_finite()
            && actual.ready == self.ready
            && self.clip.as_deref().is_none_or(|v| actual.clip == Some(v))
            && self
                .min_clip_elapsed
                .is_none_or(|v| actual.clip_elapsed >= v)
            && self.paused.is_none_or(|v| actual.paused == v)
            && self.min_transitions.is_none_or(|v| actual.transitions >= v)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HandoffCheck {
    stage: Option<String>,
    observed_places: Option<Vec<String>>,
    confirmation_count: Option<u32>,
    rejected_choices: Option<u32>,
}

impl HandoffCheck {
    fn valid(&self) -> bool {
        (self.stage.is_some()
            || self.observed_places.is_some()
            || self.confirmation_count.is_some()
            || self.rejected_choices.is_some())
            && self
                .stage
                .as_deref()
                .is_none_or(|v| ["observing", "awaiting_choice", "confirmed"].contains(&v))
            && self.observed_places.as_ref().is_none_or(|ids| {
                ids.iter()
                    .all(|id| crate::story::HANDOFF_PLACES.contains(&id.as_str()))
                    && ids.iter().collect::<std::collections::BTreeSet<_>>().len() == ids.len()
            })
    }

    fn matches(&self, actual: Option<&ShopHandoff>) -> bool {
        let Some(actual) = actual else { return false };
        self.stage
            .as_deref()
            .is_none_or(|v| v == actual.stage().as_str())
            && self.observed_places.as_ref().is_none_or(|ids| {
                ids.iter().collect::<std::collections::BTreeSet<_>>()
                    == actual.observed_places.iter().collect()
            })
            && self
                .confirmation_count
                .is_none_or(|v| v == actual.confirmation_count)
            && self
                .rejected_choices
                .is_none_or(|v| v == actual.rejected_choices)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphicsCheck {
    aa: Option<String>,
    ssao: Option<String>,
    bloom: Option<bool>,
    shadows: Option<bool>,
}

impl GraphicsCheck {
    fn valid(&self) -> bool {
        (self.aa.is_some() || self.ssao.is_some() || self.bloom.is_some() || self.shadows.is_some())
            && self.aa.as_deref().is_none_or(|value| {
                ["off", "msaa2", "msaa4", "msaa8", "fxaa", "smaa", "taa"].contains(&value)
            })
            && self
                .ssao
                .as_deref()
                .is_none_or(|value| ["off", "low", "medium", "high", "ultra"].contains(&value))
    }

    fn matches(&self, actual: Option<&GraphicsEvidence>) -> bool {
        let Some(actual) = actual else { return false };
        self.aa.as_ref().is_none_or(|value| actual.aa == *value)
            && self.ssao.as_ref().is_none_or(|value| actual.ssao == *value)
            && self.bloom.is_none_or(|value| actual.bloom == value)
            && self.shadows.is_none_or(|value| actual.shadows == value)
    }
}

const GAME_PAGES: [&str; 5] = ["title", "loading", "world", "paused", "failed"];

const KEYS: [(&str, KeyCode); 21] = [
    ("W", KeyCode::KeyW),
    ("A", KeyCode::KeyA),
    ("S", KeyCode::KeyS),
    ("D", KeyCode::KeyD),
    ("Q", KeyCode::KeyQ),
    ("E", KeyCode::KeyE),
    ("F", KeyCode::KeyF),
    ("R", KeyCode::KeyR),
    ("Shift", KeyCode::ShiftLeft),
    ("ShiftRight", KeyCode::ShiftRight),
    ("M", KeyCode::KeyM),
    ("Escape", KeyCode::Escape),
    ("Tab", KeyCode::Tab),
    ("Enter", KeyCode::Enter),
    ("Space", KeyCode::Space),
    ("Left", KeyCode::ArrowLeft),
    ("Right", KeyCode::ArrowRight),
    ("Down", KeyCode::ArrowDown),
    ("Up", KeyCode::ArrowUp),
    ("PageDown", KeyCode::PageDown),
    ("PageUp", KeyCode::PageUp),
];

const PAD: [(&str, GamepadButton); 12] = [
    ("Down", GamepadButton::DPadDown),
    ("Up", GamepadButton::DPadUp),
    ("Confirm", GamepadButton::South),
    ("Back", GamepadButton::East),
    ("Menu", GamepadButton::Start),
    ("Reset", GamepadButton::Select),
    ("ScrollDown", GamepadButton::RightTrigger),
    ("ScrollUp", GamepadButton::LeftTrigger),
    ("Jump", GamepadButton::West),
    ("Sprint", GamepadButton::LeftThumb),
    ("Left", GamepadButton::DPadLeft),
    ("Right", GamepadButton::DPadRight),
];
#[derive(Component)]
struct ScriptGamepad;
#[derive(Component)]
struct ScriptFocusWindow;

impl Script {
    fn game_scene(&self) -> bool {
        matches!(self.scene.as_str(), "game-entry" | "walk-preview")
    }

    fn validate(&self) -> Result<(), String> {
        if !matches!(
            self.scene.as_str(),
            "district" | "ui-signal" | "game-entry" | "walk-preview"
        ) || self.view.is_empty()
            || (self.game_scene() && self.view != "shop")
        {
            return Err(
                "scene must be district/ui-signal with an existing Viewer view, or game-entry/walk-preview with view shop".into(),
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
            if event.left_mouse && event.pointer.is_none()
                || event.pointer.is_some_and(|p| {
                    !p[0].is_finite()
                        || !p[1].is_finite()
                        || p[0] < 0.0
                        || p[0] >= self.width as f32
                        || p[1] < 0.0
                        || p[1] >= self.height as f32
                })
            {
                return Err("pointer must be inside the capture canvas; left_mouse requires pointer coordinates".into());
            }
            if event.focused.is_some() && !self.game_scene() {
                return Err("focused input requires game-entry or walk-preview".into());
            }
            if event.gamepad_connected.is_some() && !self.game_scene() {
                return Err("gamepad_connected input requires game-entry or walk-preview".into());
            }
            if event.start < previous_end
                || event.start >= event.end
                || event.end > self.frames
                || event.look.iter().any(|v| !v.is_finite())
                || event
                    .move_axis
                    .iter()
                    .chain(&event.look_axis)
                    .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
                || event
                    .keys
                    .iter()
                    .any(|key| !KEYS.iter().any(|(name, _)| name == key))
                || event
                    .gamepad
                    .iter()
                    .any(|button| !PAD.iter().any(|(name, _)| name == button))
            {
                return Err("input spans must be ordered, non-overlapping, within frames and use known keys, finite mouse deltas and gamepad axes in -1..1".into());
            }
            previous_end = event.end;
        }
        let mut wait_frames = std::collections::BTreeSet::new();
        if let Some(route) = &self.route
            && (self.scene != "walk-preview"
                || route.id.trim().is_empty()
                || route.start >= self.frames
                || self.events.iter().any(|event| event.end > route.start))
        {
            return Err("route requires walk-preview, a nonempty id, start within frames and all manual input ending before its start".into());
        }
        for wait in &self.waits {
            if !self.game_scene()
                || wait.frame >= self.frames
                || !wait_frames.insert(wait.frame)
                || !GAME_PAGES.contains(&wait.game_page.as_str())
            {
                return Err(
                    "waits require game-entry/walk-preview, unique frames within frames and a known game_page"
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
                || [
                    check.min_distance,
                    check.max_distance,
                    check.min_rotation,
                    check.max_rotation,
                    check.min_player_distance,
                    check.max_player_distance,
                    check.min_player_camera_distance,
                    check.max_player_camera_distance,
                ]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite() || v < 0.0)
                || (check.min_distance.is_none()
                    && check.max_distance.is_none()
                    && check.min_rotation.is_none()
                    && check.max_rotation.is_none()
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
                    && check.same_world_entities.is_none()
                    && check.gamepad_connected.is_none()
                    && check.pointer_locked.is_none()
                    && check.graphics.is_none()
                    && check.handoff.is_none()
                    && check.character.is_none()
                    && !check.has_observation_assertion()
                    && !check.has_entry_assertion()
                    && !check.has_place_assertion()
                    && !check.has_player_assertion())
                || check
                    .handoff
                    .as_ref()
                    .is_some_and(|v| self.scene != "walk-preview" || !v.valid())
                || check
                    .character
                    .as_ref()
                    .is_some_and(|v| self.scene != "walk-preview" || !v.valid())
                || (check.has_player_assertion() && self.scene != "walk-preview")
                || (check.gamepad_connected.is_some() && !self.game_scene())
                || (check.pointer_locked.is_some() && self.scene != "walk-preview")
                || check
                    .min_player_jumps
                    .zip(check.max_player_jumps)
                    .is_some_and(|(min, max)| min > max)
                || (check.has_place_assertion() && self.scene != "walk-preview")
                || (check.has_observation_assertion() && self.scene != "walk-preview")
                || [
                    check.observation_target.as_ref(),
                    check.observation_selected.as_ref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| value.trim().is_empty())
                || (check.has_entry_assertion() && !self.game_scene())
                || check.entry_focus.is_some_and(|focus| focus > 20)
                || check.graphics_page.is_some_and(|page| page > 5)
                || check.configured_resolution.is_some_and(|value| value > 3)
                || check
                    .graphics
                    .as_ref()
                    .is_some_and(|graphics| !self.game_scene() || !graphics.valid())
                || check
                    .text_scale
                    .is_some_and(|scale| ![1.0, 1.25].contains(&scale))
                || check
                    .camera_sensitivity
                    .is_some_and(|value| ![1.0, 0.65].contains(&value))
                || check
                    .entry_device
                    .as_ref()
                    .is_some_and(|device| !["keyboard_mouse", "gamepad"].contains(&device.as_str()))
                || [
                    check.place_id.as_ref(),
                    check.place_name.as_ref(),
                    check.place_room.as_ref(),
                ]
                .into_iter()
                .flatten()
                .any(|value| value.trim().is_empty())
                || check
                    .min_rotation
                    .zip(check.max_rotation)
                    .is_some_and(|(min, max)| min > max)
                || [check.min_player_height, check.max_player_height]
                    .into_iter()
                    .flatten()
                    .any(|v| !v.is_finite())
                || check
                    .min_player_distance
                    .zip(check.max_player_distance)
                    .is_some_and(|(min, max)| min > max)
                || check
                    .min_player_height
                    .zip(check.max_player_height)
                    .is_some_and(|(min, max)| min > max)
                || check
                    .min_player_camera_distance
                    .zip(check.max_player_camera_distance)
                    .is_some_and(|(min, max)| min > max)
                || check
                    .player_blocked
                    .as_ref()
                    .is_some_and(|value| value.trim().is_empty())
                || check
                    .game_page
                    .as_ref()
                    .is_some_and(|page| !self.game_scene() || !GAME_PAGES.contains(&page.as_str()))
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

impl Assertion {
    fn has_observation_assertion(&self) -> bool {
        self.observation_target.is_some()
            || self.observation_selected.is_some()
            || self.observation_open.is_some()
            || self.observation_visible.is_some()
    }

    fn check_observation(&self, observation: Option<&ObservationSample>) -> bool {
        if !self.has_observation_assertion() {
            return true;
        }
        let Some(observation) = observation else {
            return false;
        };
        self.observation_target
            .as_ref()
            .is_none_or(|target| observation.target.as_ref() == Some(target))
            && self
                .observation_selected
                .as_ref()
                .is_none_or(|selected| observation.selected.as_ref() == Some(selected))
            && self
                .observation_open
                .is_none_or(|open| observation.open == open)
            && self
                .observation_visible
                .is_none_or(|visible| observation.visible == visible)
    }

    fn has_entry_assertion(&self) -> bool {
        self.settings_open.is_some()
            || self.graphics_tab.is_some()
            || self.graphics_page.is_some()
            || self.display_pending.is_some()
            || self.configured_resolution.is_some()
            || self.configured_borderless.is_some()
            || self.entry_focus.is_some()
            || self.text_scale.is_some()
            || self.camera_sensitivity.is_some()
            || self.entry_device.is_some()
    }

    fn check_entry(&self, entry: Option<&EntrySample>) -> bool {
        if !self.has_entry_assertion() {
            return true;
        }
        let Some(entry) = entry else {
            return false;
        };
        self.settings_open
            .is_none_or(|open| entry.settings_open == open)
            && self
                .graphics_tab
                .is_none_or(|value| entry.graphics_tab == value)
            && self
                .graphics_page
                .is_none_or(|value| entry.graphics_page == value)
            && self
                .display_pending
                .is_none_or(|value| entry.display_pending == value)
            && self
                .configured_resolution
                .is_none_or(|value| entry.configured_resolution == value)
            && self
                .configured_borderless
                .is_none_or(|value| entry.configured_borderless == value)
            && self.entry_focus.is_none_or(|focus| entry.focus == focus)
            && self
                .text_scale
                .is_none_or(|scale| entry.text_scale == scale)
            && self
                .camera_sensitivity
                .is_none_or(|value| entry.camera_sensitivity == value)
            && self
                .entry_device
                .as_ref()
                .is_none_or(|device| entry.device == *device)
    }

    fn has_place_assertion(&self) -> bool {
        self.place_id.is_some()
            || self.place_room.is_some()
            || self.inside_room.is_some()
            || self.place_name.is_some()
            || self.place_visible.is_some()
            || self.place_gamepad.is_some()
            || self.place_known.is_some()
    }

    fn check_place(&self, place: Option<&PlaceSample>) -> bool {
        if !self.has_place_assertion() {
            return true;
        }
        let Some(place) = place else {
            return false;
        };
        self.place_id
            .as_ref()
            .is_none_or(|id| place.id.as_ref() == Some(id))
            && self
                .place_name
                .as_ref()
                .is_none_or(|name| place.name == *name)
            && self
                .place_room
                .as_ref()
                .is_none_or(|room| place.room.as_ref() == Some(room))
            && self
                .inside_room
                .is_none_or(|inside| place.room.is_some() == inside)
            && self
                .place_visible
                .is_none_or(|visible| place.visible == visible)
            && self
                .place_gamepad
                .is_none_or(|gamepad| place.gamepad == gamepad)
            && self
                .place_known
                .is_none_or(|known| place.id.is_some() == known)
    }

    fn has_player_assertion(&self) -> bool {
        self.min_player_distance.is_some()
            || self.max_player_distance.is_some()
            || self.min_player_height.is_some()
            || self.max_player_height.is_some()
            || self.min_player_camera_distance.is_some()
            || self.max_player_camera_distance.is_some()
            || self.player_grounded.is_some()
            || self.min_player_jumps.is_some()
            || self.max_player_jumps.is_some()
            || self.player_sprinting.is_some()
            || self.max_player_resets.is_some()
            || self.player_blocked.is_some()
    }

    fn check_player(&self, samples: &[Sample]) -> bool {
        if !self.has_player_assertion() {
            return true;
        }
        let Some(start) = samples.first().and_then(|sample| sample.player.as_ref()) else {
            return false;
        };
        let mut peak_distance = 0.0_f32;
        let mut min_height = f32::INFINITY;
        let mut max_height = f32::NEG_INFINITY;
        let mut min_camera_distance = f32::INFINITY;
        let mut max_camera_distance = f32::NEG_INFINITY;
        let mut blocked = false;
        for sample in samples {
            let Some(player) = &sample.player else {
                return false;
            };
            if self
                .player_sprinting
                .is_some_and(|expected| player.sprinting != expected)
            {
                return false;
            }
            peak_distance = peak_distance
                .max(Vec3::from_array(start.foot).distance(Vec3::from_array(player.foot)));
            min_height = min_height.min(player.foot[1]);
            max_height = max_height.max(player.foot[1]);
            min_camera_distance = min_camera_distance.min(player.camera_distance);
            max_camera_distance = max_camera_distance.max(player.camera_distance);
            blocked |= self.player_blocked.as_ref().is_some_and(|source| {
                player
                    .blocked
                    .as_ref()
                    .is_some_and(|actual| actual.contains(source))
            });
        }
        let end = samples.last().unwrap().player.as_ref().unwrap();
        self.min_player_distance.is_none_or(|limit| {
            Vec3::from_array(start.foot).distance(Vec3::from_array(end.foot)) >= limit
        }) && self
            .max_player_distance
            .is_none_or(|limit| peak_distance <= limit)
            && self
                .min_player_height
                .is_none_or(|limit| min_height >= limit)
            && self
                .max_player_height
                .is_none_or(|limit| max_height <= limit)
            && self
                .min_player_camera_distance
                .is_none_or(|limit| min_camera_distance >= limit)
            && self
                .max_player_camera_distance
                .is_none_or(|limit| max_camera_distance <= limit)
            && self
                .player_grounded
                .is_none_or(|expected| end.grounded == expected)
            && self.min_player_jumps.is_none_or(|limit| end.jumps >= limit)
            && self.max_player_jumps.is_none_or(|limit| end.jumps <= limit)
            && self
                .max_player_resets
                .is_none_or(|limit| end.resets <= limit)
            && (self.player_blocked.is_none() || blocked)
    }
}

#[derive(Serialize)]
struct PlayerSample {
    foot: [f32; 3],
    grounded: bool,
    jumps: u32,
    sprinting: bool,
    blocked: Option<String>,
    resets: u32,
    camera_distance: f32,
}

impl From<&PlayerState> for PlayerSample {
    fn from(player: &PlayerState) -> Self {
        Self {
            foot: player.foot.to_array(),
            grounded: player.grounded,
            jumps: player.jumps,
            sprinting: player.sprinting,
            blocked: player.blocked.clone(),
            resets: player.resets,
            camera_distance: player.camera_distance,
        }
    }
}

#[derive(Serialize)]
struct PlaceSample {
    id: Option<String>,
    room: Option<String>,
    name: String,
    visible: bool,
    gamepad: bool,
}

#[derive(Serialize)]
struct ObservationSample {
    target: Option<String>,
    selected: Option<String>,
    open: bool,
    visible: bool,
}

#[derive(Serialize)]
struct EntrySample {
    settings_open: bool,
    graphics_tab: bool,
    graphics_page: usize,
    display_pending: bool,
    configured_resolution: u8,
    configured_borderless: bool,
    focus: usize,
    text_scale: f32,
    camera_sensitivity: f32,
    device: &'static str,
}

#[derive(Clone, Copy, Serialize)]
struct ResourceCounts {
    entities: usize,
    meshes: usize,
    images: usize,
    materials: usize,
}

#[derive(Default)]
struct RuntimeMetrics {
    last_update: Option<Instant>,
    last_world_active: bool,
    world_update_intervals_ms: Vec<f64>,
    load_started: Option<Instant>,
    observed_load_seconds: Vec<f64>,
}

impl RuntimeMetrics {
    fn observe(&mut self, now: Instant, loading: Option<bool>, world_active: bool) {
        if let Some(previous) = self.last_update.replace(now)
            && world_active
            && self.last_world_active
        {
            self.world_update_intervals_ms
                .push(now.duration_since(previous).as_secs_f64() * 1000.0);
        }
        self.last_world_active = world_active;
        match loading {
            Some(false) => {
                self.load_started.get_or_insert(now);
            }
            Some(true) => {
                if let Some(started) = self.load_started.take() {
                    self.observed_load_seconds
                        .push(now.duration_since(started).as_secs_f64());
                }
            }
            None => self.load_started = None,
        }
    }
}

// Capture intervals include screenshot scheduling, readback waits and PNG saves
// Native frame timing and GPU timings require separate profiling
fn interval_summary(mut values: Vec<f64>) -> serde_json::Value {
    if values.is_empty() {
        return serde_json::Value::Null;
    }
    values.sort_unstable_by(f64::total_cmp);
    let percentile = |fraction: f64| values[(values.len() as f64 * fraction).ceil() as usize - 1];
    serde_json::json!({
        "count": values.len(), "mean": values.iter().sum::<f64>() / values.len() as f64,
        "p50": percentile(0.5), "p95": percentile(0.95), "max": values.last(),
        "percentile_method": "nearest_rank"
    })
}

#[derive(Serialize)]
struct Sample {
    frame: u32,
    simulation_seconds: f64,
    wall_elapsed_seconds: f64,
    resources: Option<ResourceCounts>,
    position: [f32; 3],
    rotation: [f32; 4],
    enabled: bool,
    game_page: Option<String>,
    world_ready: bool,
    world_entities: usize,
    player: Option<PlayerSample>,
    ui: Option<serde_json::Value>,
    place: Option<PlaceSample>,
    observation: Option<ObservationSample>,
    handoff: Option<ShopHandoff>,
    character: Option<CharacterStatus>,
    entry: Option<EntrySample>,
    gamepad_connected: bool,
    pointer_locked: Option<bool>,
    graphics: Option<GraphicsEvidence>,
}

#[derive(Resource)]
pub struct Recording {
    pub script: Script,
    output: PathBuf,
    started: Instant,
    metrics: RuntimeMetrics,
    warmup: u32,
    tick: bool,
    pending: bool,
    saved: u32,
    transforms_checked: u32,
    world_ready_seen: bool,
    route: Option<RouteDriver>,
    samples: Vec<Sample>,
    failure: Option<String>,
    finished: bool,
}

impl Recording {
    pub fn load(path: &Path, output: PathBuf, project_root: &Path) -> Result<Self, String> {
        let bytes =
            fs::read(path).map_err(|e| format!("[capture/script] {}: {e}", path.display()))?;
        let script: Script = serde_json::from_slice(&bytes)
            .map_err(|e| format!("[capture/script] {}: {e}", path.display()))?;
        script
            .validate()
            .map_err(|e| format!("[capture/script] {e}"))?;
        let route = script
            .route
            .as_ref()
            .map(|route| RouteDriver::load(project_root, &route.id, route.round_trip))
            .transpose()?;
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
            metrics: RuntimeMetrics::default(),
            warmup: 30,
            tick: false,
            pending: false,
            saved: 0,
            transforms_checked: 0,
            world_ready_seen: false,
            route,
            samples: vec![],
            failure: None,
            finished: false,
        })
    }

    fn finish(&mut self, loading: Option<&SceneLoading>) -> bool {
        let assets_ready = if self.script.game_scene() {
            self.world_ready_seen
        } else {
            loading.is_some_and(|loading| loading.ready && loading.failure.is_none())
        };
        let mut checks = vec![
            serde_json::json!({"name":"required_assets_ready", "passed":assets_ready}),
            serde_json::json!({"name":"all_frames_saved", "passed":self.saved == self.script.frames, "saved":self.saved, "expected":self.script.frames}),
            serde_json::json!({"name":"finite_transforms", "passed":self.transforms_checked == self.script.frames, "checked_frames":self.transforms_checked}),
        ];
        if let Some(route) = &self.route {
            checks.push(serde_json::json!({"name":"route_all_nodes_reached", "passed":route.complete(), "route":route.report()}));
        }
        for assertion in &self.script.assertions {
            let measured = self
                .samples
                .get(assertion.from as usize)
                .zip(self.samples.get(assertion.to as usize));
            let (distance, rotation, peak_rotation, enabled, passed) =
                measured.map_or((None, None, None, None, false), |(a, b)| {
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
                    let peak_rotation = self.samples
                        [assertion.from as usize..=assertion.to as usize]
                        .iter()
                        .map(|sample| {
                            Quat::from_array(a.rotation)
                                .angle_between(Quat::from_array(sample.rotation))
                        })
                        .fold(0.0_f32, f32::max);
                    let passed = assertion.min_distance.is_none_or(|v| distance >= v)
                        && assertion.check_player(
                            &self.samples[assertion.from as usize..=assertion.to as usize],
                        )
                        && assertion.character.as_ref().is_none_or(|v| {
                            v.matches_range(
                                &self.samples[assertion.from as usize..=assertion.to as usize],
                            )
                        })
                        && self.samples[assertion.from as usize..=assertion.to as usize]
                            .iter()
                            .all(|sample| {
                                assertion.check_place(sample.place.as_ref())
                                    && assertion.check_observation(sample.observation.as_ref())
                                    && assertion
                                        .handoff
                                        .as_ref()
                                        .is_none_or(|v| v.matches(sample.handoff.as_ref()))
                                    && assertion.check_entry(sample.entry.as_ref())
                                    && assertion
                                        .pointer_locked
                                        .is_none_or(|value| sample.pointer_locked == Some(value))
                                    && assertion.graphics.as_ref().is_none_or(|expected| {
                                        expected.matches(sample.graphics.as_ref())
                                    })
                            })
                        && assertion.max_distance.is_none_or(|v| peak_distance <= v)
                        && assertion.min_rotation.is_none_or(|v| rotation >= v)
                        && assertion.max_rotation.is_none_or(|v| peak_rotation <= v)
                        && assertion.enabled.is_none_or(|v| b.enabled == v)
                        && assertion
                            .game_page
                            .as_ref()
                            .is_none_or(|v| b.game_page.as_ref() == Some(v))
                        && assertion.world_ready.is_none_or(|v| b.world_ready == v)
                        && assertion
                            .gamepad_connected
                            .is_none_or(|v| b.gamepad_connected == v)
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
                    (
                        Some(distance),
                        Some(rotation),
                        Some(peak_rotation),
                        Some(b.enabled),
                        passed,
                    )
                });
            checks.push(serde_json::json!({"name":assertion.name, "passed":passed, "expected":assertion, "distance":distance, "rotation_radians":rotation, "peak_rotation_radians":peak_rotation, "enabled":enabled}));
        }
        let passed = self.failure.is_none() && checks.iter().all(|c| c["passed"] == true);
        let report = serde_json::json!({
            "status":if passed {"PASS"} else {"FAIL"}, "script":self.script,
            "elapsed_wall_seconds":self.started.elapsed().as_secs_f64(), "error":self.failure,
            "checks":checks, "samples":self.samples,
            "performance": {
                "world_update_interval_ms": interval_summary(self.metrics.world_update_intervals_ms.clone()),
                "world_capture_frame_interval_ms": interval_summary(self.samples.windows(2)
                    .filter(|pair| pair.iter().all(|sample| sample.world_ready && sample.game_page.as_deref().is_none_or(|page| page == "world")))
                    .map(|pair| (pair[1].wall_elapsed_seconds - pair[0].wall_elapsed_seconds) * 1000.0).collect()),
                "observed_scene_load_seconds": self.metrics.observed_load_seconds,
                "resource_counts": {
                    "first_ready": self.samples.iter().find_map(|sample| sample.resources),
                    "last_ready": self.samples.iter().rev().find_map(|sample| sample.resources),
                    "peak_ready": {
                        "entities": self.samples.iter().filter_map(|sample| sample.resources.map(|counts| counts.entities)).max(),
                        "meshes": self.samples.iter().filter_map(|sample| sample.resources.map(|counts| counts.meshes)).max(),
                        "images": self.samples.iter().filter_map(|sample| sample.resources.map(|counts| counts.images)).max(),
                        "materials": self.samples.iter().filter_map(|sample| sample.resources.map(|counts| counts.materials)).max()
                    }
                },
                "scope": "Monotonic wall time measured at capture PostUpdate; warmup is excluded. Update intervals include zero-simulation readback-wait updates and PNG work, while capture frame intervals also include intervening updates. Both endpoints must be world-active; these are capture throughput intervals, not one-render intervals. Video encoding runs later and is excluded. Observed load spans first loading observation to ready, excluding preparation before observation. Resource counts are live main-world entities and Assets lengths at ready captured frames, not GPU bytes or visible draw calls. Compare the same instrumentation, build profile, device, route and capture settings; these metrics do not establish native FPS or GPU render time."
            },
            "determinism":"Fixed simulated dt; manual input spans or route steering from the measured player/camera state. Scene has no randomized behavior. Seed is recorded, not consumed. GPU pixels and wall time are not cross-platform deterministic.",
            "visual_review":"NOT RUN: inspect frames/video separately; assertions cannot establish visual quality",
            "scope":"Real game entry, world assets, Viewer camera and opt-in UI/walking experiments. Handoff samples are actual session state for the implemented prologue fragment, not completion of a full quest. Script gamepad injection verifies software routing, not physical gamepad hardware. Pointer lock samples reflect application intent; native cursor confinement requires a desktop test. State checks and images do not establish author/player acceptance"
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
    if recording
        .script
        .events
        .iter()
        .any(|event| event.focused.is_some())
    {
        // Input-only window: the host disables Winit and keeps rendering to CaptureTarget
        app.world_mut().spawn((
            ScriptFocusWindow,
            PrimaryWindow,
            Window {
                visible: false,
                focused: true,
                resolution: (recording.script.width, recording.script.height).into(),
                ..default()
            },
        ));
    }
    app.insert_resource(Time::<Fixed>::from_hz(recording.script.fps as f64))
        .add_message::<WindowFocused>()
        .add_message::<GamepadConnectionEvent>()
        .insert_resource(recording)
        .add_systems(First, advance_clock.before(TimeSystems))
        .add_systems(PreUpdate, drive_connection.before(InputSystems))
        .add_systems(
            PreUpdate,
            drive_pointer.before(PickingSystems::ProcessInput),
        )
        .add_systems(
            RunFixedMainLoop,
            drive_input
                .in_set(CaptureInput)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .before(UiInput),
        )
        .add_systems(PostUpdate, record.in_set(CaptureRecord));
}

fn advance_clock(
    mut recording: ResMut<Recording>,
    loading: Option<Res<SceneLoading>>,
    phase: Option<Res<State<GamePhase>>>,
    mut strategy: ResMut<TimeUpdateStrategy>,
    ui_font: Option<Res<UiFont>>,
    assets: Res<AssetServer>,
    character: Option<Res<CharacterStatus>>,
) {
    recording.tick = ui_font
        .as_ref()
        .is_none_or(|font| assets.is_loaded_with_dependencies(font.0.id()))
        && (recording.script.game_scene() || loading.as_ref().is_some_and(|loading| loading.ready))
        && recording.script.waits.iter().all(|wait| {
            wait.frame != recording.samples.len() as u32
                || phase
                    .as_ref()
                    .is_some_and(|phase| phase.get().as_str() == wait.game_page)
        })
        && character.as_ref().is_none_or(|c| {
            !phase
                .as_ref()
                .is_some_and(|p| matches!(p.get(), GamePhase::World | GamePhase::Paused))
                || c.ready
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

fn drive_connection(
    recording: Res<Recording>,
    pads: Query<(Entity, Option<&Gamepad>), With<ScriptGamepad>>,
    mut connections: MessageWriter<GamepadConnectionEvent>,
) {
    if !recording.tick {
        return;
    }
    let frame = recording.samples.len() as u32;
    let requested = recording
        .script
        .events
        .iter()
        .find(|event| event.start <= frame && frame < event.end)
        .and_then(|event| event.gamepad_connected);
    if let Some(connected) = requested {
        for (entity, pad) in &pads {
            if connected != pad.is_some() {
                // InputPlugin owns inserting/removing Gamepad; never set gameplay results here
                connections.write(GamepadConnectionEvent::new(
                    entity,
                    if connected {
                        GamepadConnection::Connected {
                            name: "Capture gamepad".into(),
                            vendor_id: None,
                            product_id: None,
                        }
                    } else {
                        GamepadConnection::Disconnected
                    },
                ));
            }
        }
    }
}

// Feed the native UI picking backend for the same image target used by the real camera
// Unlike Interaction assignment, this exercises layout, clipping and pointer hit testing
fn drive_pointer(
    recording: Res<Recording>,
    target: Res<CaptureTarget>,
    mut events: MessageWriter<PointerInput>,
    mut prior: Local<Option<(Vec2, bool)>>,
) {
    if !recording.tick {
        return;
    }
    let frame = recording.samples.len() as u32;
    let event = recording
        .script
        .events
        .iter()
        .find(|e| e.start <= frame && frame < e.end);
    let position = event
        .and_then(|e| e.pointer)
        .map(Vec2::from_array)
        .or_else(|| prior.map(|(position, _)| position));
    let (Some(position), Some(image)) = (position, &target.0) else {
        return;
    };
    let location = Location {
        target: RenderTarget::Image(image.clone().into())
            .normalize(None)
            .unwrap(),
        position,
    };
    let pressed = event.is_some_and(|e| e.left_mouse);
    if prior.is_none_or(|(old, _)| old != position) {
        events.write(PointerInput::new(
            PointerId::Mouse,
            location.clone(),
            PointerAction::Move {
                delta: prior.map_or(Vec2::ZERO, |(old, _)| position - old),
            },
        ));
    }
    if pressed != prior.is_some_and(|(_, pressed)| pressed) {
        events.write(PointerInput::new(
            PointerId::Mouse,
            location,
            if pressed {
                PointerAction::Press(PointerButton::Primary)
            } else {
                PointerAction::Release(PointerButton::Primary)
            },
        ));
    }
    *prior = Some((position, pressed));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Capture observes player/collision/camera but only writes ordinary inputs"
)]
fn drive_input(
    mut recording: ResMut<Recording>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut pad: Query<&mut Gamepad, With<ScriptGamepad>>,
    mut windows: Query<(Entity, &mut Window), With<ScriptFocusWindow>>,
    mut focus_events: MessageWriter<WindowFocused>,
    player: Option<Res<PlayerState>>,
    collision: Option<Res<CollisionWorld>>,
    camera: Query<&Transform, With<Camera3d>>,
    settings: Option<Res<EntrySettings>>,
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
    let route_active = recording
        .script
        .route
        .as_ref()
        .is_some_and(|route| frame >= route.start);
    let route_input = if route_active {
        let fps = recording.script.fps;
        let result = match (
            recording.route.as_mut(),
            player.as_deref(),
            collision.as_deref(),
            camera.single(),
        ) {
            (Some(route), Some(player), Some(collision), Ok(camera)) => route.input(
                frame,
                fps,
                &PlayerSample::from(player),
                camera,
                collision,
                settings
                    .as_ref()
                    .map_or(1.0, |settings| settings.camera_sensitivity()),
            ),
            _ => Err(format!(
                "[capture/route] frame={frame}: player, collision, camera or route unavailable"
            )),
        };
        match result {
            Ok(input) => Some(input),
            Err(error) => {
                recording.failure = Some(error);
                return;
            }
        }
    } else {
        None
    };
    let event = recording
        .script
        .events
        .iter()
        .find(|e| e.start <= frame && frame < e.end);
    if let Some(focused) = event.and_then(|event| event.focused) {
        for (entity, mut window) in &mut windows {
            if window.focused != focused {
                window.focused = focused;
                focus_events.write(WindowFocused {
                    window: entity,
                    focused,
                });
            }
        }
    }
    for mut gamepad in &mut pad {
        let movement = route_input.map_or_else(
            || event.map_or([0.0; 2], |event| event.move_axis),
            |(movement, _)| movement,
        );
        let look = event.map_or([0.0; 2], |event| event.look_axis);
        for (axis, value) in [
            (GamepadAxis::LeftStickX, movement[0]),
            (GamepadAxis::LeftStickY, movement[1]),
            (GamepadAxis::RightStickX, look[0]),
            (GamepadAxis::RightStickY, look[1]),
        ] {
            gamepad.analog_mut().set(axis, value);
        }
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
    if event.is_some_and(|e| e.left_mouse) {
        buttons.press(MouseButton::Left);
    } else {
        buttons.release(MouseButton::Left);
    }
    if let Some((_, look)) = route_input {
        motion.delta = Vec2::from_array(look);
    } else if let Some(event) = event {
        motion.delta = Vec2::from_array(event.look);
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy injects recorder, world, UI and asset state for evidence"
)]
fn record(
    mut commands: Commands,
    mut recording: ResMut<Recording>,
    loading: Option<Res<SceneLoading>>,
    phase: Option<Res<State<GamePhase>>>,
    load_error: Option<Res<GameLoadError>>,
    player: Option<Res<PlayerState>>,
    target: Res<CaptureTarget>,
    camera: Query<&Transform, With<Camera3d>>,
    #[cfg(feature = "viewer")] controller: Query<&FreeCameraState, With<Camera3d>>,
    world_entities: Query<(), With<MapSource>>,
    transforms: Query<(Entity, &Transform)>,
    pads: Query<&Gamepad, With<ScriptGamepad>>,
    mut exit: MessageWriter<AppExit>,
    ui_state: (
        Option<Res<SignalUi>>,
        Option<Res<PlaceHud>>,
        Option<Res<Observation>>,
        Option<Res<EntryUi>>,
        Option<Res<EntrySettings>>,
        Option<Res<PointerLock>>,
        Option<Res<GraphicsEvidence>>,
        Option<Res<ShopHandoff>>,
        Option<Res<CharacterStatus>>,
    ),
    ui_font: Option<Res<UiFont>>,
    assets: (
        Res<AssetServer>,
        Query<Entity>,
        Res<Assets<Mesh>>,
        Res<Assets<Image>>,
        Res<Assets<StandardMaterial>>,
    ),
) {
    let (ui, place, observation, entry, settings, pointer_lock, graphics, handoff, character) =
        ui_state;
    let (asset_server, entities, meshes, images, materials) = assets;
    if recording.finished {
        return;
    }
    if let Some(font) = &ui_font
        && let Some(bevy::asset::LoadState::Failed(error)) =
            asset_server.get_load_state(font.0.id())
    {
        recording.failure = Some(format!("UI font loading failed: {error}"));
    }
    if let Some(error) = character.as_ref().and_then(|c| c.error.as_ref()) {
        recording.failure = Some(format!("character loading: {error}"));
    }
    let world_ready = loading.as_ref().is_some_and(|loading| loading.ready);
    let capture_warmed_up = recording.warmup == 0;
    recording.metrics.observe(
        Instant::now(),
        loading.as_ref().map(|loading| loading.ready),
        world_ready
            && capture_warmed_up
            && phase
                .as_ref()
                .is_none_or(|phase| phase.get().as_str() == "world"),
    );
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
    if !recording.script.game_scene() && !world_ready {
        return;
    }
    if recording.warmup > 0 {
        recording.warmup -= 1;
        return;
    }
    if !recording.tick {
        return;
    }
    if player
        .as_ref()
        .is_some_and(|player| !player.foot.is_finite() || !player.camera_distance.is_finite())
    {
        recording.failure = Some("non-finite player state".into());
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
    let wall_elapsed_seconds = recording.started.elapsed().as_secs_f64();
    recording.samples.push(Sample {
        frame,
        simulation_seconds,
        wall_elapsed_seconds,
        resources: world_ready.then(|| ResourceCounts {
            entities: entities.iter().len(),
            meshes: meshes.len(),
            images: images.len(),
            materials: materials.len(),
        }),
        position: transform.translation.to_array(),
        rotation: transform.rotation.to_array(),
        enabled,
        game_page: phase.as_ref().map(|phase| phase.get().as_str().to_owned()),
        world_ready,
        world_entities: world_entities.iter().count(),
        gamepad_connected: !pads.is_empty(),
        pointer_locked: pointer_lock.as_ref().map(|pointer| pointer.active),
        graphics: graphics.as_ref().map(|graphics| (**graphics).clone()),
        player: player.as_deref().map(PlayerSample::from),
        handoff: handoff.as_deref().cloned(),
        character: character.as_deref().cloned(),
        observation: observation.as_ref().map(|observation| ObservationSample {
            target: observation.target.clone(),
            selected: observation.selected.clone(),
            open: observation.open,
            visible: observation.visible,
        }),
        entry: entry.zip(settings).map(|(entry, settings)| EntrySample {
            settings_open: entry.settings_open(),
            graphics_tab: entry.graphics_tab(),
            graphics_page: entry.graphics_page(),
            display_pending: entry.display_pending(),
            configured_resolution: settings.graphics.resolution,
            configured_borderless: settings.graphics.borderless,
            focus: entry.focus(),
            text_scale: settings.text_scale(),
            camera_sensitivity: settings.camera_sensitivity(),
            device: if entry.gamepad {
                "gamepad"
            } else {
                "keyboard_mouse"
            },
        }),
        place: place.as_ref().map(|hud| PlaceSample {
            id: hud.current_id.clone(),
            room: hud.room.clone(),
            name: hud.name.clone(),
            visible: hud.visible,
            gamepad: hud.gamepad,
        }),
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
    fn character_evidence_rejects_absent_failed_or_unplayed_assets() {
        let check: CharacterCheck = serde_json::from_value(serde_json::json!({
            "ready":true,"clip":"Walk","min_clip_elapsed":0.2,"paused":false
        }))
        .unwrap();
        assert!(check.valid());
        assert!(!check.matches(None));
        let mut actual = CharacterStatus::default();
        assert!(!check.matches(Some(&actual)));
        actual.ready = true;
        actual.clip = Some("Walk");
        actual.clip_elapsed = 0.3;
        assert!(check.matches(Some(&actual)));
        actual.error = Some("missing texture".into());
        assert!(!check.matches(Some(&actual)));
        for bad in [
            serde_json::json!({"ready":true,"clip":"Jump"}),
            serde_json::json!({"ready":true,"min_clip_elapsed":-1.0}),
        ] {
            assert!(
                !serde_json::from_value::<CharacterCheck>(bad)
                    .unwrap()
                    .valid()
            );
        }
    }

    #[test]
    fn handoff_assertions_require_measured_state_and_reject_invalid_contracts() {
        let mut check: HandoffCheck = serde_json::from_value(serde_json::json!({
            "stage":"observing", "observed_places":[], "confirmation_count":0, "rejected_choices":0
        }))
        .unwrap();
        assert!(check.valid());
        assert!(!check.matches(None));
        let mut actual = ShopHandoff::default();
        assert!(check.matches(Some(&actual)));
        actual.observed_places.insert("04".into());
        assert!(!check.matches(Some(&actual)));
        check.observed_places = Some(vec!["04".into()]);
        assert!(check.matches(Some(&actual)));
        actual.confirmation_count = 1;
        assert!(!check.matches(Some(&actual)));
        for bad in [
            serde_json::json!({}),
            serde_json::json!({"stage":"done"}),
            serde_json::json!({"observed_places":["04","04"]}),
            serde_json::json!({"observed_places":["99"]}),
        ] {
            assert!(!serde_json::from_value::<HandoffCheck>(bad).unwrap().valid());
        }
    }

    #[test]
    fn capture_metrics_measure_wall_time_and_exclude_load_and_pause_boundaries() {
        let start = Instant::now();
        let mut metrics = RuntimeMetrics::default();
        for (ms, ready, active) in [
            (0, false, false),
            (100, false, false),
            (300, true, true),
            (320, true, true),
            (360, true, true),
            (500, true, false),
            (600, true, true),
            (660, true, true),
        ] {
            metrics.observe(start + Duration::from_millis(ms), Some(ready), active);
        }
        assert_eq!(metrics.observed_load_seconds, [0.3]);
        assert_eq!(metrics.world_update_intervals_ms, [20.0, 40.0, 60.0]);
        let summary = interval_summary(metrics.world_update_intervals_ms);
        assert_eq!(summary["count"], 3);
        assert_eq!(summary["mean"], 40.0);
        assert_eq!(summary["p50"], 40.0);
        assert_eq!(summary["p95"], 60.0);
        assert_eq!(summary["max"], 60.0);
        assert!(interval_summary(vec![]).is_null());
        let single = interval_summary(vec![7.0]);
        for key in ["mean", "p50", "p95", "max"] {
            assert_eq!(single[key], 7.0);
        }
        assert_eq!(single["count"], 1);
    }

    #[test]
    fn connection_script_uses_input_plugin_and_preserves_disconnected_state() {
        let mut script: Script =
            serde_json::from_str(include_str!("../capture/game-entry.json")).unwrap();
        assert!(
            script
                .events
                .iter()
                .all(|event| event.gamepad_connected.is_none())
        );
        script.events[0].gamepad_connected = Some(false);
        assert!(script.validate().is_ok());
        let mut viewer: Script =
            serde_json::from_str(include_str!("../capture/viewer-tour.json")).unwrap();
        viewer.events[0].gamepad_connected = Some(false);
        assert!(viewer.validate().unwrap_err().contains("gamepad_connected"));
        assert!(
            serde_json::from_value::<InputSpan>(serde_json::json!({
                "start": 0, "end": 1, "gamepad_connected": "false"
            }))
            .is_err()
        );
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin)
            .insert_resource(Recording {
                script,
                output: PathBuf::new(),
                started: Instant::now(),
                metrics: RuntimeMetrics::default(),
                warmup: 0,
                tick: true,
                pending: false,
                saved: 0,
                transforms_checked: 0,
                world_ready_seen: false,
                route: None,
                samples: vec![],
                failure: None,
                finished: false,
            })
            .add_systems(PreUpdate, drive_connection.before(InputSystems));
        let pad = app
            .world_mut()
            .spawn((ScriptGamepad, Gamepad::default()))
            .id();
        let mut cursor = app
            .world()
            .resource::<Messages<GamepadConnectionEvent>>()
            .get_cursor();
        for (requested, tick, connected, emitted) in [
            (Some(false), false, true, None),
            (Some(false), true, false, Some(false)),
            (None, true, false, None),
            (Some(false), true, false, None),
            (Some(true), true, true, Some(true)),
            (None, true, true, None),
        ] {
            let mut recording = app.world_mut().resource_mut::<Recording>();
            recording.tick = tick;
            recording.script.events = requested
                .into_iter()
                .map(|value| {
                    serde_json::from_value(serde_json::json!({
                        "start": 0, "end": 1, "gamepad_connected": value
                    }))
                    .unwrap()
                })
                .collect();
            app.update();
            assert_eq!(app.world().get::<Gamepad>(pad).is_some(), connected);
            assert!(app.world().get_entity(pad).is_ok());
            let events: Vec<_> = cursor
                .read(app.world().resource::<Messages<GamepadConnectionEvent>>())
                .map(|event| (event.gamepad, event.connected()))
                .collect();
            assert_eq!(
                events,
                emitted
                    .into_iter()
                    .map(|value| (pad, value))
                    .collect::<Vec<_>>()
            );
        }
        assert!(!app.world().contains_resource::<State<GamePhase>>());
        assert!(!app.world().contains_resource::<PlayerState>());
    }

    #[test]
    fn focus_input_is_game_only_and_does_not_change_legacy_scripts() {
        let mut game: Script =
            serde_json::from_str(include_str!("../capture/game-entry.json")).unwrap();
        assert!(game.events.iter().all(|event| event.focused.is_none()));
        game.events[0].focused = Some(false);
        assert!(game.validate().is_ok());
        game.scene = "walk-preview".into();
        assert!(game.validate().is_ok());
        let mut viewer: Script =
            serde_json::from_str(include_str!("../capture/viewer-tour.json")).unwrap();
        viewer.events[0].focused = Some(false);
        assert!(viewer.validate().unwrap_err().contains("focused input"));
        assert!(
            serde_json::from_value::<InputSpan>(serde_json::json!({
                "start": 0, "end": 1, "focused": "false"
            }))
            .is_err()
        );
    }

    #[test]
    fn focus_changes_are_persistent_and_emit_native_messages_only_when_ticking() {
        for uses_focus in [false, true] {
            let mut script: Script =
                serde_json::from_str(include_str!("../capture/game-entry.json")).unwrap();
            script.events[0].focused = uses_focus.then_some(false);
            let recording = Recording {
                script,
                output: PathBuf::new(),
                started: Instant::now(),
                metrics: RuntimeMetrics::default(),
                warmup: 0,
                tick: true,
                pending: false,
                saved: 0,
                transforms_checked: 0,
                world_ready_seen: false,
                route: None,
                samples: vec![],
                failure: None,
                finished: false,
            };
            let mut app = App::new();
            app.init_resource::<Time<Virtual>>()
                .init_resource::<ButtonInput<KeyCode>>()
                .init_resource::<ButtonInput<MouseButton>>()
                .init_resource::<AccumulatedMouseMotion>();
            install(&mut app, recording);
            let windows: Vec<_> = app
                .world_mut()
                .query_filtered::<Entity, With<PrimaryWindow>>()
                .iter(app.world())
                .collect();
            assert_eq!(windows.len(), usize::from(uses_focus));
            if !uses_focus {
                app.world_mut().run_schedule(RunFixedMainLoop);
                continue;
            }
            let window = windows[0];
            assert!(app.world().get::<Window>(window).unwrap().focused);
            assert!(!app.world().get::<Window>(window).unwrap().visible);
            let mut cursor = app
                .world()
                .resource::<Messages<WindowFocused>>()
                .get_cursor();
            for (requested, tick, expected, emitted) in [
                (Some(false), false, true, None),
                (Some(false), true, false, Some(false)),
                (None, true, false, None),
                (Some(false), true, false, None),
                (Some(true), true, true, Some(true)),
                (None, true, true, None),
            ] {
                let mut recording = app.world_mut().resource_mut::<Recording>();
                recording.tick = tick;
                recording.script.events = requested
                    .into_iter()
                    .map(|focused| {
                        serde_json::from_value(serde_json::json!({
                            "start": 0, "end": 1, "focused": focused
                        }))
                        .unwrap()
                    })
                    .collect();
                app.world_mut().run_schedule(RunFixedMainLoop);
                let events: Vec<_> = cursor
                    .read(app.world().resource::<Messages<WindowFocused>>())
                    .map(|event| (event.window, event.focused))
                    .collect();
                assert_eq!(app.world().get::<Window>(window).unwrap().focused, expected);
                assert_eq!(
                    events,
                    emitted
                        .into_iter()
                        .map(|focused| (window, focused))
                        .collect::<Vec<_>>()
                );
            }
            assert!(!app.world().contains_resource::<State<GamePhase>>());
            assert!(!app.world().contains_resource::<PlayerState>());
        }
    }

    #[test]
    fn waits_hold_the_clock_and_script_inputs_reach_real_gamepad_axes() {
        let mut script: Script =
            serde_json::from_str(include_str!("../capture/game-entry.json")).unwrap();
        script.waits[0].frame = 0;
        let recording = Recording {
            script,
            output: PathBuf::new(),
            started: Instant::now(),
            metrics: RuntimeMetrics::default(),
            warmup: 0,
            tick: false,
            pending: false,
            saved: 0,
            transforms_checked: 0,
            world_ready_seen: false,
            route: None,
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
        let pad = app
            .world_mut()
            .spawn((ScriptGamepad, Gamepad::default()))
            .id();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .add_message::<WindowFocused>()
            .add_systems(Update, drive_input.after(advance_clock));
        {
            let mut recording = app.world_mut().resource_mut::<Recording>();
            recording.pending = false;
            recording.script.events = vec![InputSpan {
                start: 0,
                end: 1,
                keys: vec![
                    "R".into(),
                    "Space".into(),
                    "Shift".into(),
                    "ShiftRight".into(),
                    "M".into(),
                ],
                right_mouse: true,
                pointer: None,
                left_mouse: false,
                gamepad: vec!["Reset".into()],
                look: [0.0; 2],
                move_axis: [0.5, 1.0],
                look_axis: [-1.0, 0.25],
                focused: None,
                gamepad_connected: None,
            }];
        }
        app.update();
        let gamepad = app.world().get::<Gamepad>(pad).unwrap();
        assert_eq!(gamepad.left_stick(), Vec2::new(0.5, 1.0));
        assert_eq!(gamepad.right_stick(), Vec2::new(-1.0, 0.25));
        for key in [
            KeyCode::Space,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::KeyM,
        ] {
            assert!(
                app.world()
                    .resource::<ButtonInput<KeyCode>>()
                    .just_pressed(key)
            );
        }
        assert!(
            app.world()
                .resource::<ButtonInput<MouseButton>>()
                .pressed(MouseButton::Right)
        );
        assert!(gamepad.just_pressed(GamepadButton::Select));
        assert!(
            app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::KeyR)
        );
        app.world_mut()
            .resource_mut::<Recording>()
            .script
            .events
            .clear();
        app.update();
        let gamepad = app.world().get::<Gamepad>(pad).unwrap();
        assert_eq!(gamepad.left_stick(), Vec2::ZERO);
        assert_eq!(gamepad.right_stick(), Vec2::ZERO);
        assert!(!gamepad.pressed(GamepadButton::Select));
        assert!(
            !app.world()
                .resource::<ButtonInput<MouseButton>>()
                .pressed(MouseButton::Right)
        );
    }

    #[test]
    fn script_contract_rejects_unreproducible_inputs() {
        let script: Script =
            serde_json::from_str(include_str!("../capture/viewer-tour.json")).unwrap();
        assert!(script.validate().is_ok());
        let mut pointer = script.clone();
        pointer.events[0].left_mouse = true;
        assert!(pointer.validate().is_err());
        for position in [[-1.0, 0.0], [640.0, 20.0], [20.0, 360.0], [f32::NAN, 10.0]] {
            pointer.events[0].pointer = Some(position);
            assert!(pointer.validate().is_err());
        }
        pointer.events[0].pointer = Some([320.0, 180.0]);
        assert!(pointer.validate().is_ok());
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

    #[test]
    fn script_contract_validates_walk_axes_and_player_limits() {
        let script: Script =
            serde_json::from_str(include_str!("../capture/walk-preview.json")).unwrap();
        assert!(script.validate().is_ok());
        let ramp_script: Script =
            serde_json::from_str(include_str!("../capture/walk-ramp-camera.json")).unwrap();
        assert!(ramp_script.validate().is_ok());
        let ascent_script: Script =
            serde_json::from_str(include_str!("../capture/walk-ascent-entry.json")).unwrap();
        assert!(ascent_script.validate().is_ok());
        let mut full: Script =
            serde_json::from_str(include_str!("../capture/walk-ascent-full.json")).unwrap();
        assert!(full.validate().is_ok());
        let round_trip: Script =
            serde_json::from_str(include_str!("../capture/walk-round-trip.json")).unwrap();
        assert!(round_trip.validate().is_ok());
        assert!(round_trip.route.unwrap().round_trip);
        full.route.as_mut().unwrap().start = 30;
        assert!(full.validate().is_err());
        full.route.as_mut().unwrap().start = 60;
        full.scene = "game-entry".into();
        assert!(full.validate().is_err());
        let pause_script: Script =
            serde_json::from_str(include_str!("../capture/walk-pause.json")).unwrap();
        assert!(pause_script.validate().is_ok());
        let focus_script: Script =
            serde_json::from_str(include_str!("../capture/walk-focus.json")).unwrap();
        assert!(focus_script.validate().is_ok());
        let pad_script: Script =
            serde_json::from_str(include_str!("../capture/walk-gamepad.json")).unwrap();
        assert!(pad_script.validate().is_ok());
        let mut invalid = script.clone();
        invalid.events[0].move_axis = [1.01, 0.0];
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.events[0].look_axis = [0.0, f32::NAN];
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[0].max_player_distance = Some(-0.1);
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[0].min_player_height = Some(10.0);
        invalid.assertions[0].max_player_height = Some(9.0);
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[0].player_blocked = Some(String::new());
        assert!(invalid.validate().is_err());
        let mut invalid = script.clone();
        invalid.assertions[0].min_player_camera_distance = Some(2.0);
        invalid.assertions[0].max_player_camera_distance = Some(1.0);
        assert!(invalid.validate().is_err());
        let mut invalid = script;
        invalid.scene = "game-entry".into();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn rotation_limit_detects_a_turn_even_when_the_camera_returns() {
        let mut script: Script =
            serde_json::from_str(include_str!("../capture/walk-pause.json")).unwrap();
        script.frames = 3;
        script.events.clear();
        script.waits.clear();
        script.keyframes = vec![0];
        script.assertions = vec![
            serde_json::from_value(serde_json::json!({
                "name": "paused camera", "from": 0, "to": 2, "max_rotation": 0.01
            }))
            .unwrap(),
        ];
        assert!(script.validate().is_ok());
        script.assertions[0].max_rotation = Some(-0.1);
        assert!(script.validate().is_err());
        script.assertions[0].max_rotation = Some(0.01);
        script.assertions[0].min_rotation = Some(0.02);
        assert!(script.validate().is_err());
        script.assertions[0].min_rotation = None;
        let output = std::env::temp_dir().join(format!(
            "n-side-rotation-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&output).unwrap();
        let samples = [0.0, 0.5, 0.0]
            .into_iter()
            .enumerate()
            .map(|(frame, angle)| Sample {
                frame: frame as u32,
                simulation_seconds: frame as f64 / 30.0,
                wall_elapsed_seconds: frame as f64 / 30.0,
                resources: None,
                position: [0.; 3],
                rotation: Quat::from_rotation_y(angle).to_array(),
                enabled: false,
                game_page: Some("paused".into()),
                world_ready: true,
                world_entities: 1,
                player: None,
                ui: None,
                place: None,
                observation: None,
                handoff: None,
                character: None,
                entry: None,
                gamepad_connected: true,
                pointer_locked: Some(false),
                graphics: None,
            })
            .collect();
        let mut recording = Recording {
            script,
            output: output.clone(),
            started: Instant::now(),
            metrics: RuntimeMetrics::default(),
            warmup: 0,
            tick: false,
            pending: false,
            saved: 3,
            transforms_checked: 3,
            world_ready_seen: true,
            route: None,
            samples,
            failure: None,
            finished: false,
        };
        assert!(!recording.finish(None));
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("state.json")).unwrap()).unwrap();
        let check = report["checks"].as_array().unwrap().last().unwrap();
        assert_eq!(check["passed"], false);
        assert!(check["rotation_radians"].as_f64().unwrap() < 0.001);
        assert!(check["peak_rotation_radians"].as_f64().unwrap() > 0.49);
        recording.samples[1].rotation = Quat::IDENTITY.to_array();
        assert!(recording.finish(None));
        recording.script.assertions[0].gamepad_connected = Some(false);
        assert!(!recording.finish(None));
        recording.script.assertions[0].gamepad_connected = Some(true);
        assert!(recording.finish(None));
        recording.script.assertions[0].pointer_locked = Some(true);
        assert!(!recording.finish(None));
        recording.script.assertions[0].pointer_locked = Some(false);
        assert!(recording.finish(None));
        recording.samples[1].pointer_locked = None;
        assert!(!recording.finish(None));
        recording.samples[1].pointer_locked = Some(false);
        recording.script.scene = "walk-preview".into();
        recording.script.assertions = vec![
            serde_json::from_value(serde_json::json!({
                "name":"observation remains open", "from":0, "to":2,
                "observation_target":"04", "observation_selected":"04",
                "observation_open":true, "observation_visible":true
            }))
            .unwrap(),
        ];
        assert!(recording.script.validate().is_ok());
        for sample in &mut recording.samples {
            sample.observation = Some(ObservationSample {
                target: Some("04".into()),
                selected: Some("04".into()),
                open: true,
                visible: true,
            });
        }
        assert!(recording.finish(None));
        recording.samples[1].observation.as_mut().unwrap().open = false;
        assert!(
            !recording.finish(None),
            "middle-frame close must fail despite identical endpoints"
        );
        recording.samples[1].observation = None;
        assert!(
            !recording.finish(None),
            "missing actual observation cannot pass"
        );
        recording.script.scene = "game-entry".into();
        assert!(recording.script.validate().is_err());
        recording.script.scene = "walk-preview".into();
        recording.script.assertions[0].observation_target = Some(" ".into());
        assert!(recording.script.validate().is_err());
        fs::remove_dir_all(output).unwrap();
    }

    #[test]
    fn controls_and_graphics_assertions_require_measured_values() {
        let mut controls: Script =
            serde_json::from_str(include_str!("../capture/walk-controls.json")).unwrap();
        assert!(controls.validate().is_ok());
        for source in [
            include_str!("../capture/walk-graphics.json"),
            include_str!("../capture/walk-graphics-restore.json"),
            include_str!("../capture/walk-display-confirm.json"),
            include_str!("../capture/walk-display-restore.json"),
        ] {
            assert!(
                serde_json::from_str::<Script>(source)
                    .unwrap()
                    .validate()
                    .is_ok()
            );
        }
        controls.scene = "game-entry".into();
        assert!(controls.validate().is_err());
        let check: GraphicsCheck = serde_json::from_value(serde_json::json!({
            "aa":"fxaa", "ssao":"medium", "bloom":false, "shadows":false
        }))
        .unwrap();
        assert!(check.valid());
        let actual = GraphicsEvidence {
            aa: "fxaa".into(),
            ssao: "medium".into(),
            ..default()
        };
        assert!(check.matches(Some(&actual)));
        assert!(!check.matches(None));
        for field in ["aa", "ssao", "bloom", "shadows"] {
            let mut wrong = actual.clone();
            match field {
                "aa" => wrong.aa = "msaa4".into(),
                "ssao" => wrong.ssao = "off".into(),
                "bloom" => wrong.bloom = true,
                _ => wrong.shadows = true,
            }
            assert!(!check.matches(Some(&wrong)), "wrong {field} must fail");
        }
        for invalid in [
            serde_json::json!({}),
            serde_json::json!({"aa":"unknown"}),
            serde_json::json!({"ssao":"automatic"}),
        ] {
            let check: GraphicsCheck = serde_json::from_value(invalid).unwrap();
            assert!(!check.valid());
        }
    }

    #[test]
    fn settings_assertions_validate_ranges_and_reject_wrong_or_missing_evidence() {
        let mut script: Script =
            serde_json::from_str(include_str!("../capture/walk-settings.json")).unwrap();
        assert!(script.validate().is_ok());
        let expected = serde_json::json!({
            "name":"current settings", "from":0, "to":0,
            "settings_open":true, "entry_focus":1, "text_scale":1.25,
            "camera_sensitivity":0.65, "entry_device":"gamepad"
        });
        let sample = EntrySample {
            settings_open: true,
            graphics_tab: false,
            graphics_page: 0,
            display_pending: false,
            configured_resolution: 0,
            configured_borderless: false,
            focus: 1,
            text_scale: 1.25,
            camera_sensitivity: 0.65,
            device: "gamepad",
        };
        let check: Assertion = serde_json::from_value(expected.clone()).unwrap();
        assert!(check.check_entry(Some(&sample)));
        assert!(!check.check_entry(None));
        script.assertions = vec![check];
        script.waits.clear();
        script.events.clear();
        script.scene = "ui-signal".into();
        assert!(script.validate().is_err());
        script.scene = "game-entry".into();
        assert!(script.validate().is_ok());
        script.scene = "walk-preview".into();
        for (field, wrong, valid) in [
            ("settings_open", serde_json::json!(false), true),
            ("entry_focus", serde_json::json!(2), true),
            ("display_pending", serde_json::json!(true), true),
            ("configured_resolution", serde_json::json!(1), true),
            ("configured_borderless", serde_json::json!(true), true),
            ("configured_resolution", serde_json::json!(4), false),
            ("text_scale", serde_json::json!(1.0), true),
            ("camera_sensitivity", serde_json::json!(1.0), true),
            ("entry_device", serde_json::json!("keyboard_mouse"), true),
            ("entry_focus", serde_json::json!(21), false),
            ("text_scale", serde_json::json!(1.5), false),
            ("camera_sensitivity", serde_json::json!(0.0), false),
            ("entry_device", serde_json::json!("unknown"), false),
        ] {
            let mut candidate = expected.clone();
            candidate[field] = wrong;
            let check: Assertion = serde_json::from_value(candidate).unwrap();
            assert!(!check.check_entry(Some(&sample)), "wrong {field} must fail");
            script.assertions = vec![check];
            assert_eq!(script.validate().is_ok(), valid, "{field} range");
        }
        script.assertions[0].entry_device = None;
        script.assertions[0].text_scale = Some(f32::NAN);
        assert!(script.validate().is_err());
    }

    #[test]
    fn place_assertions_require_real_hud_and_validate_fields() {
        let mut script: Script =
            serde_json::from_str(include_str!("../capture/walk-places.json")).unwrap();
        assert!(script.validate().is_ok());
        let mut check: Assertion = serde_json::from_value(serde_json::json!({
            "name":"near shop", "from":0, "to":0,
            "place_id":"04", "place_name":"月台杂货与住家", "place_visible":true,
            "place_gamepad":false, "place_known":true
        }))
        .unwrap();
        let mut place = PlaceSample {
            id: Some("04".into()),
            room: None,
            name: "月台杂货与住家".into(),
            visible: true,
            gamepad: false,
        };
        assert!(check.check_place(Some(&place)));
        assert!(!check.check_place(None));
        check.place_room = Some("接待与陈列".into());
        assert!(!check.check_place(Some(&place)));
        place.room = Some("接待与陈列".into());
        assert!(check.check_place(Some(&place)));
        check.inside_room = Some(false);
        assert!(!check.check_place(Some(&place)));
        check.inside_room = Some(true);
        assert!(check.check_place(Some(&place)));
        check.place_room = None;
        check.inside_room = None;
        place.id = None;
        assert!(!check.check_place(Some(&place)));
        check.place_id = None;
        check.place_known = Some(false);
        assert!(check.check_place(Some(&place)));
        place.gamepad = true;
        assert!(!check.check_place(Some(&place)));
        script.scene = "game-entry".into();
        assert!(script.validate().is_err());
        script.scene = "walk-preview".into();
        script.assertions[0].place_id = Some(" ".into());
        assert!(script.validate().is_err());
    }

    #[test]
    fn player_checks_require_real_snapshots_and_bound_the_whole_interval() {
        let assertion: Assertion = serde_json::from_value(serde_json::json!({
            "name": "wall approach", "from": 0, "to": 2,
            "min_player_distance": 3.0, "max_player_distance": 5.0,
            "min_player_height": 28.0, "max_player_height": 28.5,
            "min_player_camera_distance": 4.0, "max_player_camera_distance": 7.0,
            "player_grounded": true, "max_player_resets": 0,
            "player_blocked": "V-04", "max_player_jumps":0, "player_sprinting":false
        }))
        .unwrap();
        let sample = |x| Sample {
            frame: 0,
            simulation_seconds: 0.0,
            wall_elapsed_seconds: 0.0,
            resources: None,
            position: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            enabled: false,
            game_page: Some("world".into()),
            world_ready: true,
            world_entities: 1,
            ui: None,
            place: None,
            observation: None,
            handoff: None,
            character: None,
            entry: None,
            gamepad_connected: true,
            pointer_locked: Some(true),
            graphics: None,
            player: Some(PlayerSample {
                foot: [x, 28.0, 0.0],
                grounded: true,
                jumps: 0,
                sprinting: false,
                blocked: None,
                resets: 0,
                camera_distance: 6.0,
            }),
        };
        let mut samples = [sample(0.0), sample(3.0), sample(4.0)];
        samples[1].player.as_mut().unwrap().blocked = Some("/buildings/V-04/wall".into());
        assert!(assertion.check_player(&samples));
        samples[2].player.as_mut().unwrap().jumps = 1;
        assert!(!assertion.check_player(&samples));
        samples[2].player.as_mut().unwrap().jumps = 0;
        samples[1].player.as_mut().unwrap().sprinting = true;
        assert!(!assertion.check_player(&samples));
        samples[1].player.as_mut().unwrap().sprinting = false;
        samples[1].player.as_mut().unwrap().camera_distance = 8.0;
        assert!(!assertion.check_player(&samples));
        samples[1].player.as_mut().unwrap().camera_distance = 3.0;
        assert!(!assertion.check_player(&samples));
        samples[1].player.as_mut().unwrap().camera_distance = 6.0;
        samples[1].player.as_mut().unwrap().foot[0] = 6.0;
        assert!(!assertion.check_player(&samples));
        samples[1].player.as_mut().unwrap().foot = [3.0, 27.9, 0.0];
        assert!(!assertion.check_player(&samples));
        samples[1].player.as_mut().unwrap().foot[1] = 28.0;
        samples[2].player.as_mut().unwrap().resets = 1;
        assert!(!assertion.check_player(&samples));
        samples[2].player.as_mut().unwrap().resets = 0;
        samples[1].player.as_mut().unwrap().blocked = None;
        assert!(!assertion.check_player(&samples));
        samples[1].player = None;
        assert!(!assertion.check_player(&samples));

        let mut check: CharacterCheck = serde_json::from_value(serde_json::json!({
            "ready": true, "clip":"Walk", "paused":false, "min_advance":0.4
        }))
        .unwrap();
        for sample in &mut samples {
            sample.character = Some(CharacterStatus {
                ready: true,
                clip: Some("Walk"),
                clip_elapsed: 0.2,
                ..default()
            });
        }
        assert!(!check.matches_range(&samples), "frozen animation must fail");
        samples[1].character.as_mut().unwrap().clip_elapsed = 0.5;
        samples[2].character.as_mut().unwrap().clip_elapsed = 0.7;
        assert!(
            !check.matches_range(&samples),
            "speed zero must fail despite elapsed growth"
        );
        samples[1].character.as_mut().unwrap().clip_time = 0.3;
        assert!(check.matches_range(&samples));
        check.min_advance = None;
        check.max_drift = Some(0.001);
        assert!(!check.matches_range(&samples), "pause cannot advance");
        samples[2].character.as_mut().unwrap().clip_elapsed = 0.2;
        assert!(
            !check.matches_range(&samples),
            "middle-frame drift must fail"
        );
        samples[1].character.as_mut().unwrap().clip_elapsed = 0.2;
        assert!(
            !check.matches_range(&samples),
            "seek-time drift cannot hide behind constant elapsed"
        );
        samples[1].character.as_mut().unwrap().clip_time = 0.0;
        assert!(check.matches_range(&samples));
        samples[1].character.as_mut().unwrap().transitions = 1;
        assert!(
            !check.matches_range(&samples),
            "clip restarts invalidate elapsed comparisons"
        );
    }
}
