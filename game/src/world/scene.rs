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
        parts.extend(terrace_retainers(&map)?);
        parts.extend(star_screens(&map)?);
        parts.extend(business_signs(&map)?);
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
    started: Option<Instant>,
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
            started: None,
            degraded: BTreeSet::new(),
            ready: false,
            failure: None,
        }
    }
}

pub struct WorldScenePlugin;
impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                load_assets.run_if(resource_exists::<SceneLoading>),
                finish_loading,
            )
                .chain(),
        );
    }
}

/// Remove map roots and their descendants before inserting a new SceneLoading
/// Host cameras, lighting and UI keep their own lifecycle
pub fn clear_scene(world: &mut World) {
    let roots: Vec<_> = world
        .query_filtered::<Entity, (With<MapSource>, Without<ChildOf>)>()
        .iter(world)
        .collect();
    for root in roots {
        world.despawn(root);
    }
    world.remove_resource::<SceneLoading>();
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
    // Polling reinserts this resource, so an added tick is not a load guard
    if loading.started.is_some() {
        return;
    }
    loading.started = Some(Instant::now());
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
    if loading.ready || loading.failure.is_some() || loading.started.is_none() {
        world.insert_resource(loading);
        return;
    }
    let started = loading.started.expect("asset loading started");
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
    } else if started.elapsed().as_secs() > 120 {
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
                    started.elapsed().as_secs_f64()
                );
            }
            Err(error) => loading.failure = Some(error),
        }
    }
    if let Some(error) = &loading.failure {
        error!("{error}\n[world/failed] ready=false");
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
    let mut mesh = Mesh::from(Cuboid::from_size(size));
    // Bevy's cuboid maps each face to 0..1; material tiling is defined in metres
    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .unwrap()
        .as_float3()
        .unwrap();
    let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
        mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
    else {
        unreachable!("Cuboid UVs are Float32x2")
    };
    let uvs: Vec<_> = uvs
        .iter()
        .zip(normals)
        .map(|(uv, normal)| {
            let scale = if normal[0].abs() > 0.5 {
                [size.y, size.z]
            } else if normal[1].abs() > 0.5 {
                [size.x, size.z]
            } else {
                [size.x, size.y]
            };
            [uv[0] * scale[0], uv[1] * scale[1]]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    let mesh = mesh.transformed_by(Transform::from_translation(position).with_rotation(rotation));
    if let Some(batch) = batches.get_mut(role) {
        batch
            .merge(&mesh)
            .map_err(|e| format!("facade mesh merge: {e}"))?;
    } else {
        batches.insert(role.to_string(), mesh);
    }
    Ok(())
}

fn add_shop_canopy(
    batches: &mut BTreeMap<String, Mesh>,
    origin: Vec3,
    width: f32,
    depth: f32,
    outward: Vec3,
) -> Result<(), String> {
    if depth == 0. {
        return Ok(());
    }
    // Keep the 1.5 m detail proportions; shallow canopies shrink in section
    let section_scale = (depth / 1.5).min(1.);
    let design_depth = depth.max(1.5);
    let along = Vec3::Y.cross(outward);
    let run = design_depth - 0.08;
    let slope = outward * run - Vec3::Y * 0.30;
    let rotation = Quat::from_mat3(&Mat3::from_cols(
        along,
        slope.normalize().cross(along),
        slope.normalize(),
    ));
    let center = origin + outward * (depth / 2.) + Vec3::Y * 2.90;
    add_box(
        batches,
        "awning",
        center,
        Vec3::new(width, 0.12 * section_scale, slope.length() * section_scale),
        rotation,
    )?;
    for side in [-1., 1.] {
        add_box(
            batches,
            "metal",
            center + along * side * (width / 2. - 0.025),
            Vec3::new(0.05, 0.14 * section_scale, slope.length() * section_scale),
            rotation,
        )?;
    }
    let flat = Quat::from_mat3(&Mat3::from_cols(along, Vec3::Y, outward));
    add_box(
        batches,
        "awning",
        origin
            + outward * (depth - 0.04 * section_scale)
            + Vec3::Y * (2.66 + 0.24 * (1. - section_scale)),
        Vec3::new(width, 0.20 * section_scale, 0.07 * section_scale),
        flat,
    )?;
    let supports = (width / 3.8).ceil() as usize;
    for index in 0..=supports {
        let offset = along * (-width / 2. + 0.25 + (width - 0.5) * index as f32 / supports as f32);
        let root = origin
            + offset
            + outward * (0.07 * section_scale)
            + Vec3::Y * (2.60 + 0.30 * (1. - section_scale));
        let diagonal = outward * (design_depth - 0.17) + Vec3::Y * 0.11;
        add_box(
            batches,
            "metal",
            root + diagonal * (section_scale / 2.),
            Vec3::new(0.07, 0.07, diagonal.length()) * section_scale,
            Quat::from_rotation_arc(Vec3::Z, diagonal.normalize()),
        )?;
        add_box(
            batches,
            "metal",
            root + Vec3::Y * (0.20 * section_scale),
            Vec3::new(0.11, 0.46 * section_scale, 0.08 * section_scale),
            flat,
        )?;
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

fn add_shop_display_rack(
    batches: &mut BTreeMap<String, Mesh>,
    foot: Vec3,
    rotation: Quat,
) -> Result<(), String> {
    let mut piece = |role: &str, offset: Vec3, size: Vec3| {
        add_box(batches, role, foot + rotation * offset, size, rotation)
    };
    for x in [-0.68, 0.68] {
        for z in [-0.26, 0.26] {
            piece("trim", Vec3::new(x, 0.52, z), Vec3::new(0.05, 1.04, 0.05))?;
        }
    }
    // Fixed stationery displays: stock rests on the timber tiers, clear of the door
    for y in [0.18, 0.62, 0.98] {
        piece(
            "wood_siding",
            Vec3::new(0., y, 0.),
            Vec3::new(1.48, 0.055, 0.64),
        )?;
        piece(
            "wood_siding",
            Vec3::new(0., y + 0.07, -0.295),
            Vec3::new(1.48, 0.085, 0.05),
        )?;
        if y > 0.5 {
            for (x, width, height, role) in [
                (-0.51, 0.22, 0.24, "awning"),
                (-0.18, 0.28, 0.31, "terracotta"),
                (0.18, 0.20, 0.27, "awning"),
                (0.49, 0.24, 0.22, "terracotta"),
            ] {
                piece(
                    role,
                    Vec3::new(x, y + 0.0275 + height / 2., 0.02),
                    Vec3::new(width, height, 0.26),
                )?;
                piece(
                    "shop_ceiling",
                    Vec3::new(x, y + 0.0275 + height * 0.58, 0.154),
                    Vec3::new(width * 0.66, 0.06, 0.008),
                )?;
            }
        }
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

// A small authored batch along the home-to-hill streets; no random facade dressing
fn residential_sample(id: &str) -> bool {
    matches!(
        id,
        "V-04"
            | "V-A07"
            | "V-A08"
            | "V-A09"
            | "V-A13"
            | "V-A14"
            | "V-W08"
            | "V-13"
            | "V-A15"
            | "V-A16"
    )
}

fn add_residential_glazing(
    batches: &mut BTreeMap<String, Mesh>,
    center: Vec3,
    opening: Vec2,
    rotation: Quat,
    outward: Vec3,
    variant: usize,
) -> Result<(), String> {
    // Glass stays outside the shell but behind the frame, with a clear sill lip
    add_frame(batches, center + outward * 0.10, opening, rotation, true)?;
    add_box(
        batches,
        "metal",
        center + outward * 0.10,
        Vec3::new(0.06, opening.y, 0.16),
        rotation,
    )?;
    for pane in 0..2 {
        // Uniform, low-contrast pane tints are not painted environment reflections
        let tint = 0.86 + ((variant + pane) % 3) as f32 * 0.055;
        let mut colors = match batches
            .get_mut("window_glass")
            .and_then(|mesh| mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR))
        {
            Some(bevy::mesh::VertexAttributeValues::Float32x4(colors)) => colors,
            None => Vec::new(),
            _ => unreachable!("residential glazing uses Float32x4 colors"),
        };
        add_box(
            batches,
            "window_glass",
            center
                + rotation * Vec3::X * (pane as f32 - 0.5) * (opening.x + 0.06) / 2.
                + outward * 0.015,
            Vec3::new((opening.x - 0.06) / 2., opening.y, 0.025),
            rotation,
        )?;
        let mesh = batches.get_mut("window_glass").unwrap();
        // add_box merges uncoloured cuboids; preserve one colour per appended vertex
        colors.resize(mesh.count_vertices(), [tint, tint, tint, 1.]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    Ok(())
}

fn add_residential_window(
    batches: &mut BTreeMap<String, Mesh>,
    origin: Vec3,
    rotation: Quat,
    outward: Vec3,
    width: f32,
    kind: &str,
    bay: usize,
) -> Result<(), String> {
    let along = rotation * Vec3::X;
    let mut piece = |material, x, y, depth, size| {
        add_box(
            batches,
            material,
            origin + along * x + Vec3::Y * y + outward * depth,
            size,
            rotation,
        )
    };
    // A drip hood complements the recessed frame and sill in the facade batch
    piece(
        if kind == "slope" { "roof" } else { "metal" },
        0.,
        2.59,
        0.31,
        Vec3::new(width + 0.40, 0.10, 0.58),
    )?;
    if kind == "row" {
        piece(
            "wood_siding",
            0.,
            0.44,
            0.06,
            Vec3::new(width + 0.16, 0.48, 0.08),
        )?;
    }
    if bay.is_multiple_of(2) {
        // Window guards, not accessible balconies: the source has no balcony doors
        let rail_width = width + 0.28;
        for height in [0.80, 1.32] {
            piece(
                "metal",
                0.,
                height,
                0.54,
                Vec3::new(rail_width, 0.045, 0.045),
            )?;
        }
        let gaps = (rail_width / 0.28).ceil() as usize;
        for i in 0..=gaps {
            let x = rail_width * (i as f32 / gaps as f32 - 0.5);
            piece("metal", x, 1.06, 0.54, Vec3::new(0.035, 0.56, 0.035))?;
        }
        for side in [-1., 1.] {
            piece(
                "metal",
                side * rail_width / 2.,
                1.30,
                0.32,
                Vec3::new(0.045, 0.045, 0.46),
            )?;
        }
    } else {
        // Outdoor condenser below a window, with wall brackets and an exposed grille
        piece("trim", 0., 0.44, 0.25, Vec3::new(0.84, 0.46, 0.42))?;
        for y in [0.33, 0.44, 0.55] {
            piece("metal", 0., y, 0.465, Vec3::new(0.66, 0.035, 0.025))?;
        }
        for x in [-0.28, 0.28] {
            piece("metal", x, 0.18, 0.25, Vec3::new(0.06, 0.06, 0.48))?;
        }
    }
    Ok(())
}

fn facades(map: &Map, appearance: &Appearance) -> Result<Vec<GeometryPart>, String> {
    let mut parts = Vec::new();
    let ground = Ground::new(map)?;
    for building in &map.buildings {
        let Some(design) = &building.design else {
            continue;
        };
        let mut batches = BTreeMap::new();
        let mut skin = BTreeMap::new();
        let mut residential = BTreeMap::new();
        let mut rainwater = BTreeMap::new();
        let mut street_display = BTreeMap::new();
        let top = building.elevation + building.height;
        // Keep deep sills and bands outside the shell of the publicly enterable shop
        let interior_clearance = if building.shop_floor().is_some() {
            0.065
        } else {
            0.0
        };
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
                let residential_face = !court
                    && residential_sample(&building.id)
                    && design.front.map_or_else(
                        || {
                            design.entries.iter().any(|e| {
                                let p = map.nodes[&e.node];
                                on_edge([p[0], p[1]])
                            })
                        },
                        |front| front.into_iter().all(on_edge),
                    );
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
                    } else if (design.kind == "station" && floor_index == 0)
                        || (commercial && design.kind == "cinema")
                    {
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
                                p[0] + normal[0] * (0.105 + interior_clearance),
                                p[1] + normal[1] * (0.105 + interior_clearance),
                                p[2],
                            ]);
                            add_frame(
                                &mut batches,
                                center,
                                Vec2::new(width, 1.67),
                                rotation,
                                true,
                            )?;
                            // Timber skirt stays outside the authored wall and below the display
                            add_box(
                                &mut batches,
                                "wood_siding",
                                map_to_world([
                                    p[0] + normal[0] * 0.07,
                                    p[1] + normal[1] * 0.07,
                                    floor.z + 0.42,
                                ]),
                                Vec3::new(width + 0.16, 0.64, 0.10),
                                rotation,
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
                        let p = [
                            p[0] + normal[0] * interior_clearance,
                            p[1] + normal[1] * interior_clearance,
                            p[2],
                        ];
                        let detailed_window = !court
                            && residential_sample(&building.id)
                            && floor_index > 0
                            && window_exposed(
                                map,
                                &ground,
                                building,
                                [
                                    a[0] + dx * t + normal[0] * 0.4,
                                    a[1] + dy * t + normal[1] * 0.4,
                                    floor.z + 1.4,
                                ],
                                f64::from(width) / 2.0 + 0.25,
                                floor.z - 2.5,
                                normal,
                            );
                        if detailed_window {
                            add_residential_glazing(
                                &mut batches,
                                map_to_world(p),
                                Vec2::new(width, height),
                                rotation,
                                map_to_world([normal[0], normal[1], 0.]),
                                floor_index + i,
                            )?;
                        } else {
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
                        if residential_face && detailed_window {
                            add_residential_window(
                                &mut residential,
                                map_to_world([a[0] + dx * t, a[1] + dy * t, floor.z]),
                                rotation,
                                map_to_world([normal[0], normal[1], 0.]),
                                width,
                                &design.kind,
                                i,
                            )?;
                        }
                    }
                    if !court && matches!(design.kind.as_str(), "station" | "music") {
                        let at = |t: f64, outward: f64, z: f64| {
                            map_to_world([
                                a[0] + dx * t + normal[0] * outward,
                                a[1] + dy * t + normal[1] * outward,
                                z,
                            ])
                        };
                        if design.kind == "station" {
                            // Piers share the window grid; doorway bays remain unobstructed
                            for i in 1..n {
                                let t = i as f64 / n as f64;
                                if design.entries.iter().any(|entry| {
                                    let p = map.nodes[&entry.node];
                                    on_edge([p[0], p[1]])
                                        && (p[2] - floor.z).abs() < 0.3
                                        && (p[0] - a[0] - dx * t).hypot(p[1] - a[1] - dy * t)
                                            < f64::from(entry_opening(&design.kind, &entry.role).x)
                                                / 2.0
                                                + 0.35
                                }) {
                                    continue;
                                }
                                add_box(
                                    &mut skin,
                                    "concrete",
                                    at(t, 0.10, (floor.z + ceiling - 0.25) / 2.0),
                                    Vec3::new(0.32, (ceiling - floor.z - 0.25) as f32, 0.24),
                                    rotation,
                                )?;
                            }
                            add_box(
                                &mut skin,
                                "metal",
                                at(0.5, 0.06, ceiling - 0.22),
                                Vec3::new((length - 0.32) as f32, 0.24, 0.16),
                                rotation,
                            )?;
                        } else {
                            // Rainscreen panels fill opaque auditorium walls below clerestories
                            let bottom = floor.z + if floor_index == 0 { 3.0 } else { 0.35 };
                            let upper = ceiling - 1.2;
                            if upper - bottom >= 0.25 {
                                add_box(
                                    &mut skin,
                                    "metal",
                                    at(0.5, 0.06, (bottom + upper) / 2.0),
                                    Vec3::new(
                                        (length - 0.32) as f32,
                                        (upper - bottom) as f32,
                                        0.10,
                                    ),
                                    rotation,
                                )?;
                                let panels = (length / 2.4).ceil() as usize;
                                for i in 1..panels {
                                    add_box(
                                        &mut skin,
                                        "trim",
                                        at(i as f64 / panels as f64, 0.115, (bottom + upper) / 2.0),
                                        Vec3::new(0.065, (upper - bottom) as f32, 0.03),
                                        rotation,
                                    )?;
                                }
                            }
                        }
                    }
                    if floor_index > 0 {
                        let p = [
                            (a[0] + b[0]) / 2.0 + normal[0] * (0.04 + interior_clearance),
                            (a[1] + b[1]) / 2.0 + normal[1] * (0.04 + interior_clearance),
                            floor.z,
                        ];
                        add_box(
                            &mut batches,
                            "trim",
                            map_to_world(p),
                            Vec3::new((length - interior_clearance * 2.0) as f32, 0.13, 0.18),
                            rotation,
                        )?;
                    }
                }
                if residential_face {
                    // Roof-edge collection and corner downpipes leave the window bays clear
                    let outward = map_to_world([normal[0], normal[1], 0.]);
                    add_box(
                        &mut rainwater,
                        "metal",
                        map_to_world([(a[0] + b[0]) / 2., (a[1] + b[1]) / 2., top - 0.35])
                            + outward * 0.13,
                        Vec3::new((length - 0.64) as f32, 0.14, 0.16),
                        rotation,
                    )?;
                    for distance in [0.42, length - 0.42] {
                        let t = distance / length;
                        let foot = [a[0] + dx * t, a[1] + dy * t];
                        let outer = [foot[0] + normal[0] * 0.20, foot[1] + normal[1] * 0.20];
                        let bottom = ground.height(outer).max(building.elevation) + 0.06;
                        if top - 0.40 <= bottom
                            || design.entries.iter().any(|entry| {
                                let p = map.nodes[&entry.node];
                                on_edge([p[0], p[1]])
                                    && (p[0] - foot[0]).hypot(p[1] - foot[1])
                                        < f64::from(entry_opening(&design.kind, &entry.role).x) / 2.
                                            + 0.25
                            })
                        {
                            continue;
                        }
                        add_box(
                            &mut rainwater,
                            "metal",
                            map_to_world([foot[0], foot[1], (bottom + top - 0.40) / 2.])
                                + outward * 0.115,
                            Vec3::new(0.075, (top - 0.40 - bottom) as f32, 0.10),
                            rotation,
                        )?;
                        for floor in &design.floors {
                            if floor.z + 0.30 > bottom {
                                add_box(
                                    &mut rainwater,
                                    "metal",
                                    map_to_world([foot[0], foot[1], floor.z + 0.30])
                                        + outward * 0.09,
                                    Vec3::new(0.13, 0.035, 0.16),
                                    rotation,
                                )?;
                            }
                        }
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
                    let public_shop_door = building.shop_door(&map.nodes).filter(|door| {
                        (0..2).all(|axis| {
                            ((door.line[0][axis] + door.line[1][axis]) * 0.5 - p[axis]).abs() < 0.01
                        })
                    });
                    if display.is_some()
                        && public_shop_door.is_none()
                        && (p[2] - building.elevation).abs() < 0.3
                    {
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
                    let requested = public_shop_door.map_or_else(
                        || entry_opening(&design.kind, &entry.role),
                        |door| {
                            Vec2::new(
                                (door.line[1][0] - door.line[0][0])
                                    .hypot(door.line[1][1] - door.line[0][1])
                                    as f32,
                                geometry::SHOP_DOOR_HEIGHT as f32,
                            )
                        },
                    );
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
                    if public_shop_door.is_none() {
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
                    }
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
                if appearance.shopfronts.contains_key(&building.id) {
                    add_shop_canopy(
                        &mut batches,
                        map_to_world([mid[0], mid[1], building.elevation]),
                        len as f32,
                        depth as f32,
                        map_to_world([normal[0], normal[1], 0.]),
                    )?;
                } else {
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
            }
            if let Some(role) = appearance.shopfronts.get(&building.id) {
                if building.id == "V-04" {
                    let entry = design
                        .entries
                        .iter()
                        .find(|e| e.role == "public")
                        .ok_or_else(|| {
                            "[geometry/display] V-04 has no public entrance anchor".to_string()
                        })?;
                    let foot = map_to_world(map.nodes[&entry.node])
                        + map_to_world([normal[0], normal[1], 0.]) * 0.9;
                    for offset in [-3.4, 3.4] {
                        add_shop_display_rack(
                            &mut street_display,
                            foot + rotation * Vec3::X * offset,
                            rotation,
                        )?;
                    }
                }
                let sign_width = (len * 0.65).min(9.0) as f32;
                add_box(
                    &mut batches,
                    "trim",
                    map_to_world([
                        mid[0] + normal[0] * 0.14,
                        mid[1] + normal[1] * 0.14,
                        building.elevation + 3.55,
                    ]),
                    Vec3::new(sign_width + 0.20, 1.02, 0.18),
                    rotation,
                )?;
                let p = [
                    mid[0] + normal[0] * 0.24,
                    mid[1] + normal[1] * 0.24,
                    building.elevation + 3.55,
                ];
                add_sign(&mut batches, role, map_to_world(p), sign_width, 0.9, normal)?;
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
        for (suffix, batches) in [
            ("derived-facade", batches),
            ("derived-facade/skin", skin),
            ("derived-facade/residential", residential),
            ("derived-facade/rainwater", rainwater),
            ("derived-facade/street-display", street_display),
        ] {
            for (material, mesh) in batches {
                parts.push(GeometryPart {
                    source: format!("buildings[{}]/{suffix}", building.id),
                    material,
                    mesh,
                });
            }
        }
    }
    Ok(parts)
}

// Thicken exposed fill at two existing urban terraces; entry edges remain unchanged
fn terrace_retainers(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let ground = Ground::new(map)?;
    let mut parts = Vec::new();
    for id in ["shop-site", "east-lower-yard"] {
        let (index, surface) = map
            .surfaces
            .iter()
            .enumerate()
            .find(|(_, surface)| surface.id.as_deref() == Some(id))
            .ok_or_else(|| format!("[geometry/retaining] missing surface {id}"))?;
        let [a, mut b] = [surface.polygon[0], surface.polygon[1]];
        if (a[1] - b[1]).abs() > 0.001 || b[0] <= a[0] || surface.elevated {
            return Err(format!(
                "[geometry/retaining] {id}: authored south edge changed"
            ));
        }
        // The shop's southeast corner meets roads[1]'s miter; retain its capsule clearance
        if id == "shop-site" {
            b[0] -= 0.5;
        }
        let width = b[0] - a[0];
        let top = surface.elevation;
        let mut batches = BTreeMap::new();
        let mut piece = |material, x: f64, elevation: f64, outward: f64, size| {
            add_box(
                &mut batches,
                material,
                map_to_world([x, a[1] - outward, elevation]),
                size,
                Quat::IDENTITY,
            )
        };
        // Staggered shallow block facing thickens the existing retained fill, not the road
        let bays = (width / 1.2).ceil() as usize;
        let bay_width = width / bays as f64;
        let lowest = (0..=bays)
            .map(|i| ground.height([a[0] + i as f64 * bay_width, a[1] - 0.19]))
            .fold(f64::INFINITY, f64::min);
        let rows = ((top - 0.18 - lowest) / 0.40).ceil().max(0.) as usize;
        for row in 0..rows {
            let upper = top - 0.18 - row as f64 * 0.40;
            let lower = upper - 0.40;
            let offset = if row.is_multiple_of(2) { 0. } else { 0.5 };
            for bay in 0..=bays {
                let left = (a[0] + (bay as f64 - offset) * bay_width).max(a[0]);
                let right = (a[0] + (bay as f64 + 1. - offset) * bay_width).min(b[0]);
                if right - left < 0.1 {
                    continue;
                }
                let foot = [left, (left + right) / 2., right]
                    .map(|x| ground.height([x, a[1] - 0.19]))
                    .into_iter()
                    .fold(f64::INFINITY, f64::min);
                if top - foot < 0.35 {
                    return Err(format!(
                        "[geometry/retaining] {id}: insufficient exposed fill at bay {bay}"
                    ));
                }
                if upper <= foot {
                    continue;
                }
                piece(
                    "concrete",
                    (left + right) / 2.,
                    (lower + upper) / 2.,
                    0.04,
                    Vec3::new((right - left - 0.015) as f32, 0.385, 0.30),
                )?;
            }
        }
        for bay in 0..bays {
            let left = a[0] + bay as f64 * bay_width;
            let right = left + bay_width;
            piece(
                "concrete",
                (left + right) / 2.,
                top - 0.09,
                0.03,
                Vec3::new((bay_width - 0.008) as f32, 0.18, 0.42),
            )?;
            if bay % 3 == 1 {
                // Wall-mounted weep grille follows the local lower grade
                let x = (left + right) / 2.;
                let drain = ground.height([x, a[1] - 0.22]) + 0.28;
                if drain + 0.12 < top - 0.18 {
                    for dx in [-0.095, 0.095] {
                        piece("metal", x + dx, drain, 0.215, Vec3::new(0.025, 0.18, 0.06))?;
                    }
                    for dz in [-0.0775, 0.0775] {
                        piece("metal", x, drain + dz, 0.215, Vec3::new(0.215, 0.025, 0.06))?;
                    }
                    for dx in [-0.045, 0., 0.045] {
                        piece("metal", x + dx, drain, 0.235, Vec3::new(0.018, 0.13, 0.025))?;
                    }
                }
            }
        }
        for (material, mesh) in batches {
            parts.push(GeometryPart {
                source: format!("/surfaces/{index} ({id})/retaining-detail"),
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

fn business_signs(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let mut parts = Vec::new();
    // These five names belong above the real arrival doors, not the generic shop fascia
    for (id, entry, material, width, height, rise) in [
        ("V-01", "station_entry", "sign_station", 7.5, 1.6, 5.65),
        ("V-35", "game_entry", "sign_byte_beat", 6.0, 1.5, 4.0),
        ("V-36", "models_entry", "sign_frame", 6.0, 1.5, 4.0),
        ("V-39", "st_39_door", "sign_playroom", 6.0, 1.5, 4.0),
        ("V-79", "live_entry", "sign_after9", 8.0, 2.5, 4.4),
    ] {
        let building = map
            .buildings
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| format!("[geometry/sign] {id}: missing source building"))?;
        let p = map
            .nodes
            .get(entry)
            .ok_or_else(|| format!("[geometry/sign] {id}: missing entry {entry}"))?;
        if !building.design.as_ref().is_some_and(|d| {
            d.entries
                .iter()
                .any(|e| e.node == entry && e.role == "public")
        }) {
            return Err(format!(
                "[geometry/sign] {id}:{entry}: not this building's public entry"
            ));
        }
        let (a, b) = building
            .polygon
            .iter()
            .zip(building.polygon.iter().cycle().skip(1))
            .take(building.polygon.len())
            .find(|(a, b)| {
                let d = Vec2::new((b[0] - a[0]) as f32, (b[1] - a[1]) as f32);
                let offset = Vec2::new((p[0] - a[0]) as f32, (p[1] - a[1]) as f32);
                let along = offset.dot(d.normalize());
                offset.perp_dot(d).abs() / d.length() < 0.02
                    && along > width / 2.0 + 0.1
                    && along < d.length() - width / 2.0 - 0.1
            })
            .ok_or_else(|| format!("[geometry/sign] {id}:{entry}: no source wall wide enough"))?;
        let length = (b[0] - a[0]).hypot(b[1] - a[1]);
        let mut normal = [(b[1] - a[1]) / length, -(b[0] - a[0]) / length];
        if contains(
            [p[0] + normal[0] * 0.1, p[1] + normal[1] * 0.1],
            &building.polygon,
        ) {
            normal = [-normal[0], -normal[1]];
        }
        let outward = map_to_world([normal[0], normal[1], 0.]);
        let rotation = Quat::from_mat3(&Mat3::from_cols(Vec3::Y.cross(outward), Vec3::Y, outward));
        let center = map_to_world(*p) + Vec3::Y * rise;
        let mut batches = BTreeMap::new();
        add_box(
            &mut batches,
            "trim",
            center + outward * 0.24,
            Vec3::new(width + 0.16, height + 0.16, 0.24),
            rotation,
        )?;
        add_sign(
            &mut batches,
            material,
            center + outward * 0.365,
            width,
            height,
            normal,
        )?;
        for (material, mesh) in batches {
            parts.push(GeometryPart {
                source: format!("buildings[{id}]/derived-facade/name-sign"),
                material,
                mesh,
            });
        }
    }
    Ok(parts)
}

fn star_screens(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let mut parts = Vec::new();
    for building in &map.buildings {
        // Public station / music venue roofs face their northern arrival courts
        let width = match building.id.as_str() {
            "V-01" => 10.0,
            "V-79" => 8.0,
            _ => continue,
        };
        let height = width * 9.0 / 16.0;
        let west = building
            .polygon
            .iter()
            .map(|p| p[0])
            .fold(f64::INFINITY, f64::min);
        let east = building
            .polygon
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max);
        let north = building
            .polygon
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max);
        let roof = (building.elevation + building.height) as f32;
        let center = map_to_world([(west + east) / 2., north - 1., roof as f64])
            + Vec3::Y * (0.9 + height / 2.);
        let mut batches = BTreeMap::new();
        // The solid back box gives a visible bezel while the image is a one-sided front
        add_box(
            &mut batches,
            "trim",
            center,
            Vec3::new(width + 0.32, height + 0.32, 0.30),
            Quat::IDENTITY,
        )?;
        add_sign(
            &mut batches,
            "poster_anke",
            center + Vec3::NEG_Z * 0.155,
            width,
            height,
            [0., 1.],
        )?;
        for side in [-1., 1.] {
            let foot = Vec3::new(center.x + side * width * 0.32, roof, center.z + 0.55);
            add_box(
                &mut batches,
                "metal",
                foot + Vec3::Y * 0.06,
                Vec3::new(0.65, 0.12, 1.4),
                Quat::IDENTITY,
            )?;
            let top = Vec3::new(foot.x, center.y + height * 0.3, center.z + 0.22);
            let root = foot + Vec3::Y * 0.14;
            let diagonal = top - root;
            add_box(
                &mut batches,
                "metal",
                (top + root) / 2.,
                Vec3::new(0.20, 0.20, diagonal.length()),
                Quat::from_rotation_arc(Vec3::Z, diagonal.normalize()),
            )?;
        }
        for (material, mesh) in batches {
            parts.push(GeometryPart {
                source: format!("buildings[{}]/star-screen", building.id),
                material,
                mesh,
            });
        }
    }
    Ok(parts)
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
    // Authored edge groups stay inside existing gardens and leave their centers open
    for (group, (id, uv)) in [
        ("fw-f-plateau-east-court", [0.32, 0.67]),
        ("fw-e-foothill-east-garden", [0.25, 0.57]),
        ("fw-e-foothill-north-edge", [0.32, 0.68]),
        ("fw-e-foothill-north-edge", [0.71, 0.63]),
    ]
    .into_iter()
    .enumerate()
    {
        let Some((i, surface)) = map.surfaces.iter().enumerate().find(|(_, s)| {
            s.id.as_deref() == Some(id) && s.kind == "park" && !s.elevated && s.building.is_none()
        }) else {
            continue;
        };
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
        let anchor = [
            bounds.min().x + bounds.width() * uv[0],
            bounds.min().y + bounds.height() * uv[1],
        ];
        for (j, (model, offset)) in [
            ("tree_pine", [0.0, 0.0]),
            ("shrub", [3.6, 0.6]),
            ("grass", [4.5, -1.3]),
            ("rock", [-3.8, 0.4]),
            ("shrub", [0.4, -4.5]),
            ("grass", [2.4, -3.8]),
        ]
        .into_iter()
        .enumerate()
        {
            let p = [anchor[0] + offset[0], anchor[1] + offset[1]];
            let point = Point::new(p[0], p[1]);
            if area.contains(&point)
                && Euclidean.distance(&point, area.exterior()) > vegetation_radius(model) + 0.4
            {
                candidates.push((
                    format!("surfaces[{i}]/derived-vegetation[edge-group:{group}:{j}]"),
                    model,
                    p,
                ));
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
    fn terrace_retainers_follow_fill_edges_and_leave_stair_access_clear() {
        use geo::{Contains, Intersects};
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let ground = Ground::new(&map).unwrap();
        let parts = terrace_retainers(&map).unwrap();
        assert_eq!(parts.len(), 4, "two materials per authored terrace");
        let roads: Vec<_> = map
            .roads
            .iter()
            .filter(|r| r.building.is_none() && !matches!(r.kind.as_str(), "lift" | "interior"))
            .flat_map(|r| {
                let points: Vec<_> = r.nodes.iter().map(|n| map.nodes[n]).collect();
                let offsets = geometry::road_offsets(&points, r.width + 0.64);
                points
                    .windows(2)
                    .enumerate()
                    .map(|(i, pair)| geometry::ribbon(pair[0], pair[1], offsets[i], offsets[i + 1]))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut boxes = 0;
        for id in ["shop-site", "east-lower-yard"] {
            let (index, surface) = map
                .surfaces
                .iter()
                .enumerate()
                .find(|(_, s)| s.id.as_deref() == Some(id))
                .unwrap();
            let [a, b] = [surface.polygon[0], surface.polygon[1]];
            let source = format!("/surfaces/{index} ({id})/retaining-detail");
            let details: Vec<_> = parts.iter().filter(|p| p.source == source).collect();
            assert_eq!(details.len(), 2);
            let mut height = 0.0_f64;
            for part in details {
                assert!(matches!(part.material.as_str(), "concrete" | "metal"));
                let positions = part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                assert!(positions.iter().flatten().all(|v| v.is_finite()));
                assert_eq!(positions.len() % 24, 0, "box batch vertex contract changed");
                let indices: Vec<_> = part.mesh.indices().unwrap().iter().collect();
                assert_eq!(indices.len(), positions.len() / 24 * 36);
                assert!(
                    indices
                        .as_chunks::<36>()
                        .0
                        .iter()
                        .enumerate()
                        .all(|(i, face)| face.iter().all(|v| (i * 24..(i + 1) * 24).contains(v))),
                    "merged box indices do not reference their own vertices"
                );
                let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                    part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
                else {
                    panic!("missing UVs")
                };
                assert!(uvs.iter().flatten().all(|v| v.is_finite() && *v >= 0.));
                for vertices in positions.as_chunks::<24>().0 {
                    boxes += 1;
                    let (lo, hi) = vertices.iter().fold(
                        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                        |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
                    );
                    assert!(
                        hi.y <= surface.elevation as f32 + 0.0001,
                        "{id}: raised original platform"
                    );
                    assert!(lo.x >= a[0] as f32 && hi.x <= b[0] as f32);
                    assert!(lo.z >= -a[1] as f32 - 0.181 && hi.z <= -a[1] as f32 + 0.251);
                    let footprint = geo::Rect::new(
                        geo::Coord {
                            x: lo.x as f64,
                            y: -hi.z as f64,
                        },
                        geo::Coord {
                            x: hi.x as f64,
                            y: -lo.z as f64,
                        },
                    )
                    .to_polygon();
                    assert!(
                        roads.iter().all(|r| !r.intersects(&footprint)),
                        "{id}: obstructs road plus capsule clearance"
                    );
                    for building in &map.buildings {
                        assert!(
                            vertices
                                .iter()
                                .all(|p| !contains([p[0] as f64, -p[2] as f64], &building.polygon)),
                            "{id}: building intersection"
                        );
                    }
                    let center = [(lo.x + hi.x) as f64 / 2., a[1] - 0.19];
                    height = height.max(surface.elevation - ground.height(center));
                    if part.material == "metal" {
                        assert!(
                            lo.y as f64 > ground.height(center) + 0.1,
                            "{id}: buried weep outlet"
                        );
                    }
                    let cap_center = geo::Point::new((lo.x + hi.x) as f64 / 2., a[1] + 0.1);
                    if hi.y > surface.elevation as f32 - 0.001 {
                        let surface_poly = geo::Polygon::new(
                            geo::LineString::from(
                                surface
                                    .polygon
                                    .iter()
                                    .map(|p| (p[0], p[1]))
                                    .collect::<Vec<_>>(),
                            ),
                            vec![],
                        );
                        assert!(
                            surface_poly.contains(&cap_center),
                            "{id}: cap lacks platform bearing"
                        );
                    }
                }
            }
            eprintln!(
                "retaining {id}: south_edge={a:?}->{b:?}, exposed_height_max={height:.3}m, road_clearance=0.32m"
            );
        }
        let collision = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        assert_eq!(collision.triangle_count(), boxes * 12);
        eprintln!(
            "retaining details: {} batches, {boxes} boxes, {} collision triangles",
            parts.len(),
            collision.triangle_count()
        );
        let mut invalid = map.clone();
        invalid
            .surfaces
            .iter_mut()
            .find(|s| s.id.as_deref() == Some("shop-site"))
            .unwrap()
            .elevated = true;
        assert!(
            terrace_retainers(&invalid)
                .err()
                .unwrap()
                .contains("authored south edge changed")
        );
    }

    #[test]
    fn shop_street_displays_are_supported_and_keep_public_clearance() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        let displays: Vec<_> = parts
            .iter()
            .filter(|p| p.source.ends_with("/street-display"))
            .collect();
        assert_eq!(displays.len(), 5);
        let shop = map.buildings.iter().find(|b| b.id == "V-04").unwrap();
        let platform = map
            .surfaces
            .iter()
            .find(|s| s.id.as_deref() == Some("shop-site"))
            .unwrap();
        let door = map_to_world(map.nodes["shop_front_door"]);
        let bounds = |vertices: &[[f32; 3]]| {
            vertices.iter().fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
            )
        };
        let existing: Vec<_> = parts
            .iter()
            .filter(|p| p.source == "buildings[V-04]/derived-facade")
            .filter(|p| !appearance.displays.values().any(|role| role == &p.material))
            .flat_map(|p| {
                p.mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()
                    .as_chunks::<24>()
                    .0
                    .iter()
                    .map(|cube| bounds(cube))
            })
            .collect();
        let mut feet = 0;
        let mut cubes = 0;
        for part in &displays {
            assert_eq!(part.source, "buildings[V-04]/derived-facade/street-display");
            assert!(appearance.materials.contains_key(&part.material));
            let vertices = part
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
            else {
                panic!("missing UVs")
            };
            assert!(vertices.iter().flatten().all(|v| v.is_finite()));
            assert!(uvs.iter().flatten().all(|v| v.is_finite() && *v >= 0.));
            for (face, uv) in vertices
                .as_chunks::<4>()
                .0
                .iter()
                .zip(uvs.as_chunks::<4>().0.iter())
            {
                for i in 0..4 {
                    let j = (i + 1) % 4;
                    assert!(
                        (Vec3::from(face[i]).distance(Vec3::from(face[j]))
                            - Vec2::from(uv[i]).distance(Vec2::from(uv[j])))
                        .abs()
                            < 0.0001,
                        "display UVs must preserve metre scale"
                    );
                }
            }
            for cube in vertices.as_chunks::<24>().0.iter() {
                cubes += 1;
                let (lo, hi) = bounds(cube);
                assert!((hi - lo).cmpgt(Vec3::ZERO).all());
                assert!(lo.y >= platform.elevation as f32 - 0.0001);
                assert!(hi.y <= platform.elevation as f32 + 1.4);
                feet += usize::from((lo.y - platform.elevation as f32).abs() < 0.0001);
                assert!(hi.z < door.z - 1. || lo.z > door.z + 1.);
                assert!(
                    existing.iter().all(|(other_lo, other_hi)| {
                        !(hi.min(*other_hi) - lo.max(*other_lo))
                            .cmpgt(Vec3::splat(0.001))
                            .all()
                    }),
                    "rack overlaps an existing window, sill, planter or wall detail"
                );
                for p in cube {
                    let xy = [f64::from(p[0]), -f64::from(p[2])];
                    assert!(contains(xy, &platform.polygon));
                    assert!(!contains(xy, &shop.polygon));
                    assert!(
                        p[0] <= door.x + 1.221,
                        "rack projects into the walking strip"
                    );
                }
            }
        }
        assert_eq!(
            feet, 8,
            "both four-legged racks must rest on the source platform"
        );
        let displays: Vec<_> = parts
            .into_iter()
            .filter(|p| p.source.ends_with("/street-display"))
            .collect();
        let collision = super::super::collision::CollisionWorld::from_parts(&displays).unwrap();
        assert_eq!(collision.triangle_count(), cubes * 12);
        assert!(
            collision
                .capsule_cast(door + Vec3::X * 6., 1.7, 0.32, Vec3::NEG_X * 6., 0.02)
                .is_none(),
            "public entrance approach must stay clear"
        );
        for x in [2.6, 4.6] {
            for z in [-10., 4.] {
                let p = door + Vec3::new(x, 0., z);
                assert!(contains(
                    [f64::from(p.x), -f64::from(p.z)],
                    &platform.polygon
                ));
            }
        }
        assert!(
            collision
                .capsule_cast(
                    door + Vec3::new(3.6, 0., 3.),
                    2.,
                    1.,
                    Vec3::NEG_Z * 12.,
                    0.02
                )
                .is_none(),
            "the 2 m shopfront walking strip must stay clear"
        );
        assert!(
            collision
                .capsule_cast(
                    door + Vec3::new(3., 0., 3.4),
                    1.7,
                    0.32,
                    Vec3::NEG_X * 3.,
                    0.02,
                )
                .is_some(),
            "visible display racks must also block the player capsule"
        );
        eprintln!(
            "shop street displays: 2 supported racks, {cubes} boxes, {} collision triangles",
            collision.triangle_count()
        );
    }

    #[test]
    fn business_signs_keep_source_aspect_arrival_direction_and_openings() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let signs = business_signs(&map).unwrap();
        let facades = facades(&map, &appearance).unwrap();
        let bounds = |vertices: &[[f32; 3]]| {
            vertices.iter().fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
            )
        };
        let overlaps = |a: (Vec3, Vec3), b: (Vec3, Vec3)| {
            (a.1.min(b.1) - a.0.max(b.0))
                .cmpgt(Vec3::splat(0.001))
                .all()
        };
        assert_eq!(signs.len(), 10);
        for (id, entry, normal) in [
            ("V-01", "station_entry", Vec3::X),
            ("V-35", "game_entry", Vec3::NEG_Z),
            ("V-36", "models_entry", Vec3::NEG_Z),
            ("V-39", "st_39_door", Vec3::NEG_Z),
            ("V-79", "live_entry", Vec3::NEG_Z),
        ] {
            let building = map.buildings.iter().find(|b| b.id == id).unwrap();
            let source = format!("buildings[{id}]/derived-facade/name-sign");
            let image = signs
                .iter()
                .find(|p| p.source == source && p.material.starts_with("sign_"))
                .unwrap();
            let frame = signs
                .iter()
                .find(|p| p.source == source && p.material == "trim")
                .unwrap();
            let positions = image
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let normals = image
                .mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap();
            assert!(
                normals
                    .iter()
                    .all(|n| Vec3::from(*n).distance(normal) < 0.001)
            );
            let [a, b, c, _]: [Vec3; 4] = positions
                .iter()
                .copied()
                .map(Vec3::from)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            assert!(
                (b - a).cross(c - a).normalize().distance(normal) < 0.001,
                "{id}: winding"
            );
            let material = &appearance.materials[&image.material];
            assert_eq!(material.tile_meters, [1., 1.]);
            let path = root
                .join("game/assets")
                .join(material.color_texture.as_ref().unwrap());
            let png = std::fs::read(path).unwrap();
            assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
            let w = u32::from_be_bytes(png[16..20].try_into().unwrap()) as f32;
            let h = u32::from_be_bytes(png[20..24].try_into().unwrap()) as f32;
            assert!(
                ((b - a).length() / (c - b).length() - w / h).abs() < 0.001,
                "{id}: stretched source graphic"
            );
            let door = map_to_world(map.nodes[entry]);
            let (lo, hi) = bounds(positions);
            let center = (lo + hi) / 2.0;
            assert!((center - door - normal * 0.365).xz().length() < 0.001);
            let frame_bounds = bounds(
                frame
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap(),
            );
            assert!(
                frame_bounds.0.y > door.y + 2.8
                    && frame_bounds.1.y < (building.elevation + building.height) as f32
            );
            for part in facades.iter().filter(|p| {
                p.source == format!("buildings[{id}]/derived-facade")
                    && matches!(p.material.as_str(), "glass" | "awning")
            }) {
                for cube in part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()
                    .as_chunks::<24>()
                    .0
                    .iter()
                {
                    assert!(
                        !overlaps(frame_bounds, bounds(cube)),
                        "{id}: sign blocks source window or canopy"
                    );
                }
            }
        }
        assert_eq!(
            super::super::collision::CollisionWorld::from_parts(&signs)
                .unwrap()
                .triangle_count(),
            70
        );
        let mut detached = map.clone();
        detached.nodes.get_mut("station_entry").unwrap()[0] += 1.0;
        assert!(
            business_signs(&detached)
                .err()
                .unwrap()
                .contains("no source wall wide enough")
        );
    }

    #[test]
    fn star_screens_have_correct_public_facing_and_roof_supports() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let parts = star_screens(&map).unwrap();
        assert_eq!(
            parts
                .iter()
                .filter(|part| part.material == "poster_anke")
                .count(),
            2
        );
        for id in ["V-01", "V-79"] {
            let building = map
                .buildings
                .iter()
                .find(|building| building.id == id)
                .unwrap();
            let roof = (building.elevation + building.height) as f32;
            let source = format!("buildings[{id}]/star-screen");
            let mut supported = false;
            for part in parts.iter().filter(|part| part.source == source) {
                let vertices = part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                for &position in vertices {
                    assert!(
                        position[1] >= roof - 0.001,
                        "{id}: below the roof / obscures facade"
                    );
                    assert!(
                        contains([position[0] as f64, -position[2] as f64], &building.polygon),
                        "{id}: outside roof footprint"
                    );
                    supported |= (position[1] - roof).abs() < 0.001;
                }
                if part.material == "poster_anke" {
                    assert_eq!(vertices.len(), 4);
                    let a = Vec3::from(vertices[0]);
                    let b = Vec3::from(vertices[1]);
                    let c = Vec3::from(vertices[2]);
                    assert!(
                        (b - a).cross(c - a).normalize().dot(Vec3::NEG_Z) > 0.999,
                        "{id}: poster back faces public north court"
                    );
                    assert!(((b - a).length() / (c - b).length() - 16. / 9.).abs() < 0.001);
                }
            }
            assert!(supported, "{id}: screen must be anchored to source roof");
        }
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let poster = &appearance.materials["poster_anke"];
        assert!(poster.unlit);
        assert!(
            root.join("game/assets")
                .join(poster.color_texture.as_ref().unwrap())
                .is_file()
        );
    }

    #[test]
    fn shop_canopies_keep_slope_depth_headroom_and_metre_uvs() {
        let mut empty = BTreeMap::new();
        add_shop_canopy(&mut empty, Vec3::ZERO, 20., 0., Vec3::X).unwrap();
        assert!(
            empty.is_empty(),
            "zero canopy depth must not generate fixtures"
        );
        for outward in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
            for depth in [0.0001, 0.01, 0.08, 0.2, 0.9, 1.5, 3.0] {
                let mut batches = BTreeMap::new();
                add_shop_canopy(&mut batches, Vec3::ZERO, 20., depth, outward).unwrap();
                let parts: Vec<_> = batches
                    .into_iter()
                    .map(|(material, mesh)| GeometryPart {
                        source: "buildings[test]/derived-facade".into(),
                        material,
                        mesh,
                    })
                    .collect();
                let mut sloped_face = false;
                for part in &parts {
                    let positions = part
                        .mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap();
                    let normals = part
                        .mesh
                        .attribute(Mesh::ATTRIBUTE_NORMAL)
                        .unwrap()
                        .as_float3()
                        .unwrap();
                    let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                        part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
                    else {
                        panic!("UVs unavailable")
                    };
                    assert!(normals.iter().flatten().all(|value| value.is_finite()));
                    assert!(
                        uvs.iter()
                            .flatten()
                            .all(|value| value.is_finite() && *value >= 0.)
                    );
                    for position in positions {
                        let p = Vec3::from(*position);
                        let tolerance = (depth * 0.00001).max(0.0000001);
                        assert!(
                            (-tolerance..=depth + tolerance).contains(&p.dot(outward)),
                            "outside source canopy depth {depth}: {p:?}"
                        );
                        assert!(p.y >= 2.5 && p.y <= 3.2, "canopy must keep headroom: {p:?}");
                        assert!(p.dot(Vec3::Y.cross(outward)).abs() <= 10.001);
                    }
                    if part.material == "awning" {
                        sloped_face |= normals
                            .iter()
                            .map(|n| Vec3::from(*n))
                            .any(|n| n.y > 0.9 && n.dot(outward) > 0.1);
                    }
                    for (face, uv) in positions
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(uvs.as_chunks::<4>().0.iter())
                    {
                        for i in 0..4 {
                            let j = (i + 1) % 4;
                            let metres = Vec3::from(face[i]).distance(Vec3::from(face[j]));
                            let uv_metres = Vec2::from(uv[i]).distance(Vec2::from(uv[j]));
                            assert!(
                                (metres - uv_metres).abs() < 0.0001,
                                "UV edge {uv_metres} differs from geometry edge {metres}"
                            );
                        }
                    }
                }
                assert!(sloped_face, "canopy roof must fall toward its street edge");
                let collision =
                    super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
                assert!(
                    collision
                        .capsule_cast(outward * 3., 1.7, 0.3, outward * -3., 0.02)
                        .is_none(),
                    "fixtures block the route under the canopy"
                );
            }
        }
    }

    fn lifecycle_scene() -> PreparedScene {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let mut appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let (material, spec) = appearance.materials.iter_mut().next().unwrap();
        spec.color_texture = None;
        spec.normal_texture = None;
        let material = material.clone();
        PreparedScene {
            map,
            appearance,
            parts: vec![GeometryPart {
                source: "lifecycle-test".into(),
                material,
                mesh: Cuboid::default().into(),
            }],
            props: vec![],
            asset_root: root.join("game/assets"),
            warnings: vec![],
            model_contexts: BTreeMap::new(),
        }
    }

    fn lifecycle_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), WorldScenePlugin))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>();
        app.finish();
        app.cleanup();
        app
    }

    #[test]
    fn lifecycle_waits_for_a_scene_and_starts_each_scene_once() {
        let mut idle = App::new();
        idle.add_plugins(WorldScenePlugin);
        idle.update();

        let mut app = lifecycle_app();
        app.update();
        app.update();
        assert!(!app.world().contains_resource::<SceneLoading>());

        for _ in 0..2 {
            app.insert_resource(SceneLoading::new(lifecycle_scene()));
            app.update();
            let loading = app.world().resource::<SceneLoading>();
            assert!(loading.ready && loading.failure.is_none());
            assert!(loading.prepared.is_none());
            let material = loading.materials.values().next().unwrap().id();
            for _ in 0..3 {
                app.update();
                assert_eq!(
                    app.world()
                        .resource::<SceneLoading>()
                        .materials
                        .values()
                        .next()
                        .unwrap()
                        .id(),
                    material
                );
                assert_eq!(
                    app.world_mut()
                        .query::<&MapSource>()
                        .iter(app.world())
                        .count(),
                    1
                );
            }
            clear_scene(app.world_mut());
            app.update();
            assert!(!app.world().contains_resource::<SceneLoading>());
            assert_eq!(
                app.world_mut()
                    .query::<&MapSource>()
                    .iter(app.world())
                    .count(),
                0
            );
        }
    }

    #[test]
    fn lifecycle_clear_removes_map_roots_and_descendants_only() {
        let mut world = World::new();
        let root = world.spawn(MapSource("map-root".into())).id();
        let child = world.spawn((MapSource("model".into()), ChildOf(root))).id();
        let grandchild = world.spawn(ChildOf(child)).id();
        let other = world.spawn(Name::new("host-owned")).id();
        let other_child = world
            .spawn((MapSource("host-owned-child".into()), ChildOf(other)))
            .id();
        world.insert_resource(SceneLoading::new(lifecycle_scene()));

        for _ in 0..2 {
            clear_scene(&mut world);
            assert!(!world.contains_resource::<SceneLoading>());
            for entity in [root, child, grandchild] {
                assert!(world.get_entity(entity).is_err());
            }
            for entity in [other, other_child] {
                assert!(world.get_entity(entity).is_ok());
            }
        }
    }

    #[test]
    fn lifecycle_failure_is_reported_without_exiting_the_host() {
        let mut app = lifecycle_app();
        let mut loading = SceneLoading::new(lifecycle_scene());
        loading.started = Some(Instant::now() - std::time::Duration::from_secs(121));
        app.insert_resource(loading);
        app.update();
        assert!(
            app.world()
                .resource::<SceneLoading>()
                .failure
                .as_ref()
                .is_some_and(|error| error.contains("[asset/timeout]"))
        );
        assert!(!app.world().resource::<SceneLoading>().ready);
        assert!(app.should_exit().is_none());
    }

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
    fn garden_edge_groups_keep_tree_shrub_ground_layers_and_clear_arrivals() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let placements = props(&map, &appearance).unwrap();
        for (group, id) in [
            "fw-f-plateau-east-court",
            "fw-e-foothill-east-garden",
            "fw-e-foothill-north-edge",
            "fw-e-foothill-north-edge",
        ]
        .into_iter()
        .enumerate()
        {
            let index = map
                .surfaces
                .iter()
                .position(|s| s.id.as_deref() == Some(id))
                .unwrap();
            let prefix = format!("surfaces[{index}]/derived-vegetation[edge-group:{group}:");
            let models: BTreeSet<_> = placements
                .iter()
                .filter(|p| p.source.starts_with(&prefix))
                .map(|p| p.model.as_str())
                .collect();
            assert_eq!(
                models,
                BTreeSet::from(["tree_pine", "shrub", "grass", "rock"]),
                "{id}: edge group lost a canopy, understory or ground layer"
            );
        }
        // The two approached gardens keep a full 2 m diameter stopping space at their arrivals
        for (id, point) in [
            ("plateau arrival", map.nodes["fw_f_plateau_e_stay0"]),
            ("foothill street", map.nodes["fw_e_v_ef41_resident0_via0"]),
            ("plateau stopping space", [197.0, 498.5, 100.678571]),
            ("foothill stopping space", [465.0, 644.5, 169.925926]),
        ] {
            for p in placements
                .iter()
                .filter(|p| p.source.contains("edge-group:"))
            {
                let at = p.transform.translation;
                assert!(
                    (f64::from(at.x) - point[0]).hypot(-f64::from(at.z) - point[1])
                        >= vegetation_radius(&p.model) + 1.0,
                    "{id}: {} obstructs arrival",
                    p.source
                );
            }
        }
        eprintln!(
            "garden edge group instances={}",
            placements
                .iter()
                .filter(|p| p.source.contains("edge-group:"))
                .count()
        );
    }

    #[test]
    fn shop_shell_keeps_foundations_and_facade_trim_out_of_public_rooms() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let prepared = PreparedScene::load(root).unwrap();
        let building = prepared
            .map
            .buildings
            .iter()
            .find(|building| building.id == "V-04")
            .unwrap();
        let floor = building.shop_floor().unwrap();
        let foundation = prepared
            .parts
            .iter()
            .find(|part| part.source.contains("(V-04)") && part.source.ends_with("/foundation"))
            .unwrap();
        let highest_foundation = foundation
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let mut leaks = Vec::new();
        for part in prepared
            .parts
            .iter()
            .filter(|part| part.source == "buildings[V-04]/derived-facade")
        {
            for point in part
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
            {
                if point[1] > floor.z as f32 + 0.1
                    && point[1] < 32.2
                    && floor.public_rooms().any(|(_, room)| {
                        contains([f64::from(point[0]), -f64::from(point[2])], &room.polygon)
                    })
                {
                    leaks.push((part.material.as_str(), *point));
                }
            }
        }
        eprintln!(
            "shop shell: {} maximum={} floor={}; facade intrusions={} examples={:?}",
            foundation.source,
            highest_foundation,
            floor.z,
            leaks.len(),
            leaks.iter().take(6).collect::<Vec<_>>()
        );
        assert!(
            highest_foundation <= floor.z as f32 + 0.001,
            "foundation cut faces overlap the finished interior walls"
        );
        assert!(
            leaks.is_empty(),
            "facade boxes project behind the shell into the public rooms"
        );
    }

    #[test]
    fn public_building_skins_stay_attached_and_clear_of_windows_and_entries() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        let bounds = |vertices: &[[f32; 3]]| {
            vertices.iter().fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
            )
        };
        let overlaps = |a: (Vec3, Vec3), b: (Vec3, Vec3)| {
            (a.1.min(b.1) - a.0.max(b.0))
                .cmpgt(Vec3::splat(0.001))
                .all()
        };
        let mut covered = BTreeSet::new();
        let mut boxes = 0;
        for building in &map.buildings {
            let source = format!("buildings[{}]/derived-facade/skin", building.id);
            let skins: Vec<_> = parts.iter().filter(|part| part.source == source).collect();
            if skins.is_empty() {
                continue;
            }
            covered.insert(building.id.as_str());
            let design = building.design.as_ref().unwrap();
            let glass: Vec<_> = parts
                .iter()
                .filter(|part| {
                    part.source == format!("buildings[{}]/derived-facade", building.id)
                        && part.material == "glass"
                })
                .flat_map(|part| {
                    part.mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap()
                        .as_chunks::<24>()
                        .0
                        .iter()
                        .map(|cube| bounds(cube))
                })
                .collect();
            for part in skins {
                let vertices = part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                assert!(vertices.iter().flatten().all(|v| v.is_finite()));
                let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                    part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
                else {
                    panic!("missing UVs")
                };
                assert!(uvs.iter().flatten().all(|v| v.is_finite() && *v >= 0.0));
                assert_eq!(vertices.len() % 24, 0);
                for cube in vertices.as_chunks::<24>().0.iter() {
                    boxes += 1;
                    let (lo, hi) = bounds(cube);
                    assert!(
                        (hi - lo).cmpgt(Vec3::ZERO).all(),
                        "{source}: degenerate box"
                    );
                    assert!(lo.y >= building.elevation as f32 - 0.001);
                    assert!(hi.y <= (building.elevation + building.height) as f32 + 0.001);
                    let attached = building
                        .polygon
                        .iter()
                        .zip(building.polygon.iter().cycle().skip(1))
                        .take(building.polygon.len())
                        .any(|(a, b)| {
                            let a = Vec2::new(a[0] as f32, -a[1] as f32);
                            let b = Vec2::new(b[0] as f32, -b[1] as f32);
                            let tangent = (b - a).normalize();
                            let normal = Vec2::new(-tangent.y, tangent.x);
                            cube.iter().all(|p| {
                                let offset = Vec2::new(p[0], p[2]) - a;
                                offset.dot(normal).abs() <= 0.25
                                    && offset.dot(tangent) >= -0.001
                                    && offset.dot(tangent) <= (b - a).length() + 0.001
                            })
                        });
                    assert!(attached, "{source}: panel disconnected from source wall");
                    assert!(
                        glass.iter().all(|window| !overlaps((lo, hi), *window)),
                        "{source}: skin overlaps a window"
                    );
                    for entry in &design.entries {
                        let door = map_to_world(map.nodes[&entry.node]);
                        let opening = entry_opening(&design.kind, &entry.role);
                        let edge = building
                            .polygon
                            .iter()
                            .zip(building.polygon.iter().cycle().skip(1))
                            .take(building.polygon.len())
                            .find(|(a, b)| {
                                let a = Vec2::new(a[0] as f32, -a[1] as f32);
                                let b = Vec2::new(b[0] as f32, -b[1] as f32);
                                ((Vec2::new(door.x, door.z) - a).perp_dot(b - a)).abs() < 0.01
                            })
                            .unwrap();
                        let along_x = (edge.1[0] - edge.0[0]).abs() > 0.01;
                        let half = if along_x {
                            Vec3::new(opening.x / 2.0 + 0.1, 0.0, 0.4)
                        } else {
                            Vec3::new(0.4, 0.0, opening.x / 2.0 + 0.1)
                        };
                        assert!(
                            !overlaps(
                                (lo, hi),
                                (door - half, door + half + Vec3::Y * (opening.y + 0.1))
                            ),
                            "{source}: blocks entry {}",
                            entry.node
                        );
                    }
                }
            }
        }
        assert_eq!(covered, BTreeSet::from(["V-01", "V-32", "V-78", "V-79"]));
        let skins: Vec<_> = parts
            .into_iter()
            .filter(|part| part.source.ends_with("/derived-facade/skin"))
            .collect();
        let collision = super::super::collision::CollisionWorld::from_parts(&skins).unwrap();
        assert_eq!(collision.triangle_count(), boxes * 12);
        eprintln!(
            "public facade skins: {} buildings, {boxes} attached boxes, {} collision triangles",
            covered.len(),
            collision.triangle_count()
        );
    }

    #[test]
    fn residential_glazing_is_recessed_batched_and_keeps_walk_envelope() {
        let mut batches = BTreeMap::new();
        for (index, angle) in [0., 0.73].into_iter().enumerate() {
            let rotation = Quat::from_rotation_y(angle);
            let outward = rotation * Vec3::Z;
            let center = Vec3::new(index as f32 * 5., 4., 0.);
            add_residential_glazing(
                &mut batches,
                center,
                Vec2::new(1.45, 1.67),
                rotation,
                outward,
                index,
            )
            .unwrap();
            let glass = &batches["window_glass"];
            let vertices = glass
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let pair = &vertices[index * 48..(index + 1) * 48];
            for point in pair {
                let local = rotation.inverse() * (Vec3::from(*point) - center);
                assert!(local.z > 0., "pane must remain outside the shell");
                assert!(local.z < 0.04, "pane must remain behind the frame");
                assert!(local.x.abs() >= 0.029, "pane intersects central mullion");
                assert!(local.x.abs() <= 0.726, "pane overlaps side frame");
            }
            let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
                glass.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
            else {
                panic!("missing pane colors")
            };
            assert_eq!(colors.len(), vertices.len(), "merge lost pane colors");
            assert_ne!(colors[index * 48], colors[index * 48 + 24]);
        }
        assert_eq!(batches.len(), 3, "window parts must share material batches");

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let ground = Ground::new(&map).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        let mut covered = BTreeSet::new();
        let mut panes = 0;
        for part in parts.iter().filter(|part| part.material == "window_glass") {
            let building = map
                .buildings
                .iter()
                .find(|b| part.source == format!("buildings[{}]/derived-facade", b.id))
                .unwrap();
            assert!(residential_sample(&building.id));
            assert!(
                covered.insert(building.id.as_str()),
                "duplicate glass batch"
            );
            let vertices = part
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            assert_eq!(vertices.len() % 48, 0, "window must retain both panes");
            assert_eq!(part.mesh.indices().unwrap().len(), vertices.len() / 24 * 36);
            let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
                part.mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
            else {
                panic!("{}: missing colors", part.source)
            };
            assert_eq!(colors.len(), vertices.len());
            assert!(
                colors
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite() && *v > 0. && *v <= 1.)
            );
            for point in vertices {
                assert!(point.iter().all(|v| v.is_finite()));
                let position = [f64::from(point[0]), -f64::from(point[2])];
                assert!(!contains(position, &building.polygon), "pane inside shell");
                assert!(f64::from(point[1]) - ground.height(position) > 2.5);
                assert!(f64::from(point[1]) < building.elevation + building.height);
            }
            panes += vertices.len() / 24;
        }
        assert_eq!(
            covered,
            map.buildings
                .iter()
                .filter(|b| residential_sample(&b.id))
                .map(|b| b.id.as_str())
                .collect()
        );
        eprintln!(
            "residential glazing: {} shared glass batches, {} windows, {} shell triangles (before existing facade accessory removal)",
            covered.len(),
            panes / 2,
            panes / 2 * 7 * 12
        );
    }

    #[test]
    fn residential_details_keep_headroom_entries_shells_and_collision() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let ground = Ground::new(&map).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        let bounds = |vertices: &[[f32; 3]]| {
            vertices.iter().fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
            )
        };
        let overlap = |a: (Vec3, Vec3), b: (Vec3, Vec3)| {
            (a.1.min(b.1) - a.0.max(b.0))
                .cmpgt(Vec3::splat(0.001))
                .all()
        };
        let mut covered = BTreeSet::new();
        let mut boxes = 0;
        for building in &map.buildings {
            let source = format!("buildings[{}]/derived-facade/residential", building.id);
            let rainwater = format!("buildings[{}]/derived-facade/rainwater", building.id);
            let details: Vec<_> = parts
                .iter()
                .filter(|p| p.source == source || p.source == rainwater)
                .collect();
            if details.is_empty() {
                continue;
            }
            assert!(residential_sample(&building.id));
            assert_eq!(details.iter().filter(|p| p.source == rainwater).count(), 1);
            assert!(
                details.len() <= 6,
                "residential details exceed six material batches"
            );
            covered.insert(building.id.as_str());
            let design = building.design.as_ref().unwrap();
            let glass: Vec<_> = parts
                .iter()
                .filter(|p| {
                    p.source == format!("buildings[{}]/derived-facade", building.id)
                        && matches!(p.material.as_str(), "glass" | "window_glass")
                })
                .flat_map(|p| {
                    p.mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap()
                        .as_chunks::<24>()
                        .0
                        .iter()
                        .map(|cube| bounds(cube))
                })
                .collect();
            for part in details {
                let vertices = part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                assert!(vertices.iter().flatten().all(|p| p.is_finite()));
                let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                    part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
                else {
                    panic!("missing UVs")
                };
                assert!(uvs.iter().flatten().all(|v| v.is_finite() && *v >= 0.));
                assert_eq!(vertices.len() % 24, 0);
                for cube in vertices.as_chunks::<24>().0.iter() {
                    boxes += 1;
                    let (lo, hi) = bounds(cube);
                    assert!((hi - lo).cmpgt(Vec3::ZERO).all(), "{source}: zero volume");
                    assert!(
                        hi.y < (building.elevation + building.height) as f32,
                        "{source}: roof intrusion"
                    );
                    for vertex in cube {
                        let p = [f64::from(vertex[0]), -f64::from(vertex[2])];
                        assert!(
                            !contains(p, &building.polygon),
                            "{source}: inside authored shell"
                        );
                        if part.source == rainwater {
                            assert!(
                                f64::from(lo.y) >= ground.height(p),
                                "{rainwater}: buried pipe"
                            );
                        } else {
                            assert!(
                                f64::from(lo.y) - ground.height(p) >= 2.5,
                                "{source}: headroom under 2.5m"
                            );
                        }
                        assert!(
                            !map.buildings.iter().any(|other| other.id != building.id
                                && contains(p, &other.polygon)
                                && hi.y > other.elevation as f32
                                && lo.y < (other.elevation + other.height) as f32),
                            "{source}: neighbour intersection"
                        );
                    }
                    let attached = building
                        .polygon
                        .iter()
                        .zip(building.polygon.iter().cycle().skip(1))
                        .take(building.polygon.len())
                        .any(|(a, b)| {
                            let a = Vec2::new(a[0] as f32, -a[1] as f32);
                            let b = Vec2::new(b[0] as f32, -b[1] as f32);
                            let tangent = (b - a).normalize();
                            let normal = Vec2::new(-tangent.y, tangent.x);
                            cube.iter().all(|p| {
                                let offset = Vec2::new(p[0], p[2]) - a;
                                let depth = if part.source == rainwater { 0.22 } else { 0.61 };
                                offset.dot(normal).abs() < depth
                                    && offset.dot(tangent) >= 0.
                                    && offset.dot(tangent) <= (b - a).length()
                            })
                        });
                    assert!(attached, "{source}: outside 0.60m facade strip");
                    assert!(
                        glass.iter().all(|pane| !overlap((lo, hi), *pane)),
                        "{source}: blocked window"
                    );
                    for entry in &design.entries {
                        let p = map_to_world(map.nodes[&entry.node]);
                        let opening = entry_opening(&design.kind, &entry.role);
                        let radius = opening.x / 2. + 0.2;
                        assert!(
                            !overlap(
                                (lo, hi),
                                (
                                    p - Vec3::new(radius, 0., radius),
                                    p + Vec3::new(radius, opening.y + 0.1, radius)
                                )
                            ),
                            "{source}: blocked entry {}",
                            entry.node
                        );
                    }
                }
            }
        }
        assert_eq!(
            covered,
            BTreeSet::from([
                "V-04", "V-A07", "V-A08", "V-A09", "V-A13", "V-A14", "V-W08", "V-13", "V-A15",
                "V-A16"
            ])
        );
        let residential: Vec<_> = parts
            .into_iter()
            .filter(|p| {
                p.source.ends_with("/derived-facade/residential")
                    || p.source.ends_with("/derived-facade/rainwater")
            })
            .collect();
        let collision = super::super::collision::CollisionWorld::from_parts(&residential).unwrap();
        assert_eq!(collision.triangle_count(), boxes * 12);
        eprintln!(
            "residential facades: {} buildings, {boxes} boxes, {} collision triangles",
            covered.len(),
            collision.triangle_count()
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
                    count,
                    usize::from(!(building.shop_floor().is_some() && entry.role == "public")),
                    "{}:{} expected closed door panel, or clear playable shop entrance",
                    building.id,
                    entry.node
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
                .filter(|p| {
                    p.source == "buildings[V-04]/derived-facade"
                        && matches!(p.material.as_str(), "glass" | "window_glass")
                })
                .map(|p| p.mesh.count_vertices())
                .sum::<usize>()
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
