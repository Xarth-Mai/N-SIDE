#[path = "map_viewer/capture.rs"]
mod capture;

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    app::ScheduleRunnerPlugin,
    audio::AudioPlugin,
    camera::RenderTarget,
    camera_controller::free_camera::{
        FreeCamera, FreeCameraPlugin, FreeCameraState, run_freecamera_controller,
    },
    prelude::*,
    render::render_resource::TextureFormat,
    render::{
        RenderPlugin,
        settings::{Backends, RenderCreation, WgpuSettings},
        view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    },
    window::{
        CursorGrabMode, CursorOptions, ExitCondition, MonitorSelection, PresentMode, PrimaryWindow,
        WindowMode,
    },
    winit::WinitPlugin,
};
use n_side::world::{
    map::map_to_world,
    scene::{PreparedScene, SceneLoading, WorldScenePlugin},
    visual::{Antialiasing, DaylightSettings},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct CameraViews(Vec<(&'static str, Transform)>);
#[derive(Resource)]
struct CaptureTarget(Option<Handle<Image>>);
#[derive(Resource)]
struct Verification {
    output: PathBuf,
    view: usize,
    frames: Vec<f64>,
    started: Option<Instant>,
    captured: bool,
    capture_done: bool,
    failed: bool,
}

fn main() -> AppExit {
    match run() {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{error}");
            AppExit::error()
        }
    }
}

fn run() -> Result<AppExit, String> {
    let mut root = PathBuf::from(".");
    let mut validate = false;
    let mut verify = None;
    let mut headless = false;
    let mut visual_path = None;
    let mut aa = Antialiasing::Msaa4;
    let mut selected_view = None;
    let mut uncapped = false;
    let mut capture_script = None;
    let mut capture_output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project-root" => {
                root = PathBuf::from(args.next().ok_or("--project-root requires a directory")?)
            }
            "--validate" => validate = true,
            "--capture" => {
                capture_script = Some(PathBuf::from(
                    args.next().ok_or("--capture requires a script")?,
                ))
            }
            "--output" => {
                capture_output = Some(PathBuf::from(
                    args.next().ok_or("--output requires a directory")?,
                ))
            }
            "--visual" => {
                visual_path = Some(PathBuf::from(
                    args.next().ok_or("--visual requires a JSON path")?,
                ))
            }
            "--aa" => {
                aa = args
                    .next()
                    .ok_or("--aa requires msaa4, taa or taa-ssao")?
                    .parse()?
            }
            "--view" => selected_view = Some(args.next().ok_or("--view requires a camera name")?),
            "--uncapped" => uncapped = true,
            "--verify" | "--verify-headless" => {
                headless = arg == "--verify-headless";
                verify = Some(PathBuf::from(
                    args.next().ok_or("--verify requires an output directory")?,
                ))
            }
            "--help" | "-h" => {
                println!(
                    "N:SIDE Map Viewer\n--project-root PATH  Repository root (default .)\n--validate           Check map, geometry, appearance and daylight without a GPU\n--capture SCRIPT --output DIRECTORY  Scripted offscreen interaction evidence\n--verify DIRECTORY   Automated fixed-view render and frame-time check\n--verify-headless DIRECTORY  Vulkan offscreen render check (no window)\n--visual PATH        Daylight JSON (default source-assets/district-scene/daylight.json)\n--aa MODE            msaa4 (default), taa, taa-ssao\n--view NAME          Start at or verify one fixed view\n--uncapped           Disable window VSync for measurement\n\nWASD move, Q/E down/up, Shift accelerate, wheel speed, right mouse look, M toggle capture, Esc release"
                );
                return Ok(AppExit::Success);
            }
            _ => return Err(format!("unknown option {arg:?}; use --help")),
        }
    }
    if capture_script.is_some() != capture_output.is_some() {
        return Err("--capture and --output must be used together".into());
    }
    if capture_script.is_some() && (verify.is_some() || selected_view.is_some() || validate) {
        return Err("--capture selects its own scene/view; cannot combine with --verify, --view or --validate".into());
    }
    let capture = capture_script
        .map(|path| capture::Recording::load(&path, capture_output.unwrap()))
        .transpose()?;
    if let Some(recording) = &capture {
        headless = true;
        selected_view = Some(recording.script.view.clone());
    }
    let (width, height) = capture.as_ref().map_or((2560, 1440), |recording| {
        (recording.script.width, recording.script.height)
    });
    let root = root
        .canonicalize()
        .map_err(|e| format!("[startup/root] {}: {e}", root.display()))?;
    let start = Instant::now();
    let visual_path =
        visual_path.unwrap_or_else(|| root.join("source-assets/district-scene/daylight.json"));
    let visual = DaylightSettings::load(&visual_path)?;
    println!(
        "[visual/config] file={} aa={aa:?} frame_metric=cpu_frame_interval pacing={}",
        visual_path.display(),
        if headless || uncapped {
            "uncapped"
        } else {
            "vsync"
        }
    );
    let prepared = PreparedScene::load(&root)?;
    println!(
        "[map/validated] schema={} nodes={} roads={} buildings={} surfaces={} trees={} meshes={} elapsed_seconds={:.3}",
        prepared.map.version,
        prepared.map.nodes.len(),
        prepared.map.roads.len(),
        prepared.map.buildings.len(),
        prepared.map.surfaces.len(),
        prepared.map.trees.len(),
        prepared.parts.len(),
        start.elapsed().as_secs_f64()
    );
    println!(
        "[geometry/coverage] interior roads omitted={} (exterior scope); derived features retain source paths",
        prepared
            .map
            .roads
            .iter()
            .filter(|r| r.kind == "interior")
            .count()
    );
    if validate {
        for warning in &prepared.warnings {
            eprintln!("WARNING {warning}");
        }
        println!(
            "[validate/pass] CPU geometry and bindings; asynchronous image decoding and GPU rendering require normal launch or --verify"
        );
        return Ok(AppExit::Success);
    }
    let map = &prepared.map;
    let bounds = map.nodes.values().fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |b, p| {
            [
                b[0].min(p[0]),
                b[1].min(p[1]),
                b[2].max(p[0]),
                b[3].max(p[1]),
            ]
        },
    );
    let center = [
        (bounds[0] + bounds[2]) / 2.0,
        (bounds[1] + bounds[3]) / 2.0,
        20.0,
    ];
    let span = (bounds[2] - bounds[0]).max(bounds[3] - bounds[1]);
    let view = |eye: [f64; 3], target: [f64; 3]| {
        Transform::from_translation(map_to_world(eye)).looking_at(map_to_world(target), Vec3::Y)
    };
    let shop = *map
        .nodes
        .get("shop_front_door")
        .ok_or("[viewer/view] /nodes/shop_front_door: missing review anchor")?;
    let cinema = *map
        .nodes
        .get("cinema_roof")
        .ok_or("[viewer/view] /nodes/cinema_roof: missing review anchor")?;
    let mut views = vec![
        (
            "overview",
            view(
                [
                    center[0] + span * 0.15,
                    center[1] - span * 0.90,
                    span * 0.76,
                ],
                center,
            ),
        ),
        (
            "shop",
            view(
                [shop[0] + 22.0, shop[1] - 17.0, shop[2] + 5.0],
                [shop[0], shop[1] + 2.0, shop[2] + 2.0],
            ),
        ),
        (
            "street",
            view(
                [shop[0] + 40.0, shop[1] - 66.0, shop[2] + 22.0],
                [shop[0] - 28.0, shop[1] - 38.0, shop[2] - 2.0],
            ),
        ),
        (
            "cinema",
            view(
                [cinema[0] + 90.0, cinema[1] - 75.0, cinema[2] + 40.0],
                cinema,
            ),
        ),
    ];
    let bridge = map
        .roads
        .iter()
        .find(|r| r.kind == "bridge")
        .ok_or("map has no bridge for the verification view")?;
    let a = map.nodes[&bridge.nodes[0]];
    let b = map.nodes[bridge.nodes.last().unwrap()];
    let bridge_mid = [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ];
    views.push((
        "bridge",
        view(
            [bridge_mid[0] + 60.0, bridge_mid[1] + 50.0, 6.5],
            bridge_mid,
        ),
    ));
    let summit = *map
        .nodes
        .get("summit")
        .ok_or("[viewer/view] /nodes/summit missing")?;
    let foothill = *map
        .nodes
        .get("hillgate")
        .ok_or("[viewer/view] /nodes/hillgate missing")?;
    views.push((
        "hillside",
        view(
            [foothill[0] + 220.0, foothill[1] - 100.0, foothill[2] + 18.0],
            [
                (foothill[0] + summit[0]) / 2.0,
                (foothill[1] + summit[1]) / 2.0,
                (foothill[2] + summit[2]) / 2.0,
            ],
        ),
    ));
    let campus = *map
        .nodes
        .get("fw_e_school_edge_low")
        .ok_or("[viewer/view] /nodes/fw_e_school_edge_low missing")?;
    views.push((
        "campus",
        view(
            [campus[0], campus[1] - 18.0, campus[2] + 2.0],
            [campus[0], campus[1], campus[2] + 6.0],
        ),
    ));
    // Eye heights follow real road nodes rather than the distant shop datum
    for (name, anchor, target) in [
        ("eye-shop", "home", "shop_front_door"),
        ("eye-corner", "market_turn", "bakery_entry"),
        ("eye-shade", "service_shared", "shop_rear_door"),
    ] {
        let mut eye = *map
            .nodes
            .get(anchor)
            .ok_or_else(|| format!("[viewer/view] missing {anchor}"))?;
        let mut target = *map
            .nodes
            .get(target)
            .ok_or_else(|| format!("[viewer/view] missing {target}"))?;
        eye[2] += 1.7;
        target[2] += 1.7;
        println!("[visual/view] name={name} anchor={anchor} eye={eye:?} target={target:?} fov=55");
        views.push((name, view(eye, target)));
    }
    if let Some(name) = selected_view {
        let selected = views
            .iter()
            .position(|(id, _)| *id == name)
            .ok_or_else(|| format!("[viewer/view] unknown view {name:?}"))?;
        if verify.is_some() {
            views = vec![views[selected]];
        } else {
            views.swap(0, selected);
        }
    }
    let initial = views[0].1;
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
                title: "N:SIDE · Map Viewer".into(),
                resolution: (2560, 1440).into(),
                present_mode: if uncapped {
                    PresentMode::AutoNoVsync
                } else {
                    PresentMode::AutoVsync
                },
                mode: if verify.is_some() {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                } else {
                    WindowMode::Windowed
                },
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
            synchronous_pipeline_compilation: capture.is_some(),
            render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            })),
            ..default()
        });
    if headless {
        plugins = plugins.disable::<WinitPlugin>();
    }
    app.add_plugins(plugins);
    visual.install(&mut app);
    app.insert_resource(CaptureTarget(None))
        .insert_resource(SceneLoading::new(prepared))
        .insert_resource(CameraViews(views))
        .add_plugins((WorldScenePlugin, FreeCameraPlugin))
        .add_systems(
            Startup,
            move |mut commands: Commands, mut images: ResMut<Assets<Image>>| {
                let render_target = if headless {
                    let handle = images.add(Image::new_target_texture(
                        width,
                        height,
                        TextureFormat::Rgba8UnormSrgb,
                        None,
                    ));
                    commands.insert_resource(CaptureTarget(Some(handle.clone())));
                    RenderTarget::Image(handle.into())
                } else {
                    RenderTarget::default()
                };
                let camera = commands
                    .spawn((
                        Camera3d::default(),
                        render_target,
                        Projection::Perspective(PerspectiveProjection {
                            fov: 55.0_f32.to_radians(),
                            near: 0.1,
                            far: 7000.0,
                            ..default()
                        }),
                        initial,
                        FreeCamera {
                            walk_speed: 12.0,
                            run_speed: 55.0,
                            friction: 18.0,
                            sensitivity: 0.18,
                            ..default()
                        },
                    ))
                    .id();
                visual.configure_camera(&mut commands, camera, &mut images, aa);
                visual.spawn_lighting(&mut commands);
            },
        )
        .add_systems(
            RunFixedMainLoop,
            camera_focus.before(run_freecamera_controller),
        );
    if headless {
        app.add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::ZERO));
    }
    if let Some(recording) = capture {
        capture::install(&mut app, recording);
    }
    if let Some(output) = verify {
        std::fs::create_dir_all(&output)
            .map_err(|e| format!("[verify/output] {}: {e}", output.display()))?;
        app.insert_resource(bevy::winit::WinitSettings::continuous());
        app.insert_resource(Verification {
            output,
            view: 0,
            frames: vec![],
            started: None,
            captured: false,
            capture_done: false,
            failed: false,
        })
        .add_systems(Update, verify_frames);
    }
    Ok(app.run())
}

