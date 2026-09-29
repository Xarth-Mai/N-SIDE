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
use n_side::capture::{self, CaptureInput, CaptureTarget, Recording};
use n_side::ui::{SignalUi, SignalUiPlugin, UiInput};
use n_side::world::{
    geometry::Ground,
    map::{Building, Fixture, Map, map_to_world},
    scene::{PreparedScene, SceneLoading, WorldScenePlugin},
    visual::{Antialiasing, DaylightSettings},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct CameraViews(Vec<(String, Transform)>);
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
    let mut ui_preview = false;
    let mut capture_output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project-root" => {
                root = PathBuf::from(args.next().ok_or("--project-root requires a directory")?)
            }
            "--validate" => validate = true,
            "--ui-preview" => ui_preview = true,
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
                    "N:SIDE Map Viewer\n--project-root PATH  Repository root (default .)\n--ui-preview         Neighborhood Signal UI experiment over the real world\n--validate           Check map, geometry, appearance and daylight without a GPU\n--capture SCRIPT --output DIRECTORY  Scripted offscreen interaction evidence\n--verify DIRECTORY   Automated fixed-view render and frame-time check\n--verify-headless DIRECTORY  Vulkan offscreen render check (no window)\n--visual PATH        Daylight JSON (default source-assets/district-scene/daylight.json)\n--aa MODE            msaa4 (default), taa, taa-ssao\n--view NAME          Start at or verify one fixed view\n--uncapped           Disable window VSync for measurement\n\nWASD move, Q/E down/up, Shift accelerate, wheel speed, right mouse look, M toggle capture, Esc release"
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
    if ui_preview && verify.is_some() {
        return Err("--ui-preview uses --capture with scene ui-signal for font, input and UI state verification; fixed-view --verify is world-only".into());
    }
    let capture = capture_script
        .map(|path| capture::Recording::load(&path, capture_output.unwrap(), &root))
        .transpose()?;
    if let Some(recording) = &capture {
        if matches!(
            recording.script.scene.as_str(),
            "game-entry" | "walk-preview"
        ) {
            return Err("game-entry/walk-preview capture uses the n-side binary".into());
        }
        headless = true;
        ui_preview |= recording.script.scene == "ui-signal";
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
        let mut model_counts = std::collections::BTreeMap::new();
        for prop in &prepared.props {
            *model_counts.entry(prop.model.as_str()).or_insert(0usize) += 1;
        }
        for (model, instances) in model_counts {
            println!("[model/instances] model={model} instances={instances}");
        }
        for warning in &prepared.warnings {
            eprintln!("WARNING {warning}");
        }
        println!(
            "[validate/pass] CPU geometry and bindings; asynchronous image decoding and GPU rendering require normal launch or --verify"
        );
        return Ok(AppExit::Success);
    }
    let mut views = camera_views(&prepared.map)?;
    if let Some(name) = selected_view {
        let selected = views
            .iter()
            .position(|(id, _)| *id == name)
            .ok_or_else(|| format!("[viewer/view] unknown view {name:?}"))?;
        if verify.is_some() {
            views = vec![views[selected].clone()];
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
            camera_focus
                .after(CaptureInput)
                .after(UiInput)
                .before(run_freecamera_controller),
        )
        .add_systems(Update, exit_on_scene_failure);
    if ui_preview {
        app.add_plugins(SignalUiPlugin);
    }
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

fn camera_views(map: &Map) -> Result<Vec<(String, Transform)>, String> {
    let bounds = map.nodes.values().chain(&map.terrain.samples).fold(
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
    let peak = *map
        .terrain
        .samples
        .iter()
        .max_by(|a, b| a[2].total_cmp(&b[2]))
        .ok_or("[viewer/view] /terrain/samples: missing mountain controls")?;
    let ground = Ground::new(map)?;
    let center = [
        (bounds[0] + bounds[2]) / 2.0,
        (bounds[1] + bounds[3]) / 2.0,
        peak[2] * 0.5,
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
                    center[2] + span * 0.76,
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
        .get("home")
        .ok_or("[viewer/view] /nodes/home missing")?;
    let hillside_xy = [foothill[0] + 220.0, foothill[1] - 100.0];
    views.push((
        "hillside",
        view(
            [
                hillside_xy[0],
                hillside_xy[1],
                ground.height(hillside_xy) + 35.0,
            ],
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
        ("eye-slope-support", "upper", "slope_lift_high"),
        (
            "eye-upper-bridge",
            "level_upper_to_slope_platform_bypass",
            "upper",
        ),
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
    // Inspect the transfer landing from the actual low public deck, with a labelled upward view
    let from = map.nodes["cinema_deck_turn"];
    let to = map.nodes["cinema_upper"];
    let eye = [
        from[0] + (to[0] - from[0]) * 0.75,
        from[1] + (to[1] - from[1]) * 0.75,
        from[2] + (to[2] - from[2]) * 0.75 + 1.7,
    ];
    let target = map.nodes["upper_transfer_public"];
    println!("[visual/view] name=eye-transfer-support eye={eye:?} target={target:?} fov=55");
    views.push(("eye-transfer-support", view(eye, target)));
    // Free inspection pose, not a pedestrian eye: expose the short roof bearing behind the shaft
    let roof = map.nodes["cinema_roof"];
    let target = [roof[0], roof[1], roof[2] - 0.8];
    let eye = [roof[0] + 12.0, roof[1] - 7.0, roof[2] - 2.5];
    println!("[visual/view] name=inspect-cinema-bearing eye={eye:?} target={target:?} fov=55");
    views.push(("inspect-cinema-bearing", view(eye, target)));
    // The closer summit naturally leaves a level 55-degree view; keep that view and label the tilt
    let mut eye = map.nodes["home"];
    eye[2] += 1.7;
    let distance = (peak[0] - eye[0]).hypot(peak[1] - eye[1]);
    let target = [
        peak[0],
        peak[1],
        eye[2] + distance * 20.0_f64.to_radians().tan(),
    ];
    println!(
        "[visual/view] name=eye-shop-uphill eye={eye:?} target={target:?} pitch_deg=20 fov=55"
    );
    views.push(("eye-shop-uphill", view(eye, target)));
    // Level eye views keep the skyline's apparent rise visible at the shared 55-degree FOV
    for (name, anchor) in [
        ("eye-station", "station"),
        ("eye-cinema", "fw_w_cinema_s"),
        ("eye-shop-mountain", "home"),
    ] {
        let mut eye = *map
            .nodes
            .get(anchor)
            .ok_or_else(|| format!("[viewer/view] missing {anchor}"))?;
        eye[2] += 1.7;
        let distance = (peak[0] - eye[0]).hypot(peak[1] - eye[1]);
        let elevation_angle = (peak[2] - eye[2]).atan2(distance).to_degrees();
        let target = [peak[0], peak[1], eye[2]];
        println!(
            "[visual/view] name={name} anchor={anchor} eye={eye:?} target={target:?} peak={peak:?} horizontal_distance_m={distance:.1} peak_angle_deg={elevation_angle:.2} fov=55"
        );
        views.push((name, view(eye, target)));
    }
    // Side-on overview exposes the river-to-ridge elevation sequence at the same meter scale
    let profile_eye = [center[0] + span * 1.1, center[1], peak[2] + span * 0.2];
    println!(
        "[visual/view] name=mountain-profile eye={profile_eye:?} target={center:?} terrain_peak={peak:?} fov=55"
    );
    views.push(("mountain-profile", view(profile_eye, center)));
    // The tree is the existing trees[68] instance on the public low stair landing
    let tree = map
        .trees
        .get(68)
        .ok_or("[viewer/view] missing trees[68] inspection anchor")?;
    let terrace = map
        .surfaces
        .iter()
        .find(|s| s.id.as_deref() == Some("shop-steps-low"))
        .ok_or("[viewer/view] missing shop-steps-low tree support")?;
    let approach = map.nodes["shop-steps-low-junction"];
    let entrance = map.nodes["shop-steps-low-entry"];
    let eye = [
        approach[0] + (entrance[0] - approach[0]) * 0.3,
        approach[1] + (entrance[1] - approach[1]) * 0.3,
        approach[2] + (entrance[2] - approach[2]) * 0.3 + 1.7,
    ];
    let target = [tree[0], tree[1], terrace.elevation + 3.0];
    println!(
        "[visual/view] name=inspect-street-tree-detail source=trees[68] support=shop-steps-low eye={eye:?} target={target:?} fov=55"
    );
    views.push(("inspect-street-tree-detail", view(eye, target)));
    let residence = map
        .buildings
        .iter()
        .find(|b| b.id == "V-A13")
        .ok_or("[viewer/view] missing V-A13 residential inspection anchor")?;
    let doorway = map.nodes["v_a13_door"];
    let mut eye = map.nodes["east_mid_junction"];
    eye[2] += 1.7;
    let target = [
        doorway[0],
        doorway[1],
        residence.elevation + residence.height / 2.0,
    ];
    println!(
        "[visual/view] name=inspect-residential-detail source=buildings[V-A13] eye={eye:?} target={target:?} fov=55"
    );
    views.push(("inspect-residential-detail", view(eye, target)));
    views.extend([
        (
            "poster-station",
            view(
                [
                    map.nodes["square"][0],
                    map.nodes["square"][1],
                    map.nodes["square"][2] + 1.7,
                ],
                [-30., 25., 18.7125],
            ),
        ),
        (
            "poster-live",
            view(
                [
                    map.nodes["live_meeting"][0],
                    map.nodes["live_meeting"][1],
                    map.nodes["live_meeting"][2] + 1.7,
                ],
                [734., 38., 23.15],
            ),
        ),
        (
            "inspect-road-old-home",
            view([-8., 318., 52.], [0., 340., 47.]),
        ),
        (
            "inspect-road-home-north",
            view([91., 324., 51.], [109., 345., 50.]),
        ),
        (
            "inspect-road-upper-homes",
            view([143., 363., 68.], [123., 373., 63.]),
        ),
        (
            "inspect-road-foothill-east",
            view([592., 695., 191.], [578., 720., 177.]),
        ),
        (
            "inspect-road-north",
            view([56., 560., 160.], [70., 577., 154.]),
        ),
        (
            "inspect-road-school-east",
            view([564., 306., 49.], [536., 335., 34.]),
        ),
        (
            "inspect-road-school-lower",
            view([524., 304., 42.], [544., 323., 30.]),
        ),
        (
            "inspect-road-market-entry",
            view([31., 77., 26.], [2., 104., 19.]),
        ),
        (
            "inspect-road-market-turn",
            view([42., 172., 32.], [14., 151., 20.]),
        ),
        (
            "inspect-road-music-delivery",
            view([685., -66., 30.], [650., -13., 11.]),
        ),
        (
            "inspect-road-music-west",
            view([532., 74., 28.], [557., 94., 13.]),
        ),
        (
            "inspect-road-upper-residential",
            view([244., 401., 103.], [189., 436., 84.]),
        ),
        (
            "inspect-road-upper-stairs",
            view([177., 452., 103.], [185., 428., 80.]),
        ),
        (
            "inspect-road-upper-stairs-low",
            view([212., 413., 87.], [195., 428., 79.]),
        ),
        (
            "inspect-trail-east-entry",
            view([823., 878., 230.], [793., 843., 201.]),
        ),
        (
            "inspect-trail-turn-six",
            view([388., 958., 435.], [357., 927., 398.]),
        ),
        (
            "inspect-trail-turn-eight",
            view([322., 954., 459.], [298., 931., 434.]),
        ),
        (
            "inspect-road-station-square",
            view([22., 87., 29.], [0., 51., 9.]),
        ),
        (
            "inspect-road-city-west",
            view([-132., -24., 25.], [-157., -53., 7.]),
        ),
        (
            "inspect-road-music-north",
            view([704., 183., 35.], [651., 145., 11.]),
        ),
        (
            "inspect-trail-shrine",
            view([688., 796., 225.], [655., 837., 206.]),
        ),
        (
            "inspect-trail-shrine-door",
            view([627., 824., 214.], [652., 837., 206.]),
        ),
        (
            "inspect-road-foothill-gate",
            view([505., 568., 198.], [542., 617., 171.]),
        ),
        (
            "inspect-road-west-plateau",
            view([-68., 406., 114.], [-73., 457., 88.]),
        ),
        (
            "inspect-road-west-rear",
            view([-42., 537., 126.], [-75., 500., 102.]),
        ),
        (
            "inspect-road-res-north",
            view([34., 541., 153.], [-4., 511., 107.]),
        ),
        (
            "inspect-road-east-door",
            view([172., 477., 112.], [171., 486., 101.]),
        ),
        (
            "inspect-road-east-platform",
            view([309., 393., 104.], [276., 424., 84.]),
        ),
        (
            "inspect-road-wood-homes",
            view([100., 464., 117.], [70., 491., 99.]),
        ),
        (
            "inspect-road-east-plateau",
            view([202., 449., 128.], [188., 485., 101.]),
        ),
        (
            "inspect-road-east-neighbors",
            view([250., 301., 44.], [249., 276., 28.]),
        ),
        (
            "inspect-road-old-low",
            view([-221., 117., 33.], [-179., 151., 17.]),
        ),
        (
            "inspect-road-interest-north",
            view([-327., 296., 34.], [-301., 271., 16.]),
        ),
        (
            "inspect-road-west-link",
            view([-222., 255., 43.], [-190., 281., 26.]),
        ),
        (
            "inspect-road-west-north",
            view([-65., 297., 60.], [-108., 313., 34.]),
        ),
        (
            "inspect-road-old-court",
            view([-66., 254., 41.], [-104., 279., 28.]),
        ),
        (
            "inspect-road-food-lower",
            view([204., 57., 29.], [180., 86., 13.]),
        ),
        (
            "inspect-road-food-north",
            view([253., 207., 45.], [236., 167., 23.]),
        ),
        (
            "inspect-road-river-low",
            view([216., -145., 25.], [181., -108., 4.]),
        ),
        (
            "inspect-road-dock",
            view([344., -143., 24.], [308., -110., 4.]),
        ),
        (
            "inspect-road-river-service",
            view([222., -18., 26.], [210., -50., 8.]),
        ),
        (
            "inspect-road-east-platform-reverse",
            view([260., 400., 111.], [280., 426., 84.]),
        ),
        (
            "inspect-road-shop-court",
            view([76., 169., 44.], [68., 206., 25.]),
        ),
        (
            "inspect-road-shop-west",
            view([51., 199., 37.], [36., 208., 23.5]),
        ),
        (
            "inspect-road-shop-upper",
            view([152., 340., 80.], [138., 362., 59.]),
        ),
        (
            "inspect-road-west-link-upper",
            view([-220., 337., 54.], [-186., 336., 36.]),
        ),
        (
            "inspect-road-music-east-merge",
            view([580., 145., 38.], [620., 169., 15.]),
        ),
        (
            "inspect-road-clinic-approach",
            view([688., 153., 25.], [704., 173., 10.]),
        ),
        (
            "inspect-road-east-housing",
            view([623., 272., 102.], [554., 308., 35.]),
        ),
        (
            "inspect-road-east-upper",
            view([560., 339., 56.], [564., 365., 41.]),
        ),
        (
            "inspect-road-northeast-foot",
            view([559., 674., 188.], [538., 701., 174.]),
        ),
        (
            "inspect-road-platform-shop-back",
            view([108., 295., 57.], [78., 267., 29.]),
        ),
        (
            "inspect-road-platform-loading",
            view([-35., 235., 47.], [-10., 265., 28.]),
        ),
        (
            "inspect-road-platform-middle",
            view([153., 313., 78.], [125., 343., 51.]),
        ),
        (
            "inspect-road-platform-bend",
            view([263., 304., 91.], [228., 333., 53.]),
        ),
        (
            "inspect-road-platform-homes-rear",
            view([211., 449., 116.], [197., 421., 77.]),
        ),
        (
            "inspect-road-platform-neighbors",
            view([247., 245., 49.], [225., 220., 24.]),
        ),
        (
            "inspect-road-platform-river-square",
            view([216., -145., 25.], [181., -82., 6.]),
        ),
        (
            "inspect-road-platform-station-west",
            view([-141., 43., 27.], [-105., 67., 10.]),
        ),
        (
            "inspect-road-platform-summit-east",
            view([270., 930., 460.], [245., 944., 450.]),
        ),
        (
            "inspect-road-platform-summit-west",
            view([203., 988., 461.], [224., 969., 450.]),
        ),
        (
            "inspect-road-platform-dock-tip",
            view([326., -174., 17.], [300., -145., 2.]),
        ),
    ]);
    let mut views: Vec<_> = views
        .into_iter()
        .map(|(name, pose)| (name.to_owned(), pose))
        .collect();
    let mainland: Vec<_> = map
        .buildings
        .iter()
        .filter(|b| b.bank == "district")
        .collect();
    views.push(("city-overview".into(), building_view(&mainland)?));
    for block in &map.blocks {
        let buildings: Vec<_> = mainland
            .iter()
            .copied()
            .filter(|b| {
                map.parcels
                    .iter()
                    .any(|p| Some(&p.id) == b.parcel.as_ref() && p.block == block.id)
            })
            .collect();
        let pose =
            building_view(&buildings).map_err(|e| format!("[viewer/view] {}: {e}", block.id))?;
        println!(
            "[visual/view] name=block-{} buildings={} eye={:?} fov=55",
            block.id,
            buildings.len(),
            pose.translation
        );
        views.push((format!("block-{}", block.id), pose));
    }
    let ascent: Vec<_> = map
        .nodes
        .iter()
        .filter(|(name, _)| name.contains("hill_short"))
        .map(|(_, p)| map_to_world(*p))
        .collect();
    views.push(("ascent-overview".into(), frame_points(&ascent)?));
    for i in 1..=3 {
        let name = format!("hill_short_rest{i}");
        let mut eye = *map
            .nodes
            .get(&name)
            .ok_or_else(|| format!("[viewer/view] missing {name}"))?;
        // Stand within the city-facing side of the 8 m landing, retaining a 1 m edge margin
        let distance = (shop[0] - eye[0]).hypot(shop[1] - eye[1]);
        eye[0] += (shop[0] - eye[0]) / distance * 3.0;
        eye[1] += (shop[1] - eye[1]) / distance * 3.0;
        eye[2] += 1.7;
        views.push((format!("eye-ascent-{i}"), view(eye, shop)));
    }
    // TASK-033 round-trip frame 17000: measured camera at the 290m descent cut
    // Keep the exact world-space pose for winding regression; revisit it after terrain edits
    views.push((
        "eye-descent-cut".into(),
        Transform {
            translation: Vec3::new(119.85376, 292.6301, -750.14526),
            rotation: Quat::from_xyzw(0.042013165, 0.91828364, 0.10141955, -0.38040003),
            ..default()
        },
    ));
    for architecture in &map.architectures {
        for (index, fixture) in architecture.fixtures.iter().enumerate() {
            let name = format!("fixture-{}-{:02}", architecture.id, index + 1);
            let pose = fixture_view(map, fixture)
                .map_err(|error| format!("[viewer/view] {name}: {error}"))?;
            println!(
                "[visual/view] name={name} eye={:?} occlusion_check=sampled-building-centerline step_m=0.25",
                pose.translation
            );
            views.push((name, pose));
        }
    }
    Ok(views)
}

fn fixture_view(map: &Map, fixture: &Fixture) -> Result<Transform, String> {
    let corners: Vec<_> = fixture
        .polygon
        .iter()
        .flat_map(|p| {
            [fixture.elevation, fixture.elevation + 2.2].map(|h| map_to_world([p[0], p[1], h]))
        })
        .collect();
    let mut target = corners.iter().copied().sum::<Vec3>() / corners.len() as f32;
    target.y = (fixture.elevation
        + match fixture.kind.as_str() {
            "bench" => 0.48,
            "screen" => 1.45,
            "locker" => 1.0,
            _ => 0.035,
        }) as f32;
    for direction in [
        Vec3::new(0.45, 0.75, 1.0),
        Vec3::new(-0.45, 0.75, 1.0),
        Vec3::new(0.45, 0.75, -1.0),
        Vec3::new(-0.45, 0.75, -1.0),
    ] {
        let pose = frame_points_from(&corners, direction)?;
        if clear_building_segment(map, pose.translation, target) {
            return Ok(pose);
        }
    }
    Err(format!(
        "{}: four inspection directions intersect building bodies; author a clear fixture placement or inspection view",
        fixture.name
    ))
}

// Bounded source-volume sampling checks the center sightline, not foliage or full-mesh visibility
fn clear_building_segment(map: &Map, eye: Vec3, target: Vec3) -> bool {
    use geo::{Contains, LineString, Point, Polygon};
    let samples = (eye.distance(target) / 0.25).ceil().max(1.0) as usize;
    map.buildings.iter().all(|building| {
        let ring = |points: &[[f64; 2]]| {
            LineString::from(points.iter().map(|p| (p[0], p[1])).collect::<Vec<_>>())
        };
        let holes = building
            .design
            .as_ref()
            .and_then(|d| d.lightwell.as_ref())
            .map(|h| vec![ring(h)])
            .unwrap_or_default();
        let footprint = Polygon::new(ring(&building.polygon), holes);
        (0..=samples).all(|i| {
            let p = eye.lerp(target, i as f32 / samples as f32);
            f64::from(p.y) <= building.elevation
                || f64::from(p.y) >= building.elevation + building.height
                || !footprint.contains(&Point::new(f64::from(p.x), -f64::from(p.z)))
        })
    })
}

/// Frame actual buildings, excluding unused parcels and the distant mountain extent
fn building_view(buildings: &[&Building]) -> Result<Transform, String> {
    let corners: Vec<_> = buildings
        .iter()
        .flat_map(|b| {
            b.polygon.iter().flat_map(|p| {
                [b.elevation, b.elevation + b.height].map(|h| map_to_world([p[0], p[1], h]))
            })
        })
        .collect();
    frame_points(&corners)
}

fn frame_points(corners: &[Vec3]) -> Result<Transform, String> {
    frame_points_from(corners, Vec3::new(0.45, 0.75, 1.0))
}

fn frame_points_from(corners: &[Vec3], direction: Vec3) -> Result<Transform, String> {
    if corners.is_empty() {
        return Err("view has no geometry to inspect".into());
    }
    let low = corners
        .iter()
        .fold(Vec3::splat(f32::INFINITY), |a, b| a.min(*b));
    let high = corners
        .iter()
        .fold(Vec3::splat(f32::NEG_INFINITY), |a, b| a.max(*b));
    let center = (low + high) * 0.5;
    let direction = direction.normalize();
    let mut pose = Transform::from_translation(center + direction).looking_at(center, Vec3::Y);
    let tan_v = (55.0_f32.to_radians() * 0.5).tan();
    let distance = corners
        .iter()
        .map(|p| {
            let local = pose.rotation.inverse() * (*p - center);
            local.z + (local.y.abs() / tan_v).max(local.x.abs() / (tan_v * 16.0 / 9.0))
        })
        .fold(1.0_f32, f32::max)
        * 1.12;
    pose.translation = center + direction * distance;
    Ok(pose)
}

fn exit_on_scene_failure(
    loading: Res<SceneLoading>,
    recording: Option<Res<Recording>>,
    mut exit: MessageWriter<AppExit>,
) {
    // The recorder writes its failure evidence before ending a capture
    if recording.is_none()
        && let Some(error) = &loading.failure
    {
        error!("[scene/failed] {error}");
        exit.write(AppExit::error());
    }
}

fn camera_focus(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    loading: Res<SceneLoading>,
    verification: Option<Res<Verification>>,
    ui: Option<Res<SignalUi>>,
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
            || ui
                .as_ref()
                .is_some_and(|ui| ui.is_open() || ui.input_consumed)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_views_reject_building_obstructions() {
        let map = Map::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../source-assets/district-map/district.json"),
        )
        .unwrap();
        for architecture in &map.architectures {
            for fixture in &architecture.fixtures {
                let pose = fixture_view(&map, fixture).unwrap();
                assert!(pose.translation.is_finite());
            }
        }
        let rear_drain = map
            .architectures
            .iter()
            .flat_map(|a| &a.fixtures)
            .find(|f| f.name == "背侧检修排水带")
            .unwrap();
        let corners: Vec<_> = rear_drain
            .polygon
            .iter()
            .flat_map(|p| {
                [rear_drain.elevation, rear_drain.elevation + 2.2]
                    .map(|h| map_to_world([p[0], p[1], h]))
            })
            .collect();
        let mut target = corners.iter().copied().sum::<Vec3>() / corners.len() as f32;
        target.y = (rear_drain.elevation + 0.035) as f32;
        assert!(
            !clear_building_segment(&map, frame_points(&corners).unwrap().translation, target),
            "regression must include the original through-building view"
        );
        assert!(clear_building_segment(
            &map,
            fixture_view(&map, rear_drain).unwrap().translation,
            target
        ));
        let shop = map.buildings.iter().find(|b| b.id == "V-04").unwrap();
        let inside = map_to_world([80.0, 254.0, shop.elevation + 1.0]);
        assert!(!clear_building_segment(&map, inside, inside + Vec3::X));
        assert!(!clear_building_segment(
            &map,
            inside + Vec3::X * 15.0,
            inside
        ));
        let sky = inside + Vec3::Y * (shop.height as f32 + 2.0);
        assert!(clear_building_segment(&map, sky, sky + Vec3::X));
    }

    #[test]
    fn every_block_view_frames_all_its_source_buildings() {
        let map = Map::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../source-assets/district-map/district.json"),
        )
        .unwrap();
        let views = camera_views(&map).unwrap();
        let mut covered = std::collections::BTreeSet::new();
        for block in &map.blocks {
            let camera = views
                .iter()
                .find(|(name, _)| *name == format!("block-{}", block.id))
                .unwrap()
                .1;
            for building in map.buildings.iter().filter(|b| {
                map.parcels
                    .iter()
                    .any(|p| Some(&p.id) == b.parcel.as_ref() && p.block == block.id)
            }) {
                assert!(covered.insert(&building.id));
                for p in &building.polygon {
                    for h in [building.elevation, building.elevation + building.height] {
                        let local = camera
                            .compute_affine()
                            .inverse()
                            .transform_point3(map_to_world([p[0], p[1], h]));
                        let half_height = -local.z * (55.0_f32.to_radians() * 0.5).tan();
                        assert!(
                            local.z < -0.1
                                && local.z > -7000.0
                                && local.y.abs() < half_height
                                && local.x.abs() < half_height * 16.0 / 9.0,
                            "{} clipped from {}",
                            building.id,
                            block.id
                        );
                    }
                }
            }
        }
        assert_eq!(
            covered.len(),
            map.buildings
                .iter()
                .filter(|b| b.bank == "district")
                .count()
        );
        assert!(building_view(&[]).is_err());
    }

    #[test]
    fn mountain_views_use_source_scale_and_cover_the_peak() {
        let map = Map::load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../source-assets/district-map/district.json"),
        )
        .unwrap();
        let views = camera_views(&map).unwrap();
        let peak = map_to_world(
            *map.terrain
                .samples
                .iter()
                .max_by(|a, b| a[2].total_cmp(&b[2]))
                .unwrap(),
        );
        for (name, anchor) in [
            ("eye-station", "station"),
            ("eye-cinema", "fw_w_cinema_s"),
            ("eye-shop-mountain", "home"),
        ] {
            let camera = views.iter().find(|(id, _)| *id == name).unwrap().1;
            let expected_eye = map_to_world(map.nodes[anchor]) + Vec3::Y * 1.7;
            assert!(
                camera.translation.distance(expected_eye) < 0.001,
                "{name}: eye height"
            );
            let horizontal = Vec3::new(peak.x - expected_eye.x, 0.0, peak.z - expected_eye.z);
            assert!(
                camera.forward().dot(horizontal.normalize()) > 0.9999,
                "{name}: level view toward summit"
            );
        }
        for (name, camera) in &views {
            assert!(camera.is_finite(), "{name}: invalid transform");
            assert_eq!(camera.scale, Vec3::ONE, "{name}: physical scale changed");
            assert!(
                camera.translation.distance(peak) < 7000.0,
                "{name}: peak outside far plane"
            );
        }
        let uphill = views
            .iter()
            .find(|(id, _)| *id == "eye-shop-uphill")
            .unwrap()
            .1;
        assert!(
            uphill
                .translation
                .distance(map_to_world(map.nodes["home"]) + Vec3::Y * 1.7)
                < 0.001
        );
        assert!((uphill.forward().y.asin().to_degrees() - 20.0).abs() < 0.001);
        for name in ["overview", "mountain-profile", "eye-shop-uphill"] {
            let camera = views.iter().find(|(id, _)| *id == name).unwrap().1;
            let peak_in_view = camera.compute_affine().inverse().transform_point3(peak);
            let half_height = -peak_in_view.z * (55.0_f32.to_radians() * 0.5).tan();
            assert!(peak_in_view.z < -0.1, "{name}: peak behind camera");
            assert!(
                peak_in_view.y.abs() < half_height,
                "{name}: peak vertically clipped"
            );
            assert!(
                peak_in_view.x.abs() < half_height * 16.0 / 9.0,
                "{name}: peak horizontally clipped"
            );
        }
    }
}
