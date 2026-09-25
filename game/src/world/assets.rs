//! Appearance is independent of authoritative map coordinates
use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState, UntypedHandle},
    prelude::*,
};
use serde::{Deserialize, Deserializer, de};
use std::{
    collections::BTreeMap,
    path::{Component as PathComponent, Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub version: u32,
    #[serde(deserialize_with = "unique_bindings")]
    pub materials: BTreeMap<String, MaterialSpec>,
    #[serde(deserialize_with = "unique_bindings")]
    pub models: BTreeMap<String, ModelSpec>,
    #[serde(deserialize_with = "unique_bindings")]
    pub shopfronts: BTreeMap<String, String>,
    #[serde(default, deserialize_with = "unique_bindings")]
    pub displays: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialSpec {
    pub color: [f32; 4],
    pub roughness: f32,
    #[serde(default)]
    pub metallic: f32,
    pub tile_meters: [f32; 2],
    pub color_texture: Option<String>,
    pub normal_texture: Option<String>,
    #[serde(default)]
    pub unlit: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    pub file: String,
    pub scene: usize,
    pub scale: f32,
    #[serde(default)]
    pub optional: bool,
}

impl Appearance {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("[appearance/read] {}: {e}", path.display()))?;
        let mut de = serde_json::Deserializer::from_str(&text);
        let result: Self = serde_path_to_error::deserialize(&mut de)
            .map_err(|e| format!("[appearance/parse] {}:{}: {e}", path.display(), e.path()))?;
        de.end()
            .map_err(|e| format!("[appearance/parse] {}: {e}", path.display()))?;
        if result.version != 1 {
            return Err(format!(
                "[appearance/validate] {}:/version: unsupported {}",
                path.display(),
                result.version
            ));
        }
        for (id, m) in &result.materials {
            let context = format!("[appearance/validate] {}:/materials/{id}", path.display());
            if id.is_empty() {
                return Err(format!("{context}: material ID must not be empty"));
            }
            for (slot, value) in m
                .color
                .iter()
                .enumerate()
                .map(|(i, &v)| (format!("color/{i}"), v))
                .chain([
                    ("roughness".into(), m.roughness),
                    ("metallic".into(), m.metallic),
                ])
            {
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    return Err(format!(
                        "{context}/{slot}: expected finite 0..=1; actual={value}"
                    ));
                }
            }
            for (i, &value) in m.tile_meters.iter().enumerate() {
                if !value.is_finite() || value <= 0.0 || !value.recip().is_finite() {
                    return Err(format!(
                        "{context}/tile_meters/{i}: expected positive finite tiling with finite reciprocal; actual={value}"
                    ));
                }
            }
            for (slot, texture) in [
                ("color_texture", &m.color_texture),
                ("normal_texture", &m.normal_texture),
            ] {
                if let Some(texture) = texture {
                    asset_path(Path::new(""), texture)
                        .map_err(|e| format!("{context}/{slot}: {e}"))?;
                }
            }
        }
        for (id, m) in &result.models {
            if id.is_empty() {
                return Err(format!(
                    "[appearance/validate] {}:/models: model ID must not be empty",
                    path.display()
                ));
            }
            if !m.scale.is_finite() || m.scale <= 0.0 {
                return Err(format!(
                    "[appearance/validate] {}:/models/{id}/scale: {}",
                    path.display(),
                    m.scale
                ));
            }
            asset_path(Path::new(""), &m.file).map_err(|e| {
                format!(
                    "[appearance/validate] {}:/models/{id}/file: {e}",
                    path.display()
                )
            })?;
        }
        for (field, bindings) in [
            ("shopfronts", &result.shopfronts),
            ("displays", &result.displays),
        ] {
            for (building, material) in bindings {
                if !result.materials.contains_key(material) {
                    return Err(format!(
                        "[appearance/binding] {}:/{field}/{building}: material {material:?} does not exist",
                        path.display()
                    ));
                }
            }
        }
        Ok(result)
    }
}

