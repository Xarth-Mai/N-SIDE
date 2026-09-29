use crate::{app::EntrySettings, graphics::DisplayTrial};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures::check_ready},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const FILE: &str = "settings.json";
const MAX_BYTES: u64 = 4096;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Preferences {
    version: u32,
    large_text: bool,
    slow_camera: bool,
    #[serde(default)]
    graphics: Option<crate::graphics::GraphicsSettings>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SaveStatus {
    #[default]
    SessionOnly,
    Ready,
    Saving,
    Saved,
    ReadFailed,
    WriteFailed,
}

impl SaveStatus {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::SessionOnly => "即时应用 · 当前会话有效",
            Self::Ready => "即时应用 · 调整后自动保存",
            Self::Saving => "即时应用 · 正在保存",
            Self::Saved => "已保存 · 下次启动继续使用",
            Self::ReadFailed => "原设置无法读取，已保留 · 本次仅会话有效",
            Self::WriteFailed => "保存失败，旧文件已保留 · 本次调整仍有效",
        }
    }
}

#[derive(Resource)]
pub(crate) struct SettingsStore {
    path: Option<PathBuf>,
    latest: EntrySettings,
    attempted: EntrySettings,
    pending: Option<Task<Result<(), String>>>,
    pub(crate) status: SaveStatus,
}

pub(crate) fn install(app: &mut App, directory: Option<PathBuf>, capture: bool) {
    let (settings, store) = SettingsStore::load(settings_path(directory, capture));
    app.insert_resource(settings)
        .insert_resource(store)
        .add_systems(Last, save_changes);
}

pub(crate) fn reset_graphics(app: &mut App) {
    app.world_mut().resource_mut::<EntrySettings>().graphics = default();
}

fn settings_path(directory: Option<PathBuf>, capture: bool) -> Result<Option<PathBuf>, String> {
    if let Some(directory) = directory {
        return Ok(Some(directory.join(FILE)));
    }
    if capture {
        return Ok(None);
    }
    #[cfg(target_os = "windows")]
    let directory = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let directory = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    directory
        .filter(|path| path.is_absolute())
        .map(|directory| Some(directory.join("N-SIDE").join(FILE)))
        .ok_or_else(|| {
            "user configuration directory is unavailable; settings remain in memory".into()
        })
}

impl SettingsStore {
    fn load(path: Result<Option<PathBuf>, String>) -> (EntrySettings, Self) {
        let mut settings = EntrySettings::default();
        let mut status = SaveStatus::SessionOnly;
        let path = match path {
            Ok(Some(path)) => match read_file(&path) {
                Ok(bytes) => match bytes.as_deref().map(decode).transpose() {
                    Ok(loaded) => {
                        settings = loaded.unwrap_or_default();
                        status = if loaded.is_some() {
                            SaveStatus::Saved
                        } else {
                            SaveStatus::Ready
                        };
                        Some(path)
                    }
                    Err(error) => {
                        warn!(
                            "[settings/load] {}: {error}; original file retained",
                            path.display()
                        );
                        status = SaveStatus::ReadFailed;
                        None
                    }
                },
                Err(error) => {
                    warn!("[settings/load] {error}; original file retained");
                    status = SaveStatus::ReadFailed;
                    None
                }
            },
            Ok(None) => None,
            Err(error) => {
                warn!("[settings/path] {error}");
                status = SaveStatus::ReadFailed;
                None
            }
        };
        (
            settings,
            Self {
                path,
                latest: settings,
                attempted: settings,
                pending: None,
                status,
            },
        )
    }

    fn report(&mut self, result: Result<(), String>) {
        self.status = match result {
            Ok(()) => SaveStatus::Saved,
            Err(error) => {
                warn!("[settings/save] {error}; previous settings retained");
                SaveStatus::WriteFailed
            }
        };
    }
}

fn save_changes(
    settings: Res<EntrySettings>,
    trial: Option<Res<DisplayTrial>>,
    mut store: ResMut<SettingsStore>,
) {
    store.latest = trial.map_or(*settings, |trial| trial.persisted(*settings));
    if let Some(result) = store.pending.as_mut().and_then(check_ready) {
        store.pending = None;
        store.report(result);
    }
    if store.pending.is_none()
        && store.latest != store.attempted
        && let Some(path) = store.path.clone()
    {
        let snapshot = store.latest;
        store.attempted = snapshot;
        store.status = SaveStatus::Saving;
        store.pending = Some(IoTaskPool::get().spawn(async move { write_file(&path, snapshot) }));
    }
}

impl Drop for SettingsStore {
    fn drop(&mut self) {
        // Native runners consume App; finish queued writes before its resource is dropped
        if let Some(pending) = self.pending.take() {
            self.report(block_on(pending));
        }
        if self.latest != self.attempted
            && let Some(path) = self.path.clone()
        {
            let snapshot = self.latest;
            self.report(block_on(
                IoTaskPool::get().spawn(async move { write_file(&path, snapshot) }),
            ));
        }
    }
}

