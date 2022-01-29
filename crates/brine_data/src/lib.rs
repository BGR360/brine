#![doc = include_str!("../README.md")]

use std::ops::Deref;

pub(crate) use minecraft_data_rs::api::Api;

use minecraft_data_rs::{
    api::versions::{latest_stable, versions_by_minecraft_version},
    models::version::Version as McVersion,
};

pub mod block;

use block::Blocks;

/// Provides access to all Minecraft data for a specific version.
///
/// This type is intended to be initialized once at program startup and accessed
/// by reference thereafter. Construction is **not** an inexpensive operation,
/// but access **is** an inexpensive operation.
pub struct MinecraftData {
    pub blocks: Blocks,
}

impl MinecraftData {
    /// Constructs Minecraft data for the latest stable version.
    pub fn latest_stable() -> Self {
        Self::for_version(Version::latest_stable())
    }

    /// Constructs Minecraft data for the specified [`Version`].
    pub fn for_version(version: impl Into<Version>) -> Self {
        let version = version.into();
        let api = Api::new(version.0);
        Self {
            blocks: Blocks::from_api(&api),
        }
    }
}

/// Represents a version of the Minecraft game.
pub struct Version(McVersion);

impl Version {
    /// Returns the latest stable version supported by this crate.
    pub fn latest_stable() -> Self {
        Self(latest_stable().unwrap())
    }
}

impl<S: Into<String>> From<S> for Version {
    fn from(source: S) -> Self {
        Self(
            versions_by_minecraft_version()
                .unwrap()
                .get(&source.into())
                .unwrap()
                .clone(),
        )
    }
}

impl Deref for Version {
    type Target = McVersion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
