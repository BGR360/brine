//! Low-level client-server protocol implementation.

mod codec;
mod plugin;

mod backend_minecraft_protocol;

pub(crate) use backend_minecraft_protocol as backend;

pub use plugin::ProtocolBackendPlugin;
