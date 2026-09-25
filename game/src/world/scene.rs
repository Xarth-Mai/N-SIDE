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
        for (id, role) in &appearance.shopfronts {
            if !map.buildings.iter().any(|b| &b.id == id)
                || !appearance.materials.contains_key(role)
            {
                return Err(format!(
                    "[appearance/binding] {}:/shopfronts/{id}: building or material {role:?} does not exist",
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
    normal: [f64; 2],
) -> Result<(), String> {
    use bevy::{
        asset::RenderAssetUsages,
        mesh::{Indices, PrimitiveTopology},
    };
    let normal = map_to_world([normal[0], normal[1], 0.0]);
    let right = Vec3::Y.cross(normal);
    let vertices = [
        center - right * width / 2.0 - Vec3::Y * 0.45,
        center + right * width / 2.0 - Vec3::Y * 0.45,
        center + right * width / 2.0 + Vec3::Y * 0.45,
        center - right * width / 2.0 + Vec3::Y * 0.45,
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

fn facades(map: &Map, appearance: &Appearance) -> Result<Vec<GeometryPart>, String> {
    let mut parts = Vec::new();
    for building in &map.buildings {
        let Some(design) = &building.design else {
            continue;
        };
        let mut batches = BTreeMap::new();
        let polygon = &building.polygon;
        let signed_area: f64 = polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .take(polygon.len())
            .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
            .sum();
        let side = if signed_area >= 0.0 { 1.0 } else { -1.0 };
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
            for (floor_index, floor) in design.floors.iter().enumerate() {
                let ceiling = design
                    .floors
                    .get(floor_index + 1)
                    .map_or(building.elevation + building.height, |f| f.z);
                if ceiling - floor.z < 2.4 {
                    continue;
                }
                let n = (length / 3.8).floor() as usize;
                for i in 0..n {
                    let t = (i as f64 + 0.5) / n as f64;
                    let p = [
                        a[0] + dx * t + normal[0] * 0.055,
                        a[1] + dy * t + normal[1] * 0.055,
                        floor.z + 1.65,
                    ];
                    // Door nodes own their facade bays
                    if design.entries.iter().any(|e| {
                        let door = map.nodes[&e.node];
                        (door[0] - p[0]).hypot(door[1] - p[1]) < 1.8
                            && (door[2] - floor.z).abs() < 0.3
                    }) {
                        continue;
                    }
                    let width =
                        if floor_index == 0 && matches!(building.kind.as_str(), "shop" | "home") {
                            2.35
                        } else {
                            1.45
                        };
                    add_box(
                        &mut batches,
                        "trim",
                        map_to_world(p),
                        Vec3::new(width + 0.16, 1.85, 0.12),
                        rotation,
                    )?;
                    let p = [p[0] + normal[0] * 0.07, p[1] + normal[1] * 0.07, p[2]];
                    add_box(
                        &mut batches,
                        "glass",
                        map_to_world(p),
                        Vec3::new(width, 1.67, 0.025),
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
            for entry in &design.entries {
                let p = map.nodes[&entry.node];
                let t = ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (length * length);
                let distance = ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / length;
                if !(-0.001..=1.001).contains(&t) || distance > 0.02 {
                    continue;
                }
                let center = [
                    p[0] + normal[0] * 0.10,
                    p[1] + normal[1] * 0.10,
                    p[2] + 1.25,
                ];
                add_box(
                    &mut batches,
                    "trim",
                    map_to_world(center),
                    Vec3::new(1.55, 2.55, 0.16),
                    rotation,
                )?;
                let center = [
                    center[0] + normal[0] * 0.09,
                    center[1] + normal[1] * 0.09,
                    center[2],
                ];
                add_box(
                    &mut batches,
                    if entry.role == "public" {
                        "glass"
                    } else {
                        "metal"
                    },
                    map_to_world(center),
                    Vec3::new(1.3, 2.35, 0.03),
                    rotation,
                )?;
            }
        }
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
            model: if i % 2 == 0 { "tree_a" } else { "tree_b" }.into(),
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
    Ok(props)
}
