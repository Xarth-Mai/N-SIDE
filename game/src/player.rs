//! Neutral exterior controller experiment; static mesh queries use the rendered world geometry
use crate::{
    app::{EntrySettings, GamePhase},
    capture::CaptureInput,
    places::Observation,
    ui::UiInput,
    world::{
        collision::{CollisionHit, CollisionWorld},
        map::{Map, map_to_world},
    },
};
use bevy::{
    app::RunFixedMainLoopSystems,
    input::{keyboard::KeyboardInput, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

const HEIGHT: f32 = 1.7;
const RADIUS: f32 = 0.3;
const SKIN: f32 = 0.02;
// Leave one millimeter beyond the query skin to avoid zero-time tangential hits
const SEPARATION: f32 = 0.001;
const STEP: f32 = 0.26;
const WALKABLE_Y: f32 = 0.70710677;
const SPEED: f32 = 3.2;
const SPRINT_SPEED: f32 = 5.6;
const JUMP_SPEED: f32 = 6.0;
const GRAVITY: f32 = 18.0;
const CAMERA_LENGTH: f32 = 3.8;

#[derive(Resource, Default)]
pub struct WalkPreview(pub bool);

#[derive(Resource)]
pub struct PlayerState {
    pub foot: Vec3,
    pub grounded: bool,
    pub blocked: Option<String>,
    pub resets: u32,
    pub camera_distance: f32,
    pub jumps: u32,
    pub sprinting: bool,
    spawn: Vec3,
    vertical_speed: f32,
    ground_normal: Vec3,
}

impl PlayerState {
    pub fn from_map(map: &Map, collision: &CollisionWorld) -> Result<Self, String> {
        let node = map
            .nodes
            .get("home")
            .ok_or("[player/spawn] missing /nodes/home")?;
        let origin = map_to_world(*node) + Vec3::Y * 0.7;
        let hit = collision
            .capsule_cast(origin, HEIGHT, RADIUS, -Vec3::Y * 1.5, SKIN)
            .filter(|hit| hit.surface_normal.y >= WALKABLE_Y)
            .ok_or("[player/spawn] no walkable capsule support at /nodes/home")?;
        let foot = origin - Vec3::Y * (1.5 * hit.fraction - SEPARATION).max(0.0);
        Ok(Self::at(foot))
    }

    fn at(foot: Vec3) -> Self {
        Self {
            foot,
            grounded: true,
            blocked: None,
            resets: 0,
            camera_distance: CAMERA_LENGTH,
            jumps: 0,
            sprinting: false,
            spawn: foot,
            vertical_speed: 0.0,
            ground_normal: Vec3::Y,
        }
    }

    fn reset(&mut self) {
        self.foot = self.spawn;
        self.vertical_speed = 0.0;
        self.ground_normal = Vec3::Y;
        self.grounded = true;
        self.blocked = None;
        self.sprinting = false;
        self.resets += 1;
    }

    fn jump(&mut self) {
        if self.grounded {
            self.vertical_speed = JUMP_SPEED;
            self.grounded = false;
            self.jumps += 1;
        }
    }

    fn step(&mut self, collision: &CollisionWorld, direction: Vec3, dt: f32) {
        self.blocked = None;
        let was_grounded = self.grounded;
        let speed = if self.sprinting { SPRINT_SPEED } else { SPEED };
        let mut delta = direction.clamp_length_max(1.0) * speed * dt;
        if was_grounded {
            // Keep the requested horizontal speed while following a walkable ramp
            delta.y = -self.ground_normal.dot(delta) / self.ground_normal.y.max(WALKABLE_Y);
        }
        let start = self.foot;
        let (moved, blocked) = slide(collision, start, delta);
        self.foot = moved;
        self.blocked = blocked;
        if was_grounded && self.blocked.is_some() && delta.xz().length_squared() > 0.000001 {
            // A step is accepted only after a clear upward sweep, forward sweep and supported landing
            if let Some(stepped) = step_up(collision, start, Vec3::new(delta.x, 0.0, delta.z))
                && stepped.xz().distance_squared(start.xz())
                    > self.foot.xz().distance_squared(start.xz()) + 0.000001
            {
                self.foot = stepped;
                self.blocked = None;
            }
        }
        let vertical = if was_grounded {
            -Vec3::Y * (STEP + SKIN)
        } else {
            self.vertical_speed -= GRAVITY * dt;
            Vec3::Y
                * if self.vertical_speed > 0.0 {
                    self.vertical_speed * dt
                } else {
                    (self.vertical_speed * dt).min(-SKIN)
                }
        };
        self.grounded = false;
        if let Some(hit) = collision.capsule_cast(self.foot, HEIGHT, RADIUS, vertical, SKIN) {
            if vertical.y > 0.0 {
                self.foot += vertical * safe_fraction(vertical, hit.fraction);
                self.vertical_speed = 0.0;
                self.blocked = Some(hit.source.to_owned());
            } else if let Some(normal) = walkable_normal(collision, &hit) {
                self.foot += vertical * safe_fraction(vertical, hit.fraction);
                self.grounded = true;
                self.ground_normal = normal;
                self.vertical_speed = 0.0;
            } else {
                self.foot = slide(collision, self.foot, vertical).0;
            }
        } else {
            self.foot += vertical;
        }
        let (min, max) = collision.bounds();
        if !self.foot.is_finite()
            || self.foot.y < min.y - 15.0
            || self.foot.x < min.x - 10.0
            || self.foot.x > max.x + 10.0
            || self.foot.z < min.z - 10.0
            || self.foot.z > max.z + 10.0
        {
            warn!(
                "[player/recover] outside static world bounds at {:?}",
                self.foot
            );
            self.reset();
        }
    }
}

fn walkable_normal(collision: &CollisionWorld, hit: &CollisionHit<'_>) -> Option<Vec3> {
    if hit.normal.y <= 0.01 {
        return None;
    }
    if hit.surface_normal.y >= WALKABLE_Y {
        return Some(hit.surface_normal);
    }
    // A rounded foot can contact the shared edge on the riser's triangle
    // Confirm the actual tread just inside that edge instead of accepting a wall as support
    // An almost-vertical rounded contact has an unstable horizontal direction
    // At a shared mesh edge, confirm both adjacent sides with the same short tread probe
    [hit.normal, hit.surface_normal, -hit.surface_normal]
        .into_iter()
        .find_map(|normal| {
            let inside = -Vec3::new(normal.x, 0.0, normal.z).normalize_or_zero() * 0.01;
            collision
                .support(hit.point + inside + Vec3::Y * 0.05, 0.1)
                .filter(|support| support.surface_normal.y >= WALKABLE_Y)
                .map(|support| support.surface_normal)
        })
}

fn safe_fraction(delta: Vec3, fraction: f32) -> f32 {
    (fraction - SEPARATION / delta.length().max(SEPARATION)).max(0.0)
}

fn slide(collision: &CollisionWorld, mut foot: Vec3, mut delta: Vec3) -> (Vec3, Option<String>) {
    let mut blocked = None;
    for _ in 0..4 {
        if delta.length_squared() < 0.0000001 {
            break;
        }
        let Some(hit) = collision.capsule_cast(foot, HEIGHT, RADIUS, delta, SKIN) else {
            foot += delta;
            break;
        };
        foot += delta * safe_fraction(delta, hit.fraction);
        delta *= 1.0 - hit.fraction;
        let normal = if hit.normal.y < WALKABLE_Y && hit.normal.y > 0.0 && delta.y >= 0.0 {
            Vec3::new(hit.normal.x, 0.0, hit.normal.z).normalize_or_zero()
        } else {
            hit.normal
        };
        delta -= normal * delta.dot(normal).min(0.0);
        if hit.fraction < 1.0e-4 {
            // Roundoff can leave the projected dot negative; re-query a micrometre outward
            delta += normal * 1.0e-6;
        }
        if hit.normal.y < WALKABLE_Y {
            blocked = Some(hit.source.to_owned());
        }
    }
    (foot, blocked)
}

fn step_up(collision: &CollisionWorld, foot: Vec3, horizontal: Vec3) -> Option<Vec3> {
    let up = Vec3::Y * STEP;
    if collision
        .capsule_cast(foot, HEIGHT, RADIUS, up, SKIN)
        .is_some()
    {
        return None;
    }
    let raised = foot + up;
    if collision
        .capsule_cast(raised, HEIGHT, RADIUS, horizontal, SKIN)
        .is_some()
    {
        return None;
    }
    let down = -Vec3::Y * (STEP + SKIN);
    let end = raised + horizontal;
    let hit = collision.capsule_cast(end, HEIGHT, RADIUS, down, SKIN)?;
    walkable_normal(collision, &hit).map(|_| end + down * safe_fraction(down, hit.fraction))
}

#[derive(Component)]
struct PlayerBody;
#[derive(Resource, Default)]
struct Intent {
    direction: Vec3,
    reset: bool,
    jump: bool,
    sprint: bool,
    wait_for_release: bool,
}
/// Gameplay lock intent; the window backend applies CursorOptions when a window exists
#[derive(Resource)]
pub(crate) struct PointerLock {
    pub requested: bool,
    pub active: bool,
}

impl Default for PointerLock {
    fn default() -> Self {
        Self {
            requested: true,
            active: false,
        }
    }
}

#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.22,
        }
    }
}

