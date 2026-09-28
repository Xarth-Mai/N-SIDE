//! Read public place information through the real player's reach and view
use super::{Place, PlaceCatalog};
use crate::{
    app::{EntrySettings, EntryUi, GamePhase},
    capture::CaptureInput,
    player::PlayerState,
    ui::{Tokens, UiFont, UiInput, color},
    world::collision::CollisionWorld,
};
use bevy::{
    app::RunFixedMainLoopSystems,
    picking::hover::Hovered,
    prelude::*,
    text::{FontWeight, LineHeight},
};

#[derive(Resource, Default, Debug)]
pub struct Observation {
    pub target: Option<String>,
    pub selected: Option<String>,
    pub open: bool,
    pub visible: bool,
    consumed: bool,
    wait_for_release: bool,
    close_requested: bool,
}

impl Observation {
    pub fn blocks_world(&self) -> bool {
        self.open || self.consumed
    }

    /// Called by the entry router after focus/disconnection protection, before menu actions
    pub fn handle_input(&mut self, open_pressed: bool, close_pressed: bool, held: bool) -> bool {
        let close_pressed = std::mem::take(&mut self.close_requested) || close_pressed;
        if self.wait_for_release {
            self.wait_for_release = held;
            self.consumed = self.open || held;
            return self.consumed;
        }
        if self.open {
            if close_pressed {
                self.open = false;
                self.wait_for_release = true;
            }
            self.consumed = true;
            return true;
        }
        if open_pressed && !close_pressed && self.target.is_some() {
            self.selected.clone_from(&self.target);
            self.open = true;
            self.wait_for_release = true;
            self.consumed = true;
            return true;
        }
        false
    }
}

