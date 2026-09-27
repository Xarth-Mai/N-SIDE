use super::{
    assets::{self, Appearance, Readiness, TrackedAsset},
    geometry::{self, GeometryPart, Ground},
    map::{Building, Map, map_to_world},
};
use bevy::{
    asset::UntypedHandle,
    ecs::entity::EntityHashMap,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::Instant,
};

/// Stable source association for generated geometry and imported descendants
#[derive(Component, Clone, Debug)]
pub struct MapSource(pub String);

pub struct PropPlacement {
    pub source: String,
    pub model: String,
    pub transform: Transform,
}

pub struct PreparedScene {
    pub map: Map,
    pub appearance: Appearance,
    pub parts: Vec<GeometryPart>,
    pub props: Vec<PropPlacement>,
    pub asset_root: PathBuf,
    pub warnings: Vec<String>,
    model_contexts: BTreeMap<String, Vec<String>>,
}

impl PreparedScene {
    pub fn load(project_root: &Path) -> Result<Self, String> {
        let map_path = project_root.join("source-assets/district-map/district.json");
        let map = Map::load(&map_path).map_err(|e| e.to_string())?;
        let appearance_path = project_root.join("source-assets/district-scene/appearance.json");
        let appearance = Appearance::load(&appearance_path)?;
        for (id, role) in appearance.shopfronts.iter().chain(&appearance.displays) {
            if !map.buildings.iter().any(|b| &b.id == id)
                || !appearance.materials.contains_key(role)
            {
                return Err(format!(
                    "[appearance/binding] {}: building={id} material={role:?} does not exist",
                    appearance_path.display()
                ));
            }
        }
        let mut parts = geometry::generate(&map)
            .map_err(|e| format!("[geometry] {}: {e}", map_path.display()))?;
        parts.extend(facades(&map, &appearance)?);
        for part in &mut parts {
            if appearance
                .materials
                .get(&part.material)
                .is_some_and(|m| m.normal_texture.is_some())
            {
                part.mesh.generate_tangents().map_err(|e| {
                    format!(
                        "[geometry/tangent] source={} material={}: {e}",
                        part.source, part.material
                    )
                })?;
            }
        }
        let props = props(&map, &appearance)?;
        let asset_root = project_root.join("game/assets");
        for part in &parts {
            if !appearance.materials.contains_key(&part.material) {
                return Err(format!(
                    "[appearance/binding] {} asset={} source={} material slot has no binding",
                    appearance_path.display(),
                    part.material,
                    part.source
                ));
            }
        }
        let mut warnings = Vec::new();
        for (index, tree) in map.trees.iter().enumerate() {
            for building in &map.buildings {
                if contains(*tree, &building.polygon) {
                    warnings.push(format!("[source-space] {}:/trees/{index} actual={tree:?} overlaps building={}; source position preserved",map_path.display(),building.id));
                }
            }
        }
        let mut model_contexts = BTreeMap::new();
        let mut checked = BTreeSet::new();
        for prop in &props {
            let Some(spec) = appearance.models.get(&prop.model) else {
                return Err(format!(
                    "[appearance/binding] {} source={} model={} has no binding",
                    appearance_path.display(),
                    prop.source,
                    prop.model
                ));
            };
            if checked.insert(&prop.model) {
                if let Err(e) = assets::validate_model(&asset_root, &prop.model, spec) {
                    let sources = props
                        .iter()
                        .filter(|p| p.model == prop.model)
                        .map(|p| p.source.as_str())
                        .collect::<Vec<_>>()
                        .join(",");
                    let error = format!("{e}; affected sources=[{sources}]");
                    if spec.optional {
                        warnings.push(format!("{error}; explicitly optional: omitted"));
                    } else {
                        return Err(error);
                    }
                } else {
                    model_contexts.insert(
                        prop.model.clone(),
                        assets::model_material_contexts(&asset_root, &prop.model, spec)?,
                    );
                }
            }
        }
        Ok(Self {
            map,
            appearance,
            parts,
            props,
            asset_root,
            warnings,
            model_contexts,
        })
    }
}

#[derive(Resource)]
pub struct SceneLoading {
    prepared: Option<PreparedScene>,
    tracked: Vec<TrackedAsset>,
    materials: BTreeMap<String, Handle<StandardMaterial>>,
    models: BTreeMap<String, Handle<WorldAsset>>,
    started: Instant,
    degraded: BTreeSet<String>,
    pub ready: bool,
    pub failure: Option<String>,
}

impl SceneLoading {
    pub fn new(prepared: PreparedScene) -> Self {
        Self {
            prepared: Some(prepared),
            tracked: vec![],
            materials: BTreeMap::new(),
            models: BTreeMap::new(),
            started: Instant::now(),
            degraded: BTreeSet::new(),
            ready: false,
            failure: None,
        }
    }
}

pub struct WorldScenePlugin;
impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_assets)
            .add_systems(Update, finish_loading);
    }
}

fn track(
    tracked: &mut Vec<TrackedAsset>,
    handle: UntypedHandle,
    contexts: Vec<String>,
    optional: bool,
) {
    if let Some(existing) = tracked.iter_mut().find(|a| a.handle.id() == handle.id()) {
        existing.contexts.extend(contexts);
        existing.optional &= optional;
    } else {
        tracked.push(TrackedAsset {
            handle,
            contexts,
            optional,
        });
    }
}

fn load_assets(
    mut loading: ResMut<SceneLoading>,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let prepared = loading.prepared.take().expect("prepared scene");
    for warning in &prepared.warnings {
        warn!("{warning}");
    }
    for (id, spec) in &prepared.appearance.materials {
        let sources = prepared
            .parts
            .iter()
            .filter(|p| &p.material == id)
            .map(|p| p.source.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",");
        if sources.is_empty() {
            continue;
        }
        let mut textures = Vec::new();
        for (slot, path, srgb) in [
            ("base_color", &spec.color_texture, true),
            ("normal", &spec.normal_texture, false),
        ] {
            let handle = path.as_ref().map(|path| {
                let handle: Handle<Image> = server
                    .load_builder()
                    .with_settings(move |settings: &mut ImageLoaderSettings| {
                        settings.is_srgb = srgb;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            anisotropy_clamp: 8,
                            ..ImageSamplerDescriptor::linear()
                        });
                    })
                    .load(path.clone());
                track(
                    &mut loading.tracked,
                    handle.clone().untyped(),
                    vec![format!(
                        "asset={id} file={} material slot={slot} sources=[{sources}]",
                        prepared.asset_root.join(path).display()
                    )],
                    false,
                );
                handle
            });
            textures.push(handle);
        }
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(spec.color[0], spec.color[1], spec.color[2], spec.color[3]),
            base_color_texture: textures[0].clone(),
            normal_map_texture: textures[1].clone(),
            perceptual_roughness: spec.roughness,
            metallic: spec.metallic,
            unlit: spec.unlit,
            uv_transform: Affine2::from_scale(Vec2::new(
                1.0 / spec.tile_meters[0],
                1.0 / spec.tile_meters[1],
            )),
            ..default()
        });
        loading.materials.insert(id.clone(), material);
    }
    for (id, spec) in &prepared.appearance.models {
        let sources = prepared
            .props
            .iter()
            .filter(|p| &p.model == id)
            .map(|p| p.source.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if sources.is_empty() {
            continue;
        }
        if spec.optional && assets::validate_model(&prepared.asset_root, id, spec).is_err() {
            loading.degraded.insert(id.clone());
            continue;
        }
        let handle: Handle<WorldAsset> =
            server.load(GltfAssetLabel::Scene(spec.scene).from_asset(spec.file.clone()));
        let contexts = prepared.model_contexts[id]
            .iter()
            .map(|c| format!("{c} sources=[{sources}]"))
            .collect();
        track(
            &mut loading.tracked,
            handle.clone().untyped(),
            contexts,
            spec.optional,
        );
        loading.models.insert(id.clone(), handle);
    }
    loading.prepared = Some(prepared);
}