fn decode(bytes: &[u8]) -> Result<EntrySettings, String> {
    let data: Preferences = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    if ![1, 2].contains(&data.version) {
        return Err(format!(
            "unsupported settings version {} (expected 1 or 2)",
            data.version
        ));
    }
    if data.version == 2 && data.graphics.is_none() {
        return Err("version 2 requires graphics settings".into());
    }
    let graphics = data.graphics.unwrap_or_default();
    graphics.validate()?;
    Ok(EntrySettings {
        large_text: data.large_text,
        slow_camera: data.slow_camera,
        graphics,
    })
}

pub(crate) fn read_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!("{} exceeds {MAX_BYTES} bytes", path.display()));
    }
    Ok(Some(bytes))
}

fn write_file(path: &Path, settings: EntrySettings) -> Result<(), String> {
    settings.graphics.validate()?;
    let parent = path.parent().ok_or("settings path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    // Recheck before replacement: a newer application or external edit may have changed the file
    if let Some(previous) = read_file(path)? {
        decode(&previous)?;
        atomic_write(&path.with_extension("json.bak"), &previous)?;
    }
    let bytes = serde_json::to_vec_pretty(&Preferences {
        version: 2,
        large_text: settings.large_text,
        slow_camera: settings.slow_camera,
        graphics: Some(settings.graphics),
    })
    .map_err(|error| error.to_string())?;
    atomic_write(path, &bytes)
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("{}: {error}", temporary.display()))?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })()
    .map_err(|error| format!("{}: {error}", path.display()));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "n-side-settings-{}-{}",
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

    fn app(directory: Option<PathBuf>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        install(&mut app, directory, true);
        app
    }

    fn changed() -> EntrySettings {
        EntrySettings {
            large_text: true,
            slow_camera: true,
            ..default()
        }
    }

    #[test]
    fn pending_display_choices_are_not_saved_on_exit_but_confirmation_is() {
        for confirm in [false, true] {
            let directory = Directory::new();
            let mut expected = changed();
            expected.graphics.vsync = false;
            {
                let mut app = app(Some(directory.0.clone()));
                app.init_resource::<DisplayTrial>();
                let previous = app.world().resource::<EntrySettings>().graphics;
                app.world_mut()
                    .resource_mut::<DisplayTrial>()
                    .start(previous);
                *app.world_mut().resource_mut::<EntrySettings>() = expected;
                {
                    let mut settings = app.world_mut().resource_mut::<EntrySettings>();
                    settings.graphics.borderless = true;
                    settings.graphics.resolution = 2;
                }
                app.update();
                assert!(app.world().resource::<SettingsStore>().latest == expected);
                if confirm {
                    app.world_mut().resource_mut::<DisplayTrial>().confirm();
                    expected.graphics.borderless = true;
                    expected.graphics.resolution = 2;
                    app.update();
                }
            }
            assert!(decode(&fs::read(directory.file()).unwrap()).unwrap() == expected);
        }
    }

    #[test]
    fn graphics_reset_preserves_accessibility_and_cannot_overwrite_a_bad_file() {
        let directory = Directory::new();
        let mut settings = changed();
        settings.graphics.resolution = 3;
        settings.graphics.borderless = true;
        settings.graphics.vsync = false;
        write_file(&directory.file(), settings).unwrap();
        {
            let mut app = app(Some(directory.0.clone()));
            reset_graphics(&mut app);
            app.update();
        }
        assert!(decode(&fs::read(directory.file()).unwrap()).unwrap() == changed());
        let broken = b"{broken";
        fs::write(directory.file(), broken).unwrap();
        {
            let mut app = app(Some(directory.0.clone()));
            app.world_mut().resource_mut::<EntrySettings>().graphics = settings.graphics;
            reset_graphics(&mut app);
            app.update();
            assert_eq!(
                app.world().resource::<EntrySettings>().graphics,
                crate::graphics::GraphicsSettings::default()
            );
            assert_eq!(
                app.world().resource::<SettingsStore>().status,
                SaveStatus::ReadFailed
            );
        }
        assert_eq!(fs::read(directory.file()).unwrap(), broken);
    }

    #[test]
    fn version_one_migrates_and_graphics_boundaries_preserve_files() {
        let legacy = decode(br#"{"version":1,"large_text":true,"slow_camera":false}"#).unwrap();
        assert!(legacy.large_text);
        assert_eq!(
            legacy.graphics,
            crate::graphics::GraphicsSettings::default()
        );
        let directory = Directory::new();
        let mut settings = legacy;
        settings.graphics.adjust(2, false);
        write_file(&directory.file(), settings).unwrap();
        assert!(decode(&fs::read(directory.file()).unwrap()).unwrap() == settings);
        let mut invalid: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.file()).unwrap()).unwrap();
        invalid["graphics"]["antialiasing"] = serde_json::json!(255);
        let bytes = serde_json::to_vec(&invalid).unwrap();
        fs::write(directory.file(), &bytes).unwrap();
        assert!(write_file(&directory.file(), legacy).is_err());
        assert_eq!(fs::read(directory.file()).unwrap(), bytes);
    }

    #[test]
    fn capture_is_isolated_unless_a_directory_is_explicit() {
        assert_eq!(settings_path(None, true).unwrap(), None);
        let directory = Directory::new();
        assert_eq!(
            settings_path(Some(directory.0.clone()), true).unwrap(),
            Some(directory.file())
        );
        let mut app = app(None);
        *app.world_mut().resource_mut::<EntrySettings>() = changed();
        app.update();
        let store = app.world().resource::<SettingsStore>();
        assert_eq!(store.status, SaveStatus::SessionOnly);
        assert!(store.path.is_none() && store.pending.is_none());
        assert!(!directory.file().exists());
    }

    #[test]
    fn invalid_or_future_files_are_preserved_even_after_changes() {
        for contents in [
            b"{broken".to_vec(),
            br#"{"version":2,"large_text":true,"slow_camera":true}"#.to_vec(),
            br#"{"version":1,"large_text":true}"#.to_vec(),
            br#"{"version":1,"large_text":true,"slow_camera":true,"other":1}"#.to_vec(),
            vec![b' '; MAX_BYTES as usize + 1],
        ] {
            let directory = Directory::new();
            fs::write(directory.file(), &contents).unwrap();
            {
                let mut app = app(Some(directory.0.clone()));
                assert!(!app.world().resource::<EntrySettings>().large_text);
                assert_eq!(
                    app.world().resource::<SettingsStore>().status,
                    SaveStatus::ReadFailed
                );
                *app.world_mut().resource_mut::<EntrySettings>() = changed();
                app.update();
            }
            assert_eq!(fs::read(directory.file()).unwrap(), contents);
            assert!(!directory.file().with_extension("json.bak").exists());
        }
    }

    #[test]
    fn write_failure_preserves_the_last_valid_file_and_reports_failure() {
        let directory = Directory::new();
        write_file(&directory.file(), EntrySettings::default()).unwrap();
        let original = fs::read(directory.file()).unwrap();
        // An existing temporary file is not ours to overwrite or clean up
        let temporary = directory
            .file()
            .with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, b"existing partial write").unwrap();
        let mut app = app(Some(directory.0.clone()));
        *app.world_mut().resource_mut::<EntrySettings>() = changed();
        app.update();
        {
            let mut store = app.world_mut().resource_mut::<SettingsStore>();
            let result = block_on(store.pending.take().unwrap());
            assert!(result.is_err());
            store.report(result);
            assert_eq!(store.status, SaveStatus::WriteFailed);
        }
        app.update();
        assert!(app.world().resource::<SettingsStore>().pending.is_none());
        assert_eq!(fs::read(directory.file()).unwrap(), original);
        assert_eq!(fs::read(&temporary).unwrap(), b"existing partial write");
        assert_eq!(
            fs::read(directory.file().with_extension("json.bak")).unwrap(),
            original
        );
    }

    #[test]
    fn exit_finishes_the_latest_change_and_keeps_a_valid_backup() {
        let directory = Directory::new();
        write_file(&directory.file(), EntrySettings::default()).unwrap();
        {
            let mut app = app(Some(directory.0.clone()));
            app.world_mut().resource_mut::<EntrySettings>().large_text = true;
            app.update();
            app.world_mut().resource_mut::<EntrySettings>().slow_camera = true;
            app.update();
            // Drop must finish both the in-flight save and the latest queued values
        }
        assert!(decode(&fs::read(directory.file()).unwrap()).unwrap() == changed());
        assert!(decode(&fs::read(directory.file().with_extension("json.bak")).unwrap()).is_ok());
        let loaded = app(Some(directory.0.clone()));
        assert!(*loaded.world().resource::<EntrySettings>() == changed());
        assert_eq!(
            loaded.world().resource::<SettingsStore>().status,
            SaveStatus::Saved
        );
    }

    #[test]
    fn preferences_restore_in_a_second_process() {
        let directory = Directory::new();
        for mode in ["write", "read"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "settings::tests::restart_worker",
                    "--nocapture",
                ])
                .env("NSIDE_SETTINGS_TEST_DIR", &directory.0)
                .env("NSIDE_SETTINGS_TEST_MODE", mode)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    #[ignore = "invoked in two isolated processes by preferences_restore_in_a_second_process"]
    fn restart_worker() {
        let directory = PathBuf::from(std::env::var_os("NSIDE_SETTINGS_TEST_DIR").unwrap());
        let mut app = app(Some(directory));
        if std::env::var("NSIDE_SETTINGS_TEST_MODE").unwrap() == "write" {
            assert!(*app.world().resource::<EntrySettings>() == EntrySettings::default());
            *app.world_mut().resource_mut::<EntrySettings>() = changed();
            app.update();
        } else {
            assert!(*app.world().resource::<EntrySettings>() == changed());
            assert_eq!(
                app.world().resource::<SettingsStore>().status,
                SaveStatus::Saved
            );
        }
    }
}