fn unique_bindings<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Bindings<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Bindings<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique named asset bindings")
        }
        fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut result = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if result.insert(key.clone(), value).is_some() {
                    return Err(de::Error::custom(format!(
                        "duplicate asset binding {key:?}"
                    )));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Bindings(std::marker::PhantomData))
}

pub fn asset_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if relative.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, PathComponent::Normal(_)))
        || relative.contains(['#', ':', '\\', '?'])
    {
        return Err(format!("invalid repository asset path {relative:?}"));
    }
    Ok(root.join(path))
}

/// Preflight preserves material slots instead of accepting glTF's implicit white material
pub fn validate_model(root: &Path, id: &str, spec: &ModelSpec) -> Result<(), String> {
    let path = asset_path(root, &spec.file).map_err(|e| {
        format!(
            "[asset/model] asset={id} file={:?} scene={}: {e}",
            spec.file, spec.scene
        )
    })?;
    let context = format!("asset={id} file={} scene={}", path.display(), spec.scene);
    let gltf = gltf::Gltf::open(&path).map_err(|e| format!("[asset/model] {context}: {e}"))?;
    let scene = gltf
        .scenes()
        .nth(spec.scene)
        .ok_or_else(|| format!("[asset/model] {context}: scene index does not exist"))?;
    let mut nodes = scene.nodes().collect::<Vec<_>>();
    let mut visited = std::collections::BTreeSet::new();
    let mut mesh_count = 0;
    while let Some(node) = nodes.pop() {
        if !visited.insert(node.index()) {
            return Err(format!(
                "[asset/model] {context}: scene node={} repeats or forms a cycle",
                node.index()
            ));
        }
        mesh_count += usize::from(node.mesh().is_some());
        nodes.extend(node.children());
    }
    if mesh_count == 0 {
        return Err(format!(
            "[asset/model] {context}: selected scene contains no meshes"
        ));
    }
    for extension in gltf.extensions_used() {
        if !matches!(extension, "KHR_materials_unlit" | "KHR_texture_transform") {
            return Err(format!(
                "[asset/material] {context}: unsupported extension {extension}"
            ));
        }
    }
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            if primitive.material().index().is_none() {
                return Err(format!(
                    "[asset/material] {context} mesh={} primitive={} material slot=<absent>: explicit material slot required",
                    mesh.index(),
                    primitive.index()
                ));
            }
        }
    }
    Ok(())
}

/// Slot context accompanies recursive load failures, including external image dependencies
pub fn model_material_contexts(
    root: &Path,
    id: &str,
    spec: &ModelSpec,
) -> Result<Vec<String>, String> {
    let path = asset_path(root, &spec.file)?;
    let gltf = gltf::Gltf::open(&path).map_err(|e| {
        format!(
            "[asset/model] asset={id} file={} scene={}: {e}",
            path.display(),
            spec.scene
        )
    })?;
    let mut contexts = Vec::new();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let material = primitive.material();
            let prefix = format!(
                "asset={id} file={} scene={} mesh={} primitive={} material={} ({})",
                path.display(),
                spec.scene,
                mesh.index(),
                primitive.index(),
                material
                    .index()
                    .map_or_else(|| "<absent>".into(), |i| i.to_string()),
                material.name().unwrap_or("unnamed")
            );
            let pbr = material.pbr_metallic_roughness();
            let textures = [
                ("base_color", pbr.base_color_texture().map(|t| t.texture())),
                (
                    "metallic_roughness",
                    pbr.metallic_roughness_texture().map(|t| t.texture()),
                ),
                ("normal", material.normal_texture().map(|t| t.texture())),
                (
                    "occlusion",
                    material.occlusion_texture().map(|t| t.texture()),
                ),
                ("emissive", material.emissive_texture().map(|t| t.texture())),
            ];
            contexts.push(prefix.clone());
            for (slot, texture) in textures {
                if let Some(texture) = texture {
                    let image = texture.source();
                    let dependency = match image.source() {
                        gltf::image::Source::Uri { uri, .. } if uri.starts_with("data:") => {
                            "embedded data URI".to_owned()
                        }
                        gltf::image::Source::Uri { uri, .. } => format!(
                            "URI={uri:?} relative to {}",
                            path.parent().unwrap_or(root).display()
                        ),
                        gltf::image::Source::View { view, .. } => {
                            let buffer = view.buffer();
                            let source = match buffer.source() {
                                gltf::buffer::Source::Bin => "embedded GLB buffer".to_owned(),
                                gltf::buffer::Source::Uri(uri) if uri.starts_with("data:") => {
                                    "embedded data URI".to_owned()
                                }
                                gltf::buffer::Source::Uri(uri) => format!(
                                    "URI={uri:?} relative to {}",
                                    path.parent().unwrap_or(root).display()
                                ),
                            };
                            format!(
                                "buffer_view={} -> buffer={} -> {source}",
                                view.index(),
                                buffer.index()
                            )
                        }
                    };
                    contexts.push(format!(
                        "{prefix} slot={slot} -> texture={} -> image={} -> {dependency}",
                        texture.index(),
                        image.index()
                    ));
                }
            }
        }
    }
    Ok(contexts)
}