pub fn install(app: &mut App) {
    app.init_resource::<WalkPreview>()
        .init_resource::<EntrySettings>()
        .init_resource::<Intent>()
        .init_resource::<Orbit>()
        .init_resource::<PointerLock>()
        .add_systems(
            OnEnter(GamePhase::World),
            spawn_body.run_if(resource_exists::<PlayerState>),
        )
        .add_systems(OnExit(GamePhase::World), |mut intent: ResMut<Intent>| {
            *intent = Intent {
                wait_for_release: true,
                ..default()
            };
        })
        .add_systems(
            RunFixedMainLoop,
            (read_input, sync_pointer)
                .chain()
                .after(CaptureInput)
                .after(UiInput)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(
            FixedUpdate,
            move_player
                .run_if(gameplay_active)
                .run_if(resource_exists::<PlayerState>),
        )
        .add_systems(
            Update,
            show_player
                .run_if(gameplay_active)
                .run_if(resource_exists::<PlayerState>),
        );
}

fn gameplay_active(
    phase: Res<State<GamePhase>>,
    next: Res<NextState<GamePhase>>,
    observation: Option<Res<Observation>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) -> bool {
    *phase.get() == GamePhase::World
        && matches!(*next, NextState::Unchanged)
        && observation.is_none_or(|observation| !observation.blocks_world())
        && windows.iter().all(|window| window.focused)
}

pub fn clear(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<PlayerBody>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
    world.remove_resource::<PlayerState>();
    world.remove_resource::<CollisionWorld>();
    if let Some(mut pointer) = world.get_resource_mut::<PointerLock>() {
        *pointer = PointerLock::default();
    }
    if let Some(mut intent) = world.get_resource_mut::<Intent>() {
        *intent = Intent::default();
    }
    if let Some(mut orbit) = world.get_resource_mut::<Orbit>() {
        *orbit = Orbit::default();
    }
}

fn spawn_body(
    mut commands: Commands,
    player: Res<PlayerState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bodies: Query<(), With<PlayerBody>>,
) {
    if !bodies.is_empty() {
        return;
    }
    commands.spawn((
        PlayerBody,
        Mesh3d(meshes.add(Capsule3d::new(RADIUS, HEIGHT - RADIUS * 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.84, 0.95, 0.27),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_translation(player.foot + Vec3::Y * HEIGHT * 0.5),
    ));
}

fn stick(value: Vec2) -> Vec2 {
    let length = value.length();
    if !length.is_finite() || length <= 0.15 {
        Vec2::ZERO
    } else {
        value / length * ((length - 0.15) / 0.85).min(1.0)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy provides real keyboard, mouse, pad and focus input"
)]
fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    pads: Query<&Gamepad>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time<Virtual>>,
    next: Res<NextState<GamePhase>>,
    phase: Res<State<GamePhase>>,
    mut player: Option<ResMut<PlayerState>>,
    mut pointer: ResMut<PointerLock>,
    mut intent: ResMut<Intent>,
    mut orbit: ResMut<Orbit>,
    observation: Option<Res<Observation>>,
    settings: Res<EntrySettings>,
) {
    let repeated: Vec<_> = keyboard_events
        .read()
        .filter(|event| event.repeat)
        .map(|event| event.key_code)
        .collect();
    let pressed = |key| keys.just_pressed(key) && !repeated.contains(&key);
    intent.direction = Vec3::ZERO;
    intent.sprint = false;
    pointer.active = false;
    if let Some(player) = player.as_deref_mut() {
        player.sprinting = false;
    }
    if *phase.get() != GamePhase::World
        || player.is_none()
        || windows.iter().any(|window| !window.focused)
        || !matches!(*next, NextState::Unchanged)
        || observation.is_some_and(|observation| observation.blocks_world())
    {
        *intent = Intent {
            wait_for_release: true,
            ..default()
        };
        return;
    }
    let mut movement = Vec2::new(
        f32::from(keys.pressed(KeyCode::KeyD)) - f32::from(keys.pressed(KeyCode::KeyA)),
        f32::from(keys.pressed(KeyCode::KeyW)) - f32::from(keys.pressed(KeyCode::KeyS)),
    );
    let mut look = Vec2::new(
        f32::from(keys.pressed(KeyCode::KeyE)) - f32::from(keys.pressed(KeyCode::KeyQ)),
        0.0,
    );
    for pad in &pads {
        movement += stick(pad.left_stick());
        look += stick(pad.right_stick());
    }
    if intent.wait_for_release {
        intent.reset = false;
        intent.jump = false;
        intent.wait_for_release = keys.any_pressed([
            KeyCode::KeyW,
            KeyCode::KeyA,
            KeyCode::KeyS,
            KeyCode::KeyD,
            KeyCode::KeyQ,
            KeyCode::KeyE,
            KeyCode::KeyR,
            KeyCode::KeyM,
            KeyCode::Space,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ]) || buttons
            .any_pressed([MouseButton::Left, MouseButton::Right])
            || pads.iter().any(|pad| {
                pad.left_stick().length() > 0.15
                    || pad.right_stick().length() > 0.15
                    || pad.pressed(GamepadButton::Select)
                    || pad.pressed(GamepadButton::West)
                    || pad.pressed(GamepadButton::LeftThumb)
            });
        return;
    }
    if pressed(KeyCode::KeyM) {
        pointer.requested = !pointer.requested;
    }
    pointer.active = pointer.requested;
    let sensitivity = settings.camera_sensitivity();
    orbit.yaw += look.x * 1.6 * time.delta_secs() * sensitivity;
    orbit.pitch -= look.y * 1.2 * time.delta_secs() * sensitivity;
    if pointer.active || buttons.pressed(MouseButton::Left) {
        orbit.yaw -= motion.delta.x * 0.003 * sensitivity;
        orbit.pitch += motion.delta.y * 0.003 * sensitivity;
    }
    orbit.pitch = orbit.pitch.clamp(-0.12, 1.15);
    let forward = Vec3::new(-orbit.yaw.sin(), 0.0, -orbit.yaw.cos());
    let right = Vec3::new(orbit.yaw.cos(), 0.0, -orbit.yaw.sin());
    movement = movement.clamp_length_max(1.0);
    intent.direction = right * movement.x + forward * movement.y;
    intent.reset |= pressed(KeyCode::KeyR)
        || pads
            .iter()
            .any(|pad| pad.just_pressed(GamepadButton::Select));
    intent.jump |=
        pressed(KeyCode::Space) || pads.iter().any(|pad| pad.just_pressed(GamepadButton::West));
    intent.sprint = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        || buttons.pressed(MouseButton::Right)
        || pads.iter().any(|pad| pad.pressed(GamepadButton::LeftThumb));
}

fn sync_pointer(
    pointer: Res<PointerLock>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for mut cursor in &mut windows {
        if cursor.visible == pointer.active {
            cursor.visible = !pointer.active;
        }
        let grab_mode = if pointer.active {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        if cursor.grab_mode != grab_mode {
            cursor.grab_mode = grab_mode;
        }
    }
}

fn move_player(
    mut player: ResMut<PlayerState>,
    collision: Res<CollisionWorld>,
    mut intent: ResMut<Intent>,
    time: Res<Time<Fixed>>,
) {
    if intent.reset {
        player.reset();
        intent.reset = false;
        intent.jump = false;
    }
    if std::mem::take(&mut intent.jump) {
        player.jump();
    }
    player.sprinting = intent.sprint && intent.direction.length_squared() > 0.0;
    player.step(&collision, intent.direction, time.delta_secs());
}

#[expect(
    clippy::type_complexity,
    reason = "Bevy filters keep actor and camera mutable transforms disjoint"
)]
fn show_player(
    mut player: ResMut<PlayerState>,
    collision: Res<CollisionWorld>,
    orbit: Res<Orbit>,
    mut body: Query<(&mut Transform, &mut Visibility), (With<PlayerBody>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<PlayerBody>)>,
) {
    let pivot = player.foot + Vec3::Y * 1.35;
    let offset = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.cos() * orbit.pitch.cos(),
    ) * CAMERA_LENGTH;
    let distance = collision
        .sphere_cast(pivot, 0.18, offset, 0.04)
        .map_or(1.0, |hit| hit.fraction);
    player.camera_distance = CAMERA_LENGTH * distance;
    if let Ok((mut body, mut visibility)) = body.single_mut() {
        body.translation = player.foot + Vec3::Y * HEIGHT * 0.5;
        // The neutral proxy must not cover the viewport when a wall forces the camera inside it
        *visibility = if player.camera_distance < 0.8 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    if let Ok(mut camera) = camera.single_mut() {
        // Collapse immediately on obstruction; never interpolate through a wall
        *camera = Transform::from_translation(pivot + offset * distance.max(0.001))
            .looking_at(pivot, Vec3::Y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::geometry::GeometryPart;
    use std::path::Path;

    fn block(source: &str, size: Vec3, center: Vec3) -> GeometryPart {
        GeometryPart {
            source: source.into(),
            material: "test".into(),
            mesh: Mesh::from(Cuboid::from_size(size)).translated_by(center),
        }
    }

    fn input_app() -> (App, Entity) {
        let mut app = App::new();
        let collision = CollisionWorld::from_parts(&[block(
            "/terrain",
            Vec3::new(100., 1., 100.),
            Vec3::new(0., -0.5, 0.),
        )])
        .unwrap();
        let mut fixed = Time::<Fixed>::default();
        fixed.advance_by(std::time::Duration::from_secs_f32(1. / 60.));
        app.insert_resource(collision)
            .insert_resource(PlayerState::at(Vec3::Y * (SKIN + SEPARATION)))
            .insert_resource(State::new(GamePhase::World))
            .init_resource::<NextState<GamePhase>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<Time<Virtual>>()
            .insert_resource(fixed)
            .init_resource::<EntrySettings>()
            .init_resource::<Intent>()
            .init_resource::<Orbit>()
            .init_resource::<PointerLock>()
            .init_resource::<Observation>()
            .add_message::<KeyboardInput>()
            .add_systems(
                Update,
                (
                    read_input,
                    sync_pointer,
                    move_player.run_if(gameplay_active),
                )
                    .chain(),
            );
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        (app, window)
    }

    fn frame(app: &mut App) {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::ZERO;
    }

    #[test]
    fn jump_is_one_press_lands_and_stops_at_ceiling() {
        let (mut app, _) = input_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        let mut peak = 0_f32;
        for tick in 0..90 {
            // A second press in the air must not renew vertical speed
            if tick == 5 {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .release(KeyCode::Space);
            }
            if tick == 6 {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::Space);
            }
            frame(&mut app);
            peak = peak.max(app.world().resource::<PlayerState>().foot.y);
        }
        let player = app.world().resource::<PlayerState>();
        assert!((0.9..1.05).contains(&peak), "peak={peak}");
        assert_eq!(
            player.jumps, 1,
            "holding jump must not auto-jump on landing"
        );
        assert!(player.grounded && player.foot.y < 0.03);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Space);
        frame(&mut app);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        frame(&mut app);
        assert_eq!(app.world().resource::<PlayerState>().jumps, 2);
        assert!(!app.world().resource::<PlayerState>().grounded);

        let collision = CollisionWorld::from_parts(&[
            block("/terrain", Vec3::new(10., 1., 10.), Vec3::new(0., -0.5, 0.)),
            block(
                "/buildings/ceiling",
                Vec3::new(10., 0.2, 10.),
                Vec3::new(0., 2.1, 0.),
            ),
        ])
        .unwrap();
        let mut player = PlayerState::at(Vec3::Y * (SKIN + SEPARATION));
        player.jump();
        let mut ceiling_contact = false;
        for _ in 0..90 {
            player.step(&collision, Vec3::ZERO, 1. / 60.);
            assert!(player.foot.y + HEIGHT + SKIN <= 2.001);
            ceiling_contact |= player.blocked.as_deref() == Some("/buildings/ceiling");
        }
        assert!(ceiling_contact && player.grounded && player.resets == 0);
    }

    #[test]
    fn sprint_bindings_are_equivalent_and_diagonal_speed_is_normalized() {
        let mut distances = Vec::new();
        for (shift, mouse, diagonal) in [
            (None, false, false),
            (Some(KeyCode::ShiftLeft), false, false),
            (Some(KeyCode::ShiftRight), false, false),
            (None, true, false),
            (None, true, true),
        ] {
            let (mut app, _) = input_app();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::KeyW);
            if diagonal {
                keys.press(KeyCode::KeyD);
            }
            if let Some(shift) = shift {
                keys.press(shift);
            }
            if mouse {
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(MouseButton::Right);
            }
            for _ in 0..60 {
                frame(&mut app);
            }
            let player = app.world().resource::<PlayerState>();
            distances.push(player.foot.xz().length());
            assert_eq!(player.sprinting, shift.is_some() || mouse);
        }
        // Sweeps can lose one separation skin at a shared triangle edge
        assert!(
            (distances[0] - SPEED).abs() < SEPARATION * 2.0,
            "{distances:?}"
        );
        assert!(
            distances[1..]
                .iter()
                .all(|distance| (*distance - SPRINT_SPEED).abs() < SEPARATION * 2.0),
            "{distances:?}"
        );
    }

    #[test]
    fn lock_controls_camera_and_releases_for_observation_pause_and_focus() {
        let (mut app, window) = input_app();
        frame(&mut app);
        assert!(app.world().resource::<PointerLock>().active);
        assert_eq!(
            app.world().get::<CursorOptions>(window).unwrap().grab_mode,
            CursorGrabMode::Locked
        );
        assert!(!app.world().get::<CursorOptions>(window).unwrap().visible);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(20., 0.);
        frame(&mut app);
        let yaw = app.world().resource::<Orbit>().yaw;
        assert!(yaw < -0.05);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyM);
        frame(&mut app);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyM);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(20., 0.);
        frame(&mut app);
        assert_eq!(app.world().resource::<Orbit>().yaw, yaw);
        assert!(app.world().get::<CursorOptions>(window).unwrap().visible);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(20., 0.);
        frame(&mut app);
        assert!(app.world().resource::<Orbit>().yaw < yaw);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyM);
        frame(&mut app);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyM);
        for gate in 0..3 {
            let before = app.world().resource::<PlayerState>().foot;
            let yaw = app.world().resource::<Orbit>().yaw;
            match gate {
                0 => app.world_mut().resource_mut::<Observation>().open = true,
                1 => {
                    app.insert_resource(State::new(GamePhase::Paused));
                }
                _ => app.world_mut().get_mut::<Window>(window).unwrap().focused = false,
            }
            for key in [KeyCode::Space, KeyCode::ShiftLeft, KeyCode::KeyW] {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
            }
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Right);
            app.world_mut()
                .resource_mut::<AccumulatedMouseMotion>()
                .delta = Vec2::new(200., 100.);
            frame(&mut app);
            assert!(!app.world().resource::<PointerLock>().active);
            assert_eq!(
                app.world().get::<CursorOptions>(window).unwrap().grab_mode,
                CursorGrabMode::None
            );
            assert!(app.world().get::<CursorOptions>(window).unwrap().visible);
            app.world_mut().resource_mut::<Observation>().open = false;
            app.insert_resource(State::new(GamePhase::World));
            app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
            for _ in 0..3 {
                frame(&mut app);
            }
            assert_eq!(app.world().resource::<PlayerState>().foot, before);
            assert_eq!(app.world().resource::<PlayerState>().jumps, 0);
            assert_eq!(app.world().resource::<Orbit>().yaw, yaw);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release_all();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release_all();
            frame(&mut app);
            frame(&mut app);
            assert!(app.world().resource::<PointerLock>().active);
        }
    }

    #[test]
    fn body_stops_at_wall_can_leave_and_does_not_climb_tall_obstacle() {
        let world = CollisionWorld::from_parts(&[
            block(
                "/terrain",
                Vec3::new(30.0, 1.0, 30.0),
                Vec3::new(0.0, -0.5, 0.0),
            ),
            block(
                "/buildings/0",
                Vec3::new(0.1, 4.0, 20.0),
                Vec3::new(2.0, 2.0, 0.0),
            ),
        ])
        .unwrap();
        let mut player = PlayerState::at(Vec3::Y * SKIN);
        for _ in 0..120 {
            player.step(&world, Vec3::X, 1.0 / 60.0);
        }
        assert!((1.60..1.64).contains(&player.foot.x), "{:?}", player.foot);
        assert!(player.grounded && player.blocked.is_some());
        assert!(player.foot.y < 0.03);
        let before_slide = player.foot;
        for _ in 0..60 {
            player.step(&world, Vec3::Z, 1.0 / 60.0);
        }
        assert!(
            player.foot.z - before_slide.z > 3.0 && (player.foot.x - before_slide.x).abs() < 0.01,
            "{:?}",
            player.foot
        );
        for _ in 0..30 {
            player.step(&world, -Vec3::X, 1.0 / 60.0);
        }
        assert!(player.foot.x < 0.1 && player.blocked.is_none());
        player.foot.y = -30.0;
        player.step(&world, Vec3::ZERO, 1.0 / 60.0);
        assert_eq!(player.resets, 1);
        assert_eq!(player.foot, player.spawn);
    }

    #[test]
    fn steps_need_headroom_and_a_supported_landing() {
        let parts = vec![
            block(
                "/terrain",
                Vec3::new(30.0, 1.0, 30.0),
                Vec3::new(0.0, -0.5, 0.0),
            ),
            block(
                "/roads/0",
                Vec3::new(2.0, 0.2, 4.0),
                Vec3::new(2.0, 0.1, 0.0),
            ),
        ];
        let world = CollisionWorld::from_parts(&parts).unwrap();
        let mut player = PlayerState::at(Vec3::Y * SKIN);
        for _ in 0..38 {
            player.step(&world, Vec3::X, 1.0 / 60.0);
        }
        assert!(
            player.foot.x > 1.9 && player.foot.y > 0.21 && player.grounded,
            "{:?}",
            player.foot
        );
        let mut low_ceiling = parts;
        low_ceiling.push(block(
            "/buildings/1",
            Vec3::new(4.0, 0.2, 4.0),
            Vec3::new(2.0, 1.87, 0.0),
        ));
        let world = CollisionWorld::from_parts(&low_ceiling).unwrap();
        let mut player = PlayerState::at(Vec3::Y * SKIN);
        for _ in 0..60 {
            player.step(&world, Vec3::X, 1.0 / 60.0);
        }
        assert!(
            player.foot.x < 1.0 && player.foot.y < 0.06,
            "{:?}",
            player.foot
        );
    }

    #[test]
    fn steep_face_cannot_be_climbed_by_repeated_edge_contacts() {
        use bevy::{
            asset::RenderAssetUsages,
            mesh::{Indices, PrimitiveTopology},
        };
        let ramp = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [1., 0., -2.],
                [1., 0., 2.],
                [5., 6.9282, 2.],
                [5., 6.9282, -2.],
            ],
        )
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
        let world = CollisionWorld::from_parts(&[
            block("/terrain", Vec3::new(30., 1., 30.), Vec3::new(0., -0.5, 0.)),
            GeometryPart {
                source: "/roads/steep".into(),
                material: "test".into(),
                mesh: ramp,
            },
        ])
        .unwrap();
        let mut player = PlayerState::at(Vec3::Y * (SKIN + SEPARATION));
        for _ in 0..600 {
            player.step(&world, Vec3::X, 1.0 / 60.0);
        }
        assert!(
            player.foot.x < 1.2 && player.foot.y < 0.25,
            "{:?}",
            player.foot
        );
        assert_eq!(player.resets, 0);
    }

    #[test]
    fn close_camera_hides_proxy_and_clear_view_restores_it() {
        let collision = CollisionWorld::from_parts(&[block(
            "/buildings/0",
            Vec3::new(0.1, 4., 10.),
            Vec3::new(-0.5, 2., 0.),
        )])
        .unwrap();
        let mut app = App::new();
        app.insert_resource(collision)
            .insert_resource(PlayerState::at(Vec3::Y * SKIN))
            .insert_resource(Orbit {
                yaw: -std::f32::consts::FRAC_PI_2,
                pitch: 0.22,
            })
            .add_systems(Update, show_player);
        let body = app
            .world_mut()
            .spawn((PlayerBody, Transform::default(), Visibility::Visible))
            .id();
        app.world_mut()
            .spawn((Camera3d::default(), Transform::default()));
        app.update();
        assert!(app.world().resource::<PlayerState>().camera_distance < 0.8);
        assert_eq!(
            *app.world().get::<Visibility>(body).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().resource_mut::<Orbit>().yaw = 0.0;
        app.update();
        assert!(
            (app.world().resource::<PlayerState>().camera_distance - CAMERA_LENGTH).abs() < 0.001
        );
        assert_eq!(
            *app.world().get::<Visibility>(body).unwrap(),
            Visibility::Visible
        );
    }

    #[test]
    fn ramp_contacts_keep_moving_without_passing_a_wall_or_tall_step() {
        use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology};
        let origin = Vec3::new(110., 20., -260.);
        for (name, obstacle) in [
            ("clear", None),
            (
                "wall",
                Some(block(
                    "/buildings/0",
                    Vec3::new(0.1, 4., 10.),
                    origin + Vec3::new(5., 2.5, 0.),
                )),
            ),
            (
                "tall-step",
                Some(block(
                    "/roads/1",
                    Vec3::new(2., 1., 10.),
                    origin + Vec3::new(6., 1., 0.),
                )),
            ),
        ] {
            let mut parts = vec![GeometryPart {
                source: "/roads/0".into(),
                material: "test".into(),
                mesh: Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        [0., 0., -5.],
                        [0., 0., 5.],
                        [30., 3., 5.],
                        [0., 0., -5.],
                        [30., 3., 5.],
                        [30., 3., -5.],
                    ],
                )
                .translated_by(origin),
            }];
            parts.extend(obstacle);
            let collision = CollisionWorld::from_parts(&parts).unwrap();
            for dt in [1.0 / 64.0, 1.0 / 30.0] {
                let mut player = PlayerState::at(origin + Vec3::new(1., 0.13, 0.));
                for _ in 0..(6.0 / dt) as usize {
                    player.step(&collision, Vec3::X, dt);
                }
                eprintln!(
                    "[player/ramp-obstacle] name={name} dt={dt} foot={:?} blocked={:?}",
                    player.foot, player.blocked
                );
                assert!(
                    player.grounded && player.resets == 0,
                    "{name}: {:?}",
                    player.foot
                );
                if name == "clear" {
                    assert!(player.foot.x > origin.x + 18., "{name}: {:?}", player.foot);
                } else {
                    assert!(
                        (origin.x + 4.5..origin.x + 4.8).contains(&player.foot.x),
                        "{name}: {:?}",
                        player.foot
                    );
                    assert!(player.foot.y < origin.y + 0.7, "{name}: {:?}", player.foot);
                    assert!(player.blocked.is_some(), "{name}: {:?}", player.foot);
                }
            }
        }
    }

    #[test]
    fn real_shop_walk_reaches_steps_and_wall_through_swept_geometry() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let prepared = crate::world::scene::PreparedScene::load(root).unwrap();
        let world = CollisionWorld::from_parts(&prepared.parts).unwrap();
        let mut player = PlayerState::from_map(&prepared.map, &world).unwrap();
        for _ in 0..360 {
            player.step(&world, Vec3::new(0.341574, 0.0, -0.939855), 1.0 / 60.0);
        }
        eprintln!(
            "[player/steps] foot={:?} grounded={} blocked={:?}",
            player.foot, player.grounded, player.blocked
        );
        assert!(player.foot.z < -270.0 && player.foot.y > 29.0 && player.grounded);
        player.reset();
        for _ in 0..360 {
            player.step(&world, Vec3::new(0.8944272, 0.0, 0.4472136), 1.0 / 60.0);
        }
        eprintln!(
            "[player/ramp] foot={:?} grounded={} blocked={:?}",
            player.foot, player.grounded, player.blocked
        );
        assert!(
            player.foot.x > 115.0
                && player.foot.y < 27.5
                && player.foot.y > 26.5
                && player.grounded
        );
        player.reset();
        // The public door is open; the adjacent facade must still block the player
        for _ in 0..94 {
            player.step(&world, Vec3::Z, 1.0 / 60.0);
        }
        for _ in 0..360 {
            player.step(&world, -Vec3::X, 1.0 / 60.0);
        }
        eprintln!(
            "[player/wall] foot={:?} grounded={} blocked={:?}",
            player.foot, player.grounded, player.blocked
        );
        assert!((88.3..88.8).contains(&player.foot.x) && player.grounded);
        assert!(
            player
                .blocked
                .as_deref()
                .is_some_and(|source| source.contains("V-04"))
        );
        for dt in [1.0 / 64.0, 1.0 / 30.0] {
            let mut player = PlayerState::from_map(&prepared.map, &world).unwrap();
            for _ in 0..(35.0 / dt) as usize {
                player.step(&world, Vec3::new(0.8, 0.0, -0.6), dt);
            }
            eprintln!(
                "[player/ascent-entry] dt={dt} foot={:?} grounded={} blocked={:?} resets={}",
                player.foot, player.grounded, player.blocked, player.resets
            );
            assert!(
                player.foot.x > 180.0 && player.foot.y > 41.0,
                "{:?}",
                player.foot
            );
            assert!(player.grounded && player.resets == 0);
        }
    }

    #[test]
    fn walks_shop_public_rooms_and_returns_without_opening_private_rooms() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let scene = crate::world::scene::PreparedScene::load(root).unwrap();
        let collision = CollisionWorld::from_parts(&scene.parts).unwrap();
        let catalog = crate::places::PlaceCatalog::load(
            &root.join("source-assets/district-map/district.json"),
        )
        .unwrap();
        let walk_to = |player: &mut PlayerState, target: Vec3, dt: f32| {
            for _ in 0..(12.0 / dt) as usize {
                let delta = (target - player.foot) * Vec3::new(1., 0., 1.);
                if delta.length() < 0.025 {
                    break;
                }
                player.step(&collision, (delta / (SPEED * dt)).clamp_length_max(1.), dt);
            }
            assert!(
                player.foot.xz().distance(target.xz()) < 0.03,
                "target={target:?} foot={:?} blocked={:?}",
                player.foot,
                player.blocked
            );
            assert!(
                player.grounded && player.resets == 0,
                "foot={:?}",
                player.foot
            );
            assert!((player.foot.y - 28.0).abs() < 0.15);
        };
        for dt in [1. / 64., 1. / 30.] {
            let mut player = PlayerState::from_map(&scene.map, &collision).unwrap();
            for (x, y, room) in [
                (86., 255., Some("接待与陈列")),
                (86., 259.1, Some("接待与陈列")),
                (81., 259.1, Some("预约洽谈")),
                (86., 259.1, Some("接待与陈列")),
                (86., 263.6, Some("接待与陈列")),
                (81., 263.6, Some("主题陈列")),
                (86., 263.6, Some("接待与陈列")),
                (86., 255., Some("接待与陈列")),
                (100., 255., None),
            ] {
                walk_to(&mut player, Vec3::new(x, 28., -y), dt);
                assert_eq!(catalog.room_at(player.foot), room);
            }
            walk_to(&mut player, Vec3::new(86., 28., -255.), dt);
            for _ in 0..(2.0 / dt) as usize {
                player.step(&collision, -Vec3::X, dt);
            }
            player.jump();
            for _ in 0..(2.0 / dt) as usize {
                player.step(&collision, -Vec3::X, dt);
            }
            assert!(
                player.foot.x > 84.3 && player.foot.x < 84.6,
                "closed rear: {:?}",
                player.foot
            );
            assert!(player.grounded && player.resets == 0);
        }
    }

    #[test]
    fn settles_after_descending_near_the_summit() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let prepared = crate::world::scene::PreparedScene::load(root).unwrap();
        let collision = CollisionWorld::from_parts(&prepared.parts).unwrap();
        // Real 448m/386m landings expose near-vertical contacts on opposite riser faces
        for (node, start) in [
            (
                "hill_short_summit_wait",
                Vec3::new(258.2358, 448.49008, -933.72894),
            ),
            (
                "stair_hill_short_rest2_departure_hill_short_rest3_arrival_08_in",
                Vec3::new(255.9629, 386.86414, -857.21936),
            ),
        ] {
            let target = map_to_world(prepared.map.nodes[node]);
            for dt in [1.0 / 64.0, 1.0 / 30.0] {
                for vertical_speed in [-0.6, -36.000004] {
                    let mut player = PlayerState::at(start);
                    player.grounded = false;
                    player.vertical_speed = vertical_speed;
                    for _ in 0..60 {
                        let horizontal =
                            Vec3::new(target.x - player.foot.x, 0.0, target.z - player.foot.z);
                        player.step(
                            &collision,
                            (horizontal / (SPEED * dt)).clamp_length_max(1.0),
                            dt,
                        );
                        assert!(
                            player.grounded,
                            "node={node} dt={dt} initial_v={vertical_speed} foot={:?}",
                            player.foot
                        );
                        assert_eq!(player.vertical_speed, 0.0);
                        assert_eq!(player.resets, 0);
                        assert!(player.foot.xz().distance(target.xz()) < 0.001);
                        assert!((player.foot.y - start.y).abs() < 0.002);
                    }
                }
            }
        }
    }

    #[test]
    fn walks_complete_short_ascent_and_return() {
        let started = std::time::Instant::now();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let source: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("source-assets/district-map/district.json"))
                .unwrap(),
        )
        .unwrap();
        let route = source["routes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|route| route["id"] == "hill-short")
            .unwrap();
        let mut nodes: Vec<&str> = route["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| node.as_str().unwrap())
            .collect();
        assert_eq!(nodes.first(), Some(&"home"));
        assert_eq!(nodes.last(), Some(&"summit"));
        nodes.extend(nodes.clone().into_iter().rev().skip(1));

        let prepared = crate::world::scene::PreparedScene::load(root).unwrap();
        let collision = CollisionWorld::from_parts(&prepared.parts).unwrap();
        assert_eq!(
            Time::<Fixed>::default().timestep().as_secs_f32(),
            1.0 / 64.0
        );
        let preparation_seconds = started.elapsed().as_secs_f64();
        for dt in [1.0 / 64.0, 1.0 / 30.0] {
            let mut player = PlayerState::from_map(&prepared.map, &collision).unwrap();
            let mut tick = 0_u32;
            // A route node can lie at a tread/platform boundary; allow one generated riser plus skin
            let height_tolerance = 0.17 + SKIN + SEPARATION;
            eprintln!(
                "[ascent/start] route=hill-short nodes={} triangles={} dt={dt} height_tolerance={height_tolerance} preparation_seconds={:.3} spawn={:?}",
                nodes.len(),
                collision.triangle_count(),
                preparation_seconds,
                player.foot
            );
            for (index, pair) in nodes.windows(2).enumerate() {
                let [from, node] = [pair[0], pair[1]];
                let roads: Vec<usize> = prepared
                    .map
                    .roads
                    .iter()
                    .enumerate()
                    .filter_map(|(index, road)| {
                        road.nodes
                            .windows(2)
                            .any(|edge| {
                                (edge[0] == from && edge[1] == node)
                                    || (edge[0] == node && edge[1] == from)
                            })
                            .then_some(index)
                    })
                    .collect();
                assert_eq!(
                    roads.len(),
                    1,
                    "hill-short edge {from} -> {node}: roads={roads:?}"
                );
                let target = map_to_world(prepared.map.nodes[node]);
                // Node height bounds this observation only; it never sets the player's position
                let origin = target + Vec3::Y * 0.7;
                let exact_ray = collision.support(origin, 1.5);
                let centered_support = exact_ray.filter(|hit| hit.surface_normal.y >= WALKABLE_Y);
                // A ray exactly on a mesh seam can miss both neighboring upward triangles
                let support = centered_support.or_else(|| {
                    [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z]
                        .into_iter()
                        .filter_map(|offset| collision.support(origin + offset * 0.01, 1.5))
                        .find(|hit| {
                            hit.surface_normal.y >= WALKABLE_Y
                                && (hit.point.y - target.y).abs() <= 0.17 + 0.025
                        })
                });
                if centered_support.is_none() {
                    let capsule =
                        collision.capsule_cast(origin, HEIGHT, RADIUS, -Vec3::Y * 1.5, SKIN);
                    eprintln!(
                        "[ascent/target-seam] node={node} target={target:?} exact_ray={exact_ray:?} confirmed={support:?} capsule={capsule:?} capsule_foot={:?}",
                        capsule.map(|hit| origin - Vec3::Y * 1.5 * hit.fraction)
                    );
                }
                let support = support.unwrap_or_else(|| {
                    panic!("hill-short node {node} roads={roads:?}: no actual walkable target mesh")
                });
                let mut best_remaining = f32::INFINITY;
                let mut progress_tick = tick;
                loop {
                    let horizontal =
                        Vec3::new(target.x - player.foot.x, 0.0, target.z - player.foot.z);
                    let distance = horizontal.length();
                    let height_error = (player.foot.y - support.point.y).abs();
                    let remaining = distance + (height_error - height_tolerance).max(0.0);
                    if best_remaining - remaining >= 0.01 {
                        best_remaining = remaining;
                        progress_tick = tick;
                    }
                    let seconds = tick as f32 * dt;
                    let failure = if player.resets != 0 {
                        Some("automatic recovery")
                    } else if seconds >= 900.0 {
                        Some("900 simulated seconds exhausted")
                    } else if (tick - progress_tick) as f32 * dt >= 2.0 {
                        Some("no 1cm effective progress for 2 seconds")
                    } else {
                        None
                    };
                    let reached =
                        distance < 0.06 && height_error <= height_tolerance && player.grounded;
                    if reached || failure.is_some() {
                        eprintln!(
                            "[ascent/node] index={} from={from} node={node} roads={roads:?} seconds={seconds:.6} foot={:?} target={target:?} distance={distance:.6} height_error={height_error:.6} grounded={} blocked={:?} resets={} support={support:?} result={}",
                            index + 1,
                            player.foot,
                            player.grounded,
                            player.blocked,
                            player.resets,
                            failure.unwrap_or("reached")
                        );
                    }
                    if let Some(reason) = failure {
                        eprintln!(
                            "[ascent/contact] ground_normal={:?} vertical_speed={} horizontal={:?} up={:?} down={:?}",
                            player.ground_normal,
                            player.vertical_speed,
                            collision.capsule_cast(
                                player.foot,
                                HEIGHT,
                                RADIUS,
                                horizontal.clamp_length_max(SPEED * dt),
                                SKIN
                            ),
                            collision.capsule_cast(
                                player.foot,
                                HEIGHT,
                                RADIUS,
                                Vec3::Y * STEP,
                                SKIN
                            ),
                            collision.capsule_cast(
                                player.foot,
                                HEIGHT,
                                RADIUS,
                                -Vec3::Y * (STEP + SKIN),
                                SKIN
                            ),
                        );
                        panic!(
                            "hill-short stopped: {reason}; wall_seconds={:.3}",
                            started.elapsed().as_secs_f64()
                        );
                    }
                    if reached {
                        break;
                    }
                    // Route following supplies only a bounded horizontal analogue intention to the real mover
                    player.step(
                        &collision,
                        (horizontal / (SPEED * dt)).clamp_length_max(1.0),
                        dt,
                    );
                    tick += 1;
                }
            }
            eprintln!(
                "[ascent/complete] nodes={} seconds={:.6} foot={:?} resets={} wall_seconds={:.3}",
                nodes.len(),
                tick as f32 * dt,
                player.foot,
                player.resets,
                started.elapsed().as_secs_f64()
            );
        }
    }
}
