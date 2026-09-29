//! One durable checkpoint for the implemented QST-001 handoff fragment
use crate::{
    settings::{atomic_write, read_file},
    story::{DeliveryRoute, HANDOFF_PLACES, ShopHandoff},
};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures::check_ready},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const FILE: &str = "progress.json";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    version: u32,
    quest: String,
    checkpoint: String,
    observed_places: Vec<String>,
    confirmed_route: Option<DeliveryRoute>,
}

impl Checkpoint {
    fn capture(story: &ShopHandoff) -> Self {
        Self {
            version: 1,
            quest: "QST-001".into(),
            checkpoint: "home".into(),
            observed_places: story.observed_places.iter().cloned().collect(),
            confirmed_route: story.confirmed_route,
        }
    }

    fn restore(&self) -> Result<ShopHandoff, String> {
        if self.version != 1 || self.quest != "QST-001" || self.checkpoint != "home" {
            return Err("unsupported progress version, quest or checkpoint".into());
        }
        let observed_places: BTreeSet<_> = self.observed_places.iter().cloned().collect();
        if observed_places.len() != self.observed_places.len()
            || !self
                .observed_places
                .iter()
                .all(|id| HANDOFF_PLACES.contains(&id.as_str()))
            || self.confirmed_route.is_some_and(|route| {
                route != DeliveryRoute::ServiceYard || observed_places.len() != HANDOFF_PLACES.len()
            })
        {
            return Err("invalid or contradictory handoff progress".into());
        }
        Ok(ShopHandoff {
            observed_places,
            confirmed_route: self.confirmed_route,
            confirmation_count: u32::from(self.confirmed_route.is_some()),
            rejected_choices: 0,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProgressStatus {
    #[default]
    SessionOnly,
    Ready,
    Saving,
    Saved,
    ReadFailed,
    WriteFailed,
}

impl ProgressStatus {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::SessionOnly => "交接进度仅保留于本次会话",
            Self::Ready => "观察与路线确认后自动保存交接进度",
            Self::Saving => "正在保存交接进度",
            Self::Saved => "交接进度已保存 · 继续时从家门口出发",
            Self::ReadFailed => "存档无法读取，原文件已保留 · 本次进度仅在会话中",
            Self::WriteFailed => "进度保存失败，旧存档已保留 · 本次进度仅在会话中",
        }
    }
}

#[derive(Resource)]
pub(crate) struct ProgressStore {
    path: Option<PathBuf>,
    latest: Checkpoint,
    attempted: Checkpoint,
    pending: Option<Task<Result<(), String>>>,
    pub(crate) status: ProgressStatus,
}

pub(crate) fn install(app: &mut App, directory: Option<PathBuf>, capture: bool, walk: bool) {
    let (story, store) = ProgressStore::load(progress_path(directory, capture, walk));
    app.insert_resource(story)
        .insert_resource(store)
        .add_systems(Last, save_changes);
}

fn progress_path(
    directory: Option<PathBuf>,
    capture: bool,
    walk: bool,
) -> Result<Option<PathBuf>, String> {
    if !walk {
        return Ok(None);
    }
    if let Some(directory) = directory {
        return Ok(Some(directory.join(FILE)));
    }
    if capture {
        return Ok(None);
    }
    #[cfg(target_os = "windows")]
    let directory = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let directory = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    directory
        .filter(|path| path.is_absolute())
        .map(|path| Some(path.join("N-SIDE").join(FILE)))
        .ok_or_else(|| "user data directory is unavailable".into())
}

impl ProgressStore {
    fn load(path: Result<Option<PathBuf>, String>) -> (ShopHandoff, Self) {
        let mut story = ShopHandoff::default();
        let mut status = ProgressStatus::SessionOnly;
        let path = match path {
            Ok(Some(path)) => {
                match read_file(&path).and_then(|bytes| bytes.as_deref().map(decode).transpose()) {
                    Ok(saved) => {
                        status = if saved.is_some() {
                            ProgressStatus::Saved
                        } else {
                            ProgressStatus::Ready
                        };
                        story = saved.unwrap_or_default();
                        Some(path)
                    }
                    Err(error) => {
                        warn!(
                            "[progress/load] {}: {error}; original file retained",
                            path.display()
                        );
                        status = ProgressStatus::ReadFailed;
                        None
                    }
                }
            }
            Ok(None) => None,
            Err(error) => {
                warn!("[progress/path] {error}");
                status = ProgressStatus::ReadFailed;
                None
            }
        };
        let latest = Checkpoint::capture(&story);
        (
            story,
            Self {
                path,
                attempted: latest.clone(),
                latest,
                pending: None,
                status,
            },
        )
    }

    fn report(&mut self, result: Result<(), String>) {
        self.status = match result {
            Ok(()) => ProgressStatus::Saved,
            Err(error) => {
                warn!("[progress/save] {error}; previous progress retained");
                ProgressStatus::WriteFailed
            }
        };
    }
}

fn save_changes(story: Res<ShopHandoff>, mut store: ResMut<ProgressStore>) {
    store.latest = Checkpoint::capture(&story);
    if let Some(result) = store.pending.as_mut().and_then(check_ready) {
        store.pending = None;
        store.report(result);
    }
    if store.pending.is_none()
        && store.latest != store.attempted
        && let Some(path) = store.path.clone()
    {
        let checkpoint = store.latest.clone();
        store.attempted = checkpoint.clone();
        store.status = ProgressStatus::Saving;
        store.pending =
            Some(IoTaskPool::get().spawn(async move { write_file(&path, &checkpoint) }));
    }
}

impl Drop for ProgressStore {
    fn drop(&mut self) {
        if let Some(pending) = self.pending.take() {
            self.report(block_on(pending));
        }
        if self.latest != self.attempted
            && let Some(path) = self.path.clone()
        {
            let checkpoint = self.latest.clone();
            self.report(block_on(
                IoTaskPool::get().spawn(async move { write_file(&path, &checkpoint) }),
            ));
        }
    }
}

fn decode(bytes: &[u8]) -> Result<ShopHandoff, String> {
    serde_json::from_slice::<Checkpoint>(bytes)
        .map_err(|error| error.to_string())?
        .restore()
}

fn write_file(path: &Path, checkpoint: &Checkpoint) -> Result<(), String> {
    checkpoint.restore()?;
    let parent = path.parent().ok_or("progress path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    if let Some(previous) = read_file(path)? {
        decode(&previous)?;
        atomic_write(&path.with_extension("json.bak"), &previous)?;
    }
    let bytes = serde_json::to_vec_pretty(checkpoint).map_err(|error| error.to_string())?;
    atomic_write(path, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "n-side-progress-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn file(&self) -> PathBuf {
            self.0.join(FILE)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn checkpoint_roundtrip_keeps_only_observed_facts_and_derives_stage() {
        let mut story = ShopHandoff::default();
        for id in HANDOFF_PLACES {
            story.observe(id);
            let loaded =
                decode(&serde_json::to_vec(&Checkpoint::capture(&story)).unwrap()).unwrap();
            assert_eq!(loaded, story);
        }
        story.choose(Some("28"), DeliveryRoute::PublicSteps);
        story.choose(Some("28"), DeliveryRoute::ServiceYard);
        let loaded = decode(&serde_json::to_vec(&Checkpoint::capture(&story)).unwrap()).unwrap();
        assert_eq!(loaded.stage(), crate::story::HandoffStage::Confirmed);
        assert_eq!(loaded.confirmation_count, 1);
        assert_eq!(
            loaded.rejected_choices, 0,
            "unsuccessful attempts are session state"
        );
    }

    #[test]
    fn rejects_invalid_future_duplicate_and_contradictory_slots_without_replacement() {
        let directory = Directory::new();
        let valid = serde_json::to_value(Checkpoint::capture(&ShopHandoff::default())).unwrap();
        let mut invalid = vec![b"{broken".to_vec(), vec![b' '; 4097]];
        for (field, value) in [
            ("version", serde_json::json!(2)),
            ("quest", serde_json::json!("QST-002")),
            ("checkpoint", serde_json::json!("unchecked_position")),
            ("observed_places", serde_json::json!(["99"])),
            ("observed_places", serde_json::json!(["04", "04"])),
            ("confirmed_route", serde_json::json!("public_steps")),
            ("confirmed_route", serde_json::json!("service_yard")),
        ] {
            let mut data = valid.clone();
            data[field] = value;
            invalid.push(serde_json::to_vec(&data).unwrap());
        }
        for bytes in invalid {
            fs::write(directory.file(), &bytes).unwrap();
            let (story, store) = ProgressStore::load(Ok(Some(directory.file())));
            assert_eq!(story, ShopHandoff::default());
            assert_eq!(store.status, ProgressStatus::ReadFailed);
            assert!(store.path.is_none());
            assert!(write_file(&directory.file(), &Checkpoint::capture(&story)).is_err());
            assert_eq!(fs::read(directory.file()).unwrap(), bytes);
        }
    }

    #[test]
    fn restart_while_save_is_pending_flushes_the_empty_checkpoint_last() {
        let directory = Directory::new();
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            install(&mut app, Some(directory.0.clone()), true, true);
            app.world_mut().resource_mut::<ShopHandoff>().observe("04");
            app.update();
            assert!(app.world().resource::<ProgressStore>().pending.is_some());
            *app.world_mut().resource_mut::<ShopHandoff>() = ShopHandoff::default();
            app.update();
        }
        assert_eq!(
            decode(&fs::read(directory.file()).unwrap()).unwrap(),
            ShopHandoff::default()
        );
        assert_eq!(
            decode(&fs::read(directory.file().with_extension("json.bak")).unwrap())
                .unwrap()
                .observed_places,
            BTreeSet::from(["04".into()])
        );
    }

    #[test]
    fn async_exit_flush_load_reset_and_atomic_failure_preserve_the_committed_slot() {
        let directory = Directory::new();
        {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            install(&mut app, Some(directory.0.clone()), true, true);
            app.world_mut().resource_mut::<ShopHandoff>().observe("04");
            app.update();
            app.world_mut().resource_mut::<ShopHandoff>().observe("29");
            app.update();
        }
        let (loaded, _) = ProgressStore::load(Ok(Some(directory.file())));
        assert_eq!(
            loaded.observed_places,
            BTreeSet::from(["04".into(), "29".into()])
        );
        let original = fs::read(directory.file()).unwrap();
        let blocked = directory
            .file()
            .with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&blocked, b"occupied").unwrap();
        let result = write_file(
            &directory.file(),
            &Checkpoint::capture(&ShopHandoff::default()),
        );
        assert!(result.is_err());
        assert_eq!(fs::read(directory.file()).unwrap(), original);
        fs::remove_file(blocked).unwrap();
        write_file(
            &directory.file(),
            &Checkpoint::capture(&ShopHandoff::default()),
        )
        .unwrap();
        assert_eq!(
            fs::read(directory.file().with_extension("json.bak")).unwrap(),
            original
        );
        assert_eq!(
            decode(&fs::read(directory.file()).unwrap()).unwrap(),
            ShopHandoff::default()
        );
        assert!(progress_path(None, true, true).unwrap().is_none());
        assert!(
            progress_path(Some(directory.0.clone()), false, false)
                .unwrap()
                .is_none()
        );
    }
}
