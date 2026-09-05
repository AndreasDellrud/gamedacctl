use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::{BreatheDuration, BreatheMode, Color, LightingPlan, ProtocolError, Zone};

pub const PROFILE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProfileStore {
    pub schema_version: u32,
    pub last_selected: Option<String>,
    pub apply_on_reconnect: bool,
    pub profiles: Vec<Profile>,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            last_selected: None,
            apply_on_reconnect: false,
            profiles: Vec::new(),
        }
    }
}

impl ProfileStore {
    pub fn default_path() -> Result<PathBuf, ProfileError> {
        let directories = ProjectDirs::from("io.github", "AndreasDellrud", "gamedacctl")
            .ok_or(ProfileError::ConfigDirectoryUnavailable)?;
        Ok(directories.config_dir().join("profiles.json"))
    }

    pub fn load() -> Result<Self, ProfileError> {
        Self::load_from(&Self::default_path()?)
    }

    pub fn load_from(path: &Path) -> Result<Self, ProfileError> {
        match fs::read(path) {
            Ok(bytes) => {
                let store: Self = serde_json::from_slice(&bytes).map_err(ProfileError::Decode)?;
                store.validate()?;
                Ok(store)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(ProfileError::Read(error)),
        }
    }

    pub fn update(
        update: impl FnOnce(&mut Self) -> Result<(), ProfileError>,
    ) -> Result<Self, ProfileError> {
        Self::update_at(&Self::default_path()?, update)
    }

    pub fn update_at(
        path: &Path,
        update: impl FnOnce(&mut Self) -> Result<(), ProfileError>,
    ) -> Result<Self, ProfileError> {
        let parent = path
            .parent()
            .ok_or(ProfileError::ConfigDirectoryUnavailable)?;
        fs::create_dir_all(parent).map_err(ProfileError::CreateDirectory)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(ProfileError::SetPermissions)?;

        let lock_path = path.with_extension("lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(lock_path)
            .map_err(ProfileError::Lock)?;
        lock.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(ProfileError::SetPermissions)?;
        lock.lock().map_err(ProfileError::Lock)?;

        let mut store = Self::load_from(path)?;
        update(&mut store)?;
        store.write_locked(path, parent)?;
        Ok(store)
    }

    fn write_locked(&self, path: &Path, parent: &Path) -> Result<(), ProfileError> {
        self.validate()?;

        let bytes = serde_json::to_vec_pretty(self).map_err(ProfileError::Encode)?;
        let mut temporary = NamedTempFile::new_in(parent).map_err(ProfileError::Write)?;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(ProfileError::SetPermissions)?;
        temporary.write_all(&bytes).map_err(ProfileError::Write)?;
        temporary.write_all(b"\n").map_err(ProfileError::Write)?;
        temporary
            .as_file()
            .sync_all()
            .map_err(ProfileError::Write)?;
        temporary
            .persist(path)
            .map_err(|error| ProfileError::Write(error.error))?;
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(ProfileError::SyncDirectory)?;
        Ok(())
    }

    pub fn selected(&self) -> Option<&Profile> {
        let selected = self.last_selected.as_deref()?;
        self.profiles
            .iter()
            .find(|profile| profile.name == selected)
    }

    pub fn select(&mut self, name: &str) -> Result<(), ProfileError> {
        if self.profiles.iter().any(|profile| profile.name == name) {
            self.last_selected = Some(name.to_owned());
            Ok(())
        } else {
            Err(ProfileError::ProfileNotFound(name.to_owned()))
        }
    }

    pub fn upsert(&mut self, profile: Profile) -> Result<(), ProfileError> {
        profile.validate()?;
        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|existing| existing.name == profile.name)
        {
            *existing = profile;
        } else {
            self.profiles.push(profile);
            self.profiles
                .sort_by_key(|profile| profile.name.to_lowercase());
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.schema_version != PROFILE_SCHEMA_VERSION {
            return Err(ProfileError::UnsupportedSchema(self.schema_version));
        }

        let mut names = HashSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            if !names.insert(&profile.name) {
                return Err(ProfileError::DuplicateName(profile.name.clone()));
            }
        }

        if let Some(selected) = &self.last_selected
            && !names.contains(selected)
        {
            return Err(ProfileError::MissingSelection(selected.clone()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub lighting: ProfileLighting,
}

impl Profile {
    pub fn plan(&self) -> Result<LightingPlan, ProfileError> {
        self.validate()?;
        match &self.lighting {
            ProfileLighting::Static {
                left,
                right,
                microphone_live,
                microphone_muted,
            } => Ok(LightingPlan::steady([
                (Zone::Left, *left),
                (Zone::Right, *right),
                (Zone::MicrophoneLive, *microphone_live),
                (Zone::MicrophoneMuted, *microphone_muted),
            ])?),
            ProfileLighting::Breathe {
                color,
                seconds,
                mode,
                reverse,
            } => Ok(LightingPlan::breathe(
                *color,
                BreatheDuration::from_seconds(*seconds)?,
                (*mode).into(),
                *reverse,
            )?),
            ProfileLighting::ColorShift { colors, seconds } => Ok(LightingPlan::color_shift(
                colors,
                BreatheDuration::from_seconds(*seconds)?,
            )?),
            ProfileLighting::MultiColorBreathe {
                colors,
                seconds,
                mode,
                reverse,
            } => {
                if *mode == ProfileBreatheMode::Synchronized && !reverse {
                    Ok(LightingPlan::multi_color_breathe(
                        colors,
                        BreatheDuration::from_seconds(*seconds)?,
                    )?)
                } else {
                    let duration = BreatheDuration::from_seconds(*seconds)?;
                    let header = colors.first().copied().unwrap_or(Color::BLACK);
                    let features = [Zone::Right, Zone::Left]
                        .into_iter()
                        .map(|zone| {
                            crate::FeatureReport::multi_color_breathe(
                                zone,
                                header,
                                colors,
                                duration,
                                (*mode).into(),
                                *reverse,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(LightingPlan::captured(features)?)
                }
            }
        }
    }

    fn validate(&self) -> Result<(), ProfileError> {
        if self.name.trim().is_empty() {
            return Err(ProfileError::EmptyName);
        }
        if self.name != self.name.trim() {
            return Err(ProfileError::UntrimmedName(self.name.clone()));
        }
        if let Some(icon) = &self.icon {
            if icon != icon.trim() {
                return Err(ProfileError::UntrimmedIcon(icon.clone()));
            }
            if icon.chars().count() > 8 {
                return Err(ProfileError::IconTooLong(icon.clone()));
            }
        }
        self.plan_fields_only()?;
        Ok(())
    }

    fn plan_fields_only(&self) -> Result<(), ProfileError> {
        match &self.lighting {
            ProfileLighting::Breathe {
                seconds,
                mode,
                reverse,
                ..
            }
            | ProfileLighting::MultiColorBreathe {
                seconds,
                mode,
                reverse,
                ..
            } => {
                BreatheDuration::from_seconds(*seconds)?;
                if *reverse && *mode != ProfileBreatheMode::Sweep {
                    return Err(ProfileError::Protocol(ProtocolError::ReverseRequiresSweep));
                }
            }
            ProfileLighting::ColorShift { seconds, .. } => {
                BreatheDuration::from_seconds(*seconds)?;
            }
            ProfileLighting::Static { .. } => {}
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "kebab-case")]
pub enum ProfileLighting {
    Static {
        left: Color,
        right: Color,
        microphone_live: Color,
        microphone_muted: Color,
    },
    Breathe {
        color: Color,
        seconds: u16,
        mode: ProfileBreatheMode,
        reverse: bool,
    },
    ColorShift {
        colors: Vec<Color>,
        seconds: u16,
    },
    MultiColorBreathe {
        colors: Vec<Color>,
        seconds: u16,
        mode: ProfileBreatheMode,
        reverse: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileBreatheMode {
    Synchronized,
    Sweep,
}

impl From<ProfileBreatheMode> for BreatheMode {
    fn from(value: ProfileBreatheMode) -> Self {
        match value {
            ProfileBreatheMode::Synchronized => Self::Synchronized,
            ProfileBreatheMode::Sweep => Self::Sweep,
        }
    }
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("the user configuration directory is unavailable")]
    ConfigDirectoryUnavailable,
    #[error("could not read the profile store: {0}")]
    Read(#[source] std::io::Error),
    #[error("could not create the profile directory: {0}")]
    CreateDirectory(#[source] std::io::Error),
    #[error("could not set private profile-storage permissions: {0}")]
    SetPermissions(#[source] std::io::Error),
    #[error("could not lock the profile store: {0}")]
    Lock(#[source] std::io::Error),
    #[error("could not write the profile store atomically: {0}")]
    Write(#[source] std::io::Error),
    #[error("could not synchronize the profile directory: {0}")]
    SyncDirectory(#[source] std::io::Error),
    #[error("could not decode the profile store: {0}")]
    Decode(#[source] serde_json::Error),
    #[error("could not encode the profile store: {0}")]
    Encode(#[source] serde_json::Error),
    #[error("profile schema version {0} is not supported")]
    UnsupportedSchema(u32),
    #[error("profile names cannot be empty")]
    EmptyName,
    #[error("profile name must not have surrounding whitespace: {0:?}")]
    UntrimmedName(String),
    #[error("profile icon must not have surrounding whitespace: {0:?}")]
    UntrimmedIcon(String),
    #[error("profile icon must contain at most eight Unicode characters: {0:?}")]
    IconTooLong(String),
    #[error("profile name is duplicated: {0:?}")]
    DuplicateName(String),
    #[error("saved profile was not found: {0:?}")]
    ProfileNotFound(String),
    #[error("last-selected profile does not exist: {0:?}")]
    MissingSelection(String),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Barrier},
        thread,
    };

    use tempfile::tempdir;

    use super::*;

    fn static_profile(name: &str) -> Profile {
        Profile {
            name: name.to_owned(),
            icon: None,
            lighting: ProfileLighting::Static {
                left: Color::new(0xFF, 0x37, 0x00),
                right: Color::new(0x00, 0x84, 0xFF),
                microphone_live: Color::new(0x00, 0xFF, 0x00),
                microphone_muted: Color::new(0xFF, 0x00, 0x00),
            },
        }
    }

    #[test]
    fn missing_store_loads_safe_defaults() {
        let directory = tempdir().unwrap();
        let store = ProfileStore::load_from(&directory.path().join("profiles.json")).unwrap();
        assert_eq!(store, ProfileStore::default());
        assert!(!store.apply_on_reconnect);
    }

    #[test]
    fn profile_store_round_trips_atomically() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("nested/profiles.json");
        let store = ProfileStore::update_at(&path, |store| {
            store.upsert(static_profile("Everyday"))?;
            store.select("Everyday")?;
            store.apply_on_reconnect = true;
            Ok(())
        })
        .unwrap();

        assert_eq!(ProfileStore::load_from(&path).unwrap(), store);
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(path.with_extension("lock"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn rejected_and_malformed_updates_leave_the_existing_store_unchanged() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("profiles.json");
        let original = br#"{"schema_version":1,"last_selected":null,"apply_on_reconnect":false,"profiles":[]}"#;
        fs::write(&path, original).unwrap();

        let error = ProfileStore::update_at(&path, |_| Err(ProfileError::EmptyName)).unwrap_err();
        assert!(matches!(error, ProfileError::EmptyName));
        assert_eq!(fs::read(&path).unwrap(), original);

        let malformed = b"{not valid json\n";
        fs::write(&path, malformed).unwrap();
        let error = ProfileStore::update_at(&path, |store| {
            store.upsert(static_profile("Must not appear"))
        })
        .unwrap_err();
        assert!(matches!(error, ProfileError::Decode(_)));
        assert_eq!(fs::read(&path).unwrap(), malformed);
    }

    #[test]
    fn concurrent_updates_serialize_without_losing_profiles() {
        let directory = tempdir().unwrap();
        let path = Arc::new(directory.path().join("profiles.json"));
        let barrier = Arc::new(Barrier::new(3));
        let mut workers = Vec::new();

        for name in ["First", "Second"] {
            let path = path.clone();
            let barrier = barrier.clone();
            workers.push(thread::spawn(move || {
                barrier.wait();
                ProfileStore::update_at(&path, |store| store.upsert(static_profile(name))).unwrap();
            }));
        }

        barrier.wait();
        for worker in workers {
            worker.join().unwrap();
        }

        let store = ProfileStore::load_from(&path).unwrap();
        assert_eq!(
            store
                .profiles
                .iter()
                .map(|profile| profile.name.as_str())
                .collect::<Vec<_>>(),
            ["First", "Second"]
        );
    }

    #[test]
    fn concurrent_updates_preserve_independent_selection_and_policy_changes() {
        let directory = tempdir().unwrap();
        let path = Arc::new(directory.path().join("profiles.json"));
        ProfileStore::update_at(&path, |store| {
            store.upsert(static_profile("First"))?;
            store.upsert(static_profile("Second"))?;
            store.select("First")
        })
        .unwrap();

        let barrier = Arc::new(Barrier::new(3));
        let selection_path = path.clone();
        let selection_barrier = barrier.clone();
        let selection = thread::spawn(move || {
            selection_barrier.wait();
            ProfileStore::update_at(&selection_path, |store| store.select("Second")).unwrap();
        });
        let policy_path = path.clone();
        let policy_barrier = barrier.clone();
        let policy = thread::spawn(move || {
            policy_barrier.wait();
            ProfileStore::update_at(&policy_path, |store| {
                store.apply_on_reconnect = true;
                Ok(())
            })
            .unwrap();
        });

        barrier.wait();
        selection.join().unwrap();
        policy.join().unwrap();

        let store = ProfileStore::load_from(&path).unwrap();
        assert_eq!(store.last_selected.as_deref(), Some("Second"));
        assert!(store.apply_on_reconnect);
        assert_eq!(store.profiles.len(), 2);
    }

    #[test]
    fn default_path_uses_the_xdg_configuration_location() {
        let path = ProfileStore::default_path().unwrap();
        assert!(path.ends_with("gamedacctl/profiles.json"));
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            assert!(path.starts_with(config_home));
        } else if let Some(home) = std::env::var_os("HOME") {
            assert!(path.starts_with(Path::new(&home).join(".config")));
        }
    }

    #[test]
    fn store_rejects_invalid_references_and_effects() {
        let mut store = ProfileStore {
            last_selected: Some("Missing".to_owned()),
            ..ProfileStore::default()
        };
        assert!(matches!(
            store.validate(),
            Err(ProfileError::MissingSelection(_))
        ));

        store.last_selected = None;
        assert!(matches!(
            store.upsert(Profile {
                name: "Invalid".to_owned(),
                icon: None,
                lighting: ProfileLighting::Breathe {
                    color: Color::new(1, 2, 3),
                    seconds: 10,
                    mode: ProfileBreatheMode::Synchronized,
                    reverse: true,
                },
            }),
            Err(ProfileError::Protocol(ProtocolError::ReverseRequiresSweep))
        ));
    }

    #[test]
    fn profile_builds_only_verified_lighting_plans() {
        let static_plan = static_profile("Static").plan().unwrap();
        assert_eq!(static_plan.zone_mask(), 0x0F);

        let breathe_plan = Profile {
            name: "Pulse".to_owned(),
            icon: Some("💜".to_owned()),
            lighting: ProfileLighting::Breathe {
                color: Color::new(0x7A, 0x21, 0xE6),
                seconds: 10,
                mode: ProfileBreatheMode::Synchronized,
                reverse: false,
            },
        }
        .plan()
        .unwrap();
        assert_eq!(breathe_plan.zone_mask(), 0x03);

        for lighting in [
            ProfileLighting::ColorShift {
                colors: vec![Color::new(0xFF, 0, 0), Color::new(0, 0, 0xFF)],
                seconds: 10,
            },
            ProfileLighting::MultiColorBreathe {
                colors: vec![Color::new(0xFF, 0, 0), Color::new(0, 0, 0xFF)],
                seconds: 10,
                mode: ProfileBreatheMode::Synchronized,
                reverse: false,
            },
        ] {
            let plan = Profile {
                name: "Palette".to_owned(),
                icon: None,
                lighting,
            }
            .plan()
            .unwrap();
            assert_eq!(plan.zone_mask(), 0x03);
        }
    }

    #[test]
    fn legacy_single_color_breathe_profile_remains_compatible() {
        let profile: Profile = serde_json::from_str(
            r#"{
                "name": "Legacy",
                "lighting": {
                    "effect": "breathe",
                    "color": "7A21E6",
                    "seconds": 10,
                    "mode": "synchronized",
                    "reverse": false
                }
            }"#,
        )
        .unwrap();
        assert!(matches!(profile.lighting, ProfileLighting::Breathe { .. }));
        assert_eq!(profile.plan().unwrap().zone_mask(), 0x03);
    }

    #[test]
    fn profile_icons_are_optional_bounded_unicode_strings() {
        let mut profile = static_profile("Icon");
        profile.icon = Some("🎧".to_owned());
        assert!(profile.plan().is_ok());

        profile.icon = Some("  🎧".to_owned());
        assert!(matches!(
            profile.plan(),
            Err(ProfileError::UntrimmedIcon(_))
        ));

        profile.icon = Some("123456789".to_owned());
        assert!(matches!(profile.plan(), Err(ProfileError::IconTooLong(_))));
    }
}
