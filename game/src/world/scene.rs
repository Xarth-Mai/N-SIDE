use super::{
    assets::{self, Appearance, Readiness, TrackedAsset},
    geometry::{self, GeometryPart, Ground},
    map::{Building, Map, map_to_world},
    terrain_material::{self, TerrainMaterial},
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
        let court_retaining = court_retaining_details(&map, &parts)?;
        parts.extend(court_retaining);
        parts.extend(star_screens(&map)?);
        parts.extend(business_signs(&map)?);
        parts.extend(shop_stair_handrails(&map)?);
        parts.extend(music_brick_cladding(&map)?);
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
    terrain_material: Option<Handle<TerrainMaterial>>,
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
            terrain_material: None,
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
        if app.is_plugin_added::<AssetPlugin>() {
            app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
        }
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
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
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
    let natural_terrain = prepared.parts.iter().any(|part| part.source == "/terrain");
    if natural_terrain && !prepared.appearance.materials.contains_key("terrain_rock") {
        loading.failure =
            Some("[appearance/binding] terrain requires terrain_rock material".into());
        loading.prepared = Some(prepared);
        return;
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
        if sources.is_empty() && !(natural_terrain && id == "terrain_rock") {
            continue;
        }
        let sources = if sources.is_empty() {
            "/terrain".into()
        } else {
            sources
        };
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
    if natural_terrain {
        let ground = materials
            .get(&loading.materials["terrain"])
            .expect("ground material added");
        let rock = materials
            .get(&loading.materials["terrain_rock"])
            .expect("rock material added");
        match terrain_material::material(ground, rock) {
            Ok(material) => loading.terrain_material = Some(terrain_materials.add(material)),
            Err(error) => {
                loading.failure = Some(format!("[appearance/terrain] {error}"));
                loading.prepared = Some(prepared);
                return;
            }
        }
        let shader: Handle<bevy::shader::Shader> = server.load(terrain_material::SHADER);
        track(
            &mut loading.tracked,
            shader.untyped(),
            vec![format!("terrain slope shader {}", terrain_material::SHADER)],
            false,
        );
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
            loading.terrain_material.as_ref(),
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
    terrain_material: Option<&Handle<TerrainMaterial>>,
    models: &BTreeMap<String, Handle<WorldAsset>>,
    degraded: &mut BTreeSet<String>,
) -> Result<(usize, usize), String> {
    let count = prepared.parts.len();
    for part in prepared.parts {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(part.mesh);
        let natural_terrain = part.source == "/terrain";
        let mut entity = world.spawn((
            Name::new(part.source.clone()),
            MapSource(part.source),
            Mesh3d(mesh),
        ));
        if natural_terrain {
            entity.insert(MeshMaterial3d(
                terrain_material
                    .ok_or("terrain material unavailable")?
                    .clone(),
            ));
        } else {
            entity.insert(MeshMaterial3d(materials[&part.material].clone()));
        }
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
                prop.transform.with_scale(prop.transform.scale * spec.scale),
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

fn music_brick_cladding(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let building = map
        .buildings
        .iter()
        .find(|b| b.id == "V-W10")
        .ok_or("[geometry/cladding] missing V-W10")?;
    let design = building
        .design
        .as_ref()
        .ok_or("[geometry/cladding] V-W10 lacks floors")?;
    let bottom = building.elevation;
    let top = design
        .floors
        .get(1)
        .ok_or("[geometry/cladding] V-W10 lacks upper floor")?
        .z;
    let west = building
        .polygon
        .iter()
        .map(|p| p[0])
        .fold(f64::INFINITY, f64::min);
    let south = building
        .polygon
        .iter()
        .map(|p| p[1])
        .fold(f64::INFINITY, f64::min);
    let north = building
        .polygon
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let door = map.nodes["fw_w_v-w10_door"];
    if (door[0] - west).abs() > 0.01 || top <= bottom + 2.6 {
        return Err("[geometry/cladding] V-W10 west entry or first floor changed".into());
    }
    let mut batches = BTreeMap::new();
    // Shallow decorative veneer leaves the existing entry and its surround open
    for (a, b, low, high) in [
        (south, door[1] - 1.1, bottom, top),
        (door[1] + 1.1, north, bottom, top),
        (door[1] - 1.1, door[1] + 1.1, bottom + 2.55, top),
    ] {
        add_sign(
            &mut batches,
            "brick_music",
            map_to_world([west - 0.012, (a + b) / 2., (low + high) / 2.]),
            (b - a) as f32,
            (high - low) as f32,
            [-1., 0.],
        )?;
    }
    let mut mesh = batches.remove("brick_music").unwrap();
    // One continuous metre-space projection keeps mortar courses aligned across the doorway
    let uvs = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|p| [north as f32 + p[2], p[1] - bottom as f32])
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    Ok(vec![GeometryPart {
        source: "buildings[V-W10]/brick-cladding".into(),
        material: "brick_music".into(),
        mesh,
    }])
}

fn add_shop_window_display(
    batches: &mut BTreeMap<String, Mesh>,
    center: Vec3,
    width: f32,
    rotation: Quat,
    outward: Vec3,
) -> Result<(), String> {
    // The existing metal crossbar supports the upper tier; all stock stays behind the frame lip
    let shelf_width = (width - 0.045) / 2.;
    for side in [-1., 1.] {
        add_box(
            batches,
            "wood_siding",
            center + rotation * Vec3::X * side * (shelf_width + 0.045) / 2. - Vec3::Y * 0.62,
            Vec3::new(shelf_width, 0.045, 0.18),
            rotation,
        )?;
    }
    // Four upright books and a low two-book stack give the window a different rhythm from the door racks
    for (x, foot_y, thickness, height, angle, role) in [
        (-0.86, -0.0975, 0.15, 0.62, 0., "awning"),
        (-0.63, -0.0975, 0.16, 0.74, 0., "terracotta"),
        (-0.39, -0.0975, 0.16, 0.54, 0., "awning"),
        (0.91, -0.0975, 0.12, 0.56, 0., "terracotta"),
        (
            -0.62,
            -0.5975,
            0.08,
            0.56,
            std::f32::consts::FRAC_PI_2,
            "terracotta",
        ),
        (
            -0.59,
            -0.5175,
            0.07,
            0.46,
            std::f32::consts::FRAC_PI_2,
            "awning",
        ),
    ] {
        let book_rotation = rotation * Quat::from_rotation_z(angle);
        let book_height = if angle == 0. { height } else { thickness };
        let book_center = center + rotation * Vec3::X * x + Vec3::Y * (foot_y + book_height / 2.)
            - outward * 0.02;
        add_box(
            batches,
            "shop_ceiling",
            book_center,
            Vec3::new(thickness - 0.024, height - 0.02, 0.14),
            book_rotation,
        )?;
        for side in [-1., 1.] {
            add_box(
                batches,
                role,
                book_center + book_rotation * Vec3::X * side * (thickness - 0.012) / 2.,
                Vec3::new(0.012, height, 0.16),
                book_rotation,
            )?;
        }
        add_box(
            batches,
            role,
            book_center + outward * 0.073,
            Vec3::new(thickness, height, 0.014),
            book_rotation,
        )?;
    }
    // Native low-resolution primitives keep a real opening and handle in the shallow window
    let foot = center + rotation * Vec3::X * 0.43 - Vec3::Y * 0.0975;
    let mut piece = |role: &str, mesh: Mesh, offset: Vec3, local_rotation: Quat| {
        let mesh = mesh.transformed_by(
            Transform::from_translation(foot + rotation * offset)
                .with_rotation(rotation * local_rotation),
        );
        if let Some(batch) = batches.get_mut(role) {
            batch
                .merge(&mesh)
                .map_err(|e| format!("shop cup mesh merge: {e}"))?;
        } else {
            batches.insert(role.into(), mesh);
        }
        Ok::<(), String>(())
    };
    piece(
        "shop_ceiling",
        Cylinder::new(0.085, 0.27)
            .mesh()
            .resolution(12)
            .without_caps()
            .build(),
        Vec3::Y * 0.165,
        Quat::IDENTITY,
    )?;
    let mut inside = Cylinder::new(0.060, 0.27)
        .mesh()
        .resolution(12)
        .without_caps()
        .build();
    inside
        .invert_winding()
        .map_err(|e| format!("shop cup inner winding: {e}"))?;
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
        inside.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for normal in normals {
            *normal = (-Vec3::from(*normal)).to_array();
        }
    }
    piece("trim", inside, Vec3::Y * 0.165, Quat::IDENTITY)?;
    piece(
        "shop_ceiling",
        Cylinder::new(0.085, 0.03).mesh().resolution(12).build(),
        Vec3::Y * 0.015,
        Quat::IDENTITY,
    )?;
    piece(
        "shop_ceiling",
        Annulus::new(0.060, 0.085).mesh().resolution(12).build(),
        Vec3::Y * 0.30,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    )?;
    piece(
        "shop_ceiling",
        Torus::new(0.055, 0.09)
            .mesh()
            .major_resolution(8)
            .minor_resolution(4)
            .angle_range(-std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2)
            .build(),
        Vec3::new(0.072, 0.15, 0.),
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
    )?;
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
        "V-04" | "V-A07" | "V-A09" | "V-A13" | "V-A14" | "V-W08" | "V-13" | "V-A15" | "V-A16"
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

fn add_residential_screen(
    batches: &mut BTreeMap<String, Mesh>,
    center: Vec3,
    opening: Vec2,
    rotation: Quat,
    outward: Vec3,
    roller: bool,
) -> Result<(), String> {
    let along = rotation * Vec3::X;
    if roller {
        // One externally mounted fabric shade leaves the other sliding pane clear
        let width = (opening.x - 0.06) / 2. - 0.04;
        let side = -(opening.x + 0.06) / 4.;
        let top = opening.y / 2. - 0.045;
        add_box(
            batches,
            "awning",
            center + along * side + Vec3::Y * (top - 0.5) + outward * 0.12,
            Vec3::new(width, 1., 0.025),
            rotation,
        )?;
        for height in [top + 0.015, top - 1.] {
            add_box(
                batches,
                "metal",
                center + along * side + Vec3::Y * height + outward * 0.12,
                Vec3::new(width + 0.04, 0.05, 0.07),
                rotation,
            )?;
        }
    } else {
        // Fixed upper louvers limit afternoon exposure without closing the window
        for level in 0..4 {
            add_box(
                batches,
                "wood_siding",
                center + Vec3::Y * (opening.y / 2. - 0.10 - level as f32 * 0.18) + outward * 0.20,
                Vec3::new(opening.x + 0.08, 0.035, 0.19),
                rotation * Quat::from_rotation_x(25_f32.to_radians()),
            )?;
        }
        for side in [-1., 1.] {
            add_box(
                batches,
                "metal",
                center
                    + along * side * (opening.x + 0.02) / 2.
                    + Vec3::Y * (opening.y / 2. - 0.37)
                    + outward * 0.10,
                Vec3::new(0.04, 0.68, 0.05),
                rotation,
            )?;
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
        // This building's four exterior faces are authored in Blender
        if building.id == "V-A08" {
            continue;
        }
        let mut batches = BTreeMap::new();
        let mut skin = BTreeMap::new();
        let mut residential = BTreeMap::new();
        let mut rainwater = BTreeMap::new();
        let mut street_display = BTreeMap::new();
        let mut window_display = BTreeMap::new();
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
                let cinema_blender_face = building.id == "V-15"
                    && !court
                    && ((a[1] == 220. && b[1] == 220.) || (a[0] == 300. && b[0] == 300.));
                let workshop_blender_face = building.id == "V-55"
                    && !court
                    && ((a[0] == 148. && b[0] == 148.) || (a[1] == 198. && b[1] == 198.));
                let arcade_blender_face =
                    building.id == "V-35" && !court && a[1] == 153. && b[1] == 153.;
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
                        // Authored attachments own the windows on their covered faces
                        if cinema_blender_face || workshop_blender_face || arcade_blender_face {
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
                            if building.id == "V-04" {
                                add_shop_window_display(
                                    &mut window_display,
                                    center,
                                    width,
                                    rotation,
                                    map_to_world([normal[0], normal[1], 0.]),
                                )?;
                            }
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
                            // Street-edge rooms get a partial privacy shade on the first home level
                            let privacy = floor_index == 1 && (i == 0 || i + 1 == n);
                            // Map +Y is north: west/south street faces receive fixed sun protection
                            if privacy || normal[0] < -0.5 || normal[1] < -0.5 {
                                add_residential_screen(
                                    &mut residential,
                                    map_to_world(p),
                                    Vec2::new(width, height),
                                    rotation,
                                    map_to_world([normal[0], normal[1], 0.]),
                                    privacy,
                                )?;
                            }
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
                    if floor_index > 0
                        && !cinema_blender_face
                        && !workshop_blender_face
                        && !arcade_blender_face
                    {
                        let p = [
                            (a[0] + b[0]) / 2.0 + normal[0] * (0.04 + interior_clearance),
                            (a[1] + b[1]) / 2.0 + normal[1] * (0.04 + interior_clearance),
                            floor.z,
                        ];
                        let band_width = length - interior_clearance * 2.;
                        add_box(
                            &mut batches,
                            "trim",
                            map_to_world(p),
                            Vec3::new(band_width as f32, 0.13, 0.18),
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
                    // Preserve entry/terrain validation above; the authored closed doors own their frames
                    if workshop_blender_face || arcade_blender_face {
                        continue;
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
                        && !(building.id == "V-15" && entry.node == "cinema_entry")
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
                if !workshop_blender_face
                    && !arcade_blender_face
                    && !design.floors.iter().any(|f| f.name == "RF")
                {
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
            // V-35's authored north facade includes the full source-depth canopy
            if let Some(depth) = design.canopy.filter(|_| building.id != "V-35") {
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
            // Display stock is visual-only; the existing facade continues to own collision
            ("window-display", window_display),
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

fn court_retaining_details(
    map: &Map,
    generated: &[GeometryPart],
) -> Result<Vec<GeometryPart>, String> {
    let (index, surface) = map
        .surfaces
        .iter()
        .enumerate()
        .find(|(_, surface)| surface.id.as_deref() == Some("shop-tree-court"))
        .ok_or("[geometry/court-retaining] missing shop-tree-court")?;
    if surface.polygon.len() != 5 || surface.elevated {
        return Err("[geometry/court-retaining] authored court boundary changed".into());
    }
    let [left, right] = [surface.polygon[4], surface.polygon[3]];
    if (left[1] - right[1]).abs() > 0.001 || right[0] <= left[0] {
        return Err("[geometry/court-retaining] authored north edge changed".into());
    }
    let source = format!("/surfaces/{index} (shop-tree-court)");
    let wall = generated
        .iter()
        .find(|part| part.source == format!("{source}/structure"))
        .ok_or("[geometry/court-retaining] missing generated cut wall")?;
    // Follow the actual rendered cut crest, including native triangulation breakpoints
    let mut crest: Vec<_> = wall
        .mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|values| values.as_float3())
        .ok_or("[geometry/court-retaining] cut wall has no positions")?
        .iter()
        .filter(|p| {
            (f64::from(p[2]) + left[1]).abs() < 0.001
                && f64::from(p[1]) > surface.elevation + 0.25
                && f64::from(p[0]) >= left[0] - 0.001
                && f64::from(p[0]) <= right[0] + 0.001
        })
        .map(|p| [p[0], p[1]])
        .collect();
    crest.sort_by(|a, b| a[0].total_cmp(&b[0]));
    crest.dedup_by(|a, b| (a[0] - b[0]).abs() < 0.001);
    if crest.len() < 2
        || (f64::from(crest[0][0]) - left[0]).abs() > 0.001
        || (f64::from(crest.last().unwrap()[0]) - right[0]).abs() > 0.001
    {
        return Err("[geometry/court-retaining] north crest is incomplete".into());
    }
    let mut batches = BTreeMap::new();
    for edge in crest.windows(2) {
        let dx = edge[1][0] - edge[0][0];
        let dz = edge[1][1] - edge[0][1];
        let length = dx.hypot(dz);
        add_box(
            &mut batches,
            "concrete",
            Vec3::new(
                (edge[0][0] + edge[1][0]) / 2.,
                (edge[0][1] + edge[1][1]) / 2. - 0.02,
                -(left[1] + 0.06) as f32,
            ),
            Vec3::new(length - 0.005_f32.min(length * 0.1), 0.16, 0.46),
            Quat::from_rotation_z(dz.atan2(dx)),
        )?;
    }
    let height = |x: f32| {
        let edge = crest
            .windows(2)
            .find(|p| x >= p[0][0] && x <= p[1][0])
            .unwrap();
        edge[0][1] + (edge[1][1] - edge[0][1]) * (x - edge[0][0]) / (edge[1][0] - edge[0][0])
    };
    // Narrow construction joints divide the existing concrete face, without another wall skin
    let bays = ((right[0] - left[0]) / 3.).ceil() as usize;
    for bay in 0..bays {
        let x = (left[0] + (bay as f64 + 0.5) * (right[0] - left[0]) / bays as f64) as f32;
        let top = height(x) - 0.10;
        add_box(
            &mut batches,
            "trim",
            Vec3::new(
                x,
                (surface.elevation as f32 + top) / 2.,
                -(left[1] - 0.02) as f32,
            ),
            Vec3::new(0.035, top - surface.elevation as f32, 0.04),
            Quat::IDENTITY,
        )?;
    }
    for x in [left[0] as f32 + 0.14, right[0] as f32 - 0.14] {
        let top = height(x) - 0.08;
        add_box(
            &mut batches,
            "concrete",
            Vec3::new(
                x,
                (surface.elevation as f32 + top) / 2.,
                -(left[1] - 0.01) as f32,
            ),
            Vec3::new(0.28, top - surface.elevation as f32, 0.26),
            Quat::IDENTITY,
        )?;
    }
    Ok(batches
        .into_iter()
        .map(|(material, mesh)| GeometryPart {
            source: format!("{source}/retaining-north-detail"),
            material,
            mesh,
        })
        .collect())
}

const SHOP_STAIR_FLIGHTS: [[&str; 2]; 3] = [
    [
        "level_home_to_shop_north_junction",
        "level_shop_north_junction_from_home",
    ],
    [
        "level_shop_north_junction_to_shop_upper_junction",
        "level_shop_upper_junction_from_shop_north_junction",
    ],
    [
        "level_shop_upper_junction_to_steps_mid",
        "pause_steps_mid_level_shop_upper_junction_to_steps_mid",
    ],
];

// Visual street furniture only; the existing walking collision remains unchanged
fn shop_stair_handrails(map: &Map) -> Result<Vec<GeometryPart>, String> {
    let ground = Ground::new(map)?;
    let mut parts = Vec::new();
    for [start, end] in SHOP_STAIR_FLIGHTS {
        let road = map
            .roads
            .iter()
            .find(|road| road.kind == "steps" && road.nodes == [start, end])
            .ok_or_else(|| format!("[geometry/handrail] missing stair flight {start} -> {end}"))?;
        let [a, b] = [map.nodes[start], map.nodes[end]];
        let length = (b[0] - a[0]).hypot(b[1] - a[1]);
        if length <= 0.8 {
            return Err(format!(
                "[geometry/handrail] stair flight too short: {start} -> {end}"
            ));
        }
        let along = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
        let normal = [-along[1], along[0]];
        let rotation = Quat::from_rotation_y(along[1].atan2(along[0]) as f32);
        let bays = ((length - 0.8) / 1.8).ceil() as usize;
        let mut batches = BTreeMap::new();
        for side in [-1.0, 1.0] {
            let point = |distance: f64| {
                let t = distance / length;
                [
                    a[0] + along[0] * distance + normal[0] * (road.width / 2.0 + 0.36) * side,
                    a[1] + along[1] * distance + normal[1] * (road.width / 2.0 + 0.36) * side,
                    a[2] + (b[2] - a[2]) * t,
                ]
            };
            // Separate flights leave the existing cross-lanes and rest landing open
            for height in [0.525, 1.025] {
                let [from, to] = [0.4, length - 0.4].map(|distance| {
                    let mut p = point(distance);
                    p[2] += height;
                    map_to_world(p)
                });
                let direction = (to - from).normalize();
                let cross = direction.cross(Vec3::Y).normalize();
                let up = cross.cross(direction);
                add_box(
                    &mut batches,
                    "metal",
                    (from + to) / 2.,
                    Vec3::new(from.distance(to), 0.06, 0.06),
                    Quat::from_mat3(&Mat3::from_cols(direction, up, cross)),
                )?;
            }
            for i in 0..=bays {
                let p = point(0.4 + (length - 0.8) * i as f64 / bays as f64);
                // Embed the complete foot section in real terrain, not the stair centreline
                let bottom = [-0.03, 0.03]
                    .into_iter()
                    .flat_map(|u| {
                        [-0.03, 0.03].map(|v| {
                            ground.height([
                                p[0] + along[0] * u + normal[0] * v,
                                p[1] + along[1] * u + normal[1] * v,
                            ])
                        })
                    })
                    .fold(f64::INFINITY, f64::min)
                    - 0.06;
                let top = p[2] + 1.025;
                if bottom >= top {
                    return Err(format!("[geometry/handrail] buried rail at {p:?}"));
                }
                add_box(
                    &mut batches,
                    "metal",
                    map_to_world([p[0], p[1], (bottom + top) / 2.]),
                    Vec3::new(0.06, (top - bottom) as f32, 0.06),
                    rotation,
                )?;
            }
        }
        for (material, mesh) in batches {
            parts.push(GeometryPart {
                source: format!("nodes[{start}]/derived-handrail[{end}]"),
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

fn vegetation_clearance(prop: &PropPlacement) -> f64 {
    let tilt = (prop.transform.rotation * Vec3::Y).xz().length();
    let height = match prop.model.as_str() {
        "grass" => 0.35,
        "rock" => 0.65,
        _ => 0.0,
    };
    (vegetation_radius(&prop.model) + height * f64::from(tilt))
        * f64::from(prop.transform.scale.max_element())
}

fn vegetation_space_clear(
    map: &Map,
    obstacles: &[geo::Polygon],
    props: &[PropPlacement],
    p: [f64; 2],
    radius: f64,
) -> bool {
    use geo::{Distance, Euclidean, Point};
    let point = Point::new(p[0], p[1]);
    !obstacles
        .iter()
        .any(|area| Euclidean.distance(&point, area) < radius + 0.4)
        && !map
            .buildings
            .iter()
            .filter_map(|b| b.design.as_ref())
            .flat_map(|d| &d.entries)
            .any(|e| {
                let door = map.nodes[&e.node];
                (door[0] - p[0]).hypot(door[1] - p[1]) < radius + 2.0
            })
        && !props.iter().any(|prop| {
            let q = prop.transform.translation;
            (f64::from(q.x) - p[0]).hypot(-f64::from(q.z) - p[1])
                < radius + vegetation_clearance(prop) + 0.25
        })
}

fn summit_views_clear(map: &Map, p: [f64; 2], radius: f64, top: f64) -> bool {
    let summit = map.nodes["summit"];
    !["home", "station", "cinema_entry"].into_iter().any(|id| {
        let target = map.nodes[id];
        let d = [target[0] - summit[0], target[1] - summit[1]];
        let t = (((p[0] - summit[0]) * d[0] + (p[1] - summit[1]) * d[1])
            / (d[0] * d[0] + d[1] * d[1]))
            .clamp(0.0, 1.0);
        let line = [summit[0] + d[0] * t, summit[1] + d[1] * t];
        let view_height = summit[2] + 1.7 + (target[2] - summit[2]) * t;
        (p[0] - line[0]).hypot(p[1] - line[1]) < radius + 3.0 && top + 1.0 >= view_height
    })
}

// Three authored B12 middle-distance stands; no global forest scatter
const FOREST_GROUPS: [(&str, [f64; 2], f64, usize); 3] = [
    ("hill_short_rest2", [66.0, 48.0], 24.0, 14),
    ("hill_short_rest3", [-58.0, -90.0], 26.0, 14),
    ("hill_east_curve_08", [20.0, -6.0], 24.0, 10),
];

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
                candidates.push((
                    format!("surfaces[{i}]/derived-vegetation[{j}]"),
                    model,
                    p,
                    None,
                ));
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
                        None,
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
                    None,
                ));
            }
        }
    }
    if let Some(block) = map.blocks.iter().find(|b| b.id == "B12") {
        for (group, (id, offset, extent, _)) in FOREST_GROUPS.iter().enumerate() {
            let Some(anchor) = map.nodes.get(*id) else {
                continue;
            };
            // A bounded spiral provides irregular alternatives; actual spacing is accepted below
            for slot in 0..96 {
                let angle = slot as f64 * 2.399963 + group as f64 * 0.71;
                let radius = extent * ((slot as f64 + 0.5) / 96.0).sqrt();
                let p = [
                    anchor[0] + offset[0] + radius * angle.cos(),
                    anchor[1] + offset[1] + radius * angle.sin(),
                ];
                if contains(p, &block.polygon) {
                    candidates.push((
                        format!("nodes[{id}]/derived-vegetation[forest:{group}:{slot}]"),
                        "tree_pine",
                        p,
                        Some((group, 1.4 + ((slot * 37 + group * 19) % 17) as f64 * 0.025)),
                    ));
                }
            }
        }
    }
    let mut forest_counts = [0; 3];
    let initial = props.len();
    let height = |p: [f64; 2]| {
        map.surfaces
            .iter()
            .find(|s| {
                s.kind == "park" && !s.elevated && s.building.is_none() && contains(p, &s.polygon)
            })
            .map_or_else(|| ground.height(p), |s| s.elevation)
    };
    for (source, model, p, forest) in candidates {
        if props.len() - initial >= 240 {
            break;
        }
        if let Some((group, _)) = forest
            && forest_counts[group] >= FOREST_GROUPS[group].3
        {
            continue;
        }
        let scale = forest.map_or(1.0, |(_, scale)| scale);
        let radius = vegetation_radius(model) * scale;
        if !vegetation_space_clear(map, &obstacles, props, p, radius) {
            continue;
        }
        let (foot, tolerance) = if forest.is_some() {
            (0.2 * scale, 0.55)
        } else if model.starts_with("tree_") {
            (0.2, 0.25)
        } else if model == "rock" {
            (radius, 0.2)
        } else {
            (radius, 0.08)
        };
        let heights: Vec<_> = if forest.is_some() {
            (0..16)
                .map(|i| {
                    let a = i as f64 * std::f64::consts::TAU / 16.0;
                    height([p[0] + foot * a.cos(), p[1] + foot * a.sin()])
                })
                .collect()
        } else {
            [[-foot, -foot], [foot, -foot], [foot, foot], [-foot, foot]]
                .map(|d| height([p[0] + d[0], p[1] + d[1]]))
                .to_vec()
        };
        let low = heights
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
            .min(height(p));
        let high = heights
            .into_iter()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(height(p));
        if high - low > tolerance {
            continue;
        }
        let root = low - if forest.is_some() { 0.03 } else { 0.015 };
        if let Some((group, _)) = forest {
            if !summit_views_clear(map, p, radius, root + 6.5 * scale) {
                continue;
            }
            forest_counts[group] += 1;
        }
        props.push(PropPlacement {
            source,
            model: model.into(),
            transform: Transform::from_translation(map_to_world([p[0], p[1], root]))
                .with_rotation(Quat::from_rotation_y(
                    (props.len() - initial) as f32 * 2.399,
                ))
                .with_scale(Vec3::splat(scale as f32)),
        });
    }
    add_forest_understory(map, ground, &obstacles, initial, props);
    add_urban_bank(map, ground, &obstacles, props);
    add_shop_street_trees(map, ground, &obstacles, props);
}

fn add_shop_street_trees(
    map: &Map,
    ground: &Ground,
    obstacles: &[geo::Polygon],
    props: &mut Vec<PropPlacement>,
) {
    // Two existing grass pockets frame the shop; preserve the ramps and uphill entries
    for (id, offset, scale, yaw) in [
        ("home", [19.0, -17.0], 0.85, 0.35),
        ("v_a08_door_landing", [8.0, 9.0], 0.95, 1.75),
    ] {
        let Some(anchor) = map.nodes.get(id) else {
            continue;
        };
        let p = [anchor[0] + offset[0], anchor[1] + offset[1]];
        let radius = vegetation_radius("tree_a") * scale;
        if !vegetation_space_clear(map, obstacles, props, p, radius) {
            continue;
        }
        let mut low = ground.height(p);
        let mut high = low;
        // Existing GLB has a 0.182m root radius; this ring covers the actual trunk base
        for j in 0..16 {
            let angle = j as f64 * std::f64::consts::TAU / 16.;
            let h = ground.height([
                p[0] + angle.cos() * 0.2 * scale,
                p[1] + angle.sin() * 0.2 * scale,
            ]);
            low = low.min(h);
            high = high.max(h);
        }
        let root = low - 0.02;
        if high - low > 0.25 || !summit_views_clear(map, p, radius, root + 6.0 * scale) {
            continue;
        }
        props.push(PropPlacement {
            source: format!("nodes[{id}]/derived-vegetation[shop-street-tree]"),
            model: "tree_a".into(),
            transform: Transform::from_translation(map_to_world([p[0], p[1], root]))
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(Vec3::splat(scale as f32)),
        });
    }
}

fn add_urban_bank(
    map: &Map,
    ground: &Ground,
    obstacles: &[geo::Polygon],
    props: &mut Vec<PropPlacement>,
) {
    let Some(anchor) = map.nodes.get("v_a08_door_landing") else {
        return;
    };
    // Two low, tended groups above the existing entry lane leave its center and ends open
    for (i, (offset, scale)) in [
        ([2.2, 2.8], 1.05),
        ([4.4, 4.0], 1.20),
        ([6.7, 3.5], 1.05),
        ([11.0, 3.9], 1.15),
        ([13.0, 5.5], 1.00),
    ]
    .into_iter()
    .enumerate()
    {
        let p = [anchor[0] + offset[0], anchor[1] + offset[1]];
        if !vegetation_space_clear(map, obstacles, props, p, vegetation_radius("shrub") * scale) {
            continue;
        }
        let mut low = ground.height(p);
        let mut high = low;
        for j in 0..16 {
            let angle = j as f64 * std::f64::consts::TAU / 16.;
            let h = ground.height([
                p[0] + angle.cos() * 0.1 * scale,
                p[1] + angle.sin() * 0.1 * scale,
            ]);
            low = low.min(h);
            high = high.max(h);
        }
        if high - low > 0.28 {
            continue;
        }
        props.push(PropPlacement {
            source: format!("nodes[v_a08_door_landing]/derived-vegetation[urban-bank:{i}]"),
            model: "shrub".into(),
            transform: Transform::from_translation(map_to_world([p[0], p[1], low - 0.025]))
                .with_rotation(Quat::from_rotation_y(i as f32 * 0.67))
                .with_scale(Vec3::splat(scale as f32)),
        });
    }
}

fn add_forest_understory(
    map: &Map,
    ground: &Ground,
    obstacles: &[geo::Polygon],
    initial: usize,
    props: &mut Vec<PropPlacement>,
) {
    for (group, (id, offset, extent, _)) in FOREST_GROUPS.iter().enumerate() {
        let trees: Vec<_> = props
            .iter()
            .filter(|p| {
                p.source
                    .starts_with(&format!("nodes[{id}]/derived-vegetation[forest:"))
            })
            .collect();
        if trees.is_empty() {
            continue;
        }
        let center = trees
            .iter()
            .fold([0.0; 2], |a, tree| {
                let p = tree.transform.translation;
                [a[0] + f64::from(p.x), a[1] - f64::from(p.z)]
            })
            .map(|v| v / trees.len() as f64);
        let anchor = map.nodes[*id];
        let length = (center[0] - anchor[0]).hypot(center[1] - anchor[1]);
        let toward = [
            (center[0] - anchor[0]) / length,
            (center[1] - anchor[1]) / length,
        ];
        // Two forest-edge lobes leave the arrival-facing six-metre opening unplanted
        for (patch, side) in [-1.0, 1.0].into_iter().enumerate() {
            let lobe = [
                center[0] - toward[1] * extent * 0.65 * side + toward[0] * 2.0,
                center[1] + toward[0] * extent * 0.65 * side + toward[1] * 2.0,
            ];
            let mut counts = [0; 3];
            // Reserve the larger middle layer before ground cover competes for its gaps
            for kind in [0, 2, 1] {
                for slot in 0..80 {
                    if [0, 1, 1, 2, 1][slot % 5] != kind {
                        continue;
                    }
                    if props.len() - initial >= 280 {
                        return;
                    }
                    let limit = [2, 6, if patch == 0 { 2 } else { 1 }][kind];
                    if counts[kind] >= limit {
                        continue;
                    }
                    let model = ["shrub", "grass", "rock"][kind];
                    let phase = ((slot * 7 + group * 3 + patch * 5) % 9) as f64 / 8.0;
                    let scale = [1.4 + phase * 0.4, 1.3 + phase * 0.7, 0.8 + phase * 0.3][kind];
                    let angle = slot as f64 * 2.399963 + group as f64 * 0.6 + patch as f64;
                    let distance = 6.0 * ((slot as f64 + 0.5) / 80.0).sqrt();
                    let p = [
                        lobe[0] + distance * angle.cos(),
                        lobe[1] + distance * angle.sin(),
                    ];
                    let delta = [p[0] - center[0], p[1] - center[1]];
                    if (p[0] - anchor[0] - offset[0]).hypot(p[1] - anchor[1] - offset[1]) > *extent
                        || (delta[0] * toward[0] + delta[1] * toward[1] < 0.0
                            && (delta[0] * toward[1] - delta[1] * toward[0]).abs() < 3.0)
                    {
                        continue;
                    }
                    let dx = (ground.height([p[0] + 0.35, p[1]])
                        - ground.height([p[0] - 0.35, p[1]]))
                        / 0.7;
                    let dy = (ground.height([p[0], p[1] + 0.35])
                        - ground.height([p[0], p[1] - 0.35]))
                        / 0.7;
                    let normal = Vec3::new(-dx as f32, 1.0, dy as f32).normalize();
                    let align = if model == "shrub" {
                        Quat::IDENTITY
                    } else {
                        Quat::from_rotation_arc(Vec3::Y, normal)
                    };
                    let rotation = align * Quat::from_rotation_y(angle as f32);
                    let foot = if model == "shrub" {
                        0.1
                    } else {
                        vegetation_radius(model)
                    } * scale;
                    let mut low = ground.height(p);
                    let mut high = low;
                    for i in 0..16 {
                        let angle = i as f32 * std::f32::consts::TAU / 16.0;
                        let q = rotation
                            * Vec3::new(angle.cos() * foot as f32, 0.0, angle.sin() * foot as f32);
                        let support = ground.height([p[0] + f64::from(q.x), p[1] - f64::from(q.z)])
                            - f64::from(q.y);
                        low = low.min(support);
                        high = high.max(support);
                    }
                    if high - low > if model == "shrub" { 0.28 } else { 0.08 } {
                        continue;
                    }
                    let root = low - 0.025;
                    let prop = PropPlacement {
                        source: format!(
                            "nodes[{id}]/derived-vegetation[understory:{group}:{patch}:{slot}]"
                        ),
                        model: model.into(),
                        transform: Transform::from_translation(map_to_world([p[0], p[1], root]))
                            .with_rotation(rotation)
                            .with_scale(Vec3::splat(scale as f32)),
                    };
                    let radius = vegetation_clearance(&prop);
                    let axis = rotation * Vec3::Y;
                    let top = root
                        + scale
                            * ([0.8, 0.35, 0.65][kind] * f64::from(axis.y)
                                + vegetation_radius(model) * f64::from(axis.xz().length()));
                    if !vegetation_space_clear(map, obstacles, props, p, radius)
                        || !summit_views_clear(map, p, radius, top)
                    {
                        continue;
                    }
                    props.push(prop);
                    counts[kind] += 1;
                }
            }
        }
    }
}

fn props(map: &Map, appearance: &Appearance) -> Result<Vec<PropPlacement>, String> {
    let ground = Ground::new(map)?;
    let mut props = Vec::new();
    for (owner, model, anchor) in [
        ("V-A08", "v_a08_roof_eaves", [79., 277.5, 40.421515]),
        ("V-A08", "v_a08_facade", [79., 277.5, 30.021515]),
        ("V-15", "v15_mirror_hall_facade", [300., 220., 25.]),
        ("V-55", "v55_workshop_facade", [154., 206., 24.]),
        ("V-35", "v35_byte_beat_facade", [-386., 153., 12.]),
    ] {
        props.push(PropPlacement {
            source: format!("buildings[{owner}]/blender-attachment"),
            model: model.into(),
            transform: Transform::from_translation(map_to_world(anchor)),
        });
    }

    props.push(PropPlacement {
        source: "/surfaces/cinema_roof_surface/street-bench".into(),
        model: "street_bench".into(),
        transform: Transform::from_translation(map_to_world([312., 237., 37.])),
    });
    props.push(PropPlacement {
        source: "buildings[V-W10]/aircon-wall".into(),
        model: "aircon_wall".into(),
        transform: Transform::from_translation(map_to_world([567.79, 95.6, 12.666667]))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
    });
    // These small planted edges belong to the authored forecourt, not the general park scatter
    for (index, (surface_id, model, xy, yaw, scale)) in [
        (
            "cinema-arrival-court",
            "street_bench",
            [295.8, 233.5],
            std::f32::consts::FRAC_PI_2,
            1.,
        ),
        (
            "cinema-west-garden-south",
            "shrub",
            [295.25, 227.3],
            0.,
            0.9,
        ),
        (
            "cinema-west-garden-south",
            "shrub",
            [295.25, 230.35],
            0.7,
            1.,
        ),
        (
            "cinema-west-garden-north",
            "shrub",
            [295.25, 236.55],
            1.4,
            0.85,
        ),
        (
            "cinema-west-garden-north",
            "shrub",
            [295.25, 240.15],
            2.1,
            0.95,
        ),
        (
            "cinema-west-garden-south",
            "flowers",
            [295.25, 228.75],
            0.2,
            1.,
        ),
        (
            "cinema-west-garden-north",
            "flowers",
            [295.25, 238.2],
            1.1,
            1.,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let surface = map
            .surfaces
            .iter()
            .find(|s| s.id.as_deref() == Some(surface_id))
            .ok_or_else(|| format!("[scene/forecourt] missing {surface_id}"))?;
        props.push(PropPlacement {
            source: format!("/surfaces/{surface_id}/{model}/{index}"),
            model: model.into(),
            transform: Transform::from_translation(map_to_world([xy[0], xy[1], surface.elevation]))
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(Vec3::splat(scale)),
        });
    }

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
    fn music_brick_veneer_keeps_doorway_uv_scale_and_collision() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let mut parts = music_brick_cladding(&map).unwrap();
        assert_eq!(parts.len(), 1);
        let part = &mut parts[0];
        assert_eq!(part.mesh.indices().unwrap().len(), 18);
        part.mesh.generate_tangents().unwrap();
        let positions = part
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
            part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("UV must be float2")
        };
        for (p, uv) in positions.iter().zip(uvs) {
            assert!((p[0] - 567.988).abs() < 0.001);
            assert!((uv[0] - (100. + p[2])).abs() < 0.0001);
            assert!((uv[1] - (p[1] - 12.666667)).abs() < 0.0001);
            assert!(
                !(p[2] > -90.099 && p[2] < -87.901 && p[1] < 15.2165),
                "veneer intrudes into doorway: {p:?}"
            );
        }
        let mut original = geometry::generate(&map).unwrap();
        let before = super::super::collision::CollisionWorld::from_parts(&original).unwrap();
        original.extend(parts);
        let after = super::super::collision::CollisionWorld::from_parts(&original).unwrap();
        assert_eq!(before.triangle_count(), after.triangle_count());
        assert_eq!(before.source_count(), after.source_count());
    }

    #[test]
    fn blender_building_attachments_fit_existing_shells_and_routes() {
        use super::super::collision::CollisionWorld;
        use bevy::{
            asset::RenderAssetUsages,
            mesh::{Indices, PrimitiveTopology},
        };
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let placements = props(&map, &appearance).unwrap();
        let mut probe_parts = Vec::new();
        let mut arcade_door_projection = 0.0_f32;
        let bounds = |points: &[[f32; 3]]| {
            points.iter().fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
            )
        };
        for (owner, model, budget) in [
            ("V-A08", "v_a08_roof_eaves", 128),
            ("V-A08", "v_a08_facade", 7000),
            ("V-15", "v15_mirror_hall_facade", 16000),
            ("V-55", "v55_workshop_facade", 8000),
            ("V-35", "v35_byte_beat_facade", 8000),
        ] {
            let placed: Vec<_> = placements.iter().filter(|p| p.model == model).collect();
            assert_eq!(placed.len(), 1, "one attachment per authored building");
            let placed = placed[0];
            assert_eq!(
                placed.source,
                format!("buildings[{owner}]/blender-attachment")
            );
            let spec = &appearance.models[model];
            assets::validate_model(&root.join("game/assets"), model, spec).unwrap();
            let glb = gltf::Gltf::open(root.join("game/assets").join(&spec.file)).unwrap();
            let mut points = Vec::new();
            let mut triangles = 0;
            for node in glb.scenes().nth(spec.scene).unwrap().nodes() {
                assert!(
                    node.children().next().is_none(),
                    "these authored attachments have a single mesh root"
                );
                let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
                for primitive in node.mesh().unwrap().primitives() {
                    let reader = primitive.reader(|_| glb.blob.as_deref());
                    let vertices: Vec<_> = reader
                        .read_positions()
                        .unwrap()
                        .map(|p| {
                            placed
                                .transform
                                .transform_point(
                                    matrix.transform_point3(Vec3::from(p)) * spec.scale,
                                )
                                .to_array()
                        })
                        .collect();
                    assert!(vertices.iter().flatten().all(|v| v.is_finite()));
                    points.extend_from_slice(&vertices);
                    let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();
                    triangles += indices.len() / 3;
                    if model == "v35_byte_beat_facade" {
                        for triangle in indices.chunks_exact(3) {
                            let (lo, hi) = bounds(&[
                                vertices[triangle[0] as usize],
                                vertices[triangle[1] as usize],
                                vertices[triangle[2] as usize],
                            ]);
                            // Actual exported geometry intersecting the approaching capsule's central strip
                            if lo.x < -385.7 && hi.x > -386.3 && lo.y < 13.721 && hi.y > 12.021 {
                                arcade_door_projection = arcade_door_projection.max(-153. - lo.z);
                            }
                        }
                    }
                    // Query attachments in isolation to keep approach clearance independent of the base shell
                    probe_parts.push(GeometryPart {
                        source: format!("/buildings/{owner}/probe"),
                        material: model.into(),
                        mesh: Mesh::new(
                            PrimitiveTopology::TriangleList,
                            RenderAssetUsages::default(),
                        )
                        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
                        .with_inserted_indices(Indices::U32(indices)),
                    });
                }
            }
            assert!(triangles > 0 && triangles <= budget);
            let (lo, hi) = bounds(&points);
            let building = map.buildings.iter().find(|b| b.id == owner).unwrap();
            let top = (building.elevation + building.height) as f32;
            if model == "v_a08_roof_eaves" {
                assert!(
                    (lo - Vec3::new(69.55, top - 0.35, -284.45))
                        .abs()
                        .max_element()
                        < 0.001
                );
                assert!((hi - Vec3::new(88.45, top, -270.55)).abs().max_element() < 0.001);
            } else if model == "v_a08_facade" {
                assert!(
                    (lo - Vec3::new(69.343, 30.021515, -284.255))
                        .abs()
                        .max_element()
                        < 0.001
                );
                assert!(
                    (hi - Vec3::new(89.35, 40.081515, -270.343))
                        .abs()
                        .max_element()
                        < 0.001
                );
            } else if model == "v35_byte_beat_facade" {
                assert_eq!(triangles, 3612);
                assert!((lo - Vec3::new(-402., 11.82, -154.5)).abs().max_element() < 0.001);
                assert!((hi - Vec3::new(-370., 23., -153.02)).abs().max_element() < 0.001);
                assert_eq!(
                    building.polygon,
                    vec![[-402., 113.], [-370., 113.], [-370., 153.], [-402., 153.]]
                );
                assert_eq!(map.nodes["game_entry"], [-386., 153., 12.]);
                assert_eq!(map.nodes["game_service_entry"], [-386., 113., 12.]);
                assert!(
                    (arcade_door_projection - 0.18).abs() < 0.001,
                    "actual door projection={arcade_door_projection}"
                );
            } else if model == "v55_workshop_facade" {
                assert!((lo - Vec3::new(147.25, 23.45, -214.)).abs().max_element() < 0.001);
                assert!((hi - Vec3::new(160., 33., -197.75)).abs().max_element() < 0.001);
                assert_eq!(
                    building.polygon,
                    vec![[148., 198.], [160., 198.], [160., 214.], [148., 214.]]
                );
                assert_eq!(map.nodes["fw_f_v_55_public"], [148., 202., 24.]);
                assert_eq!(map.nodes["fw_f_v_55_service"], [148., 210., 24.]);
            } else {
                assert!(
                    (lo - Vec3::new(299.65, building.elevation as f32, -250.08))
                        .abs()
                        .max_element()
                        < 0.001
                );
                assert!(
                    (hi - Vec3::new(343.15, top - 0.17, -216.965))
                        .abs()
                        .max_element()
                        < 0.001
                );
            }
            eprintln!("{owner} actual GLB: {triangles} triangles, bounds={lo:?}..{hi:?}");
        }
        let visual_probe = CollisionWorld::from_parts(&probe_parts).unwrap();
        let base_geometry = geometry::generate(&map).unwrap();
        let ground_probe = CollisionWorld::from_parts(&base_geometry).unwrap();
        for (start, end) in [
            ("shop_north_junction", "v_a08_door_landing"),
            ("v_a08_door_landing", "v_a08_door"),
            ("fw_w_cinema_front", "cinema_entry_landing"),
            ("cinema_roof", "cinema_deck_turn"),
            ("cinema_deck_turn", "cinema_upper"),
            ("fw_f_junction20", "fw_f_v_55_public"),
            ("fw_f_junction21", "fw_f_v_55_service"),
            ("game_front_court", "game_entry"),
        ] {
            assert!(
                map.roads.iter().any(|r| r
                    .nodes
                    .windows(2)
                    .any(|n| (n[0] == start && n[1] == end) || (n[0] == end && n[1] == start))),
                "probe follows an actual source road"
            );
            let a = map_to_world(map.nodes[start]);
            let mut b = map_to_world(map.nodes[end]);
            // Closed exterior doors remain closed; approach to standing distance
            if end == "v_a08_door" {
                b -= (b - a).normalize() * 0.65;
            } else if end.starts_with("fw_f_v_55_") {
                b -= (b - a).normalize() * 0.55;
            } else if end == "game_entry" {
                // 0.18 m actual handle projection + 0.30 m radius + 0.02 m skin leaves 0.05 m
                assert!(0.55 - arcade_door_projection - 0.3 - 0.02 > 0.049);
                b -= (b - a).normalize() * 0.55;
                let support = ground_probe.support(b + Vec3::Y * 0.7, 1.4).unwrap();
                let closed = visual_probe
                    .capsule_cast(
                        support.point + Vec3::Y * 0.021,
                        1.7,
                        0.3,
                        Vec3::Z * 0.3,
                        0.02,
                    )
                    .expect("authored public door remains closed");
                assert!(
                    closed.source.contains("/buildings/V-35/probe"),
                    "{closed:?}"
                );
                assert!(
                    (-153.181..=-153.023).contains(&closed.point.z),
                    "door contact={closed:?}"
                );
            }
            for i in 0..20 {
                let at = a.lerp(b, i as f32 / 20.);
                let next = a.lerp(b, (i + 1) as f32 / 20.);
                let support = ground_probe
                    .support(at + Vec3::Y * 0.7, 1.4)
                    .expect("route retains actual ground support");
                assert!(
                    (support.point.y - at.y).abs() < 0.3,
                    "{start}->{end}: authored road elevation changed"
                );
                assert!(
                    visual_probe
                        .capsule_cast(support.point + Vec3::Y * 0.021, 1.7, 0.3, next - at, 0.02)
                        .is_none(),
                    "{start}->{end}: new visible geometry obstructs the player capsule"
                );
            }
        }
        let current = facades(&map, &appearance).unwrap();
        assert!(
            current
                .iter()
                .all(|p| !p.source.contains("blender-attachment"))
        );
        let triangles_for = |parts: &[GeometryPart], owner: &str| {
            parts
                .iter()
                .filter(|p| p.source.starts_with(&format!("buildings[{owner}]/")))
                .map(|p| p.mesh.indices().unwrap().len() / 3)
                .sum::<usize>()
        };
        // V-15 has no other ID-specific facade rules: an in-memory alias recovers its original generic details
        assert!(
            !residential_sample("V-15")
                && !appearance.displays.contains_key("V-15")
                && !appearance.shopfronts.contains_key("V-15")
        );
        let mut original_map = map.clone();
        original_map
            .buildings
            .iter_mut()
            .find(|b| b.id == "V-15")
            .unwrap()
            .id = "V-15-before".into();
        let original = facades(&original_map, &appearance).unwrap();
        let old_cinema = triangles_for(&original, "V-15-before");
        let new_cinema = triangles_for(&current, "V-15");
        assert!(
            new_cinema < old_cinema,
            "covered generic windows and old public canopy are removed"
        );
        let cubes = |parts: &[GeometryPart], owner: &str| {
            parts
                .iter()
                .filter(|p| p.source.starts_with(&format!("buildings[{owner}]/")))
                .flat_map(|p| {
                    p.mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .unwrap()
                        .as_float3()
                        .unwrap()
                        .chunks_exact(24)
                        .map(|v| (p.material.clone(), v.to_vec()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        let old_boxes = cubes(&original, "V-15-before");
        let new_boxes = cubes(&current, "V-15");
        let mut old_bands = 0;
        let mut old_canopy = 0;
        for (role, points) in old_boxes.iter().filter(|b| !new_boxes.contains(b)) {
            let (lo, hi) = bounds(points);
            let middle = (lo + hi) / 2.;
            let south = (middle.z + 219.96).abs() < 0.001 && (hi.x - lo.x - 40.).abs() < 0.001;
            let west = (middle.x - 299.96).abs() < 0.001 && (hi.z - lo.z - 30.).abs() < 0.001;
            if south || west {
                assert!(role == "trim" && [29., 33.].iter().any(|y| (middle.y - y).abs() < 0.001));
                old_bands += 1;
            } else if role == "awning" {
                assert!(
                    (middle - Vec3::new(306., 27.95, -219.45))
                        .abs()
                        .max_element()
                        < 0.001
                );
                old_canopy += 1;
            } else {
                assert!(
                    (lo.z > -220.3 && hi.z < -219.6 && lo.x >= 300.3 && hi.x <= 340.)
                        || (lo.x > 299.6 && hi.x < 300.3 && lo.z >= -250. && hi.z <= -220.),
                    "uncovered generic detail removed: {role} {lo:?}..{hi:?}"
                );
            }
        }
        assert_eq!((old_bands, old_canopy), (4, 1));
        assert!(new_boxes.iter().all(|b| old_boxes.contains(b)));
        assert!(
            current
                .iter()
                .all(|p| !p.source.starts_with("buildings[V-A08]/"))
        );
        // V-55 has no other ID-specific detail selectors; its untouched two faces must remain byte-for-byte
        assert!(
            !residential_sample("V-55")
                && !appearance.displays.contains_key("V-55")
                && !appearance.shopfronts.contains_key("V-55")
        );
        original_map
            .buildings
            .iter_mut()
            .find(|b| b.id == "V-55")
            .unwrap()
            .id = "V-55-before".into();
        let original_workshop = facades(&original_map, &appearance).unwrap();
        let old_workshop = cubes(&original_workshop, "V-55-before");
        let new_workshop = cubes(&current, "V-55");
        assert!(!new_workshop.is_empty() && new_workshop.len() < old_workshop.len());
        assert!(new_workshop.iter().all(|b| old_workshop.contains(b)));
        let mut preserved_east = 0;
        let mut preserved_north = 0;
        for old in &old_workshop {
            let (lo, hi) = bounds(&old.1);
            let middle = (lo + hi) / 2.;
            if (middle.x - 160.).abs() < 0.3 && hi.z - lo.z > hi.x - lo.x {
                assert!(
                    new_workshop.contains(old),
                    "east generic detail removed: {old:?}"
                );
                preserved_east += 1;
            } else if (middle.z + 214.).abs() < 0.3 && hi.x - lo.x > hi.z - lo.z {
                assert!(
                    new_workshop.contains(old),
                    "north generic detail removed: {old:?}"
                );
                preserved_north += 1;
            } else {
                assert!(
                    !new_workshop.contains(old),
                    "covered west/south generic detail remains: {old:?}"
                );
                assert!(
                    (hi.x < 148.3 && lo.z >= -214.001 && hi.z <= -197.99)
                        || (lo.z > -198.3 && hi.z < -197.7 && lo.x >= 147.99 && hi.x <= 160.01),
                    "unexpected removed workshop detail: {lo:?}..{hi:?}"
                );
            }
        }
        assert!(preserved_east > 0 && preserved_north > 0);
        // Only the arcade north face is authored; the other three faces and service door stay unchanged
        assert!(
            !residential_sample("V-35")
                && !appearance.displays.contains_key("V-35")
                && !appearance.shopfronts.contains_key("V-35")
        );
        original_map
            .buildings
            .iter_mut()
            .find(|b| b.id == "V-35")
            .unwrap()
            .id = "V-35-before".into();
        let original_arcade = facades(&original_map, &appearance).unwrap();
        let old_arcade = cubes(&original_arcade, "V-35-before");
        let new_arcade = cubes(&current, "V-35");
        assert!(!new_arcade.is_empty() && new_arcade.len() < old_arcade.len());
        assert!(new_arcade.iter().all(|b| old_arcade.contains(b)));
        let mut removed_arcade_canopies = 0;
        let mut preserved_arcade_faces = [0; 3];
        for old in &old_arcade {
            let (lo, hi) = bounds(&old.1);
            let middle = (lo + hi) / 2.;
            if middle.z < -152.7 {
                assert!(
                    !new_arcade.contains(old),
                    "covered north detail remains: {old:?}"
                );
                assert!(
                    lo.x >= -402.001 && hi.x <= -369.999 && lo.z >= -154.501 && hi.z <= -152.7,
                    "unexpected removed arcade detail: {lo:?}..{hi:?}"
                );
                if old.0 == "awning" {
                    removed_arcade_canopies += 1;
                }
            } else {
                assert!(
                    new_arcade.contains(old),
                    "uncovered arcade detail removed: {old:?}"
                );
                if (middle.x + 402.).abs() < 0.3 {
                    preserved_arcade_faces[0] += 1;
                } else if (middle.x + 370.).abs() < 0.3 {
                    preserved_arcade_faces[1] += 1;
                } else if (middle.z + 113.).abs() < 0.3 {
                    preserved_arcade_faces[2] += 1;
                }
            }
        }
        assert_eq!(removed_arcade_canopies, 1);
        assert!(preserved_arcade_faces.iter().all(|n| *n > 0));
        eprintln!(
            "V-35 removed {} north boxes including its old canopy; preserves west/east/south {preserved_arcade_faces:?}; actual central door projection={arcade_door_projection:.3} m",
            old_arcade.len() - new_arcade.len()
        );
        eprintln!(
            "V-A08 fully replaced; V-15 removed {} triangles; V-55 removed {} boxes and preserves east/north {preserved_east}/{preserved_north}; eight approaches clear",
            old_cinema - new_cinema,
            old_workshop.len() - new_workshop.len()
        );
    }

    #[test]
    fn shop_stair_handrails_follow_ground_and_keep_routes_and_collision() {
        use geo::{ConvexHull, Intersects};
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let ground = Ground::new(&map).unwrap();
        let mut generated = geometry::generate(&map).unwrap();
        let collision = super::super::collision::CollisionWorld::from_parts(&generated).unwrap();
        let parts = shop_stair_handrails(&map).unwrap();
        assert_eq!(parts.len(), 3);
        let routes: Vec<_> = map
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
        let mut post_count = 0;
        let mut bar_count = 0;
        let mut triangles = 0;
        let mut post_height = [f64::INFINITY, f64::NEG_INFINITY];
        for (part, [start, end]) in parts.iter().zip(SHOP_STAIR_FLIGHTS) {
            assert_eq!(
                part.source,
                format!("nodes[{start}]/derived-handrail[{end}]")
            );
            assert_eq!(part.material, "metal");
            let [a, b] = [map.nodes[start], map.nodes[end]];
            let length = (b[0] - a[0]).hypot(b[1] - a[1]);
            let along = [(b[0] - a[0]) / length, (b[1] - a[1]) / length];
            let normal = [-along[1], along[0]];
            let positions = part
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            assert!(positions.iter().flatten().all(|v| v.is_finite()));
            assert_eq!(positions.len() % 24, 0);
            triangles += part.mesh.indices().unwrap().len() / 3;
            for vertices in positions.as_chunks::<24>().0 {
                let planar: Vec<_> = vertices
                    .iter()
                    .map(|p| geo::Point::new(p[0] as f64, -p[2] as f64))
                    .collect();
                let footprint = geo::MultiPoint::new(planar).convex_hull();
                assert!(
                    routes.iter().all(|road| !road.intersects(&footprint)),
                    "{} crosses a road or 0.32m capsule margin",
                    part.source
                );
                assert!(
                    map.buildings.iter().all(|building| {
                        !geo::Polygon::new(
                            geo::LineString::from(
                                building
                                    .polygon
                                    .iter()
                                    .map(|p| (p[0], p[1]))
                                    .collect::<Vec<_>>(),
                            ),
                            vec![],
                        )
                        .intersects(&footprint)
                    }),
                    "{} intersects a building or entrance",
                    part.source
                );
                let rest = map
                    .surfaces
                    .iter()
                    .find(|s| s.id.as_deref() == Some("stairs-rest"))
                    .unwrap();
                assert!(
                    !geo::Polygon::new(
                        geo::LineString::from(
                            rest.polygon
                                .iter()
                                .map(|p| (p[0], p[1]))
                                .collect::<Vec<_>>()
                        ),
                        vec![]
                    )
                    .intersects(&footprint),
                    "rest landing must stay open"
                );
                let mut distances = [f64::INFINITY, f64::NEG_INFINITY];
                for p in vertices {
                    let distance =
                        (p[0] as f64 - a[0]) * along[0] + (-p[2] as f64 - a[1]) * along[1];
                    distances = [distances[0].min(distance), distances[1].max(distance)];
                    let lateral = ((p[0] as f64 - a[0]) * normal[0]
                        + (-p[2] as f64 - a[1]) * normal[1])
                        .abs();
                    assert!(lateral >= 1.8299 && lateral <= 1.8901);
                }
                assert!(
                    distances[0] > 0.34 && distances[1] < length - 0.34,
                    "flight ends and cross-lane approaches must remain open"
                );
                let center = vertices.iter().map(|p| Vec3::from(*p)).sum::<Vec3>() / 24.;
                let lo = vertices.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min) as f64;
                let hi = vertices
                    .iter()
                    .map(|p| p[1])
                    .fold(f32::NEG_INFINITY, f32::max) as f64;
                if distances[1] - distances[0] < 0.1 {
                    post_count += 1;
                    let minimum_ground = vertices
                        .iter()
                        .map(|p| ground.height([p[0] as f64, -p[2] as f64]))
                        .fold(f64::INFINITY, f64::min);
                    assert!(
                        (minimum_ground - lo - 0.06).abs() < 0.0001,
                        "entire post foot must enter actual terrain"
                    );
                    let distance =
                        (center.x as f64 - a[0]) * along[0] + (-center.z as f64 - a[1]) * along[1];
                    let rail = a[2] + (b[2] - a[2]) * distance / length + 1.025;
                    assert!((hi - rail).abs() < 0.0001, "post must join the upper rail");
                    assert!(
                        (0.5..1.4).contains(&(hi - minimum_ground)),
                        "unreviewed post height"
                    );
                    post_height = [
                        post_height[0].min(hi - minimum_ground),
                        post_height[1].max(hi - minimum_ground),
                    ];
                } else {
                    bar_count += 1;
                    let side = if (center.x as f64 - a[0]) * normal[0]
                        + (-center.z as f64 - a[1]) * normal[1]
                        < 0.
                    {
                        -1.
                    } else {
                        1.
                    };
                    let centre_height = center.y as f64 - (a[2] + b[2]) / 2.;
                    let upper = centre_height > 0.75;
                    for p in vertices {
                        let distance =
                            (p[0] as f64 - a[0]) * along[0] + (-p[2] as f64 - a[1]) * along[1];
                        let offset = p[1] as f64 - a[2] - (b[2] - a[2]) * distance / length;
                        assert!(
                            (offset - centre_height).abs() < 0.04,
                            "actual rail mesh must slope with the stairs, not only its midpoint"
                        );
                    }
                    for i in 0..=12 {
                        let distance = 0.4 + (length - 0.8) * i as f64 / 12.;
                        let centreline = [a[0] + along[0] * distance, a[1] + along[1] * distance];
                        let slope_height = a[2] + (b[2] - a[2]) * distance / length;
                        let support = collision
                            .support(
                                map_to_world([centreline[0], centreline[1], slope_height + 0.5]),
                                1.,
                            )
                            .unwrap();
                        assert!(support.source.starts_with("/roads/"));
                        let rail_height = slope_height + centre_height;
                        let clearance = rail_height - support.point.y as f64;
                        assert!(
                            if upper {
                                (0.88..1.12).contains(&clearance)
                            } else {
                                (0.38..0.62).contains(&clearance)
                            },
                            "handrail height must follow actual generated stair treads: {clearance}"
                        );
                        let foot = [
                            centreline[0] + normal[0] * 1.86 * side,
                            centreline[1] + normal[1] * 1.86 * side,
                        ];
                        assert!(
                            rail_height - ground.height(foot) > 0.10,
                            "rail buried in slope"
                        );
                    }
                }
            }
        }
        assert_eq!((post_count, bar_count, triangles), (48, 12, 720));
        generated.extend(parts);
        let after = super::super::collision::CollisionWorld::from_parts(&generated).unwrap();
        assert_eq!(after.triangle_count(), collision.triangle_count());
        assert_eq!(after.source_count(), collision.source_count());
        assert_eq!(after.bounds(), collision.bounds());
        eprintln!(
            "shop stair rails: 3 metal batches, {post_count} posts, {bar_count} bars, {triangles} triangles; native Ground post height={:.3}..{:.3}m; unchanged collision={} triangles",
            post_height[0],
            post_height[1],
            collision.triangle_count()
        );
        let mut missing = map.clone();
        missing.roads.retain(|r| r.nodes != SHOP_STAIR_FLIGHTS[0]);
        assert!(
            shop_stair_handrails(&missing)
                .err()
                .unwrap()
                .contains("missing stair flight")
        );
    }

    #[test]
    fn court_retaining_details_follow_real_cut_crest_and_preserve_routes() {
        use geo::Intersects;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let ground = Ground::new(&map).unwrap();
        let generated = geometry::generate(&map).unwrap();
        let parts = court_retaining_details(&map, &generated).unwrap();
        assert_eq!(parts.len(), 2);
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
        let mut caps = 0;
        let mut crest_min = f32::INFINITY;
        let mut crest_max = f32::NEG_INFINITY;
        for part in &parts {
            assert_eq!(
                part.source,
                "/surfaces/2 (shop-tree-court)/retaining-north-detail"
            );
            assert!(matches!(part.material.as_str(), "concrete" | "trim"));
            let positions = part
                .mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            assert!(positions.iter().flatten().all(|p| p.is_finite()));
            assert_eq!(positions.len() % 24, 0);
            assert_eq!(
                part.mesh.indices().unwrap().len(),
                positions.len() / 24 * 36
            );
            let bevy::mesh::VertexAttributeValues::Float32x2(uvs) =
                part.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
            else {
                panic!("missing metre UVs")
            };
            assert!(uvs.iter().flatten().all(|v| v.is_finite() && *v >= 0.));
            for cube in positions.as_chunks::<24>().0 {
                boxes += 1;
                let (lo, hi) = cube.iter().fold(
                    (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                    |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
                );
                assert!((hi - lo).cmpgt(Vec3::ZERO).all());
                assert!(lo.y >= 23. - 0.001, "new footing extends below the court");
                assert!(lo.x >= 44.9 && hi.x <= 69.1);
                assert!(
                    lo.z >= -223.30 && hi.z <= -222.82,
                    "detail leaves north wall strip"
                );
                let footprint = geo::Rect::new(
                    geo::Coord {
                        x: f64::from(lo.x),
                        y: -f64::from(hi.z),
                    },
                    geo::Coord {
                        x: f64::from(hi.x),
                        y: -f64::from(lo.z),
                    },
                )
                .to_polygon();
                assert!(
                    roads.iter().all(|r| !r.intersects(&footprint)),
                    "wall detail obstructs road plus capsule clearance"
                );
                assert!(
                    map.buildings
                        .iter()
                        .all(|building| cube.iter().all(|p| !contains(
                            [f64::from(p[0]), -f64::from(p[2])],
                            &building.polygon
                        ))),
                    "wall detail intersects building"
                );
                if ((hi.z - lo.z) - 0.46).abs() < 0.001 {
                    caps += 1;
                    let center = (lo + hi) / 2.;
                    let crest = ground.height([f64::from(center.x), 223.]) as f32;
                    assert!(
                        (center.y + 0.02 - crest).abs() < 0.002,
                        "cap floats above or cuts across the real wall crest"
                    );
                    crest_min = crest_min.min(crest);
                    crest_max = crest_max.max(crest);
                }
            }
        }
        assert!(
            caps > 1 && crest_max - crest_min > 0.2,
            "crest should follow actual sloping ground"
        );
        let collision = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        assert_eq!(collision.triangle_count(), boxes * 12);
        eprintln!(
            "court retaining: crest={crest_min:.3}..{crest_max:.3}m, court=23m, {caps} cap pieces, {boxes} total boxes, {} collision triangles, 2 material batches; road clearance=0.32m",
            collision.triangle_count()
        );
        assert!(
            court_retaining_details(&map, &[])
                .err()
                .unwrap()
                .contains("missing generated cut wall")
        );
    }

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
    fn shop_window_display_has_depth_support_and_no_collision_change() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let mut parts = facades(&map, &appearance).unwrap();
        let bounds = |vertices: &[[f32; 3]]| {
            vertices.iter().fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(Vec3::from(*p)), hi.max(Vec3::from(*p))),
            )
        };
        let positions = |mesh: &Mesh| {
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap()
                .to_vec()
        };
        let wall = geometry::generate(&map)
            .unwrap()
            .into_iter()
            .find(|p| p.source.ends_with("(V-04)"))
            .unwrap();
        let wall_front = bounds(&positions(&wall.mesh)).1.x;
        let pane_part = parts
            .iter()
            .find(|p| p.source == "buildings[V-04]/derived-facade" && p.material == "display_shop")
            .unwrap();
        let panes: Vec<_> = positions(&pane_part.mesh)
            .chunks_exact(4)
            .map(bounds)
            .collect();
        assert!(
            (1..=3).contains(&panes.len()),
            "dress only the existing exposed shop windows"
        );
        let trim: Vec<_> = parts
            .iter()
            .filter(|p| p.source == "buildings[V-04]/derived-facade" && p.material == "trim")
            .flat_map(|p| {
                positions(&p.mesh)
                    .chunks_exact(24)
                    .map(bounds)
                    .collect::<Vec<_>>()
            })
            .collect();
        let metal: Vec<_> = parts
            .iter()
            .filter(|p| p.source == "buildings[V-04]/derived-facade" && p.material == "metal")
            .flat_map(|p| {
                positions(&p.mesh)
                    .chunks_exact(24)
                    .map(bounds)
                    .collect::<Vec<_>>()
            })
            .collect();
        let visual: Vec<_> = parts
            .iter()
            .filter(|p| p.source.ends_with("/window-display"))
            .collect();
        assert_eq!(
            visual.len(),
            5,
            "existing materials form five visual batches"
        );
        let mut triangles = 0;
        let mut cubes = Vec::new();
        for part in &visual {
            assert_eq!(part.source, "buildings[V-04]/window-display");
            assert!(appearance.materials.contains_key(&part.material));
            triangles += part.mesh.indices().unwrap().len() / 3;
            let points = positions(&part.mesh);
            assert!(points.iter().flatten().all(|v| v.is_finite()));
            if matches!(
                part.material.as_str(),
                "wood_siding" | "awning" | "terracotta"
            ) {
                cubes.extend(points.chunks_exact(24).map(|p| {
                    let (lo, hi) = bounds(p);
                    (part.material.clone(), lo, hi)
                }));
            }
            let indices: Vec<_> = part.mesh.indices().unwrap().iter().collect();
            for triangle in indices.chunks_exact(3) {
                let (lo, hi) = bounds(&triangle.iter().map(|&i| points[i]).collect::<Vec<_>>());
                let (pane_lo, pane_hi) = panes
                    .iter()
                    .find(|(a, b)| {
                        lo.z >= a.z - 0.0001 && hi.z <= b.z + 0.0001 && lo.y > a.y && hi.y < b.y
                    })
                    .expect("stock must remain inside an existing pane opening");
                let center = (*pane_lo + *pane_hi) / 2.;
                let frame_front = trim
                    .iter()
                    .filter(|(a, b)| {
                        ((a.y + b.y) / 2. - center.y).abs() < 0.0001
                            && ((a.z + b.z) / 2. - center.z).abs() > 1.1
                            && ((a.z + b.z) / 2. - center.z).abs() < 1.3
                    })
                    .map(|(_, hi)| hi.x)
                    .reduce(f32::min)
                    .unwrap();
                assert!(
                    lo.x > wall_front + 0.08 && lo.x > pane_hi.x + 0.02,
                    "goods must be in front of the real wall and background"
                );
                assert!(
                    hi.x < frame_front - 0.01,
                    "goods must remain behind the existing frame lip"
                );
                assert!(
                    hi.z <= center.z - 0.0225 + 0.0001 || lo.z >= center.z + 0.0225 - 0.0001,
                    "display must leave the central mullion clear"
                );
                if part.material != "wood_siding" {
                    assert!(
                        metal.iter().all(|(a, b)| !(hi.min(*b) - lo.max(*a))
                            .cmpgt(Vec3::splat(0.001))
                            .all()),
                        "merchandise must not intersect the existing crossbar"
                    );
                }
            }
        }
        for (_, lo, hi) in cubes.iter().filter(|(role, _, _)| role == "wood_siding") {
            let supports = trim
                .iter()
                .filter(|(a, b)| {
                    lo.x < b.x
                        && hi.x > a.x
                        && lo.y < b.y
                        && hi.y > a.y
                        && ((lo.z - b.z).abs() < 0.0001 || (hi.z - a.z).abs() < 0.0001)
                })
                .count();
            assert!(
                supports >= 2,
                "each lower shelf must meet the actual side frame and center mullion"
            );
        }
        let shelves: Vec<_> = cubes
            .iter()
            .filter(|(role, _, _)| role == "wood_siding")
            .map(|(_, lo, hi)| (*lo, *hi))
            .chain(metal.iter().copied())
            .chain(
                cubes
                    .iter()
                    .filter(|(role, _, _)| role != "wood_siding")
                    .map(|(_, lo, hi)| (*lo, *hi)),
            )
            .collect();
        let stock: Vec<_> = cubes
            .iter()
            .filter(|(role, lo, hi)| role != "wood_siding" && (hi.x - lo.x - 0.014).abs() < 0.0001)
            .collect();
        assert_eq!(stock.len(), panes.len() * 6);
        assert!(
            stock.iter().any(|(_, lo, hi)| hi.y - lo.y > 0.65),
            "upright books fill the upper window"
        );
        assert_eq!(
            stock.iter().filter(|(_, lo, hi)| hi.y - lo.y < 0.1).count(),
            2,
            "two horizontal books form a low stack"
        );
        for (_, lo, hi) in stock {
            assert!(
                shelves.iter().any(|(a, b)| (lo.y - b.y).abs() < 0.0001
                    && lo.x >= a.x - 0.0001
                    && hi.x <= b.x + 0.0001
                    && lo.z >= a.z - 0.0001
                    && hi.z <= b.z + 0.0001),
                "each book spine must rest on a shelf or the book below"
            );
        }
        // Probe the generated visual mesh through the existing query engine without changing runtime collision
        let probe_parts: Vec<_> = visual
            .iter()
            .map(|part| GeometryPart {
                source: "buildings[V-04]/derived-facade/probe".into(),
                material: part.material.clone(),
                mesh: part.mesh.clone(),
            })
            .collect();
        let probe = super::super::collision::CollisionWorld::from_parts(&probe_parts).unwrap();
        for (pane_lo, pane_hi) in &panes {
            let pane_center = (*pane_lo + *pane_hi) / 2.;
            let cup_points: Vec<_> = visual
                .iter()
                .flat_map(|part| positions(&part.mesh))
                .filter(|p| {
                    p[2] < pane_center.z - 0.2
                        && p[2] > pane_center.z - 0.7
                        && p[1] > pane_center.y - 0.1
                })
                .collect();
            let (lo, hi) = bounds(&cup_points);
            assert!(
                metal.iter().any(|(a, b)| (lo.y - b.y).abs() < 0.0001
                    && lo.x >= a.x
                    && hi.x <= b.x
                    && lo.z >= a.z
                    && hi.z <= b.z),
                "the cup base rests on the original metal shelf"
            );
            let axis = Vec3::new((lo.x + hi.x) / 2., hi.y + 0.1, pane_center.z - 0.43);
            let floor = probe.support(axis, 1.).expect("cup has a solid bottom");
            let rim = probe
                .support(axis + Vec3::X * 0.073, 1.)
                .expect("cup has a rim");
            assert!(
                rim.point.y - floor.point.y > 0.22,
                "cup opening must remain hollow"
            );
            let handle = Vec3::new(hi.x + 0.1, (lo.y + hi.y) / 2., pane_center.z - 0.54);
            assert!(
                probe
                    .sphere_cast(handle, 0.002, -Vec3::X * 0.5, 0.)
                    .is_none(),
                "the handle must have a real see-through gap"
            );
            assert!(
                probe
                    .sphere_cast(handle + Vec3::Y * 0.06, 0.002, -Vec3::X * 0.5, 0.)
                    .is_some(),
                "the handle ring must surround the gap"
            );
        }
        assert!(triangles <= 600, "single-shop display triangle budget");
        let with_display = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        parts.retain(|p| !p.source.ends_with("/window-display"));
        let without_display = super::super::collision::CollisionWorld::from_parts(&parts).unwrap();
        assert_eq!(
            with_display.triangle_count(),
            without_display.triangle_count()
        );
        assert_eq!(with_display.source_count(), without_display.source_count());
        assert_eq!(
            with_display.mesh_memory_bytes(),
            without_display.mesh_memory_bytes()
        );
        assert_eq!(with_display.bounds(), without_display.bounds());
        eprintln!(
            "shop window display: {} panes, {} colored book/shelf boxes plus paper blocks and native cup, {triangles} triangles, 5 visual batches, wall x={wall_front:.3}; collision unchanged: {} triangles, {} sources",
            panes.len(),
            cubes.len(),
            with_display.triangle_count(),
            with_display.source_count()
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
    fn lifecycle_missing_terrain_rock_fails_immediately_and_stays_failed() {
        let mut prepared = lifecycle_scene();
        prepared.parts[0].source = "/terrain".into();
        prepared.parts[0].material = "terrain".into();
        assert!(
            prepared
                .appearance
                .materials
                .remove("terrain_rock")
                .is_some()
        );
        let mut app = lifecycle_app();
        app.insert_resource(SceneLoading::new(prepared));
        app.update();
        let failure = app
            .world()
            .resource::<SceneLoading>()
            .failure
            .clone()
            .expect("missing rock binding fails on the first update");
        assert!(failure.contains("[appearance/binding]") && failure.contains("terrain_rock"));
        assert!(!failure.contains("timeout"));
        for _ in 0..3 {
            app.update();
            let loading = app.world().resource::<SceneLoading>();
            assert_eq!(loading.failure.as_deref(), Some(failure.as_str()));
            assert!(!loading.ready && loading.tracked.is_empty());
            assert!(app.should_exit().is_none());
        }
        assert_eq!(
            app.world_mut()
                .query::<&MapSource>()
                .iter(app.world())
                .count(),
            0
        );
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
        assert!(!additions.is_empty() && additions.len() <= 285);
        assert!(additions.iter().any(|p| p.source.starts_with("surfaces[")));
        assert!(additions.iter().any(|p| p.source.starts_with("roads[")));
        let obstacles = vegetation_obstacles(&map);
        let ground = Ground::new(&map).unwrap();
        for p in &additions {
            let at = p.transform.translation;
            assert!(at.is_finite());
            let point = Point::new(f64::from(at.x), -f64::from(at.z));
            assert!(obstacles.iter().all(|area| Euclidean.distance(&point, area)
                >= vegetation_clearance(p) + 0.399));
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
                f64::from(at.y) <= height + 0.001
                    && height - f64::from(at.y)
                        < if p.source.contains("[forest:") {
                            0.6
                        } else {
                            0.27
                        },
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
                        >= vegetation_clearance(p) + vegetation_clearance(other) + 0.249,
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
    fn shop_street_trees_keep_actual_roots_supported_and_routes_clear() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let placements = props(&map, &appearance).unwrap();
        let trees: Vec<_> = placements
            .iter()
            .filter(|p| p.source.ends_with("/derived-vegetation[shop-street-tree]"))
            .collect();
        assert_eq!(
            trees.len(),
            2,
            "both authored shop trees must survive real Ground, route, prop and summit-view checks"
        );
        let ground = Ground::new(&map).unwrap();
        let obstacles = vegetation_obstacles(&map);
        let spec = &appearance.models["tree_a"];
        assert_eq!(spec.file, "environment/vegetation/street-tree.glb");
        let asset = gltf::Gltf::open(root.join("game/assets").join(&spec.file)).unwrap();
        let mut vertices = Vec::new();
        for node in asset.scenes().nth(spec.scene).unwrap().nodes() {
            assert!(node.children().next().is_none());
            let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
            for primitive in node.mesh().unwrap().primitives() {
                let reader = primitive.reader(|_| asset.blob.as_deref());
                vertices.extend(
                    reader
                        .read_positions()
                        .unwrap()
                        .map(|v| matrix.transform_point3(Vec3::from(v)) * spec.scale),
                );
            }
        }
        assert!(vertices.iter().any(|v| v.y < 0.03));
        for (p, (expected, scale)) in trees
            .iter()
            .zip([([119.0, 238.0], 0.85_f32), ([99.0, 286.5], 0.95_f32)])
        {
            let at = p.transform.translation;
            let xy = [f64::from(at.x), -f64::from(at.z)];
            assert_eq!(xy, expected);
            assert_eq!(p.transform.scale, Vec3::splat(scale));
            assert!(at.is_finite());
            assert!(vegetation_space_clear(
                &map,
                &obstacles,
                &[],
                xy,
                vegetation_clearance(p)
            ));
            let mut burial_range = [f64::INFINITY, f64::NEG_INFINITY];
            let mut top = f64::NEG_INFINITY;
            for v in &vertices {
                let actual = p.transform.transform_point(*v);
                top = top.max(f64::from(actual.y));
                assert!(
                    (actual.x - at.x).hypot(actual.z - at.z) as f64
                        <= vegetation_clearance(p) + 0.001
                );
                if v.y < 0.03 {
                    let burial = ground.height([f64::from(actual.x), -f64::from(actual.z)])
                        - f64::from(actual.y);
                    burial_range[0] = burial_range[0].min(burial);
                    burial_range[1] = burial_range[1].max(burial);
                    assert!(
                        (-0.004..0.27).contains(&burial),
                        "{} actual GLB root unsupported/overburied by {burial:.6}m",
                        p.source
                    );
                }
            }
            assert!(summit_views_clear(&map, xy, vegetation_clearance(p), top));
            for other in placements.iter().filter(|other| other.source != p.source) {
                let q = other.transform.translation;
                assert!(
                    f64::from((at.x - q.x).hypot(at.z - q.z))
                        >= vegetation_clearance(p) + vegetation_clearance(other) + 0.249,
                    "{} overlaps {}",
                    p.source,
                    other.source
                );
            }
            eprintln!(
                "shop-street-tree {} x={:.3} north={:.3} root={:.6} top={top:.6} actual_root_burial={burial_range:?}",
                p.source, at.x, -at.z, at.y
            );
        }
    }

    #[test]
    fn urban_bank_keeps_real_shrub_roots_supported_and_lane_clear() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let placements = props(&map, &appearance).unwrap();
        let bank: Vec<_> = placements
            .iter()
            .filter(|p| p.source.contains("[urban-bank:"))
            .collect();
        assert_eq!(
            bank.len(),
            5,
            "both authored groups must survive real terrain and clearance checks"
        );
        let ground = Ground::new(&map).unwrap();
        let obstacles = vegetation_obstacles(&map);
        let spec = &appearance.models["shrub"];
        let asset = gltf::Gltf::open(root.join("game/assets").join(&spec.file)).unwrap();
        let mut vertices = Vec::new();
        for node in asset.scenes().nth(spec.scene).unwrap().nodes() {
            assert!(node.children().next().is_none());
            let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
            for primitive in node.mesh().unwrap().primitives() {
                let reader = primitive.reader(|_| asset.blob.as_deref());
                vertices.extend(
                    reader
                        .read_positions()
                        .unwrap()
                        .map(|v| matrix.transform_point3(Vec3::from(v)) * spec.scale),
                );
            }
        }
        assert!(vertices.iter().any(|p| p.y < 0.03));
        for p in &bank {
            let at = p.transform.translation;
            assert!(vegetation_space_clear(
                &map,
                &obstacles,
                &[],
                [f64::from(at.x), -f64::from(at.z)],
                vegetation_clearance(p)
            ));
            for v in &vertices {
                let actual = p.transform.transform_point(*v);
                assert!(
                    (actual.x - at.x).hypot(actual.z - at.z) as f64
                        <= vegetation_clearance(p) + 0.001
                );
                if v.y < 0.03 {
                    let burial = ground.height([f64::from(actual.x), -f64::from(actual.z)])
                        - f64::from(actual.y);
                    assert!(
                        (-0.004..0.32).contains(&burial),
                        "{} root unsupported/overburied by {burial:.6}m",
                        p.source
                    );
                }
            }
            eprintln!(
                "urban-bank {} x={:.3} north={:.3} root={:.3} scale={:.3}",
                p.source, at.x, -at.z, at.y, p.transform.scale.x
            );
        }
        let gap = (bank[3].transform.translation - bank[2].transform.translation)
            .xz()
            .length() as f64
            - vegetation_clearance(bank[3])
            - vegetation_clearance(bank[2]);
        assert!(gap > 2., "the two groups must keep a visible two-metre gap");
    }

    #[test]
    fn forest_stands_keep_actual_roots_supported_and_summit_views_clear() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let placements = props(&map, &appearance).unwrap();
        let forest: Vec<_> = placements
            .iter()
            .filter(|p| p.source.contains("[forest:"))
            .collect();
        let mut original_map = map.clone();
        original_map.blocks.retain(|b| b.id != "B12");
        let original = props(&original_map, &appearance).unwrap();
        assert_eq!(
            placements.len(),
            original.len()
                + forest.len()
                + placements
                    .iter()
                    .filter(|p| p.source.contains("[understory:"))
                    .count()
        );
        for before in &original {
            let after = placements
                .iter()
                .find(|p| p.source == before.source && p.model == before.model)
                .unwrap();
            assert!(
                before.source == after.source
                    && before.model == after.model
                    && before.transform == after.transform,
                "forest changed an existing placement"
            );
        }
        let ground = Ground::new(&map).unwrap();
        let spec = &appearance.models["tree_pine"];
        let asset = gltf::Gltf::open(root.join("game/assets").join(&spec.file)).unwrap();
        let mut roots = Vec::new();
        let mut tree_height = 0.0_f32;
        for node in asset.scenes().nth(spec.scene).unwrap().nodes() {
            assert!(node.children().next().is_none());
            let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
            for primitive in node.mesh().unwrap().primitives() {
                let reader = primitive.reader(|_| asset.blob.as_deref());
                for point in reader.read_positions().unwrap() {
                    let p = matrix.transform_point3(Vec3::from(point)) * spec.scale;
                    tree_height = tree_height.max(p.y);
                    if p.y < 0.04 {
                        roots.push(p);
                    }
                }
            }
        }
        assert!(
            roots.len() >= 9 && roots.iter().all(|p| p.x.hypot(p.z) <= 0.2),
            "actual root geometry exceeds the sampled support circle"
        );
        assert!(tree_height <= 6.501);
        let mut counts = [0; 3];
        let mut maximum_root_burial = 0.0_f64;
        for p in forest {
            let group = FOREST_GROUPS
                .iter()
                .position(|(id, _, _, _)| p.source.starts_with(&format!("nodes[{id}]/")))
                .unwrap();
            counts[group] += 1;
            let (id, offset, extent, _) = FOREST_GROUPS[group];
            let at = p.transform.translation;
            let scale = p.transform.scale;
            assert!((1.399..=1.801).contains(&scale.x) && scale.x == scale.y && scale.y == scale.z);
            let anchor = map.nodes[id];
            assert!(
                (f64::from(at.x) - anchor[0] - offset[0])
                    .hypot(-f64::from(at.z) - anchor[1] - offset[1])
                    <= extent + 0.001
            );
            for point in &roots {
                let actual = p.transform.transform_point(*point);
                let support = ground.height([f64::from(actual.x), -f64::from(actual.z)]);
                let buried = support - f64::from(actual.y);
                assert!(
                    (-0.003..0.6).contains(&buried),
                    "{}: actual root vertex floats or is overburied ({buried:.4}m)",
                    p.source
                );
                maximum_root_burial = maximum_root_burial.max(buried);
            }
            let summit = map_to_world(map.nodes["summit"]) + Vec3::Y * 1.7;
            for id in ["home", "station", "cinema_entry"] {
                let target = map_to_world(map.nodes[id]) + Vec3::Y * 1.7;
                let direction = target - summit;
                let t = (((at.x - summit.x) * direction.x + (at.z - summit.z) * direction.z)
                    / (direction.x * direction.x + direction.z * direction.z))
                    .clamp(0.0, 1.0);
                let line = summit.lerp(target, t);
                if (at.x - line.x).hypot(at.z - line.z)
                    < vegetation_radius(&p.model) as f32 * scale.x + 3.0
                {
                    assert!(
                        at.y + tree_height * scale.y + 1.0 < line.y,
                        "{}: obstructs summit to {id} sight corridor",
                        p.source
                    );
                }
            }
            eprintln!(
                "forest placement {} x={:.3} north={:.3} root={:.3} scale={:.3}",
                p.source, at.x, -at.z, at.y, scale.x
            );
        }
        for (group, count) in counts.iter().enumerate() {
            assert!(
                (1..=FOREST_GROUPS[group].3).contains(count),
                "{}: no safe mature trees or exceeded authored budget, got {}",
                FOREST_GROUPS[group].0,
                counts[group]
            );
        }
        eprintln!(
            "forest accepted={counts:?} total={} maximum_actual_root_burial={maximum_root_burial:.6}m",
            counts.iter().sum::<usize>()
        );
    }

    #[test]
    fn understory_keeps_actual_roots_patch_openings_and_scene_clearance() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let placements = props(&map, &appearance).unwrap();
        let ground = Ground::new(&map).unwrap();
        let mut models = BTreeMap::new();
        for model in ["shrub", "grass", "rock"] {
            let spec = &appearance.models[model];
            let asset = gltf::Gltf::open(root.join("game/assets").join(&spec.file)).unwrap();
            let mut points = Vec::new();
            let mut triangles = 0;
            for node in asset.scenes().nth(spec.scene).unwrap().nodes() {
                assert!(node.children().next().is_none());
                let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
                for primitive in node.mesh().unwrap().primitives() {
                    let reader = primitive.reader(|_| asset.blob.as_deref());
                    triangles += reader.read_indices().unwrap().into_u32().count() / 3;
                    points.extend(
                        reader
                            .read_positions()
                            .unwrap()
                            .map(|p| matrix.transform_point3(Vec3::from(p)) * spec.scale),
                    );
                }
            }
            assert!(points.iter().any(|p| p.y < 0.03));
            models.insert(model, (points, triangles));
        }
        let mut counts = [[0; 3]; 3];
        let mut root_burial = [0.0_f64; 3];
        let mut triangle_total = 0;
        for (group, (id, offset, extent, _)) in FOREST_GROUPS.iter().enumerate() {
            let trees: Vec<_> = placements
                .iter()
                .filter(|p| {
                    p.source
                        .starts_with(&format!("nodes[{id}]/derived-vegetation[forest:"))
                })
                .collect();
            let mean =
                trees.iter().map(|p| p.transform.translation).sum::<Vec3>() / trees.len() as f32;
            let anchor = map_to_world(map.nodes[*id]);
            let direction = Vec2::new(mean.x - anchor.x, mean.z - anchor.z).normalize();
            let mut patches = BTreeSet::new();
            for p in placements.iter().filter(|p| {
                p.source
                    .starts_with(&format!("nodes[{id}]/derived-vegetation[understory:"))
            }) {
                let kind = ["shrub", "grass", "rock"]
                    .iter()
                    .position(|m| *m == p.model)
                    .unwrap();
                counts[group][kind] += 1;
                patches.insert(p.source.split(':').nth(2).unwrap());
                let at = p.transform.translation;
                let delta = Vec2::new(at.x - mean.x, at.z - mean.z);
                assert!(
                    delta.dot(direction) >= 0.0 || delta.perp_dot(direction).abs() >= 2.999,
                    "{} closes the arrival-facing opening",
                    p.source
                );
                assert!(
                    (f64::from(at.x) - map.nodes[*id][0] - offset[0])
                        .hypot(-f64::from(at.z) - map.nodes[*id][1] - offset[1])
                        <= extent + 0.001
                );
                let (points, triangles) = &models[p.model.as_str()];
                triangle_total += triangles;
                let mut radius = 0.0_f32;
                let mut top = at.y;
                for vertex in points {
                    let actual = p.transform.transform_point(*vertex);
                    radius = radius.max((actual.x - at.x).hypot(actual.z - at.z));
                    top = top.max(actual.y);
                    // Ground cover has curved leaves and rock sides above its planar basal row
                    if vertex.y < if kind == 0 { 0.03 } else { 0.001 } {
                        let buried = ground.height([f64::from(actual.x), -f64::from(actual.z)])
                            - f64::from(actual.y);
                        assert!(
                            (-0.004..if kind == 0 { 0.32 } else { 0.16 }).contains(&buried),
                            "{} {} basal vertex {vertex:?} unsupported/overburied {buried:.6}m",
                            p.source,
                            p.model
                        );
                        root_burial[kind] = root_burial[kind].max(buried);
                    }
                }
                assert!(
                    f64::from(radius) <= vegetation_clearance(p) + 0.001,
                    "{} tilted exported geometry exceeds its horizontal clearance",
                    p.source
                );
                let summit = map_to_world(map.nodes["summit"]) + Vec3::Y * 1.7;
                for target in ["home", "station", "cinema_entry"] {
                    let end = map_to_world(map.nodes[target]) + Vec3::Y * 1.7;
                    let d = (end - summit).xz();
                    let t = ((at - summit).xz().dot(d) / d.length_squared()).clamp(0.0, 1.0);
                    let line = summit.lerp(end, t);
                    if (at - line).xz().length() < radius + 3.0 {
                        assert!(
                            top + 1.0 < line.y,
                            "{} interrupts summit->{target}",
                            p.source
                        );
                    }
                }
                eprintln!(
                    "understory {} model={} x={:.3} north={:.3} root={:.3} scale={:.3}",
                    p.source, p.model, at.x, -at.z, at.y, p.transform.scale.x
                );
            }
            assert_eq!(
                patches.len(),
                2,
                "{id}: one intended edge lobe has no supported placements"
            );
            for (kind, limit) in [4, 12, 3].into_iter().enumerate() {
                assert!(
                    (1..=limit).contains(&counts[group][kind]),
                    "{id}: missing layer or over budget: {:?}",
                    counts[group]
                );
            }
        }
        // Current family ceiling: 12 shrubs, 36 curved grass clumps, 9 textured rocks
        assert!(triangle_total <= 12 * 5_824 + 36 * 1_320 + 9 * 320);
        eprintln!(
            "understory accepted={counts:?} total={} triangles={triangle_total} max_actual_root_burial_by_shrub_grass_rock={root_burial:?}",
            counts.iter().flatten().sum::<usize>()
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
    fn residential_screens_leave_partial_views_and_share_existing_batches() {
        for (roller, angle) in [(false, 0.0), (true, 0.73)] {
            let rotation = Quat::from_rotation_y(angle);
            let center = Vec3::new(5., 8., -3.);
            let mut batches = BTreeMap::new();
            add_residential_screen(
                &mut batches,
                center,
                Vec2::new(1.45, 1.67),
                rotation,
                rotation * Vec3::Z,
                roller,
            )
            .unwrap();
            assert_eq!(batches.len(), 2);
            for (role, mesh) in &batches {
                assert!(matches!(role.as_str(), "awning" | "wood_siding" | "metal"));
                for point in mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()
                {
                    let local = rotation.inverse() * (Vec3::from(*point) - center);
                    assert!(local.is_finite());
                    assert!(local.x.abs() < 0.83, "screen extends outside window frame");
                    assert!(local.y > -0.25, "screen closes the lower view");
                    assert!(local.y < 0.835, "screen overlaps drip hood");
                    assert!(local.z > 0.07, "screen intersects recessed glass");
                    assert!(local.z < 0.31, "screen exceeds existing facade strip");
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let map = Map::load(root.join("source-assets/district-map/district.json")).unwrap();
        let appearance =
            Appearance::load(&root.join("source-assets/district-scene/appearance.json")).unwrap();
        let parts = facades(&map, &appearance).unwrap();
        let mut rollers = 0;
        let mut slats = 0;
        let mut materials = BTreeMap::<_, BTreeSet<_>>::new();
        for part in parts
            .iter()
            .filter(|part| part.source.ends_with("/derived-facade/residential"))
        {
            assert!(appearance.materials.contains_key(&part.material));
            assert!(
                materials
                    .entry(&part.source)
                    .or_default()
                    .insert(&part.material)
            );
            if part.material == "awning" {
                rollers += part.mesh.count_vertices() / 24;
            } else if part.material == "wood_siding" {
                // Existing wooden skirts are 0.48m tall; only new tilted slats are under 0.2m
                slats += part
                    .mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap()
                    .as_chunks::<24>()
                    .0
                    .iter()
                    .filter(|cube| {
                        let low = cube.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                        let high = cube.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
                        high - low < 0.2
                    })
                    .count();
            }
        }
        assert!(
            rollers > 0 && slats > 0,
            "authored street faces need both treatments"
        );
        assert_eq!(slats % 4, 0);
        assert_eq!(materials.len(), 9);
        assert!(materials.values().all(|roles| roles.len() <= 5));
        let louvers = slats / 4;
        let boxes = rollers * 3 + louvers * 6;
        eprintln!(
            "residential screens: {rollers} partial rollers, {louvers} fixed louver sets; net +{boxes} boxes, +{} triangles; no new material definitions or per-window entities",
            boxes * 12
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
                "V-04", "V-A07", "V-A09", "V-A13", "V-A14", "V-W08", "V-13", "V-A15", "V-A16"
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
        for building in map
            .buildings
            .iter()
            .filter(|b| b.design.is_some() && b.id != "V-A08")
        {
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
                // Replaced closed panels are checked against actual imported collision geometry
                if building.id == "V-55" || (building.id == "V-35" && entry.node == "game_entry") {
                    continue;
                }
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
            map.buildings
                .iter()
                .filter(|b| b.design.is_some() && b.id != "V-A08")
                .count(),
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