fn eligible(place: &Place, foot: Vec3, camera: &Transform, collision: &CollisionWorld) -> bool {
    if place.public_info.is_none()
        || !foot.is_finite()
        || !camera.translation.is_finite()
        || !camera.rotation.is_finite()
        || foot.xz().distance(place.position.xz()) > 3.0
        || (foot.y - place.position.y).abs() > 1.0
    {
        return false;
    }
    let target = place.position + Vec3::Y * 1.35;
    let from_camera = target - camera.translation;
    if camera.forward().dot(from_camera.normalize_or_zero()) < 0.75 {
        return false;
    }
    let eye = foot + Vec3::Y * 1.35;
    let sight = target - eye;
    // The door facade projects beyond its map anchor (5.5cm at the shop)
    // Include its thickness and the 1.5cm probe radius at the endpoint
    collision
        .sphere_cast(eye, 0.015, sight, 0.0)
        .is_none_or(|hit| (1.0 - hit.fraction) * sight.length() <= 0.15)
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Observation>()
        .add_systems(
            RunFixedMainLoop,
            (
                update_target.after(CaptureInput).before(UiInput),
                expire_pointer_request.after(UiInput),
            )
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(Update, (draw, style_return).chain())
        .add_systems(OnEnter(GamePhase::Title), reset)
        .add_systems(OnEnter(GamePhase::Loading), reset)
        .add_systems(OnEnter(GamePhase::Failed), reset);
}

fn reset(mut observation: ResMut<Observation>) {
    *observation = Observation::default();
}

// Focus loss or pause can return from the entry router without consuming this frame's click
fn expire_pointer_request(mut observation: ResMut<Observation>) {
    observation.close_requested = false;
}

fn request_pointer_close(
    mut press: On<Pointer<Press>>,
    mut observation: ResMut<Observation>,
    phase: Res<State<GamePhase>>,
    next: Res<NextState<GamePhase>>,
) {
    if press.button == PointerButton::Primary
        && observation.open
        && observation.visible
        && *phase.get() == GamePhase::World
        && matches!(*next, NextState::Unchanged)
    {
        observation.close_requested = true;
        press.propagate(false);
    }
}

fn update_target(
    player: Option<Res<PlayerState>>,
    catalog: Option<Res<PlaceCatalog>>,
    collision: Option<Res<CollisionWorld>>,
    cameras: Query<&Transform, With<Camera3d>>,
    phase: Res<State<GamePhase>>,
    mut observation: ResMut<Observation>,
) {
    observation.consumed = false;
    let target = player
        .as_ref()
        .zip(catalog.as_ref())
        .zip(collision.as_ref())
        .zip(cameras.single().ok())
        .and_then(|(((player, catalog), collision), camera)| {
            catalog
                .0
                .iter()
                .filter(|place| eligible(place, player.foot, camera, collision))
                .min_by(|a, b| {
                    player
                        .foot
                        .distance_squared(a.position)
                        .total_cmp(&player.foot.distance_squared(b.position))
                })
        })
        .map(|place| place.id.clone());
    observation.target = if *phase.get() == GamePhase::World {
        target
    } else {
        None
    };
}

#[derive(Component)]
struct Panel;

#[derive(Component)]
struct ReturnButton;

#[expect(
    clippy::type_complexity,
    reason = "Bevy query updates only hovered return buttons"
)]
fn style_return(
    tokens: Res<Tokens>,
    mut buttons: Query<(&Hovered, &mut BorderColor), (With<ReturnButton>, Changed<Hovered>)>,
) {
    for (hovered, mut border) in &mut buttons {
        *border = BorderColor::all(color(if hovered.get() {
            &tokens.colors.focus
        } else {
            &tokens.colors.secondary
        }));
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Observation reads the real phase, metadata, shared UI resources and camera"
)]
fn draw(
    mut commands: Commands,
    mut observation: ResMut<Observation>,
    phase: Res<State<GamePhase>>,
    next: Res<NextState<GamePhase>>,
    catalog: Option<Res<PlaceCatalog>>,
    entry: Res<EntryUi>,
    tokens: Res<Tokens>,
    settings: Res<EntrySettings>,
    font: Res<UiFont>,
    camera: Query<Entity, With<Camera3d>>,
    roots: Query<Entity, With<Panel>>,
    mut prior: Local<Option<(bool, Option<String>, bool, bool)>>,
) {
    observation.visible = observation.open
        && *phase.get() == GamePhase::World
        && matches!(*next, NextState::Unchanged);
    let key = (
        observation.visible,
        observation.selected.clone(),
        entry.gamepad,
        settings.large_text,
    );
    if prior.as_ref() == Some(&key) {
        return;
    }
    *prior = Some(key);
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    if !observation.visible {
        return;
    }
    let Some(place) = catalog.as_ref().and_then(|catalog| {
        catalog
            .0
            .iter()
            .find(|place| observation.selected.as_ref() == Some(&place.id))
    }) else {
        return;
    };
    let (Some(info), Ok(camera)) = (&place.public_info, camera.single()) else {
        return;
    };
    commands
        .spawn((
            Panel,
            GlobalZIndex(10),
            UiTargetCamera(camera),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(40)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color(&tokens.colors.base).with_alpha(0.74)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: px(1000),
                    max_width: percent(100),
                    max_height: percent(100),
                    padding: UiRect::all(px(32)),
                    row_gap: px(24),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(tokens.panel_radius)),
                    ..default()
                },
                BackgroundColor(color(&tokens.colors.base)),
                BorderColor::all(color(&tokens.colors.focus)),
            ))
            .with_children(|panel| {
                for (value, size, ink) in [
                    (
                        place.name.as_str(),
                        tokens.heading_size,
                        &tokens.colors.focus,
                    ),
                    (info.as_str(), tokens.body_size, &tokens.colors.text),
                ] {
                    panel.spawn((
                        Text::new(value),
                        TextFont {
                            font: font.0.clone().into(),
                            font_size: FontSize::Px(size * settings.text_scale()),
                            weight: FontWeight(550),
                            ..default()
                        },
                        TextColor(color(ink)),
                        LineHeight::RelativeToFont(tokens.line_height),
                        Node {
                            width: percent(100),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                }
                panel
                    .spawn((Node {
                        column_gap: px(24),
                        row_gap: px(16),
                        align_items: AlignItems::Center,
                        flex_wrap: FlexWrap::Wrap,
                        flex_shrink: 0.0,
                        ..default()
                    },))
                    .with_children(|footer| {
                        footer
                            .spawn((
                                Button,
                                ReturnButton,
                                Name::new("observation-return"),
                                AccessibleLabel::new("返回街区"),
                                Hovered::default(),
                                Node {
                                    padding: UiRect::axes(px(24), px(12)),
                                    border: UiRect::all(px(3)),
                                    border_radius: BorderRadius::all(px(tokens.button_radius)),
                                    ..default()
                                },
                                BackgroundColor(color(&tokens.colors.raised)),
                                BorderColor::all(color(&tokens.colors.secondary)),
                            ))
                            .observe(request_pointer_close)
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("返回街区"),
                                    TextFont {
                                        font: font.0.clone().into(),
                                        font_size: FontSize::Px(
                                            tokens.body_size * settings.text_scale(),
                                        ),
                                        weight: FontWeight(650),
                                        ..default()
                                    },
                                    TextColor(color(&tokens.colors.text)),
                                ));
                            });
                        footer.spawn((
                            Text::new(if entry.gamepad {
                                "B 返回 · Start 暂停"
                            } else {
                                "Esc 返回 · Tab 暂停"
                            }),
                            TextFont {
                                font: font.0.clone().into(),
                                font_size: FontSize::Px(tokens.body_size * settings.text_scale()),
                                ..default()
                            },
                            TextColor(color(&tokens.colors.secondary)),
                        ));
                    });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{geometry::GeometryPart, map::map_to_world, scene::PreparedScene};
    use bevy::{
        camera::NormalizedRenderTarget,
        ecs::system::RunSystemOnce,
        picking::{
            backend::HitData,
            pointer::{Location, PointerId},
        },
    };

    #[test]
    fn pointer_request_uses_close_gate_and_expires_when_router_is_blocked() {
        // This checks event handling and the input gate; GPU capture verifies native UI hit testing
        let mut world = World::new();
        world.insert_resource(Observation {
            target: Some("04".into()),
            selected: Some("04".into()),
            open: true,
            visible: true,
            ..default()
        });
        world.insert_resource(State::new(GamePhase::World));
        world.insert_resource(NextState::<GamePhase>::default());
        let button = world.spawn_empty().observe(request_pointer_close).id();
        let press = |button_kind| {
            Pointer::new(
                PointerId::Mouse,
                Location {
                    target: NormalizedRenderTarget::Image(Handle::<Image>::default().into()),
                    position: Vec2::ZERO,
                },
                Press {
                    button: button_kind,
                    hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
                    count: 1,
                },
                button,
            )
        };
        world.trigger(press(PointerButton::Secondary));
        assert!(!world.resource::<Observation>().close_requested);
        world.trigger(press(PointerButton::Primary));
        {
            let mut state = world.resource_mut::<Observation>();
            assert!(
                state.open,
                "pointer requests must pass the entry input gate"
            );
            assert!(state.close_requested);
            assert!(state.handle_input(false, false, true));
            assert!(!state.open && state.blocks_world());
            assert!(!state.close_requested, "consume each request once");
            state.consumed = false;
            assert!(state.handle_input(true, false, true));
            assert!(!state.open, "held mouse must not reopen or move the player");
            assert!(!state.handle_input(false, false, false));
            assert!(state.handle_input(true, false, true));
            assert!(state.open);
        }
        world.trigger(press(PointerButton::Primary));
        {
            let mut state = world.resource_mut::<Observation>();
            assert!(state.handle_input(false, false, true));
            assert!(
                state.open,
                "opening input must release before pointer close"
            );
            assert!(state.handle_input(false, false, false));
        }
        world.trigger(press(PointerButton::Primary));
        assert!(world.resource::<Observation>().close_requested);
        world.run_system_once(expire_pointer_request).unwrap();
        {
            let mut state = world.resource_mut::<Observation>();
            assert!(
                !state.close_requested,
                "blocked routers cannot leave stale clicks"
            );
            assert!(state.handle_input(false, false, false));
            assert!(state.open);
            state.visible = false;
        }
        world.trigger(press(PointerButton::Primary));
        assert!(!world.resource::<Observation>().close_requested);
        world.resource_mut::<Observation>().visible = true;
        world.insert_resource(State::new(GamePhase::Paused));
        world.trigger(press(PointerButton::Primary));
        assert!(!world.resource::<Observation>().close_requested);
        world.insert_resource(State::new(GamePhase::World));
        world
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Paused);
        world.trigger(press(PointerButton::Primary));
        assert!(!world.resource::<Observation>().close_requested);
    }

    #[test]
    fn real_shop_recorded_approach_can_observe_its_public_entrance() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let path = root.join("source-assets/district-map/district.json");
        let scene = PreparedScene::load(root).unwrap();
        let catalog = PlaceCatalog::load(&path).unwrap();
        let collision = CollisionWorld::from_parts(&scene.parts).unwrap();
        let place = catalog.0.iter().find(|place| place.id == "04").unwrap();
        // TASK-035 approach2 frame 189: real walk and camera input before opening
        let foot = Vec3::new(90.826675, 28.04576, -255.);
        let camera = Transform {
            translation: Vec3::new(94.5335, 30.225033, -255.10828),
            rotation: Quat::from_xyzw(-0.0764835, 0.7130198, 0.07875021, 0.6924966),
            ..default()
        };
        let target = place.position + Vec3::Y * 1.35;
        let eye = foot + Vec3::Y * 1.35;
        let sight = target - eye;
        let hit = collision.sphere_cast(eye, 0.015, sight, 0.0);
        eprintln!(
            "[observation/shop] horizontal={} vertical={} facing={} endpoint_distance={:?} sight={hit:?}",
            foot.xz().distance(place.position.xz()),
            (foot.y - place.position.y).abs(),
            camera
                .forward()
                .dot((target - camera.translation).normalize_or_zero()),
            hit.map(|hit| (1.0 - hit.fraction) * sight.length()),
        );
        assert!(eligible(place, foot, &camera, &collision));
    }

    #[test]
    fn real_summit_public_entry_is_observable_from_its_walkable_platform() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let path = root.join("source-assets/district-map/district.json");
        let scene = PreparedScene::load(root).unwrap();
        let catalog = PlaceCatalog::load(&path).unwrap();
        let collision = CollisionWorld::from_parts(&scene.parts).unwrap();
        let place = catalog.0.iter().find(|place| place.id == "23").unwrap();
        let inside = (map_to_world(scene.map.nodes["summit"]) - place.position).normalize();
        let support = collision
            .support(place.position + inside * 2.0 + Vec3::Y * 0.5, 1.0)
            .expect("public summit arrival must have a real platform surface");
        assert!(support.surface_normal.y > 0.7, "{support:?}");
        let foot = support.point + Vec3::Y * 0.021;
        let camera = Transform::from_translation(foot + inside * 3.8 + Vec3::Y * 2.2)
            .looking_at(foot + Vec3::Y * 1.05, Vec3::Y);
        assert!(
            eligible(place, foot, &camera, &collision),
            "real summit entry must pass distance, height, facing and sight: foot={foot:?} entry={:?}",
            place.position
        );
    }

    #[test]
    fn observation_requires_reach_facing_and_clear_sight_then_owns_open_close_input() {
        let ground = || GeometryPart {
            source: "/terrain".into(),
            material: "test".into(),
            mesh: Mesh::from(Cuboid::new(20.0, 1.0, 20.0)).translated_by(Vec3::Y * -0.5),
        };
        let clear = CollisionWorld::from_parts(&[ground()]).unwrap();
        let place = Place {
            id: "04".into(),
            name: "门口".into(),
            position: Vec3::new(0.0, 0.0, -2.0),
            public_info: Some("公开信息".into()),
        };
        let camera =
            Transform::from_xyz(0.0, 1.35, 3.8).looking_at(Vec3::new(0.0, 1.35, 0.0), Vec3::Y);
        assert!(eligible(&place, Vec3::ZERO, &camera, &clear));
        assert!(!eligible(
            &place,
            Vec3::new(0.0, 0.0, 1.01),
            &camera,
            &clear
        ));
        assert!(!eligible(&place, Vec3::Y * 1.01, &camera, &clear));
        for wall_z in [-1.0, -1.85] {
            let blocked = CollisionWorld::from_parts(&[
                ground(),
                GeometryPart {
                    source: "/buildings/wall".into(),
                    material: "test".into(),
                    mesh: Mesh::from(Cuboid::new(4.0, 3.0, 0.1))
                        .translated_by(Vec3::new(0.0, 1.5, wall_z)),
                },
            ])
            .unwrap();
            assert!(
                !eligible(&place, Vec3::ZERO, &camera, &blocked),
                "an intervening wall must block even 20cm before the target"
            );
        }
        let back = camera.looking_at(Vec3::new(0.0, 1.35, 10.0), Vec3::Y);
        assert!(!eligible(&place, Vec3::ZERO, &back, &clear));

        let mut state = Observation::default();
        assert!(!state.handle_input(true, false, true));
        state.target = Some("04".into());
        assert!(state.handle_input(true, false, true));
        assert!(state.open && state.blocks_world());
        assert!(state.handle_input(true, true, true));
        assert!(
            state.open,
            "held opening input cannot immediately close the panel"
        );
        assert!(state.handle_input(false, false, false));
        assert!(state.handle_input(false, true, true));
        assert!(
            !state.open && state.blocks_world(),
            "closing frame still owns movement"
        );
        state.consumed = false;
        assert!(state.handle_input(true, false, true));
        assert!(!state.open, "held close must release before another action");
        assert!(!state.handle_input(false, false, false));
        assert!(state.handle_input(true, false, true));
        assert_eq!(state.selected.as_deref(), Some("04"));
    }
}
