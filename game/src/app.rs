use crate::{
    capture::{self, CaptureInput, CaptureTarget},
    player::{self, PlayerState, WalkPreview},
    ui::{FONT, Tokens, UiFont, UiInput, color},
    world::{
        collision::CollisionWorld,
        map::{Map, map_to_world},
        scene::{PreparedScene, SceneLoading, WorldScenePlugin, clear_scene},
        visual::{Antialiasing, DaylightSettings},
    },
};
use bevy::{
    app::{RunFixedMainLoopSystems, ScheduleRunnerPlugin},
    audio::AudioPlugin,
    camera::RenderTarget,
    input::{
        gamepad::GamepadConnectionEvent, keyboard::KeyboardInput, mouse::AccumulatedMouseMotion,
    },
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::TextureFormat,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
    text::{FontWeight, LineHeight},
    window::{ExitCondition, PrimaryWindow, WindowFocused},
    winit::WinitPlugin,
};
use std::path::PathBuf;

#[derive(Resource)]
pub struct GameLoadError(pub String);

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamePhase {
    #[default]
    Title,
    Loading,
    World,
    Paused,
    Failed,
}

impl GamePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Loading => "loading",
            Self::World => "world",
            Self::Paused => "paused",
            Self::Failed => "failed",
        }
    }
}

#[derive(Resource)]
struct ProjectRoot(PathBuf);
#[derive(Resource)]
struct Preparation(Task<Result<PreparedWorld, String>>);
struct PreparedWorld {
    scene: PreparedScene,
    player: Option<(CollisionWorld, PlayerState)>,
}
#[derive(Resource, Default)]
struct EntryUi {
    focus: usize,
    gamepad: bool,
    wait_for_release: bool,
    pause_after_loading: bool,
    gamepad_recovery: Option<bool>,
}
#[derive(Clone, Copy)]
enum Action {
    Enter,
    Pause,
    Resume,
    Title,
    Quit,
}

fn actions(phase: GamePhase) -> &'static [(&'static str, Action)] {
    match phase {
        GamePhase::Title => &[("进入街区", Action::Enter), ("退出", Action::Quit)],
        GamePhase::Loading => &[("取消并返回标题", Action::Title)],
        GamePhase::World => &[("暂停", Action::Pause)],
        GamePhase::Paused => &[("继续", Action::Resume), ("返回标题", Action::Title)],
        GamePhase::Failed => &[
            ("重试", Action::Enter),
            ("返回标题", Action::Title),
            ("退出", Action::Quit),
        ],
    }
}

#[derive(Component)]
struct ShellRoot;
#[derive(Component)]
struct ShellButton {
    phase: GamePhase,
    index: usize,
}
type ShellSnapshot = (GamePhase, bool, UVec2, Option<bool>);

pub fn run() -> Result<AppExit, String> {
    let mut root = PathBuf::from(".");
    let mut script = None;
    let mut walk = false;
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project-root" => {
                root = PathBuf::from(args.next().ok_or("--project-root requires a directory")?)
            }
            "--walk-preview" => walk = true,
            "--capture" => {
                script = Some(PathBuf::from(
                    args.next().ok_or("--capture requires a script")?,
                ))
            }
            "--output" => {
                output = Some(PathBuf::from(
                    args.next().ok_or("--output requires a directory")?,
                ))
            }
            "--help" | "-h" => {
                println!(
                    "N:SIDE\n--project-root PATH  Project root (default .)\n--capture SCRIPT --output DIRECTORY  Offscreen evidence\n--walk-preview  Neutral exterior movement experiment\n\nArrow keys / gamepad D-pad select, Enter / South confirm, Escape / East return"
                );
                return Ok(AppExit::Success);
            }
            _ => return Err(format!("unknown argument {arg:?}")),
        }
    }
    if script.is_some() != output.is_some() {
        return Err("--capture and --output must be used together".into());
    }
    let recording = script
        .map(|path| capture::Recording::load(&path, output.unwrap()))
        .transpose()?;
    if recording
        .as_ref()
        .is_some_and(|r| r.script.scene != if walk { "walk-preview" } else { "game-entry" })
    {
        return Err(
            "n-side capture scene must be game-entry, or walk-preview with --walk-preview".into(),
        );
    }
    let root = root
        .canonicalize()
        .map_err(|error| format!("[startup/root] {}: {error}", root.display()))?;
    let visual = DaylightSettings::load(&root.join("source-assets/district-scene/daylight.json"))?;
    let headless = recording.is_some();
    let (width, height) = recording
        .as_ref()
        .map_or((1280, 720), |r| (r.script.width, r.script.height));
    let mut app = App::new();
    let mut plugins = DefaultPlugins
        .build()
        .disable::<AudioPlugin>()
        .set(AssetPlugin {
            file_path: root.join("game/assets").to_string_lossy().into_owned(),
            watch_for_changes_override: Some(false),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: (!headless).then_some(Window {
                title: "N:SIDE".into(),
                resolution: (width, height).into(),
                ..default()
            }),
            exit_condition: if headless {
                ExitCondition::DontExit
            } else {
                ExitCondition::OnPrimaryClosed
            },
            ..default()
        })
        .set(RenderPlugin {
            synchronous_pipeline_compilation: headless,
            render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            })),
            ..default()
        });
    if headless {
        plugins = plugins.disable::<WinitPlugin>();
    }
    app.add_plugins(plugins)
        .insert_resource(ProjectRoot(root))
        .insert_resource(
            serde_json::from_str::<Tokens>(include_str!("../../source-assets/ui-kit/tokens.json"))
                .expect("checked UI tokens"),
        )
        .init_resource::<CaptureTarget>();
    visual.install(&mut app);
    app.insert_resource(WalkPreview(walk));
    install_lifecycle(&mut app);
    player::install(&mut app);
    app.add_systems(
        Startup,
        move |mut commands: Commands,
              mut images: ResMut<Assets<Image>>,
              assets: Res<AssetServer>| {
            let target = if headless {
                let image = images.add(Image::new_target_texture(
                    width,
                    height,
                    TextureFormat::Rgba8UnormSrgb,
                    None,
                ));
                commands.insert_resource(CaptureTarget(Some(image.clone())));
                RenderTarget::Image(image.into())
            } else {
                RenderTarget::default()
            };
            let camera = commands
                .spawn((
                    Camera3d::default(),
                    target,
                    Projection::Perspective(PerspectiveProjection {
                        fov: 55.0_f32.to_radians(),
                        near: 0.1,
                        far: 7000.0,
                        ..default()
                    }),
                    Transform::from_xyz(0.0, 1.7, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                ))
                .id();
            visual.configure_camera(&mut commands, camera, &mut images, Antialiasing::Msaa4);
            visual.spawn_lighting(&mut commands);
            commands.insert_resource(UiFont(assets.load(FONT)));
        },
    )
    .add_systems(Update, (check_font, draw_shell, style_buttons).chain());
    if let Some(recording) = recording {
        app.add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::ZERO));
        capture::install(&mut app, recording);
    }
    Ok(app.run())
}

