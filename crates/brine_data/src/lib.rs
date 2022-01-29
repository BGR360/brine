#![doc = include_str!("../README.md")]

pub(crate) use minecraft_data_rs::api::Api;

pub use minecraft_data_rs::models::version::Version;

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
    /// Constructs Minecraft data for the specified [`Version`].
    pub fn for_version(version: impl Into<Version>) -> Self {
        let api = Api::new(version.into());
        Self {
            blocks: Blocks::from_api(&api),
        }
    }
}