fn finish_loading(world: &mut World) {
    let Some(mut loading) = world.remove_resource::<SceneLoading>() else {
        return;
    };
    if loading.ready || loading.failure.is_some() {
        world.insert_resource(loading);
        return;
    }
    let server = world.resource::<AssetServer>();
    let mut pending = false;
    let mut required_failed = false;
    for asset in &loading.tracked {
        match assets::readiness(server, asset) {
            Readiness::Loading => pending = true,
            Readiness::Failed(_) if !asset.optional => required_failed = true,
            _ => {}
        }
    }
    if required_failed {
        let errors = assets::dependency_failures(server, &loading.tracked)
            .into_iter()
            .map(|(error, contexts)| {
                format!(
                    "[asset/dependency] {}\n  cause: {error}",
                    contexts.join("\n  affected: ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        loading.failure = Some(errors);
    } else if loading.started.elapsed().as_secs() > 120 {
        let contexts = loading
            .tracked
            .iter()
            .filter(|a| assets::readiness(server, a) == Readiness::Loading)
            .flat_map(|a| a.contexts.iter().cloned())
            .collect::<Vec<_>>()
            .join("\n");
        loading.failure = Some(format!(
            "[asset/timeout] dependencies incomplete after 120s\n{contexts}"
        ));
    } else if !pending {
        for (id, handle) in &loading.models {
            if server
                .get_load_states(handle.id())
                .is_some_and(|(s, _, r)| s.is_failed() || r.is_failed())
            {
                loading.degraded.insert(id.clone());
                if let Some(asset) = loading
                    .tracked
                    .iter()
                    .find(|a| a.handle.id() == handle.id().untyped())
                {
                    warn!(
                        "[asset/degraded] {} state={:?}; explicitly optional: omitted",
                        asset.contexts.join("; "),
                        assets::readiness(server, asset)
                    );
                }
            }
        }
        let prepared = loading.prepared.take().expect("prepared scene");
        match spawn(
            world,
            prepared,
            &loading.materials,
            &loading.models,
            &mut loading.degraded,
        ) {
            Ok((meshes, props)) => {
                loading.ready = true;
                info!(
                    "[world/ready] meshes={meshes} model_instances={props} dependencies={} failed=0 degraded={} startup_seconds={:.3}",
                    loading.tracked.len(),
                    loading.degraded.len(),
                    loading.started.elapsed().as_secs_f64()
                );
            }
            Err(error) => loading.failure = Some(error),
        }
    }
    if let Some(error) = &loading.failure {
        error!("{error}\n[world/failed] ready=false");
        world.write_message(AppExit::error());
    }
    world.insert_resource(loading);
}

fn spawn(
    world: &mut World,
    prepared: PreparedScene,
    materials: &BTreeMap<String, Handle<StandardMaterial>>,
    models: &BTreeMap<String, Handle<WorldAsset>>,
    degraded: &mut BTreeSet<String>,
) -> Result<(usize, usize), String> {
    let count = prepared.parts.len();
    for part in prepared.parts {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(part.mesh);
        world.spawn((
            Name::new(part.source.clone()),
            MapSource(part.source),
            Mesh3d(mesh),
            MeshMaterial3d(materials[&part.material].clone()),
        ));
    }
    let mut instances = 0;
    for prop in prepared.props {
        if degraded.contains(&prop.model) {
            continue;
        }
        let handle = &models[&prop.model];
        let spec = &prepared.appearance.models[&prop.model];
        let context = format!(
            "asset={} file={} scene={} source={}",
            prop.model,
            prepared.asset_root.join(&spec.file).display(),
            spec.scene,
            prop.source
        );
        let entities = match instantiate_model(world, handle, &context) {
            Ok(entities) => entities,
            Err(error) if spec.optional => {
                warn!("[asset/degraded] {error}; explicitly optional model omitted");
                degraded.insert(prop.model.clone());
                continue;
            }
            Err(error) => return Err(error),
        };
        let parent = world
            .spawn((
                Name::new(prop.source.clone()),
                MapSource(prop.source.clone()),
                prop.transform.with_scale(Vec3::splat(spec.scale)),
                Visibility::default(),
            ))
            .id();
        for entity in entities {
            let root = world.get::<ChildOf>(entity).is_none();
            world
                .entity_mut(entity)
                .insert(MapSource(prop.source.clone()));
            if root {
                world.entity_mut(entity).insert(ChildOf(parent));
            }
        }
        instances += 1;
    }
    Ok((count, instances))
}

fn instantiate_model(
    world: &mut World,
    handle: &Handle<WorldAsset>,
    context: &str,
) -> Result<Vec<Entity>, String> {
    let mut entities = EntityHashMap::default();
    let result=world.resource_scope(|world,assets:Mut<Assets<WorldAsset>>| {
        let asset=assets.get(handle).ok_or_else(||format!("[asset/instantiate] {context}: loaded scene unavailable"))?;
        asset.write_to_world_with(world,&mut entities,&world.resource::<AppTypeRegistry>().clone()).map_err(|e|format!("[asset/instantiate] {context}: {e}"))
    }).and_then(|()| {
        let mut visible=0;
        for &entity in entities.values() {
            if world.get::<Mesh3d>(entity).is_some() {
                visible+=1;
                let material=world.get::<MeshMaterial3d<StandardMaterial>>(entity).ok_or_else(||format!("[asset/material] {context} entity={entity}: missing material after instantiation"))?;
                if world.resource::<Assets<StandardMaterial>>().get(&material.0).is_none() {return Err(format!("[asset/material] {context} entity={entity}: StandardMaterial unavailable"));}
            }
        }
        if visible==0 {return Err(format!("[asset/instantiate] {context}: scene contains no meshes"));}
        Ok(())
    });
    if let Err(error) = result {
        for &entity in entities.values() {
            if let Ok(entity) = world.get_entity_mut(entity) {
                entity.despawn();
            }
        }
        return Err(error);
    }
    Ok(entities.values().copied().collect())
}

fn add_box(
    batches: &mut BTreeMap<String, Mesh>,
    role: &str,
    position: Vec3,
    size: Vec3,
    rotation: Quat,
) -> Result<(), String> {
    let mesh = Mesh::from(Cuboid::from_size(size))
        .transformed_by(Transform::from_translation(position).with_rotation(rotation));
    if let Some(batch) = batches.get_mut(role) {
        batch
            .merge(&mesh)
            .map_err(|e| format!("facade mesh merge: {e}"))?;
    } else {
        batches.insert(role.to_string(), mesh);
    }
    Ok(())
}

fn add_sign(
    batches: &mut BTreeMap<String, Mesh>,
    role: &str,
    center: Vec3,
    width: f32,
    height: f32,
    normal: [f64; 2],
) -> Result<(), String> {
    use bevy::{
        asset::RenderAssetUsages,
        mesh::{Indices, PrimitiveTopology},
    };
    let normal = map_to_world([normal[0], normal[1], 0.0]);
    let right = Vec3::Y.cross(normal);
    let vertices = [
        center - right * width / 2.0 - Vec3::Y * height / 2.0,
        center + right * width / 2.0 - Vec3::Y * height / 2.0,
        center + right * width / 2.0 + Vec3::Y * height / 2.0,
        center - right * width / 2.0 + Vec3::Y * height / 2.0,
    ];
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices.map(|v| v.to_array()).to_vec(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![normal.to_array(); 4])
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
    )
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    if let Some(batch) = batches.get_mut(role) {
        batch
            .merge(&mesh)
            .map_err(|e| format!("sign mesh merge: {e}"))?;
    } else {
        batches.insert(role.into(), mesh);
    }
    Ok(())
}

fn add_frame(
    batches: &mut BTreeMap<String, Mesh>,
    center: Vec3,
    opening: Vec2,
    rotation: Quat,
    sill: bool,
) -> Result<(), String> {
    for x in [-1.0, 1.0] {
        add_box(
            batches,
            "trim",
            center + rotation * Vec3::new(x * (opening.x + 0.1) / 2.0, 0.0, 0.0),
            Vec3::new(0.1, opening.y, 0.28),
            rotation,
        )?;
    }
    add_box(
        batches,
        "trim",
        center + Vec3::Y * (opening.y + 0.1) / 2.0,
        Vec3::new(opening.x + 0.2, 0.1, 0.28),
        rotation,
    )?;
    if sill {
        add_box(
            batches,
            "metal",
            center - Vec3::Y * (opening.y + 0.1) / 2.0,
            Vec3::new(opening.x + 0.28, 0.1, 0.4),
            rotation,
        )?;
    }
    Ok(())
}

fn entry_opening(kind: &str, role: &str) -> Vec2 {
    match (kind, role) {
        ("station" | "cinema", "public") => Vec2::new(3.6, 2.75),
        ("school", "student") => Vec2::new(2.4, 2.6),
        ("interest" | "maker" | "music", "service") => Vec2::new(2.4, 2.6),
        (_, "public") => Vec2::new(1.8, 2.35),
        _ => Vec2::new(1.3, 2.35),
    }
}

fn window_exposed(
    map: &Map,
    ground: &Ground,
    building: &Building,
    p: [f64; 3],
    half: f64,
    bottom: f64,
    normal: [f64; 2],
) -> bool {
    [-half, 0.0, half].into_iter().all(|offset| {
        let sample = [
            p[0] + normal[0] * 0.25 - normal[1] * offset,
            p[1] + normal[1] * 0.25 + normal[0] * offset,
        ];
        ground.height(sample) < bottom - 0.1
            && !map.buildings.iter().any(|other| {
                other.id != building.id
                    && other.elevation < p[2] + (p[2] - bottom)
                    && other.elevation + other.height > bottom
                    && contains(sample, &other.polygon)
                    && !other
                        .design
                        .as_ref()
                        .and_then(|d| d.lightwell.as_ref())
                        .is_some_and(|hole| contains(sample, hole))
            })
    })
}

fn facades(map: &Map, appearance: &Appearance) -> Result<Vec<GeometryPart>, String> {
    let mut parts = Vec::new();
    let ground = Ground::new(map)?;
    for building in &map.buildings {
        let Some(design) = &building.design else {
            continue;
        };
        let mut batches = BTreeMap::new();
        let top = building.elevation + building.height;
        // Inner-court normals face the opening, independent of authored polygon winding
        for (polygon, court) in std::iter::once((&building.polygon, false))
            .chain(design.lightwell.iter().map(|p| (p, true)))
        {
            let signed_area: f64 = polygon
                .iter()
                .zip(polygon.iter().cycle().skip(1))
                .take(polygon.len())
                .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
                .sum();
            let side = if (signed_area >= 0.0) != court {
                1.0
            } else {
                -1.0
            };
            for (a, b) in polygon
                .iter()
                .zip(polygon.iter().cycle().skip(1))
                .take(polygon.len())
            {
                let dx = b[0] - a[0];
                let dy = b[1] - a[1];
                let length = dx.hypot(dy);
                let normal = [side * dy / length, -side * dx / length];
                let rotation = Quat::from_rotation_y(dy.atan2(dx) as f32);
                let on_edge = |p: [f64; 2]| {
                    let t = ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (length * length);
                    (-0.001..=1.001).contains(&t)
                        && ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / length < 0.02
                };
                let public_face = !court
                    && design.entries.iter().any(|e| {
                        let p = map.nodes[&e.node];
                        matches!(e.role.as_str(), "public" | "student") && on_edge([p[0], p[1]])
                    });
                let display = appearance
                    .displays
                    .get(&building.id)
                    .filter(|_| !court)
                    .filter(|_| {
                        design.front.is_some_and(|[u, v]| {
                            [u, v].iter().all(|p| {
                                ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / length < 0.02
                            })
                        })
                    });
                for (floor_index, floor) in design.floors.iter().enumerate() {
                    let ceiling = design
                        .floors
                        .get(floor_index + 1)
                        .map_or(building.elevation + building.height, |f| f.z);
                    if ceiling - floor.z < 2.4 {
                        continue;
                    }
                    let commercial = floor_index == 0
                        && public_face
                        && !matches!(design.kind.as_str(), "school" | "shrine");
                    let (pitch, width, height, center) = if court {
                        (3.8, 1.2, 1.4, 1.65)
                    } else if design.kind == "music" && !commercial {
                        (5.0, 1.4, 0.7, (ceiling - floor.z - 0.7).max(1.7))
                    } else if design.kind == "school" {
                        (3.8, 2.6, 1.6, 1.8)
                    } else if commercial && matches!(design.kind.as_str(), "station" | "cinema") {
                        (
                            4.2,
                            3.1,
                            (ceiling - floor.z - 1.0).min(4.0),
                            (ceiling - floor.z).min(5.0) / 2.0,
                        )
                    } else if commercial {
                        (3.8, 2.35, 1.67, 1.65)
                    } else {
                        (3.8, 1.45, 1.67, 1.65)
                    };
                    let n = (length / pitch).floor() as usize;
                    for i in 0..n {
                        let t = (i as f64 + 0.5) / n as f64;
                        let p = [
                            a[0] + dx * t + normal[0] * 0.055,
                            a[1] + dy * t + normal[1] * 0.055,
                            floor.z + center,
                        ];
                        let width = width as f32;
                        let height = height as f32;
                        if !window_exposed(
                            map,
                            &ground,
                            building,
                            p,
                            f64::from(width) / 2.0,
                            p[2] - f64::from(height) / 2.0,
                            normal,
                        ) {
                            continue;
                        }
                        // Door nodes own their facade bays
                        if design.entries.iter().any(|e| {
                            let door = map.nodes[&e.node];
                            (door[0] - p[0]).hypot(door[1] - p[1])
                                < f64::from(width + entry_opening(&design.kind, &e.role).x) / 2.0
                                    + 0.2
                                && (door[2] - floor.z).abs() < 0.3
                        }) {
                            continue;
                        }
                        if floor_index == 0
                            && let Some(material) = display
                        {
                            let center = map_to_world([
                                p[0] + normal[0] * 0.105,
                                p[1] + normal[1] * 0.105,
                                p[2],
                            ]);
                            add_frame(
                                &mut batches,
                                center,
                                Vec2::new(width, 1.67),
                                rotation,
                                true,
                            )?;
                            // Shallow display boxes keep the original building envelope intact
                            add_sign(&mut batches, material, map_to_world(p), width, 1.67, normal)?;
                            add_box(
                                &mut batches,
                                "trim",
                                center,
                                Vec3::new(0.045, 1.67, 0.12),
                                rotation,
                            )?;
                            add_box(
                                &mut batches,
                                "metal",
                                center - Vec3::Y * 0.12,
                                Vec3::new(width * 0.89, 0.045, 0.18),
                                rotation,
                            )?;
                            continue;
                        }
                        add_box(
                            &mut batches,
                            "trim",
                            map_to_world(p),
                            Vec3::new(width + 0.16, height + 0.18, 0.12),
                            rotation,
                        )?;
                        let p = [p[0] + normal[0] * 0.07, p[1] + normal[1] * 0.07, p[2]];
                        add_box(
                            &mut batches,
                            "glass",
                            map_to_world(p),
                            Vec3::new(width, height, 0.025),
                            rotation,
                        )?;
                    }
                    if floor_index > 0 {
                        let p = [
                            (a[0] + b[0]) / 2.0 + normal[0] * 0.04,
                            (a[1] + b[1]) / 2.0 + normal[1] * 0.04,
                            floor.z,
                        ];
                        add_box(
                            &mut batches,
                            "trim",
                            map_to_world(p),
                            Vec3::new(length as f32, 0.13, 0.18),
                            rotation,
                        )?;
                    }
                }
                // Roof-access nodes describe a route onto a roof, not a door floating above it
                for (_, entry) in design.entries.iter().enumerate().filter(|(i, e)| {
                    !court
                        && !design.entries[..*i]
                            .iter()
                            .any(|previous| previous.node == e.node)
                }) {
                    let p = map.nodes[&entry.node];
                    if p[2] >= top - 0.2 {
                        continue;
                    }
                    let t = ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (length * length);
                    let distance = ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / length;
                    if !(-0.001..=1.001).contains(&t) || distance > 0.02 {
                        continue;
                    }
                    let approach = [p[0] + normal[0] * 0.3, p[1] + normal[1] * 0.3];
                    if let Some(other) = map.buildings.iter().find(|other| {
                        other.id != building.id
                            && other.elevation < p[2] + 1.9
                            && other.elevation + other.height > p[2] + 0.1
                            && contains(approach, &other.polygon)
                    }) {
                        return Err(format!(
                            "[geometry/entry] {}:{} blocked by building {}",
                            building.id, entry.node, other.id
                        ));
                    }
                    if ground.height(approach) > p[2] + 0.35 {
                        return Err(format!(
                            "[geometry/entry] {}:{} terrain obstructs approach: ground={} entry={}",
                            building.id,
                            entry.node,
                            ground.height(approach),
                            p[2]
                        ));
                    }
                    if display.is_some() && (p[2] - building.elevation).abs() < 0.3 {
                        let center = map_to_world([
                            p[0] + normal[0] * 0.16,
                            p[1] + normal[1] * 0.16,
                            p[2] + 1.175,
                        ]);
                        add_frame(&mut batches, center, Vec2::new(1.3, 2.35), rotation, false)?;
                        add_box(
                            &mut batches,
                            if entry.role == "public" {
                                "glass"
                            } else {
                                "metal"
                            },
                            map_to_world([
                                p[0] + normal[0] * 0.04,
                                p[1] + normal[1] * 0.04,
                                p[2] + 1.175,
                            ]),
                            Vec3::new(1.3, 2.35, 0.03),
                            rotation,
                        )?;
                        add_box(
                            &mut batches,
                            "metal",
                            map_to_world([
                                p[0] + normal[0] * 0.1 + dx / length * 0.45,
                                p[1] + normal[1] * 0.1 + dy / length * 0.45,
                                p[2] + 1.1,
                            ]),
                            Vec3::new(0.04, 0.34, 0.08),
                            rotation,
                        )?;
                        continue;
                    }
                    let available_height = design
                        .floors
                        .iter()
                        .find(|f| f.z > p[2] + 0.1)
                        .map_or(top, |f| f.z)
                        - p[2];
                    let requested = entry_opening(&design.kind, &entry.role);
                    let opening = Vec2::new(
                        requested
                            .x
                            .min((2.0 * length * t.min(1.0 - t) - 0.3) as f32),
                        requested.y.min((available_height - 0.3) as f32),
                    );
                    if opening.x < 0.6 || opening.y < 1.9 {
                        return Err(format!(
                            "[geometry/entry] {}:{} lacks room for a doorway",
                            building.id, entry.node
                        ));
                    }
                    let center = map_to_world([
                        p[0] + normal[0] * 0.16,
                        p[1] + normal[1] * 0.16,
                        p[2] + f64::from(opening.y) / 2.0,
                    ]);
                    add_frame(&mut batches, center, opening, rotation, false)?;
                    add_box(
                        &mut batches,
                        if matches!(entry.role.as_str(), "public" | "student") {
                            "glass"
                        } else {
                            "metal"
                        },
                        center - map_to_world([normal[0] * 0.08, normal[1] * 0.08, 0.0]),
                        Vec3::new(opening.x, opening.y, 0.03),
                        rotation,
                    )?;
                    if opening.x > 1.8 {
                        add_box(
                            &mut batches,
                            "metal",
                            center,
                            Vec3::new(0.08, opening.y, 0.10),
                            rotation,
                        )?;
                    }
                    if design.canopy.is_none()
                        && matches!(entry.role.as_str(), "public" | "student")
                    {
                        add_box(
                            &mut batches,
                            "awning",
                            map_to_world([
                                p[0] + normal[0] * 0.55,
                                p[1] + normal[1] * 0.55,
                                p[2] + f64::from(opening.y) + 0.2,
                            ]),
                            Vec3::new(opening.x + 0.8, 0.16, 1.1),
                            rotation,
                        )?;
                    }
                }
                // Roof coping keeps the authored top height and leaves roof paths open
                if !design.floors.iter().any(|f| f.name == "RF") {
                    add_box(
                        &mut batches,
                        "trim",
                        map_to_world([
                            (a[0] + b[0]) / 2.0 + normal[0] * 0.04,
                            (a[1] + b[1]) / 2.0 + normal[1] * 0.04,
                            top - 0.15,
                        ]),
                        Vec3::new(length as f32, 0.3, 0.16),
                        rotation,
                    )?;
                }
            }
        }
        let polygon = &building.polygon;
        if let Some([a, b]) = design.front {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let len = dx.hypot(dy);
            let rotation = Quat::from_rotation_y(dy.atan2(dx) as f32);
            // Orient the actual facade normal away from the footprint centroid
            let center = [
                polygon.iter().map(|p| p[0]).sum::<f64>() / polygon.len() as f64,
                polygon.iter().map(|p| p[1]).sum::<f64>() / polygon.len() as f64,
            ];
            let mut normal = [dy / len, -dx / len];
            let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
            if (mid[0] - center[0]) * normal[0] + (mid[1] - center[1]) * normal[1] < 0.0 {
                normal = [-normal[0], -normal[1]];
            }
            if let Some(depth) = design.canopy {
                add_box(
                    &mut batches,
                    "awning",
                    map_to_world([
                        mid[0] + normal[0] * depth / 2.0,
                        mid[1] + normal[1] * depth / 2.0,
                        building.elevation + 3.0,
                    ]),
                    Vec3::new(len as f32, 0.16, depth as f32),
                    rotation,
                )?;
            }
            if let Some(role) = appearance.shopfronts.get(&building.id) {
                let p = [
                    mid[0] + normal[0] * 0.24,
                    mid[1] + normal[1] * 0.24,
                    building.elevation + 3.55,
                ];
                add_sign(
                    &mut batches,
                    role,
                    map_to_world(p),
                    (len * 0.65).min(9.0) as f32,
                    0.9,
                    normal,
                )?;
                // Original small street-life details stay alongside, clear of the source doorway
                for p in planter_positions(map, building) {
                    add_box(
                        &mut batches,
                        "terracotta",
                        map_to_world([p[0], p[1], p[2] + 0.4]),
                        Vec3::new(0.65, 0.8, 0.65),
                        rotation,
                    )?;
                }
                let p = [
                    a[0] + dx * 0.2 + normal[0] * 0.32,
                    a[1] + dy * 0.2 + normal[1] * 0.32,
                    building.elevation + 5.0,
                ];
                add_box(
                    &mut batches,
                    "metal",
                    map_to_world(p),
                    Vec3::new(0.9, 0.55, 0.45),
                    rotation,
                )?;
            }
        }
        for (material, mesh) in batches {
            parts.push(GeometryPart {
                source: format!("buildings[{}]/derived-facade", building.id),
                material,
                mesh,
            });
        }
    }
    Ok(parts)
}

fn contains(p: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut inside = false;
    for (a, b) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

fn planter_positions(map: &Map, building: &Building) -> Vec<[f64; 3]> {
    let Some(design) = &building.design else {
        return vec![];
    };
    let Some([a, b]) = design.front else {
        return vec![];
    };
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = d[0].hypot(d[1]);
    let mut normal = [d[1] / length, -d[0] / length];
    let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    if contains(
        [mid[0] + normal[0] * 0.1, mid[1] + normal[1] * 0.1],
        &building.polygon,
    ) {
        normal = [-normal[0], -normal[1]];
    }
    [0.1, 0.9]
        .into_iter()
        .map(|t| {
            [
                a[0] + d[0] * t + normal[0] * 0.6,
                a[1] + d[1] * t + normal[1] * 0.6,
                building.elevation,
            ]
        })
        .filter(|p| {
            !design.entries.iter().any(|e| {
                let door = map.nodes[&e.node];
                (door[0] - p[0]).hypot(door[1] - p[1]) < 1.6 && (door[2] - p[2]).abs() < 0.3
            })
        })
        .collect()
}

// Conservative horizontal radii of the normalized models; tests check the actual exported GLBs
fn vegetation_radius(model: &str) -> f64 {
    match model {
        "tree_pine" => 2.3,
        "tree_autumn" => 1.35,
        "rock" => 0.9,
        "grass" => 0.45,
        "flowers" => 0.3,
        "tree_a" | "tree_b" => 3.5,
        "streetlight" => 1.4,
        _ => 0.7,
    }
}

fn vegetation_obstacles(map: &Map) -> Vec<geo::Polygon> {
    let polygon = |points: &[[f64; 2]]| {
        geo::Polygon::new(
            geo::LineString::from(points.iter().map(|p| (p[0], p[1])).collect::<Vec<_>>()),
            vec![],
        )
    };
    let mut areas: Vec<_> = map
        .buildings
        .iter()
        .map(|b| polygon(&b.polygon))
        .chain(
            map.surfaces
                .iter()
                .filter(|s| s.kind != "park" || s.elevated || s.building.is_some())
                .map(|s| polygon(&s.polygon)),
        )
        .chain(
            map.architectures
                .iter()
                .flat_map(|a| &a.fixtures)
                .map(|f| polygon(&f.polygon)),
        )
        .chain(std::iter::once(polygon(&map.terrain.water)))
        .collect();
    for road in &map.roads {
        if road.building.is_some() || matches!(road.kind.as_str(), "interior" | "lift") {
            continue;
        }
        let points: Vec<_> = road.nodes.iter().map(|n| map.nodes[n]).collect();
        let offsets = geometry::road_offsets(&points, road.width);
        for (i, pair) in points.windows(2).enumerate() {
            areas.push(geometry::ribbon(
                pair[0],
                pair[1],
                offsets[i],
                offsets[i + 1],
            ));
        }
    }
    areas
}

fn add_vegetation(map: &Map, ground: &Ground, props: &mut Vec<PropPlacement>) {
    use geo::{BoundingRect, Contains, Distance, Euclidean, LineString, Point, Polygon};
    let obstacles = vegetation_obstacles(map);
    let mut candidates = Vec::new();
    for (i, surface) in map
        .surfaces
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == "park" && !s.elevated && s.building.is_none())
    {
        let area = Polygon::new(
            LineString::from(
                surface
                    .polygon
                    .iter()
                    .map(|p| (p[0], p[1]))
                    .collect::<Vec<_>>(),
            ),
            vec![],
        );
        let bounds = area.bounding_rect().unwrap();
        for (j, uv) in [
            [0.2, 0.2],
            [0.8, 0.8],
            [0.8, 0.2],
            [0.2, 0.8],
            [0.5, 0.2],
            [0.5, 0.8],
        ]
        .iter()
        .enumerate()
        {
            let p = [
                bounds.min().x + bounds.width() * uv[0],
                bounds.min().y + bounds.height() * uv[1],
            ];
            let model = [
                "flowers",
                "grass",
                "rock",
                "flowers",
                "tree_autumn",
                "grass",
            ][j];
            let point = Point::new(p[0], p[1]);
            if area.contains(&point)
                && Euclidean.distance(&point, area.exterior()) > vegetation_radius(model) + 0.4
            {
                candidates.push((format!("surfaces[{i}]/derived-vegetation[{j}]"), model, p));
            }
        }
    }
    for (i, road) in map.roads.iter().enumerate().filter(|(_, r)| {
        r.building.is_none()
            && (r.kind == "trail"
                || (r.kind == "steps" && r.nodes.iter().any(|n| n.starts_with("hill_short_"))))
    }) {
        for (j, pair) in road.nodes.windows(2).enumerate() {
            let a = map.nodes[&pair[0]];
            let b = map.nodes[&pair[1]];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let length = dx.hypot(dy);
            let count = (length / 24.0).ceil().max(1.0) as usize;
            for slot in 0..count {
                for (side_index, side) in [-1.0, 1.0].into_iter().enumerate() {
                    let index = i * 31 + j * 7 + slot * 2 + side_index;
                    let model =
                        ["tree_pine", "grass", "rock", "flowers", "grass", "rock"][index % 6];
                    let distance = road.width / 2.0 + vegetation_radius(model) + 1.0;
                    let t = (slot as f64 + 0.5) / count as f64;
                    let p = [
                        a[0] + dx * t - dy / length * distance * side,
                        a[1] + dy * t + dx / length * distance * side,
                    ];
                    candidates.push((
                        format!("roads[{i}]/segment[{j}]/derived-vegetation[{slot}:{side_index}]"),
                        model,
                        p,
                    ));
                }
            }
        }
    }
    let initial = props.len();
    let height = |p: [f64; 2]| {
        map.surfaces
            .iter()
            .find(|s| {
                s.kind == "park" && !s.elevated && s.building.is_none() && contains(p, &s.polygon)
            })
            .map_or_else(|| ground.height(p), |s| s.elevation)
    };
    for (source, model, p) in candidates {
        if props.len() - initial >= 200 {
            break;
        }
        let radius = vegetation_radius(model);
        let point = Point::new(p[0], p[1]);
        if obstacles
            .iter()
            .any(|area| Euclidean.distance(&point, area) < radius + 0.4)
            || map
                .buildings
                .iter()
                .filter_map(|b| b.design.as_ref())
                .flat_map(|d| &d.entries)
                .any(|e| {
                    let door = map.nodes[&e.node];
                    (door[0] - p[0]).hypot(door[1] - p[1]) < radius + 2.0
                })
            || props.iter().any(|prop| {
                let q = prop.transform.translation;
                (f64::from(q.x) - p[0]).hypot(-f64::from(q.z) - p[1])
                    < radius + vegetation_radius(&prop.model) + 0.25
            })
        {
            continue;
        }
        let (foot, tolerance) = if model.starts_with("tree_") {
            (0.2, 0.25)
        } else if model == "rock" {
            (radius, 0.2)
        } else {
            (radius, 0.08)
        };
        let heights = [[-foot, -foot], [foot, -foot], [foot, foot], [-foot, foot]]
            .map(|d| height([p[0] + d[0], p[1] + d[1]]));
        let low = heights
            .into_iter()
            .fold(f64::INFINITY, f64::min)
            .min(height(p));
        let high = heights
            .into_iter()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(height(p));
        if high - low > tolerance {
            continue;
        }
        props.push(PropPlacement {
            source,
            model: model.into(),
            transform: Transform::from_translation(map_to_world([p[0], p[1], low - 0.015]))
                .with_rotation(Quat::from_rotation_y(
                    (props.len() - initial) as f32 * 2.399,
                )),
        });
    }
}

fn props(map: &Map, appearance: &Appearance) -> Result<Vec<PropPlacement>, String> {
    let ground = Ground::new(map)?;
    let mut props = Vec::new();
    for (i, p) in map.trees.iter().enumerate() {
        let supported = map
            .surfaces
            .iter()
            .filter(|s| s.building.is_none() && contains(*p, &s.polygon))
            .map(|s| s.elevation)
            .max_by(f64::total_cmp);
        let h = supported.unwrap_or_else(|| ground.height(*p));
        props.push(PropPlacement {
            source: format!("trees[{i}]"),
            model: if h >= map.nodes["hillgate"][2] {
                "tree_pine"
            } else if i % 11 == 0 {
                "tree_autumn"
            } else if i % 2 == 0 {
                "tree_a"
            } else {
                "tree_b"
            }
            .into(),
            transform: Transform::from_translation(map_to_world([p[0], p[1], h]))
                .with_rotation(Quat::from_rotation_y(i as f32 * 2.399)),
        });
    }
    for building in map
        .buildings
        .iter()
        .filter(|b| appearance.shopfronts.contains_key(&b.id))
    {
        for (index, p) in planter_positions(map, building).into_iter().enumerate() {
            props.push(PropPlacement {
                source: format!("buildings[{}]/derived-planter[{index}]", building.id),
                model: "shrub".into(),
                transform: Transform::from_translation(map_to_world([p[0], p[1], p[2] + 0.8])),
            });
        }
    }
    for (i, road) in map.roads.iter().enumerate() {
        if !matches!(road.kind.as_str(), "main" | "avenue" | "shore") {
            continue;
        }
        for (j, pair) in road.nodes.windows(2).enumerate() {
            let a = map.nodes[&pair[0]];
            let b = map.nodes[&pair[1]];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let len = dx.hypot(dy);
            if len < 30.0 {
                continue;
            }
            let p = [
                (a[0] + b[0]) / 2.0 + dy / len * (road.width / 2.0 + 0.65),
                (a[1] + b[1]) / 2.0 - dx / len * (road.width / 2.0 + 0.65),
            ];
            if map.buildings.iter().any(|b| contains(p, &b.polygon))
                || contains(p, &map.terrain.water)
            {
                continue;
            }
            let h = map
                .surfaces
                .iter()
                .filter(|s| s.building.is_none() && contains(p, &s.polygon))
                .map(|s| s.elevation)
                .max_by(f64::total_cmp)
                .unwrap_or_else(|| ground.height(p));
            props.push(PropPlacement {
                source: format!("roads[{i}]/segment[{j}]/derived-lamp"),
                model: "streetlight".into(),
                transform: Transform::from_translation(map_to_world([p[0], p[1], h]))
                    .with_rotation(Quat::from_rotation_y(dy.atan2(dx) as f32)),
            });
        }
    }
    add_vegetation(map, &ground, &mut props);
    Ok(props)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vegetation_uses_real_assets_and_keeps_public_space_clear() {
        use geo::{Distance, Euclidean, Point};
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let first = props(&map, &appearance).unwrap();
        let second = props(&map, &appearance).unwrap();
        assert_eq!(first.len(), second.len());
        assert!(first.iter().zip(&second).all(|(a, b)| a.source == b.source
            && a.model == b.model
            && a.transform == b.transform));
        let additions: Vec<_> = first
            .iter()
            .filter(|p| p.source.contains("/derived-vegetation["))
            .collect();
        assert!(!additions.is_empty() && additions.len() <= 200);
        assert!(additions.iter().any(|p| p.source.starts_with("surfaces[")));
        assert!(additions.iter().any(|p| p.source.starts_with("roads[")));
        let obstacles = vegetation_obstacles(&map);
        let ground = Ground::new(&map).unwrap();
        for p in &additions {
            let at = p.transform.translation;
            assert!(at.is_finite());
            let point = Point::new(f64::from(at.x), -f64::from(at.z));
            assert!(obstacles.iter().all(
                |area| Euclidean.distance(&point, area) >= vegetation_radius(&p.model) + 0.399
            ));
            let height = map
                .surfaces
                .iter()
                .find(|s| {
                    s.kind == "park"
                        && !s.elevated
                        && s.building.is_none()
                        && contains([point.x(), point.y()], &s.polygon)
                })
                .map_or_else(|| ground.height([point.x(), point.y()]), |s| s.elevation);
            assert!(
                f64::from(at.y) <= height + 0.001 && height - f64::from(at.y) < 0.27,
                "{} floats or sinks too far",
                p.source
            );
            for other in &first {
                if p.source == other.source {
                    continue;
                }
                let q = other.transform.translation;
                assert!(
                    (at.x - q.x).hypot(at.z - q.z) as f64
                        >= vegetation_radius(&p.model) + vegetation_radius(&other.model) + 0.249,
                    "{} overlaps {}",
                    p.source,
                    other.source
                );
            }
        }
        let mut counts = BTreeMap::new();
        for p in &first {
            *counts.entry(p.model.as_str()).or_insert(0) += 1;
        }
        for name in [
            "tree_pine",
            "tree_autumn",
            "rock",
            "grass",
            "flowers",
            "tree_a",
            "tree_b",
            "shrub",
            "streetlight",
        ] {
            assert!(
                counts.get(name).is_some_and(|n| *n > 0),
                "{name} has no actual placements"
            );
            let spec = appearance.models.get(name).expect("asset binding exists");
            assets::validate_model(&root.join("game/assets"), name, spec).unwrap();
            let asset = gltf::Gltf::open(root.join("game/assets").join(&spec.file)).unwrap();
            let mut radius = 0.0_f64;
            for node in asset.scenes().nth(spec.scene).unwrap().nodes() {
                assert!(
                    node.children().next().is_none(),
                    "bounds probe expects these normalized direct-mesh scene roots"
                );
                let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
                for primitive in node.mesh().unwrap().primitives() {
                    let reader = primitive.reader(|_| asset.blob.as_deref());
                    for p in reader.read_positions().unwrap() {
                        let p = matrix.transform_point3(Vec3::from(p)) * spec.scale;
                        radius = radius.max(f64::from(p.x.hypot(p.z)));
                    }
                }
            }
            assert!(
                radius > 0.0 && radius <= vegetation_radius(name),
                "{name}: exported radius={radius} exceeds placement allowance"
            );
        }
        let mut added_counts = BTreeMap::new();
        for p in &additions {
            *added_counts.entry(p.model.as_str()).or_insert(0) += 1;
        }
        let short_roads: Vec<_> = map
            .roads
            .iter()
            .enumerate()
            .filter(|(_, r)| r.nodes.iter().any(|n| n.starts_with("hill_short_")))
            .map(|(i, _)| format!("roads[{i}]/"))
            .collect();
        if !short_roads.is_empty() {
            assert!(
                additions.iter().any(|p| short_roads
                    .iter()
                    .any(|prefix| p.source.starts_with(prefix))),
                "new uphill route has no accepted vegetation placements"
            );
        }
        eprintln!(
            "vegetation additions={} added={added_counts:?} parks={} trails_or_uphill={} model counts={counts:?}",
            additions.len(),
            additions
                .iter()
                .filter(|p| p.source.starts_with("surfaces["))
                .count(),
            additions
                .iter()
                .filter(|p| p.source.starts_with("roads["))
                .count()
        );
    }

    #[test]
    fn all_authored_facades_cover_entries_courts_and_keep_roofs_clear() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        let mut entries = 0;
        let mut courts = 0;
        let mut kinds = BTreeSet::new();
        for building in map.buildings.iter().filter(|b| b.design.is_some()) {
            let design = building.design.as_ref().unwrap();
            kinds.insert(&design.kind);
            let source = format!("buildings[{}]/derived-facade", building.id);
            let meshes: Vec<_> = parts.iter().filter(|p| p.source == source).collect();
            assert!(!meshes.is_empty(), "{} has no facade geometry", building.id);
            let top = building.elevation + building.height;
            for mesh in &meshes {
                let positions = mesh
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                assert!(positions.iter().flatten().all(|v| v.is_finite()));
                assert!(
                    positions.iter().all(|v| f64::from(v[1]) <= top + 0.001),
                    "{} extends above authored roof",
                    building.id
                );
            }
            let boxes: Vec<_> = meshes
                .iter()
                .filter(|p| matches!(p.material.as_str(), "glass" | "metal"))
                .flat_map(|p| {
                    p.mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap()
                        .as_chunks::<24>()
                        .0
                        .iter()
                })
                .map(|vertices| {
                    vertices.iter().fold(
                        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                        |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
                    )
                })
                .collect();
            let mut seen = BTreeSet::new();
            for entry in &design.entries {
                if !seen.insert(&entry.node) {
                    continue;
                }
                let p = map_to_world(map.nodes[&entry.node]);
                if f64::from(p.y) >= top - 0.2 {
                    continue;
                }
                let count = boxes
                    .iter()
                    .filter(|(lo, hi)| {
                        let c = (*lo + *hi) / 2.0;
                        (lo.y - p.y).abs() < 0.001
                            && hi.y - lo.y >= 1.9
                            && (hi.x - lo.x).max(hi.z - lo.z) > 0.6
                            && Vec2::new(c.x - p.x, c.z - p.z).length() < 0.2
                    })
                    .count();
                assert_eq!(
                    count, 1,
                    "{}:{} missing or duplicated doorway",
                    building.id, entry.node
                );
                entries += 1;
            }
            if let Some(hole) = &design.lightwell {
                assert!(
                    boxes.iter().any(|(lo, hi)| {
                        let c = (*lo + *hi) / 2.0;
                        contains([f64::from(c.x), -f64::from(c.z)], hole)
                    }),
                    "{} inner court has no inward-facing windows",
                    building.id
                );
                courts += 1;
            }
        }
        assert!(entries > 0 && courts > 0 && kinds.len() > 1);
        eprintln!(
            "building facade coverage: buildings={} types={} unique doorways={entries} lightwells={courts}",
            map.buildings.iter().filter(|b| b.design.is_some()).count(),
            kinds.len()
        );
    }

    #[test]
    fn slope_and_neighbour_occlusion_remove_windows_without_hiding_entries() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let ground = Ground::new(&map).unwrap();
        let building = map
            .buildings
            .iter()
            .find(|b| b.id == "V-04")
            .unwrap()
            .clone();
        let p = [76.75, 246.945, building.elevation + 5.85];
        assert!(window_exposed(
            &map,
            &ground,
            &building,
            p,
            0.725,
            p[2] - 0.835,
            [0.0, -1.0]
        ));
        assert!(!window_exposed(
            &map,
            &ground,
            &building,
            [p[0], p[1], building.elevation],
            0.725,
            building.elevation - 0.835,
            [0.0, -1.0]
        ));
        let baseline = facades(&map, &appearance).unwrap();
        let glass_vertices = |parts: &[GeometryPart]| {
            parts
                .iter()
                .find(|p| p.source == "buildings[V-04]/derived-facade" && p.material == "glass")
                .unwrap()
                .mesh
                .count_vertices()
        };
        let mut neighbour = building.clone();
        neighbour.id = "test-occluding-neighbour".into();
        neighbour.polygon = vec![[75.0, 244.0], [78.5, 244.0], [78.5, 246.9], [75.0, 246.9]];
        neighbour.design = None;
        map.buildings.push(neighbour);
        assert!(!window_exposed(
            &map,
            &ground,
            &building,
            p,
            0.725,
            p[2] - 0.835,
            [0.0, -1.0]
        ));
        let occluded = facades(&map, &appearance).unwrap();
        assert!(glass_vertices(&occluded) < glass_vertices(&baseline));
        map.buildings.last_mut().unwrap().elevation = building.elevation + 20.0;
        assert!(window_exposed(
            &map,
            &ground,
            &building,
            p,
            0.725,
            p[2] - 0.835,
            [0.0, -1.0]
        ));
        let neighbour = map.buildings.last_mut().unwrap();
        neighbour.elevation = building.elevation;
        neighbour.polygon = vec![[88.1, 254.0], [89.5, 254.0], [89.5, 256.0], [88.1, 256.0]];
        let error = facades(&map, &appearance)
            .err()
            .expect("blocked doorway must not disappear silently");
        assert!(
            error.contains("V-04:shop_front_door blocked by building test-occluding-neighbour")
        );
    }

    #[test]
    fn displays_stay_on_ground_floor_front_windows() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        for (id, material) in &appearance.displays {
            let building = map.buildings.iter().find(|b| &b.id == id).unwrap();
            let [a, b] = building.design.as_ref().unwrap().front.unwrap();
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let mesh = &parts.iter().find(|p| &p.material == material).unwrap().mesh;
            for p in mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
            {
                let x = f64::from(p[0]);
                let y = -f64::from(p[2]);
                let z = f64::from(p[1]);
                assert!(
                    (((x - a[0]) * dy - (y - a[1]) * dx).abs() / dx.hypot(dy) - 0.055).abs()
                        < 0.001
                );
                assert!((building.elevation + 0.8..building.elevation + 2.5).contains(&z));
            }
        }
    }

    #[test]
    fn sample_frames_leave_recessed_displays_and_doorways_clear() {
        fn hits(mesh: &Mesh, origin: Vec3, direction: Vec3) -> bool {
            let vertices = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            mesh.indices()
                .unwrap()
                .iter()
                .collect::<Vec<_>>()
                .as_chunks::<3>()
                .0
                .iter()
                .any(|ids| {
                    let [a, b, c] = [ids[0], ids[1], ids[2]].map(|i| Vec3::from(vertices[i]));
                    let normal = (b - a).cross(c - a);
                    let denominator = normal.dot(direction);
                    if denominator.abs() < 1e-6 {
                        return false;
                    }
                    let t = normal.dot(a - origin) / denominator;
                    if !(0.0..0.45).contains(&t) {
                        return false;
                    }
                    let p = origin + direction * t;
                    [(a, b), (b, c), (c, a)]
                        .iter()
                        .all(|(u, v)| (*v - *u).cross(p - *u).dot(normal) >= -1e-6)
                })
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        for (id, material) in &appearance.displays {
            let source = format!("buildings[{id}]/derived-facade");
            let trim = &parts
                .iter()
                .find(|p| p.source == source && p.material == "trim")
                .unwrap()
                .mesh;
            let display = &parts.iter().find(|p| &p.material == material).unwrap().mesh;
            let normal = Vec3::from(
                display
                    .attribute(Mesh::ATTRIBUTE_NORMAL)
                    .unwrap()
                    .as_float3()
                    .unwrap()[0],
            );
            let right = Vec3::Y.cross(normal);
            for pane in display
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
                .as_chunks::<4>()
                .0
            {
                let center = pane.iter().map(|p| Vec3::from(*p)).sum::<Vec3>() / 4.0;
                let half_width = (Vec3::from(pane[1]) - Vec3::from(pane[0])).length() / 2.0;
                assert!(
                    !hits(
                        trim,
                        center + normal * 0.4 + right * half_width * 0.5 + Vec3::Y * 0.3,
                        -normal
                    ),
                    "{id}: display hidden behind a solid frame panel"
                );
                assert!(
                    hits(
                        trim,
                        center + normal * 0.4 + right * (half_width + 0.05),
                        -normal
                    ),
                    "{id}: missing projecting jamb"
                );
            }
            let building = map.buildings.iter().find(|b| &b.id == id).unwrap();
            for entry in building
                .design
                .as_ref()
                .unwrap()
                .entries
                .iter()
                .filter(|e| e.role == "public")
            {
                let p = map_to_world(map.nodes[&entry.node]) + Vec3::Y * 1.1;
                assert!(
                    !hits(trim, p + normal * 0.4, -normal),
                    "{id}: door center obstructed by trim"
                );
            }
        }
    }
}
