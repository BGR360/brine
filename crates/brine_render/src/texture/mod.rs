mod atlas;
mod manager;
mod plugin;

pub use atlas::TextureAtlas;
pub use manager::TextureManager;
pub use plugin::TextureManagerPlugin;

pub(crate) use atlas::PendingAtlas;
