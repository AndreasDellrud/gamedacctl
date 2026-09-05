use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
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

    pub fn save(&self) -> Result<(), ProfileError> {
        self.save_to(&Self::default_path()?)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), ProfileError> {
        self.validate()?;
        let parent = path
            .parent()
            .ok_or(ProfileError::ConfigDirectoryUnavailable)?;
        fs::create_dir_all(parent).map_err(ProfileError::CreateDirectory)?;

        let temporary = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(self).map_err(ProfileError::Encode)?;
        let mut file = fs::File::create(&temporary).map_err(ProfileError::Write)?;
        file.write_all(&bytes).map_err(ProfileError::Write)?;
        file.write_all(b"\n").map_err(ProfileError::Write)?;
        file.sync_all().map_err(ProfileError::Write)?;
        fs::rename(temporary, path).map_err(ProfileError::Write)?;
        Ok(())
    }

    pub fn selected(&self) -> Option<&Profile> {
        let selected = self.last_selected.as_deref()?;
        self.profiles
            .iter()
            .find(|profile| profile.name == selected)
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
        match self.lighting {
            ProfileLighting::Static {
                left,
                right,
                microphone_live,
                microphone_muted,
            } => Ok(LightingPlan::steady([
                (Zone::Left, left),
                (Zone::Right, right),
                (Zone::MicrophoneLive, microphone_live),
                (Zone::MicrophoneMuted, microphone_muted),
            ])?),
            ProfileLighting::Breathe {
                color,
                seconds,
                mode,
                reverse,
            } => Ok(LightingPlan::breathe(
                color,
                BreatheDuration::from_seconds(seconds)?,
                mode.into(),
                reverse,
            )?),
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
        if let ProfileLighting::Breathe {
            seconds,
            mode,
            reverse,
            ..
        } = self.lighting
        {
            BreatheDuration::from_seconds(seconds)?;
            if reverse && mode != ProfileBreatheMode::Sweep {
                return Err(ProfileError::Protocol(ProtocolError::ReverseRequiresSweep));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    #[error("could not write the profile store atomically: {0}")]
    Write(#[source] std::io::Error),
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
    #[error("last-selected profile does not exist: {0:?}")]
    MissingSelection(String),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}

#[cfg(test)]
mod tests {
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
        let mut store = ProfileStore::default();
        store.upsert(static_profile("Everyday")).unwrap();
        store.last_selected = Some("Everyday".to_owned());
        store.apply_on_reconnect = true;

        store.save_to(&path).unwrap();
        assert_eq!(ProfileStore::load_from(&path).unwrap(), store);
        assert!(!path.with_extension("json.tmp").exists());
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
