use std::sync::Arc;

use crate::{Api, Blocks, Version};

/// Provides access to all Minecraft data for a specific version.
///
/// This type is intended to be initialized once at program startup and accessed
/// by reference thereafter. Construction is **not** an inexpensive operation,
/// but access **is** an inexpensive operation.
#[derive(Clone)]
pub struct MinecraftData {
    inner: Arc<MinecraftDataInner>,
}

impl MinecraftData {
    /// Constructs Minecraft data for the latest stable version supported by
    /// this crate.
    pub fn latest_stable() -> Self {
        Self::for_version(Version::latest_stable())
    }

    /// Constructs Minecraft data for the specified [`Version`].
    pub fn for_version(version: impl Into<Version>) -> Self {
        let version = version.into();
        let api = Api::new(version.0.clone());
        Self {
            inner: Arc::new(MinecraftDataInner {
                blocks: Blocks::from_api(&api),
                version,
            }),
        }
    }

    pub fn blocks(&self) -> &Blocks {
        &self.inner.blocks
    }

    pub fn version(&self) -> &Version {
        &self.inner.version
    }
}

struct MinecraftDataInner {
    pub blocks: Blocks,
    pub version: Version,
}
