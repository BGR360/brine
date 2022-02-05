use std::ops::Deref;

use minecraft_data_rs::{
    api::versions::{latest_stable, versions_by_minecraft_version},
    models::version::Version as McVersion,
};

/// Represents a version of the Minecraft game.
pub struct Version(pub(crate) McVersion);

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
