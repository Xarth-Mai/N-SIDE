//! Neutral exterior controller experiment; static mesh queries use the rendered world geometry
use crate::{
    app::GamePhase,
    capture::CaptureInput,
    world::{
        collision::{CollisionHit, CollisionWorld},
        map::{Map, map_to_world},
    },
};
use bevy::{
    app::RunFixedMainLoopSystems, input::mouse::AccumulatedMouseMotion, prelude::*,
    window::PrimaryWindow,
};

const HEIGHT: f32 = 1.7;
const RADIUS: f32 = 0.3;
const SKIN: f32 = 0.02;
// Leave one millimeter beyond the query skin to avoid zero-time tangential hits
const SEPARATION: f32 = 0.001;
const STEP: f32 = 0.26;
const WALKABLE_Y: f32 = 0.70710677;
const SPEED: f32 = 3.2;
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
        self.resets += 1;
    }

    fn step(&mut self, collision: &CollisionWorld, direction: Vec3, dt: f32) {
        self.blocked = None;
        let was_grounded = self.grounded;
        let mut delta = direction.clamp_length_max(1.0) * SPEED * dt;
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
        self.vertical_speed = if was_grounded {
            0.0
        } else {
            self.vertical_speed - 18.0 * dt
        };
        let fall = if was_grounded {
            STEP + SKIN
        } else {
            (-self.vertical_speed * dt).max(SKIN)
        };
        let down = -Vec3::Y * fall;
        self.grounded = false;
        if let Some(hit) = collision.capsule_cast(self.foot, HEIGHT, RADIUS, down, SKIN) {
            if let Some(normal) = walkable_normal(collision, &hit) {
                self.foot += down * safe_fraction(down, hit.fraction);
                self.grounded = true;
                self.ground_normal = normal;
                self.vertical_speed = 0.0;
            } else {
                self.foot = slide(collision, self.foot, down).0;
            }
        } else {
            self.foot += down;
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
    let inside = -Vec3::new(hit.normal.x, 0.0, hit.normal.z).normalize_or_zero() * 0.01;
    collision
        .support(hit.point + inside + Vec3::Y * 0.05, 0.1)
        .filter(|support| support.surface_normal.y >= WALKABLE_Y)
        .map(|support| support.surface_normal)
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
        .init_resource::<Intent>()
        .init_resource::<Orbit>()
        .add_systems(
            OnEnter(GamePhase::World),
            spawn_body.run_if(resource_exists::<PlayerState>),
        )
        .add_systems(
            RunFixedMainLoop,
            read_input
                .after(CaptureInput)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                .run_if(in_state(GamePhase::World))
                .run_if(resource_exists::<PlayerState>),
        )
        .add_systems(
            FixedUpdate,
            move_player
                .run_if(in_state(GamePhase::World))
                .run_if(resource_exists::<PlayerState>),
        )
        .add_systems(
            Update,
            show_player
                .run_if(in_state(GamePhase::World))
                .run_if(resource_exists::<PlayerState>),
        );
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
) {
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
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    pads: Query<&Gamepad>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time<Virtual>>,
    mut intent: ResMut<Intent>,
    mut orbit: ResMut<Orbit>,
) {
    intent.direction = Vec3::ZERO;
    if windows.iter().any(|window| !window.focused) {
        intent.reset = false;
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
        intent.reset |= pad.just_pressed(GamepadButton::Select);
    }
    orbit.yaw += look.x * 1.6 * time.delta_secs();
    orbit.pitch -= look.y * 1.2 * time.delta_secs();
    if buttons.pressed(MouseButton::Right) {
        orbit.yaw -= motion.delta.x * 0.003;
        orbit.pitch += motion.delta.y * 0.003;
    }
    orbit.pitch = orbit.pitch.clamp(-0.12, 1.15);
    let forward = Vec3::new(-orbit.yaw.sin(), 0.0, -orbit.yaw.cos());
    let right = Vec3::new(orbit.yaw.cos(), 0.0, -orbit.yaw.sin());
    movement = movement.clamp_length_max(1.0);
    intent.direction = right * movement.x + forward * movement.y;
    intent.reset |= keys.just_pressed(KeyCode::KeyR);
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
    }
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
        for _ in 0..360 {
            player.step(&world, -Vec3::X, 1.0 / 60.0);
        }
        eprintln!(
            "[player/wall] foot={:?} grounded={} blocked={:?}",
            player.foot, player.grounded, player.blocked
        );
        // The visible door panel projects beyond the original x=88 wall
        assert!((88.37..88.8).contains(&player.foot.x) && player.grounded);
        assert!(
            player
                .blocked
                .as_deref()
                .is_some_and(|source| source.contains("V-04"))
        );
    }
}
