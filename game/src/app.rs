use crate::{
    capture::{self, CaptureInput, CaptureTarget},
    ui::{FONT, Tokens, UiFont, UiInput, color},
    world::{
        map::{Map, map_to_world},
        scene::{PreparedScene, SceneLoading, WorldScenePlugin, clear_scene},
        visual::{Antialiasing, DaylightSettings},
    },
};
use bevy::{
    app::ScheduleRunnerPlugin,
    audio::AudioPlugin,
    camera::RenderTarget,
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::TextureFormat,
        settings::{Backends, RenderCreation, WgpuSettings},
    },
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
    text::{FontWeight, LineHeight},
    window::{ExitCondition, PrimaryWindow},
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
    Failed,
}

impl GamePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Loading => "loading",
            Self::World => "world",
            Self::Failed => "failed",
        }
    }
}

#[derive(Resource)]
struct ProjectRoot(PathBuf);
#[derive(Resource)]
struct Preparation(Task<Result<PreparedScene, String>>);
#[derive(Resource, Default)]
struct EntryUi {
    focus: usize,
    gamepad: bool,
}
#[derive(Clone, Copy)]
enum Action {
    Enter,
    Title,
    Quit,
}

fn actions(phase: GamePhase) -> &'static [(&'static str, Action)] {
    match phase {
        GamePhase::Title => &[("进入街区", Action::Enter), ("退出", Action::Quit)],
        GamePhase::Loading => &[("取消并返回标题", Action::Title)],
        GamePhase::World => &[("返回标题", Action::Title)],
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

pub fn run() -> Result<AppExit, String> {
    let mut root = PathBuf::from(".");
    let mut script = None;
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project-root" => {
                root = PathBuf::from(args.next().ok_or("--project-root requires a directory")?)
            }
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
                    "N:SIDE\n--project-root PATH  Project root (default .)\n--capture SCRIPT --output DIRECTORY  Offscreen game-entry evidence\n\nArrow keys / gamepad D-pad select, Enter / South confirm, Escape / East return"
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
        .is_some_and(|r| r.script.scene != "game-entry")
    {
        return Err(
            "n-side --capture requires scene game-entry; district and ui-signal use map_viewer"
                .into(),
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
    install_lifecycle(&mut app);
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
        .add_plugins(WorldScenePlugin)
        .add_systems(OnEnter(GamePhase::Title), enter_title)
        .add_systems(OnEnter(GamePhase::Loading), begin_loading)
        .add_systems(OnEnter(GamePhase::Failed), enter_failed)
        .add_systems(OnEnter(GamePhase::World), |mut ui: ResMut<EntryUi>| {
            ui.focus = 0
        })
        .add_systems(
            RunFixedMainLoop,
            entry_input.after(CaptureInput).in_set(UiInput),
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
    clear_scene(world);
    world.resource_mut::<EntryUi>().focus = 0;
    info!("[game/state] title");
}

fn begin_loading(world: &mut World) {
    clear_scene(world);
    world.remove_resource::<GameLoadError>();
    world.resource_mut::<EntryUi>().focus = 0;
    let root = world.resource::<ProjectRoot>().0.clone();
    world.insert_resource(Preparation(
        AsyncComputeTaskPool::get().spawn(async move { PreparedScene::load(&root) }),
    ));
    info!("[game/state] loading");
}

fn enter_failed(world: &mut World) {
    world.remove_resource::<Preparation>();
    clear_scene(world);
    world.resource_mut::<EntryUi>().focus = 0;
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
    let result = result.and_then(|prepared| Ok((shop_view(&prepared.map)?, prepared)));
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
            commands.insert_resource(SceneLoading::new(prepared));
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
    gamepads: Query<&Gamepad>,
    buttons: Query<(&Interaction, &ShellButton), Changed<Interaction>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    phase: Res<State<GamePhase>>,
    mut next: ResMut<NextState<GamePhase>>,
    mut ui: ResMut<EntryUi>,
    mut exit: MessageWriter<AppExit>,
) {
    if windows.iter().any(|window| !window.focused) || !matches!(*next, NextState::Unchanged) {
        return;
    }
    let pressed = |button| gamepads.iter().any(|pad| pad.just_pressed(button));
    if gamepads
        .iter()
        .any(|pad| pad.get_just_pressed().next().is_some())
    {
        ui.gamepad = true;
    } else if keys.get_just_pressed().next().is_some() {
        ui.gamepad = false;
    }
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::Tab])
        || pressed(GamepadButton::East)
        || pressed(GamepadButton::Start)
    {
        if *phase.get() != GamePhase::Title {
            (*next).set_if_neq(GamePhase::Title);
        }
        return;
    }
    let options = actions(*phase.get());
    if keys.just_pressed(KeyCode::ArrowDown) || pressed(GamepadButton::DPadDown) {
        ui.focus = (ui.focus + 1) % options.len();
    }
    if keys.just_pressed(KeyCode::ArrowUp) || pressed(GamepadButton::DPadUp) {
        ui.focus = (ui.focus + options.len() - 1) % options.len();
    }
    let mut activate = keys.just_pressed(KeyCode::Enter) || pressed(GamepadButton::South);
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
    mut prior: Local<Option<(GamePhase, bool, UVec2)>>,
) {
    let Ok((camera_id, camera)) = cameras.single() else {
        return;
    };
    let Some(size) = camera.physical_viewport_size() else {
        return;
    };
    let page = *phase.get();
    let key = (page, ui.gamepad, size);
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
            } else {
                color(&tokens.colors.base)
            }),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: px(if world { 480.0 } else { 720.0 }),
                        max_width: percent(100),
                        padding: UiRect::all(px(32)),
                        row_gap: px(24),
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
                        GamePhase::World => ("月台杂货 · 街景", "街区预览 · 固定镜头"),
                        GamePhase::Failed => (
                            "暂时无法进入街区",
                            "街区资料或素材加载失败，请重试或返回标题",
                        ),
                    };
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
                            (GamePhase::World, false) => "Esc 返回标题",
                            (GamePhase::World, true) => "B 返回标题",
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("missing-entry-project");
        assert!(!root.exists());
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ProjectRoot(root));
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
}