#[derive(Debug)]
pub struct TrackedAsset {
    pub handle: UntypedHandle,
    pub contexts: Vec<String>,
    pub optional: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Readiness {
    Loading,
    Ready,
    Failed(String),
}

pub fn readiness(server: &AssetServer, asset: &TrackedAsset) -> Readiness {
    let Some((state, _, recursive)) = server.get_load_states(asset.handle.id()) else {
        return Readiness::Loading;
    };
    match (state, recursive) {
        (LoadState::Failed(e), _) | (_, RecursiveDependencyLoadState::Failed(e)) => {
            Readiness::Failed(format!("{e:#}"))
        }
        (LoadState::Loaded, RecursiveDependencyLoadState::Loaded) => Readiness::Ready,
        _ => Readiness::Loading,
    }
}

/// One failure per dependency with every affected source retained
pub fn dependency_failures(
    server: &AssetServer,
    tracked: &[TrackedAsset],
) -> BTreeMap<String, Vec<String>> {
    let mut errors: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for asset in tracked {
        if let Readiness::Failed(error) = readiness(server, asset) {
            errors
                .entry(error)
                .or_default()
                .extend(asset.contexts.clone());
        }
    }
    for contexts in errors.values_mut() {
        contexts.sort();
        contexts.dedup();
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::{AssetLoader, LoadContext, io::Reader},
        image::{CompressedImageFormats, ImageLoader, ImageLoaderError, ImageLoaderSettings},
    };
    use std::{
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        task::{Poll, Waker},
        time::{Duration, Instant},
    };

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "nside-assets-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn appearance_rejects_invalid_paths_values_and_bindings() {
        let dir = TestDir::new();
        let path = dir.0.join("appearance.json");
        let original = serde_json::json!({
            "version":1,
            "materials":{"wall":{"color":[1,1,1,1],"roughness":0.8,"tile_meters":[1,1],"color_texture":"wall.png"}},
            "models":{"tree":{"file":"tree.glb","scene":0,"scale":1}},
            "shopfronts":{"V-04":"wall"},
            "displays":{"V-04":"wall"}
        });
        std::fs::write(&path, original.to_string()).unwrap();
        Appearance::load(&path).unwrap();
        for (pointer, value) in [
            ("/materials/wall/color/0", serde_json::json!(2)),
            ("/materials/wall/roughness", serde_json::json!(-1)),
            ("/materials/wall/tile_meters/1", serde_json::json!(1e-40)),
            (
                "/materials/wall/color_texture",
                serde_json::json!("../outside.png"),
            ),
            ("/models/tree/file", serde_json::json!("")),
            ("/models/tree/scale", serde_json::json!(0)),
            ("/shopfronts/V-04", serde_json::json!("missing_material")),
            ("/displays/V-04", serde_json::json!("missing_material")),
        ] {
            let mut value_json = original.clone();
            *value_json.pointer_mut(pointer).unwrap() = value;
            std::fs::write(&path, value_json.to_string()).unwrap();
            let error = Appearance::load(&path).unwrap_err();
            assert!(
                error.contains(pointer) && error.contains("appearance.json"),
                "{error}"
            );
        }
        for invalid in [
            "",
            "../outside.png",
            "/absolute.png",
            "model.glb#Scene0",
            "remote://x",
            "C:\\model.glb",
        ] {
            assert!(asset_path(&dir.0, invalid).is_err(), "{invalid}");
        }
        std::fs::write(&path,r#"{"version":1,"materials":{},"models":{},"shopfronts":{"V-04":"wall","V-04":"other"}}"#).unwrap();
        assert!(
            Appearance::load(&path)
                .unwrap_err()
                .contains("duplicate asset binding")
        );
    }

    #[test]
    fn model_preflight_and_context_preserve_material_slots() {
        let dir = TestDir::new();
        let mut document = serde_json::json!({
            "asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],
            "buffers":[{"uri":"triangle.bin","byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36}],
            "accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[1,1,0]}],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0},"material":0}]}],
            "materials":[{"name":"shop_wall","pbrMetallicRoughness":{"baseColorTexture":{"index":0}}}],
            "textures":[{"source":0}],"images":[{"uri":"missing.png"}]
        });
        let path = dir.0.join("sample.gltf");
        std::fs::write(&path, document.to_string()).unwrap();
        let spec = ModelSpec {
            file: "sample.gltf".into(),
            scene: 0,
            scale: 1.0,
            optional: false,
        };
        validate_model(&dir.0, "shop", &spec).unwrap();
        let contexts = model_material_contexts(&dir.0, "shop", &spec)
            .unwrap()
            .join("\n");
        for expected in [
            "asset=shop",
            "scene=0",
            "mesh=0",
            "primitive=0",
            "material=0 (shop_wall)",
            "slot=base_color",
            "missing.png",
        ] {
            assert!(contexts.contains(expected), "{contexts}");
        }
        document["meshes"][0]["primitives"][0]
            .as_object_mut()
            .unwrap()
            .remove("material");
        std::fs::write(&path, document.to_string()).unwrap();
        let error = validate_model(&dir.0, "shop", &spec).unwrap_err();
        assert!(
            error.contains("material slot=<absent>") && error.contains("sample.gltf"),
            "{error}"
        );
        assert!(
            validate_model(
                &dir.0,
                "shop",
                &ModelSpec {
                    scene: 1,
                    ..spec.clone()
                }
            )
            .unwrap_err()
            .contains("scene=1")
        );
        assert!(
            validate_model(
                &dir.0,
                "shop",
                &ModelSpec {
                    file: "missing.glb".into(),
                    ..spec
                }
            )
            .unwrap_err()
            .contains("missing.glb")
        );
    }

    #[derive(Asset, TypePath)]
    struct ImageParent {
        #[dependency]
        _image: Handle<Image>,
    }

    #[derive(TypePath)]
    struct ParentLoader;
    impl AssetLoader for ParentLoader {
        type Asset = ImageParent;
        type Settings = ();
        type Error = std::io::Error;
        async fn load(
            &self,
            reader: &mut dyn Reader,
            _: &(),
            context: &mut LoadContext<'_>,
        ) -> Result<ImageParent, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let path = String::from_utf8(bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(ImageParent {
                _image: context.load(path),
            })
        }
        fn extensions(&self) -> &[&str] {
            &["parent"]
        }
    }

    #[derive(Default)]
    struct Gate {
        open: AtomicBool,
        waker: Mutex<Option<Waker>>,
    }
    impl Gate {
        fn release(&self) {
            self.open.store(true, Ordering::SeqCst);
            if let Some(waker) = self.waker.lock().unwrap().take() {
                waker.wake();
            }
        }
    }

    #[derive(TypePath)]
    struct DelayedImageLoader(Arc<Gate>);
    impl AssetLoader for DelayedImageLoader {
        type Asset = Image;
        type Settings = ImageLoaderSettings;
        type Error = ImageLoaderError;
        async fn load(
            &self,
            reader: &mut dyn Reader,
            settings: &Self::Settings,
            context: &mut LoadContext<'_>,
        ) -> Result<Image, Self::Error> {
            std::future::poll_fn(|cx| {
                let mut waker = self.0.waker.lock().unwrap();
                if self.0.open.load(Ordering::SeqCst) {
                    Poll::Ready(())
                } else {
                    *waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
            .await;
            ImageLoader::new(CompressedImageFormats::empty())
                .load(reader, settings, context)
                .await
        }
        fn extensions(&self) -> &[&str] {
            &["png"]
        }
    }

    fn update_until(app: &mut App, check: impl Fn(&World) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            app.update();
            if check(app.world()) {
                return;
            }
            assert!(Instant::now() < deadline, "asset load test timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn loaded_parent_waits_for_real_png_dependency_and_reports_failures() {
        let dir = TestDir::new();
        std::fs::write(dir.0.join("corrupt.png"), b"not a PNG").unwrap();
        std::fs::write(dir.0.join("corrupt.parent"), "corrupt.png").unwrap();
        std::fs::write(dir.0.join("missing.parent"), "missing.png").unwrap();
        std::fs::write(
            dir.0.join("valid.png"),
            include_bytes!("../../assets/branding/n-logo.png"),
        )
        .unwrap();
        std::fs::write(dir.0.join("valid.parent"), "valid.png").unwrap();
        let gate = Arc::new(Gate::default());
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: dir.0.to_string_lossy().into_owned(),
                watch_for_changes_override: Some(false),
                ..default()
            },
        ))
        .init_asset::<Image>()
        .init_asset::<ImageParent>()
        .register_asset_loader(ParentLoader)
        .register_asset_loader(DelayedImageLoader(gate.clone()));
        app.finish();
        app.cleanup();
        let server = app.world().resource::<AssetServer>().clone();
        let parent: Handle<ImageParent> = server.load("corrupt.parent");
        let contexts = vec![
            "asset=wall scene=0 material slot=base_color source=/buildings/1".into(),
            "asset=wall source=/buildings/2".into(),
        ];
        let tracked = TrackedAsset {
            handle: parent.clone().untyped(),
            contexts: contexts.clone(),
            optional: false,
        };
        assert_eq!(readiness(&server, &tracked), Readiness::Loading);
        update_until(&mut app, |_| {
            matches!(server.get_load_state(parent.id()), Some(LoadState::Loaded))
        });
        assert!(
            app.world()
                .resource::<Assets<ImageParent>>()
                .contains(parent.id())
        );
        assert_eq!(
            readiness(&server, &tracked),
            Readiness::Loading,
            "parent existence is not recursive readiness"
        );
        gate.release();
        update_until(&mut app, |_| {
            matches!(readiness(&server, &tracked), Readiness::Failed(_))
        });
        assert!(matches!(
            server.get_load_state(parent.id()),
            Some(LoadState::Loaded)
        ));
        let Readiness::Failed(error) = readiness(&server, &tracked) else {
            unreachable!()
        };
        assert!(error.contains("corrupt.png"), "{error}");
        let duplicate = TrackedAsset {
            handle: parent.clone().untyped(),
            contexts: vec![contexts[0].clone()],
            optional: true,
        };
        let failures = dependency_failures(&server, &[tracked, duplicate]);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures.values().next().unwrap().len(), 2);

        for (path, succeeds) in [("missing.parent", false), ("valid.parent", true)] {
            let handle: Handle<ImageParent> = server.load(path);
            let tracked = TrackedAsset {
                handle: handle.untyped(),
                contexts: vec![path.into()],
                optional: false,
            };
            update_until(&mut app, |_| {
                !matches!(readiness(&server, &tracked), Readiness::Loading)
            });
            match readiness(&server, &tracked) {
                Readiness::Ready => assert!(succeeds),
                Readiness::Failed(error) => {
                    assert!(!succeeds && error.contains("missing.png"), "{error}")
                }
                Readiness::Loading => unreachable!(),
            }
        }
    }
}