fn install_lifecycle(app: &mut App) {
    app.init_state::<GamePhase>()
        .init_resource::<EntryUi>()
        .add_message::<WindowFocused>()
        .add_message::<KeyboardInput>()
        .add_message::<GamepadConnectionEvent>()
        .add_plugins(WorldScenePlugin)
        .add_systems(OnEnter(GamePhase::Title), enter_title)
        .add_systems(OnEnter(GamePhase::Loading), begin_loading)
        .add_systems(OnEnter(GamePhase::Failed), enter_failed)
        .add_systems(OnEnter(GamePhase::World), |mut ui: ResMut<EntryUi>| {
            ui.focus = 0;
            if !ui.pause_after_loading {
                ui.gamepad_recovery = None;
            }
        })
        .add_systems(OnEnter(GamePhase::Paused), |mut ui: ResMut<EntryUi>| {
            ui.focus = 0
        })
        .add_systems(
            RunFixedMainLoop,
            entry_input
                .after(CaptureInput)
                .in_set(UiInput)
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(
            Update,
            (poll_preparation, poll_scene)
                .chain()
                .run_if(in_state(GamePhase::Loading)),
        );
}

fn enter_title(world: &mut World) {
    // Dropping the task discards any late result; the task never writes into World
    world.remove_resource::<Preparation>();
    world.remove_resource::<GameLoadError>();
    player::clear(world);
    clear_scene(world);
    {
        let mut ui = world.resource_mut::<EntryUi>();
        ui.focus = 0;
        ui.pause_after_loading = false;
        ui.gamepad_recovery = None;
    }
    info!("[game/state] title");
}

fn begin_loading(world: &mut World) {
    player::clear(world);
    clear_scene(world);
    world.remove_resource::<GameLoadError>();
    {
        let mut ui = world.resource_mut::<EntryUi>();
        ui.focus = 0;
        ui.pause_after_loading = false;
        ui.gamepad_recovery = None;
    }
    let root = world.resource::<ProjectRoot>().0.clone();
    let walk = world
        .get_resource::<WalkPreview>()
        .is_some_and(|preview| preview.0);
    world.insert_resource(Preparation(AsyncComputeTaskPool::get().spawn(async move {
        let scene = PreparedScene::load(&root)?;
        let player = if walk {
            let collision = CollisionWorld::from_parts(&scene.parts)?;
            info!(
                "[player/collision] triangles={} sources={}",
                collision.triangle_count(),
                collision.source_count()
            );
            let player = PlayerState::from_map(&scene.map, &collision)?;
            Some((collision, player))
        } else {
            None
        };
        Ok(PreparedWorld { scene, player })
    })));
    info!("[game/state] loading");
}

fn enter_failed(world: &mut World) {
    world.remove_resource::<Preparation>();
    player::clear(world);
    clear_scene(world);
    {
        let mut ui = world.resource_mut::<EntryUi>();
        ui.focus = 0;
        ui.pause_after_loading = false;
        ui.gamepad_recovery = None;
    }
    info!("[game/state] failed");
}

fn shop_view(map: &Map) -> Result<Transform, String> {
    let shop = map
        .nodes
        .get("shop_front_door")
        .ok_or("[game/view] missing /nodes/shop_front_door")?;
    // Same shop composition as the Viewer; this is a preview camera, not a player spawn
    Ok(Transform::from_translation(map_to_world([
        shop[0] + 22.0,
        shop[1] - 17.0,
        shop[2] + 5.0,
    ]))
    .looking_at(
        map_to_world([shop[0], shop[1] + 2.0, shop[2] + 2.0]),
        Vec3::Y,
    ))
}

fn poll_preparation(
    mut commands: Commands,
    task: Option<ResMut<Preparation>>,
    mut next: ResMut<NextState<GamePhase>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    if !matches!(*next, NextState::Unchanged) {
        return;
    }
    let Some(mut task) = task else {
        return;
    };
    let Some(result) = check_ready(&mut task.0) else {
        return;
    };
    commands.remove_resource::<Preparation>();
    let result = result.and_then(|prepared| Ok((shop_view(&prepared.scene.map)?, prepared)));
    match result {
        Ok((view, prepared)) => {
            let Ok(mut camera) = camera.single_mut() else {
                let error = "[game/camera] expected exactly one world camera".to_owned();
                error!("{error}");
                commands.insert_resource(GameLoadError(error));
                (*next).set_if_neq(GamePhase::Failed);
                return;
            };
            *camera = view;
            if let Some((collision, player)) = prepared.player {
                commands.insert_resource(collision);
                commands.insert_resource(player);
            }
            commands.insert_resource(SceneLoading::new(prepared.scene));
        }
        Err(error) => {
            error!("{error}");
            commands.insert_resource(GameLoadError(error));
            (*next).set_if_neq(GamePhase::Failed);
        }
    }
}

fn poll_scene(
    mut commands: Commands,
    loading: Option<Res<SceneLoading>>,
    mut next: ResMut<NextState<GamePhase>>,
) {
    if !matches!(*next, NextState::Unchanged) {
        return;
    }
    let Some(loading) = loading else {
        return;
    };
    if let Some(error) = &loading.failure {
        commands.insert_resource(GameLoadError(error.clone()));
        (*next).set_if_neq(GamePhase::Failed);
    } else if loading.ready {
        (*next).set_if_neq(GamePhase::World);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the real input, window and state resources"
)]
fn entry_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut focus_events: MessageReader<WindowFocused>,
    mut connections: MessageReader<GamepadConnectionEvent>,
    gamepads: Query<&Gamepad>,
    buttons: Query<(&Interaction, &ShellButton), Changed<Interaction>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    phase: Res<State<GamePhase>>,
    mut next: ResMut<NextState<GamePhase>>,
    mut ui: ResMut<EntryUi>,
    mut exit: MessageWriter<AppExit>,
) {
    let repeated: Vec<_> = keyboard_events
        .read()
        .filter(|event| event.repeat)
        .map(|event| event.key_code)
        .collect();
    let mut focus_changed = false;
    let mut lost_focus = false;
    for event in focus_events.read() {
        if windows.contains(event.window) {
            debug!(
                "[game/focus] focused={} phase={:?}",
                event.focused,
                phase.get()
            );
            focus_changed = true;
            lost_focus |= !event.focused;
        }
    }
    let mut connection_changed = false;
    let mut disconnected = false;
    for event in connections.read() {
        connection_changed = true;
        disconnected |= event.disconnected();
        if event.disconnected() {
            ui.gamepad = false;
        }
        if matches!(
            *phase.get(),
            GamePhase::Loading | GamePhase::World | GamePhase::Paused
        ) && (event.disconnected() || ui.gamepad_recovery.is_some())
        {
            ui.gamepad_recovery = Some(event.connected());
        }
    }
    ui.pause_after_loading |= disconnected && *phase.get() == GamePhase::Loading;
    let unfocused = windows.iter().any(|(_, window)| !window.focused);
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Escape, KeyCode::Tab]) {
        debug!(
            "[game/input] phase={:?} next={:?} unfocused={} changed={} gate={} repeated={:?}",
            phase.get(),
            *next,
            unfocused,
            focus_changed,
            ui.wait_for_release,
            repeated
        );
    }
    if unfocused || focus_changed || connection_changed {
        ui.wait_for_release = true;
    }
    if *phase.get() == GamePhase::World
        && (unfocused || lost_focus || disconnected || ui.pause_after_loading)
        && matches!(*next, NextState::Unchanged)
    {
        ui.pause_after_loading = false;
        (*next).set_if_neq(GamePhase::Paused);
    }
    if unfocused || focus_changed || connection_changed || !matches!(*next, NextState::Unchanged) {
        return;
    }
    if ui.wait_for_release {
        ui.wait_for_release = keys.any_pressed([
            KeyCode::Escape,
            KeyCode::Tab,
            KeyCode::Enter,
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
        ]) || mouse.pressed(MouseButton::Left)
            || gamepads.iter().any(|pad| {
                [
                    GamepadButton::South,
                    GamepadButton::East,
                    GamepadButton::Start,
                    GamepadButton::DPadUp,
                    GamepadButton::DPadDown,
                ]
                .into_iter()
                .any(|button| pad.pressed(button))
            });
        return;
    }
    // Focus loss clears Bevy's pressed keys; a late OS repeat must not become a fresh action
    let key_pressed = |key| keys.just_pressed(key) && !repeated.contains(&key);
    let pressed = |button| gamepads.iter().any(|pad| pad.just_pressed(button));
    if keys.get_just_pressed().next().is_some()
        || mouse.get_just_pressed().next().is_some()
        || motion.delta != Vec2::ZERO
    {
        ui.gamepad = false;
    } else if gamepads.iter().any(|pad| {
        pad.get_just_pressed().next().is_some()
            || pad.left_stick().length() > 0.15
            || pad.right_stick().length() > 0.15
    }) {
        ui.gamepad = true;
    }
    if key_pressed(KeyCode::Escape)
        || key_pressed(KeyCode::Tab)
        || pressed(GamepadButton::East)
        || pressed(GamepadButton::Start)
    {
        match *phase.get() {
            GamePhase::World => (*next).set_if_neq(GamePhase::Paused),
            GamePhase::Paused => (*next).set_if_neq(GamePhase::World),
            GamePhase::Loading | GamePhase::Failed => (*next).set_if_neq(GamePhase::Title),
            GamePhase::Title => {}
        }
        return;
    }
    let options = actions(*phase.get());
    if key_pressed(KeyCode::ArrowDown) || pressed(GamepadButton::DPadDown) {
        ui.focus = (ui.focus + 1) % options.len();
    }
    if key_pressed(KeyCode::ArrowUp) || pressed(GamepadButton::DPadUp) {
        ui.focus = (ui.focus + options.len() - 1) % options.len();
    }
    let mut activate = key_pressed(KeyCode::Enter) || pressed(GamepadButton::South);
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed && button.phase == *phase.get() {
            ui.focus = button.index;
            ui.gamepad = false;
            activate = true;
        }
    }
    if activate {
        match options[ui.focus].1 {
            Action::Enter => (*next).set_if_neq(GamePhase::Loading),
            Action::Pause => (*next).set_if_neq(GamePhase::Paused),
            Action::Resume => (*next).set_if_neq(GamePhase::World),
            Action::Title => (*next).set_if_neq(GamePhase::Title),
            Action::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

fn check_font(font: Res<UiFont>, assets: Res<AssetServer>, mut exit: MessageWriter<AppExit>) {
    if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(font.0.id()) {
        error!("[ui/font] {FONT}: {error}");
        exit.write(AppExit::error());
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects shell state, reusable tokens and the real camera viewport"
)]
fn draw_shell(
    mut commands: Commands,
    phase: Res<State<GamePhase>>,
    ui: Res<EntryUi>,
    tokens: Res<Tokens>,
    font: Res<UiFont>,
    cameras: Query<(Entity, &Camera), With<Camera3d>>,
    roots: Query<Entity, With<ShellRoot>>,
    mut scale: ResMut<UiScale>,
    mut prior: Local<Option<ShellSnapshot>>,
    walk: Res<WalkPreview>,
) {
    let Ok((camera_id, camera)) = cameras.single() else {
        return;
    };
    let Some(size) = camera.physical_viewport_size() else {
        return;
    };
    let page = *phase.get();
    let key = (page, ui.gamepad, size, ui.gamepad_recovery);
    if *prior == Some(key) {
        return;
    }
    *prior = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    scale.0 = (size.y as f32 / 1080.0).min(size.x as f32 / 1440.0);
    let world = page == GamePhase::World;
    let text = |value: &str, size: f32, ink: Color| {
        (
            Text::new(value),
            TextFont {
                font: font.0.clone().into(),
                font_size: FontSize::Px(size),
                weight: FontWeight(650),
                ..default()
            },
            TextColor(ink),
            LineHeight::RelativeToFont(tokens.line_height),
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                ..default()
            },
        )
    };
    commands
        .spawn((
            ShellRoot,
            UiTargetCamera(camera_id),
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(40)),
                align_items: if world {
                    AlignItems::FlexStart
                } else {
                    AlignItems::Center
                },
                justify_content: if world {
                    JustifyContent::FlexEnd
                } else {
                    JustifyContent::Center
                },
                ..default()
            },
            BackgroundColor(if world {
                Color::NONE
            } else if page == GamePhase::Paused {
                color(&tokens.colors.base).with_alpha(0.82)
            } else {
                color(&tokens.colors.base)
            }),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: px(if world { 280.0 } else { 720.0 }),
                        max_width: percent(100),
                        padding: UiRect::all(px(if world { 16.0 } else { 32.0 })),
                        row_gap: px(if world { 12.0 } else { 24.0 }),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(px(3)),
                        border_radius: BorderRadius::all(px(tokens.panel_radius)),
                        ..default()
                    },
                    BackgroundColor(color(&tokens.colors.base)),
                    BorderColor::all(color(&tokens.colors.raised)),
                ))
                .with_children(|panel| {
                    let (heading, description) = match page {
                        GamePhase::Title => ("N:SIDE", "街区信号  :  生活仍在继续"),
                        GamePhase::Loading => ("正在进入街区", "正在准备街景与素材"),
                        GamePhase::World => ("", ""),
                        GamePhase::Paused => ("暂停", match ui.gamepad_recovery {
                            Some(false) => "手柄已断开，可重新连接或使用键鼠继续",
                            Some(true) => "手柄已重新连接，确认后继续",
                            None => "继续当前行程，或返回标题",
                        }),
                        GamePhase::Failed => (
                            "暂时无法进入街区",
                            "街区资料或素材加载失败，请重试或返回标题",
                        ),
                    };
                    if !heading.is_empty() {
                        panel.spawn(text(
                        heading,
                        if world {
                            tokens.heading_size
                        } else {
                            tokens.title_size
                        },
                        color(if page == GamePhase::Failed {
                            &tokens.colors.warning
                        } else {
                            &tokens.colors.focus
                        }),
                        ));
                    }
                    if !description.is_empty() {
                        panel.spawn(text(
                            description,
                            tokens.body_size,
                            color(&tokens.colors.text),
                        ));
                    }
                    for (index, (label, _)) in actions(page).iter().enumerate() {
                        panel
                            .spawn((
                                Button,
                                AccessibleLabel::new(*label),
                                ShellButton { phase: page, index },
                                Node {
                                    width: percent(100),
                                    min_height: px(76),
                                    padding: UiRect::axes(px(24), px(14)),
                                    border: UiRect::all(px(3)),
                                    border_radius: BorderRadius::all(px(tokens.button_radius)),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(color(&tokens.colors.raised)),
                                BorderColor::all(color(&tokens.colors.secondary)),
                            ))
                            .with_children(|button| {
                                button.spawn(text(
                                    label,
                                    tokens.body_size,
                                    color(&tokens.colors.text),
                                ));
                            });
                    }
                    panel.spawn(text(
                        match (page, ui.gamepad) {
                            (GamePhase::Title, false) => "↑ ↓ 选择 · Enter 确认",
                            (GamePhase::Title, true) => "方向键选择 · A 确认",
                            (GamePhase::World, false) => "Esc / Tab 暂停",
                            (GamePhase::World, true) => "Start / B 暂停",
                            (GamePhase::Paused, false) if walk.0 => {
                                "↑ ↓ 选择 · Enter 确认 · Esc 继续\nWASD 移动 · 右键 / Q E 镜头 · R 回到起点"
                            }
                            (GamePhase::Paused, true) if walk.0 => {
                                "方向键选择 · A 确认 · B 继续\n左摇杆移动 · 右摇杆镜头 · Select 回到起点"
                            }
                            (GamePhase::Paused, false) => "↑ ↓ 选择 · Enter 确认 · Esc 继续",
                            (GamePhase::Paused, true) => "方向键选择 · A 确认 · B 继续",
                            (GamePhase::Loading, false) => "Esc 取消并返回",
                            (GamePhase::Loading, true) => "B 取消并返回",
                            (GamePhase::Failed, false) => "↑ ↓ 选择 · Enter 确认 · Esc 返回",
                            (GamePhase::Failed, true) => "方向键选择 · A 确认 · B 返回",
                        },
                        tokens.body_size,
                        color(&tokens.colors.secondary),
                    ));
                });
        });
}