fn camera_focus(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    loading: Res<SceneLoading>,
    verification: Option<Res<Verification>>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut cameras: Query<&mut FreeCameraState>,
) {
    // Offscreen capture uses the same focus policy without a physical window
    let mut window = windows.single_mut().ok();
    let focused = window.as_ref().is_none_or(|(window, _)| window.focused);
    for mut state in &mut cameras {
        if !focused
            || keys.just_pressed(KeyCode::Escape)
            || !loading.ready
            || verification.is_some()
        {
            state.enabled = false;
            state.velocity = Vec3::ZERO;
            state.rotation_curve = None;
            if let Some((_, cursor)) = &mut window {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
        } else if mouse.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::KeyM) {
            state.enabled = true;
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects verification resources and queries"
)]
fn verify_frames(
    mut commands: Commands,
    mut verify: ResMut<Verification>,
    views: Res<CameraViews>,
    loading: Res<SceneLoading>,
    time: Res<Time<Real>>,
    target: Res<CaptureTarget>,
    mut camera: Query<(&mut Transform, Option<&mut TemporalAntiAliasing>), With<Camera3d>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !loading.ready {
        return;
    }
    let started = *verify.started.get_or_insert_with(Instant::now);
    let elapsed = started.elapsed().as_secs_f64();
    if elapsed > 30.0 && !verify.capture_done {
        error!("[verify/capture] screenshot did not complete in 30 seconds");
        exit.write(AppExit::error());
        return;
    }
    // Three warm-up seconds per view exclude shader compilation and resource upload
    if elapsed > 3.0 {
        verify.frames.push(time.delta_secs_f64() * 1000.0);
    }
    if elapsed > 5.0 && !verify.captured {
        let file = verify
            .output
            .join(format!("{}.png", views.0[verify.view].0));
        commands
            .spawn(
                target
                    .0
                    .as_ref()
                    .map_or_else(Screenshot::primary_window, |handle| {
                        Screenshot::image(handle.clone())
                    }),
            )
            .observe(save_to_disk(file))
            .observe(
                |event: On<ScreenshotCaptured>, mut verify: ResMut<Verification>| {
                    let size = event.image.size();
                    if size.x != 2560 || size.y != 1440 {
                        error!(
                            "[verify/resolution] expected=2560x1440 actual={}x{}",
                            size.x, size.y
                        );
                        verify.failed = true;
                    }
                    verify.capture_done = true;
                },
            );
        verify.captured = true;
    }
    if elapsed > 8.0 && verify.capture_done {
        verify.frames.sort_by(f64::total_cmp);
        let n = verify.frames.len();
        let mean = verify.frames.iter().sum::<f64>() / n.max(1) as f64;
        let p95 = verify.frames[((n as f64 * 0.95).floor() as usize).min(n.saturating_sub(1))];
        info!(
            "[verify/view] view={} frames={n} mean_ms={mean:.2} p95_ms={p95:.2} fps={:.1} profile={} mode={} metric=cpu_frame_interval",
            views.0[verify.view].0,
            1000.0 / mean,
            if cfg!(debug_assertions) {
                "dev"
            } else {
                "release"
            },
            if target.0.is_some() {
                "offscreen"
            } else {
                "window"
            }
        );
        if n < 120 || mean > 1000.0 / 60.0 || p95 > 1000.0 / 60.0 {
            verify.failed = true;
            warn!("[verify/performance] target not met or insufficient samples");
        }
        verify.view += 1;
        if verify.view == views.0.len() {
            exit.write(if verify.failed {
                AppExit::error()
            } else {
                AppExit::Success
            });
            return;
        }
        if let Ok((mut camera, taa)) = camera.single_mut() {
            *camera = views.0[verify.view].1;
            if let Some(mut taa) = taa {
                taa.reset = true;
            }
        }
        verify.started = Some(Instant::now());
        verify.frames.clear();
        verify.captured = false;
        verify.capture_done = false;
    }
}