fn style_buttons(
    ui: Res<EntryUi>,
    tokens: Res<Tokens>,
    mut focus: ResMut<InputFocus>,
    mut buttons: Query<(
        Entity,
        &ShellButton,
        &Interaction,
        &Children,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut texts: Query<&mut TextColor>,
) {
    for (entity, button, interaction, children, mut background, mut border) in &mut buttons {
        let active = ui.focus == button.index;
        if active && focus.get() != Some(entity) {
            focus.set(entity, FocusCause::Navigated);
        }
        let background_color = color(if active {
            &tokens.colors.focus
        } else {
            &tokens.colors.raised
        });
        if background.0 != background_color {
            background.0 = background_color;
        }
        let border_color = BorderColor::all(color(if active {
            &tokens.colors.focus
        } else if *interaction == Interaction::Hovered {
            &tokens.colors.text
        } else {
            &tokens.colors.secondary
        }));
        if *border != border_color {
            *border = border_color;
        }
        for child in children {
            if let Ok(mut ink) = texts.get_mut(*child) {
                let foreground = color(if active {
                    &tokens.colors.base
                } else {
                    &tokens.colors.text
                });
                if ink.0 != foreground {
                    ink.0 = foreground;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::scene::MapSource;
    use bevy::state::app::StatesPlugin;
    use std::time::{Duration, Instant};

    fn lifecycle_app() -> App {
        lifecycle_app_with_input(false)
    }

    fn lifecycle_app_with_input(native_input: bool) -> App {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("missing-entry-project");
        assert!(!root.exists());
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .add_message::<bevy::window::WindowFocused>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .insert_resource(ProjectRoot(root));
        if native_input {
            app.add_plugins(bevy::input::InputPlugin);
        }
        app.insert_resource(WalkPreview(false));
        install_lifecycle(&mut app);
        app.finish();
        app.cleanup();
        app.update();
        app
    }

    #[test]
    fn keyboard_gamepad_and_mouse_use_the_real_entry_actions() {
        let mut app = lifecycle_app();
        let window = app
            .world_mut()
            .spawn((
                Window {
                    focused: false,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();
        assert_eq!(app.world().resource::<EntryUi>().focus, 0);
        app.world_mut().despawn(window);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();
        assert_eq!(app.world().resource::<EntryUi>().focus, 1);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        let pad = app.world_mut().spawn(Gamepad::default()).id();
        app.world_mut()
            .get_mut::<Gamepad>(pad)
            .unwrap()
            .digital_mut()
            .press(GamepadButton::DPadUp);
        app.update();
        assert_eq!(app.world().resource::<EntryUi>().focus, 0);
        assert!(app.world().resource::<EntryUi>().gamepad);
        app.world_mut()
            .get_mut::<Gamepad>(pad)
            .unwrap()
            .digital_mut()
            .clear();
        app.world_mut()
            .get_mut::<Gamepad>(pad)
            .unwrap()
            .analog_mut()
            .set(GamepadAxis::LeftStickX, 0.5);
        app.world_mut().resource_mut::<EntryUi>().gamepad = false;
        app.update();
        assert!(app.world().resource::<EntryUi>().gamepad);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::X;
        app.update();
        assert!(!app.world().resource::<EntryUi>().gamepad);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::ZERO;
        app.world_mut().spawn((
            Interaction::Pressed,
            ShellButton {
                phase: GamePhase::Title,
                index: 1,
            },
        ));
        app.update();
        assert!(matches!(app.should_exit(), Some(AppExit::Success)));
        assert!(!app.world().resource::<EntryUi>().gamepad);
    }

    #[test]
    fn real_preparation_failure_can_return_and_retry() {
        let mut app = lifecycle_app();
        for _ in 0..2 {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::Enter);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.update();
            let deadline = Instant::now() + Duration::from_secs(5);
            while *app.world().resource::<State<GamePhase>>().get() != GamePhase::Failed {
                assert!(
                    Instant::now() < deadline,
                    "preparation failure did not reach Failed"
                );
                app.update();
                std::thread::yield_now();
            }
            assert!(
                app.world()
                    .resource::<GameLoadError>()
                    .0
                    .contains("missing-entry-project")
            );
            assert!(!app.world().contains_resource::<Preparation>());
            assert!(!app.world().contains_resource::<SceneLoading>());
            assert!(app.should_exit().is_none());
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Escape);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Title
        );
        assert!(!app.world().contains_resource::<GameLoadError>());
    }

    #[test]
    fn returning_to_title_cancels_pending_preparation_without_removing_the_camera() {
        let mut app = lifecycle_app();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Loading);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .reset();
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Loading
        );
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        let map = app.world_mut().spawn(MapSource("test-map".into())).id();
        let child = app.world_mut().spawn(ChildOf(map)).id();
        app.insert_resource(Preparation(
            AsyncComputeTaskPool::get().spawn(std::future::pending()),
        ));
        app.insert_resource(GameLoadError("old failure".into()));
        app.world_mut().spawn((
            Interaction::Pressed,
            ShellButton {
                phase: GamePhase::Title,
                index: 1,
            },
        ));
        app.update();
        assert!(matches!(
            app.world().resource::<NextState<GamePhase>>(),
            NextState::Unchanged
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(matches!(
            app.world().resource::<NextState<GamePhase>>(),
            NextState::PendingIfNeq(GamePhase::Title)
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Title
        );
        assert!(!app.world().contains_resource::<Preparation>());
        assert!(!app.world().contains_resource::<GameLoadError>());
        assert!(!app.world().contains_resource::<SceneLoading>());
        assert!(app.world().get_entity(map).is_err());
        assert!(app.world().get_entity(child).is_err());
        assert!(app.world().get_entity(camera).is_ok());
        assert_eq!(app.world().resource::<EntryUi>().focus, 0);
    }

    fn enter_walk_fixture(app: &mut App) -> (Entity, Entity) {
        use crate::world::geometry::GeometryPart;
        let map = Map::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../source-assets/district-map/district.json"),
        )
        .unwrap();
        let home = map_to_world(map.nodes["home"]);
        let collision = CollisionWorld::from_parts(&[GeometryPart {
            source: "/terrain".into(),
            material: "test".into(),
            mesh: Mesh::from(Cuboid::new(100.0, 1.0, 100.0)).translated_by(home - Vec3::Y * 0.5),
        }])
        .unwrap();
        app.insert_resource(PlayerState::from_map(&map, &collision).unwrap());
        app.insert_resource(collision);
        let map_entity = app
            .world_mut()
            .spawn(MapSource("pause-fixture".into()))
            .id();
        let map_child = app.world_mut().spawn(ChildOf(map_entity)).id();
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::World);
        frame(app);
        (map_entity, map_child)
    }

    fn walk_app() -> (App, Entity, Entity) {
        walk_app_with_input(false)
    }

    fn walk_app_with_input(native_input: bool) -> (App, Entity, Entity) {
        let mut app = lifecycle_app_with_input(native_input);
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(1));
        player::install(&mut app);
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        let pad = app.world_mut().spawn(Gamepad::default()).id();
        (app, camera, pad)
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
        for mut pad in app
            .world_mut()
            .query::<&mut Gamepad>()
            .iter_mut(app.world_mut())
        {
            pad.digital_mut().clear();
        }
    }

    #[test]
    fn pause_blocks_same_frame_and_held_gameplay_input_until_release() {
        for fixed_ticks in [0, 1, 3] {
            let (mut app, camera, pad) = walk_app();
            enter_walk_fixture(&mut app);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyD);
            for _ in 0..10 {
                frame(&mut app);
            }
            let before = app.world().resource::<PlayerState>().foot;
            let view = *app.world().get::<Transform>(camera).unwrap();
            app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(fixed_ticks));
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.press(KeyCode::Escape);
                keys.press(KeyCode::KeyR);
                keys.press(KeyCode::KeyE);
            }
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Right);
            app.world_mut()
                .resource_mut::<AccumulatedMouseMotion>()
                .delta = Vec2::new(50.0, 20.0);
            {
                let mut pad = app.world_mut().get_mut::<Gamepad>(pad).unwrap();
                pad.analog_mut().set(GamepadAxis::LeftStickX, 0.8);
                pad.analog_mut().set(GamepadAxis::RightStickX, 0.8);
                pad.digital_mut().press(GamepadButton::Select);
            }
            frame(&mut app);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::PendingIfNeq(GamePhase::Paused)
            ));
            assert_eq!(
                app.world().resource::<PlayerState>().foot,
                before,
                "opening frame {fixed_ticks}"
            );
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            for _ in 0..12 {
                frame(&mut app);
            }
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            assert_eq!(app.world().resource::<PlayerState>().foot, before);
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            frame(&mut app);
            for _ in 0..12 {
                frame(&mut app);
            }
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::World
            );
            assert_eq!(
                app.world().resource::<PlayerState>().foot,
                before,
                "held input after resume {fixed_ticks}"
            );
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            // Opposite held controls are still held; their summed direction is not a release
            *app.world_mut().get_mut::<Gamepad>(pad).unwrap() = Gamepad::default();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .reset_all();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.release(KeyCode::KeyR);
                keys.press(KeyCode::KeyA);
                keys.press(KeyCode::KeyQ);
            }
            frame(&mut app);
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.release(KeyCode::KeyA);
                keys.release(KeyCode::KeyQ);
            }
            frame(&mut app);
            assert_eq!(app.world().resource::<PlayerState>().foot, before);
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .reset_all();
            *app.world_mut().get_mut::<Gamepad>(pad).unwrap() = Gamepad::default();
            app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(1));
            frame(&mut app);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyD);
            frame(&mut app);
            assert!(app.world().resource::<PlayerState>().foot.x > before.x + 0.04);
        }
    }

    #[test]
    fn pause_actions_resume_one_body_and_title_cleans_reentry() {
        let (mut app, camera, pad) = walk_app();
        let (map, child) = enter_walk_fixture(&mut app);
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<Mesh3d>>()
            .single(app.world())
            .unwrap();
        for (key, button) in [
            (Some(KeyCode::Escape), None),
            (Some(KeyCode::Tab), None),
            (None, Some(GamepadButton::Start)),
            (None, Some(GamepadButton::East)),
        ] {
            if let Some(key) = key {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
            }
            if let Some(button) = button {
                app.world_mut()
                    .get_mut::<Gamepad>(pad)
                    .unwrap()
                    .digital_mut()
                    .press(button);
            }
            frame(&mut app);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            if let Some(key) = key {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .release(key);
            }
            if let Some(button) = button {
                app.world_mut()
                    .get_mut::<Gamepad>(pad)
                    .unwrap()
                    .digital_mut()
                    .release(button);
            }
            frame(&mut app);
            if let Some(key) = key {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
            }
            if let Some(button) = button {
                app.world_mut()
                    .get_mut::<Gamepad>(pad)
                    .unwrap()
                    .digital_mut()
                    .press(button);
            }
            frame(&mut app);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::World
            );
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<Mesh3d>>()
                    .single(app.world())
                    .unwrap(),
                body
            );
            if let Some(key) = key {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .release(key);
            }
            if let Some(button) = button {
                app.world_mut()
                    .get_mut::<Gamepad>(pad)
                    .unwrap()
                    .digital_mut()
                    .release(button);
            }
            frame(&mut app);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        frame(&mut app);
        frame(&mut app);
        let continue_button = app
            .world_mut()
            .spawn((
                Interaction::Pressed,
                ShellButton {
                    phase: GamePhase::Paused,
                    index: 0,
                },
            ))
            .id();
        frame(&mut app);
        frame(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::World
        );
        app.world_mut().despawn(continue_button);
        app.world_mut()
            .get_mut::<Gamepad>(pad)
            .unwrap()
            .digital_mut()
            .press(GamepadButton::Start);
        frame(&mut app);
        frame(&mut app);
        app.world_mut()
            .get_mut::<Gamepad>(pad)
            .unwrap()
            .digital_mut()
            .press(GamepadButton::DPadDown);
        frame(&mut app);
        app.world_mut()
            .get_mut::<Gamepad>(pad)
            .unwrap()
            .digital_mut()
            .press(GamepadButton::South);
        frame(&mut app);
        frame(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Title
        );
        for entity in [map, child, body] {
            assert!(app.world().get_entity(entity).is_err());
        }
        assert!(app.world().get_entity(camera).is_ok());
        assert!(!app.world().contains_resource::<PlayerState>());
        assert!(!app.world().contains_resource::<CollisionWorld>());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        *app.world_mut().get_mut::<Gamepad>(pad).unwrap() = Gamepad::default();
        enter_walk_fixture(&mut app);
        assert_eq!(
            app.world_mut().query::<&Mesh3d>().iter(app.world()).count(),
            1
        );
        let before = app.world().resource::<PlayerState>().foot;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        frame(&mut app);
        assert!(app.world().resource::<PlayerState>().foot.x > before.x);
    }

    fn connect_pad(app: &mut App, pad: Entity, connected: bool) {
        use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};
        app.world_mut().write_message(GamepadConnectionEvent::new(
            pad,
            if connected {
                GamepadConnection::Connected {
                    name: "test controller".into(),
                    vendor_id: None,
                    product_id: None,
                }
            } else {
                GamepadConnection::Disconnected
            },
        ));
    }

    fn pad_button(app: &mut App, pad: Entity, button: GamepadButton, value: f32) {
        use bevy::input::gamepad::{RawGamepadButtonChangedEvent, RawGamepadEvent};
        app.world_mut()
            .write_message(RawGamepadEvent::Button(RawGamepadButtonChangedEvent::new(
                pad, button, value,
            )));
    }

    fn native_key(app: &mut App, key_code: KeyCode, pressed: bool) {
        use bevy::input::{ButtonState, keyboard::Key};
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }

    fn connected_walk_app() -> (App, Entity, Entity) {
        let (mut app, camera, pad) = walk_app_with_input(true);
        connect_pad(&mut app, pad, true);
        for _ in 0..3 {
            frame(&mut app);
        }
        assert!(app.world().get::<Gamepad>(pad).is_some());
        (app, camera, pad)
    }

    #[test]
    fn gamepad_disconnect_freezes_falling_world_and_allows_keyboard_takeover() {
        use crate::world::geometry::GeometryPart;
        for fixed_ticks in [0, 1, 3] {
            let (mut app, camera, pad) = connected_walk_app();
            let (map, child) = enter_walk_fixture(&mut app);
            let body = app
                .world_mut()
                .query_filtered::<Entity, With<Mesh3d>>()
                .single(app.world())
                .unwrap();
            let spawn = app.world().resource::<PlayerState>().foot;
            app.insert_resource(
                CollisionWorld::from_parts(&[GeometryPart {
                    source: "/terrain".into(),
                    material: "test".into(),
                    mesh: Mesh::from(Cuboid::new(2.0, 1.0, 2.0))
                        .translated_by(Vec3::new(spawn.x, 27.5, spawn.z)),
                }])
                .unwrap(),
            );
            native_key(&mut app, KeyCode::KeyD, true);
            for _ in 0..80 {
                frame(&mut app);
                if !app.world().resource::<PlayerState>().grounded
                    && app.world().resource::<PlayerState>().foot.y < spawn.y - 0.5
                {
                    break;
                }
            }
            assert!(!app.world().resource::<PlayerState>().grounded);
            let before = app.world().resource::<PlayerState>().foot;
            let view = *app.world().get::<Transform>(camera).unwrap();
            app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(fixed_ticks));
            native_key(&mut app, KeyCode::KeyE, true);
            connect_pad(&mut app, pad, false);
            frame(&mut app);
            assert!(
                app.world().get::<Gamepad>(pad).is_none(),
                "InputPlugin must process disconnect"
            );
            eprintln!(
                "[pad/disconnect] ticks={fixed_ticks} before={before:?} after={:?} next={:?}",
                app.world().resource::<PlayerState>().foot,
                app.world().resource::<NextState<GamePhase>>()
            );
            assert_eq!(
                app.world().resource::<PlayerState>().foot,
                before,
                "disconnect frame must stop gravity and movement"
            );
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::PendingIfNeq(GamePhase::Paused)
            ));
            native_key(&mut app, KeyCode::KeyR, true);
            for _ in 0..240 {
                frame(&mut app);
            }
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            assert_eq!(app.world().resource::<PlayerState>().foot, before);
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            for entity in [map, child, body, camera, pad] {
                assert!(
                    app.world().get_entity(entity).is_ok(),
                    "session entity must survive"
                );
            }
            app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(1));
            native_key(&mut app, KeyCode::Enter, true);
            frame(&mut app);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::World
            );
            assert_eq!(app.world().resource::<PlayerState>().foot.xz(), before.xz());
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            assert!(
                app.world()
                    .get::<Transform>(camera)
                    .unwrap()
                    .rotation
                    .angle_between(view.rotation)
                    < 0.001
            );
            for key in [KeyCode::KeyD, KeyCode::KeyE, KeyCode::KeyR, KeyCode::Enter] {
                native_key(&mut app, key, false);
            }
            frame(&mut app);
            native_key(&mut app, KeyCode::KeyR, true);
            frame(&mut app);
            assert_eq!(app.world().resource::<PlayerState>().resets, 1);
            let recovered = app.world().resource::<PlayerState>().foot;
            native_key(&mut app, KeyCode::KeyD, true);
            frame(&mut app);
            assert!(app.world().resource::<PlayerState>().foot.x > recovered.x);
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<Mesh3d>>()
                    .single(app.world())
                    .unwrap(),
                body
            );
        }
    }

    #[test]
    fn gamepad_disconnect_reconnect_same_frame_rejects_held_confirm_and_start() {
        for button in [GamepadButton::South, GamepadButton::Start] {
            let (mut app, _, pad) = connected_walk_app();
            let spare = app.world_mut().spawn_empty().id();
            connect_pad(&mut app, spare, true);
            frame(&mut app);
            frame(&mut app);
            enter_walk_fixture(&mut app);
            connect_pad(&mut app, pad, false);
            connect_pad(&mut app, pad, true);
            frame(&mut app);
            assert!(
                matches!(
                    app.world().resource::<NextState<GamePhase>>(),
                    NextState::PendingIfNeq(GamePhase::Paused)
                ),
                "button-free round trip must pause"
            );
            frame(&mut app);
            connect_pad(&mut app, pad, false);
            frame(&mut app);
            assert_eq!(
                app.world().resource::<EntryUi>().gamepad_recovery,
                Some(false)
            );
            assert!(!app.world().resource::<EntryUi>().gamepad);
            frame(&mut app);
            connect_pad(&mut app, pad, true);
            pad_button(&mut app, pad, button, 1.0);
            frame(&mut app);
            assert!(app.world().get::<Gamepad>(pad).unwrap().pressed(button));
            assert!(app.world().get::<Gamepad>(spare).is_some());
            assert_eq!(
                app.world().resource::<EntryUi>().gamepad_recovery,
                Some(true)
            );
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
            for _ in 0..12 {
                frame(&mut app);
            }
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
            pad_button(&mut app, pad, button, 0.0);
            frame(&mut app);
            pad_button(&mut app, pad, button, 1.0);
            frame(&mut app);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::World
            );
        }
    }

    #[test]
    fn gamepad_disconnect_during_loading_waits_for_explicit_resume_without_menu_navigation() {
        let (mut app, _, pad) = connected_walk_app();
        app.add_systems(
            OnEnter(GamePhase::Loading),
            (|mut commands: Commands| {
                commands.insert_resource(Preparation(
                    AsyncComputeTaskPool::get().spawn(std::future::pending()),
                ));
            })
            .after(begin_loading),
        );
        for phase in [GamePhase::Title, GamePhase::Failed] {
            app.world_mut()
                .resource_mut::<NextState<GamePhase>>()
                .set(phase);
            connect_pad(&mut app, pad, false);
            connect_pad(&mut app, pad, true);
            pad_button(&mut app, pad, GamepadButton::South, 1.0);
            frame(&mut app);
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), phase);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
            pad_button(&mut app, pad, GamepadButton::South, 0.0);
            frame(&mut app);
        }
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Loading);
        connect_pad(&mut app, pad, false);
        frame(&mut app);
        for _ in 0..4 {
            frame(&mut app);
        }
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Loading
        );
        assert!(matches!(
            app.world().resource::<NextState<GamePhase>>(),
            NextState::Unchanged
        ));
        app.world_mut().remove_resource::<Preparation>();
        enter_walk_fixture(&mut app);
        assert!(
            matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::PendingIfNeq(GamePhase::Paused)
            ),
            "loading disconnect must survive until World"
        );
        frame(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Paused
        );
        native_key(&mut app, KeyCode::Enter, true);
        frame(&mut app);
        frame(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::World
        );
        native_key(&mut app, KeyCode::Enter, false);
        app.world_mut()
            .resource_mut::<NextState<GamePhase>>()
            .set(GamePhase::Title);
        frame(&mut app);
        enter_walk_fixture(&mut app);
        assert!(
            matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ),
            "a prior disconnect must not pause a new session"
        );
    }

    fn focus_window(app: &mut App, window: Entity, focused: bool) {
        // Match Winit: update the component, then deliver the native focus notification
        app.world_mut().get_mut::<Window>(window).unwrap().focused = focused;
        app.world_mut()
            .write_message(bevy::window::WindowFocused { window, focused });
    }

    #[test]
    fn focus_loss_freezes_falling_session_and_requires_explicit_release_to_resume() {
        use crate::world::geometry::GeometryPart;
        for fixed_ticks in [1, 0, 3] {
            let (mut app, camera, pad) = walk_app();
            enter_walk_fixture(&mut app);
            let window = app
                .world_mut()
                .spawn((Window::default(), PrimaryWindow))
                .id();
            let spawn = app.world().resource::<PlayerState>().foot;
            // Walk off a small, real collision platform to obtain a falling state
            app.insert_resource(
                CollisionWorld::from_parts(&[GeometryPart {
                    source: "/terrain".into(),
                    material: "test".into(),
                    mesh: Mesh::from(Cuboid::new(2.0, 1.0, 2.0))
                        .translated_by(Vec3::new(spawn.x, 27.5, spawn.z)),
                }])
                .unwrap(),
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyD);
            for _ in 0..80 {
                frame(&mut app);
                if !app.world().resource::<PlayerState>().grounded
                    && app.world().resource::<PlayerState>().foot.y < spawn.y - 0.5
                {
                    break;
                }
            }
            assert!(!app.world().resource::<PlayerState>().grounded);
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            let before = app.world().resource::<PlayerState>().foot;
            let view = *app.world().get::<Transform>(camera).unwrap();
            app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(fixed_ticks));
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            app.world_mut()
                .get_mut::<Gamepad>(pad)
                .unwrap()
                .analog_mut()
                .set(GamepadAxis::RightStickX, 0.8);
            focus_window(&mut app, window, false);
            frame(&mut app);
            eprintln!(
                "[focus/open] ticks={fixed_ticks} before={before:?} after={:?} resets={} next={:?}",
                app.world().resource::<PlayerState>().foot,
                app.world().resource::<PlayerState>().resets,
                app.world().resource::<NextState<GamePhase>>()
            );
            assert_eq!(
                app.world().resource::<PlayerState>().foot,
                before,
                "the focus-loss frame must not fall"
            );
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::PendingIfNeq(GamePhase::Paused)
            ));
            // Long enough for the unpaused falling fixture to cross the recovery bounds
            for _ in 0..240 {
                frame(&mut app);
            }
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            assert_eq!(app.world().resource::<PlayerState>().foot, before);
            assert_eq!(*app.world().get::<Transform>(camera).unwrap(), view);
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            focus_window(&mut app, window, true);
            for _ in 0..4 {
                frame(&mut app);
            }
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            assert_eq!(app.world().resource::<PlayerState>().foot, before);
            app.insert_resource(bevy::time::TimeUpdateStrategy::FixedTimesteps(1));
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            frame(&mut app);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::World
            );
            assert_eq!(app.world().resource::<PlayerState>().foot.xz(), before.xz());
            assert_eq!(app.world().resource::<PlayerState>().resets, 0);
            assert!(
                app.world()
                    .get::<Transform>(camera)
                    .unwrap()
                    .rotation
                    .angle_between(view.rotation)
                    < 0.001
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            *app.world_mut().get_mut::<Gamepad>(pad).unwrap() = Gamepad::default();
            frame(&mut app);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            frame(&mut app);
            assert_eq!(app.world().resource::<PlayerState>().resets, 1);
            let recovered = app.world().resource::<PlayerState>().foot;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyD);
            frame(&mut app);
            assert!(app.world().resource::<PlayerState>().foot.x > recovered.x);
        }
    }

    #[test]
    fn focus_events_preserve_menu_loading_and_windowless_world() {
        let mut app = lifecycle_app();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.add_systems(
            OnEnter(GamePhase::Loading),
            (|mut commands: Commands| {
                commands.insert_resource(Preparation(
                    AsyncComputeTaskPool::get().spawn(std::future::pending()),
                ));
            })
            .after(begin_loading),
        );
        for phase in [GamePhase::Title, GamePhase::Failed, GamePhase::Loading] {
            app.world_mut()
                .resource_mut::<NextState<GamePhase>>()
                .set(phase);
            focus_window(&mut app, window, false);
            frame(&mut app);
            assert_eq!(*app.world().resource::<State<GamePhase>>().get(), phase);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
        }
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>();
        player::install(&mut app);
        enter_walk_fixture(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::World
        );
        assert!(matches!(
            app.world().resource::<NextState<GamePhase>>(),
            NextState::PendingIfNeq(GamePhase::Paused)
        ));
        frame(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Paused
        );
        assert_eq!(
            app.world_mut().query::<&Mesh3d>().iter(app.world()).count(),
            1
        );

        let (mut headless, _, _) = walk_app();
        enter_walk_fixture(&mut headless);
        let secondary = headless.world_mut().spawn(Window::default()).id();
        focus_window(&mut headless, secondary, false);
        let before = headless.world().resource::<PlayerState>().foot;
        headless
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        for _ in 0..3 {
            frame(&mut headless);
        }
        assert_eq!(
            *headless.world().resource::<State<GamePhase>>().get(),
            GamePhase::World
        );
        assert!(headless.world().resource::<PlayerState>().foot.x > before.x);
    }

    #[test]
    fn focus_round_trip_without_input_still_pauses() {
        let (mut app, _, _) = walk_app();
        enter_walk_fixture(&mut app);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        focus_window(&mut app, window, false);
        focus_window(&mut app, window, true);
        frame(&mut app);
        assert!(matches!(
            app.world().resource::<NextState<GamePhase>>(),
            NextState::PendingIfNeq(GamePhase::Paused)
        ));
        frame(&mut app);
        assert_eq!(
            *app.world().resource::<State<GamePhase>>().get(),
            GamePhase::Paused
        );
    }

    #[test]
    fn focus_round_trip_rejects_same_frame_actions_and_late_keyboard_repeats() {
        use bevy::input::{
            ButtonState,
            keyboard::{Key, KeyboardFocusLost, KeyboardInput, keyboard_input_system},
        };
        for (key_code, logical_key) in [
            (KeyCode::Enter, Key::Enter),
            (KeyCode::Escape, Key::Escape),
            (KeyCode::Tab, Key::Tab),
        ] {
            let (mut app, _, pad) = walk_app();
            enter_walk_fixture(&mut app);
            app.init_resource::<ButtonInput<Key>>()
                .add_message::<KeyboardFocusLost>()
                .add_systems(PreUpdate, keyboard_input_system);
            let window = app
                .world_mut()
                .spawn((Window::default(), PrimaryWindow))
                .id();
            let send_key = |app: &mut App, state, repeat| {
                app.world_mut().write_message(KeyboardInput {
                    key_code,
                    logical_key: logical_key.clone(),
                    state,
                    text: None,
                    repeat,
                    window,
                });
            };
            send_key(&mut app, ButtonState::Pressed, false);
            focus_window(&mut app, window, false);
            focus_window(&mut app, window, true);
            app.world_mut()
                .get_mut::<Gamepad>(pad)
                .unwrap()
                .digital_mut()
                .press(GamepadButton::South);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            frame(&mut app);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::PendingIfNeq(GamePhase::Paused)
            ));
            app.world_mut().write_message(KeyboardFocusLost);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            for _ in 0..3 {
                frame(&mut app);
            }
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
            app.world_mut()
                .get_mut::<Gamepad>(pad)
                .unwrap()
                .digital_mut()
                .release(GamepadButton::South);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            focus_window(&mut app, window, false);
            frame(&mut app);
            focus_window(&mut app, window, true);
            app.world_mut()
                .get_mut::<Gamepad>(pad)
                .unwrap()
                .digital_mut()
                .press(GamepadButton::South);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            let button = app
                .world_mut()
                .spawn((
                    Interaction::Pressed,
                    ShellButton {
                        phase: GamePhase::Paused,
                        index: 0,
                    },
                ))
                .id();
            frame(&mut app);
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            app.world_mut().despawn(button);
            app.world_mut()
                .get_mut::<Gamepad>(pad)
                .unwrap()
                .digital_mut()
                .release(GamepadButton::South);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            for _ in 0..3 {
                frame(&mut app);
            }
            send_key(&mut app, ButtonState::Pressed, true);
            frame(&mut app);
            eprintln!(
                "[focus/repeat] key={key_code:?} phase={:?} next={:?}",
                app.world().resource::<State<GamePhase>>().get(),
                app.world().resource::<NextState<GamePhase>>()
            );
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::Paused
            );
            assert!(matches!(
                app.world().resource::<NextState<GamePhase>>(),
                NextState::Unchanged
            ));
            send_key(&mut app, ButtonState::Released, false);
            frame(&mut app);
            send_key(&mut app, ButtonState::Pressed, false);
            frame(&mut app);
            frame(&mut app);
            assert_eq!(
                *app.world().resource::<State<GamePhase>>().get(),
                GamePhase::World
            );
        }
    }
}
