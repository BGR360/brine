//! Low-level client-server protocol implementation.

mod codec;
mod plugin;
mod version;

#[cfg(any(test, feature = "minecraft_proto"))]
mod backend_minecraft_protocol;
#[cfg(feature = "minecraft_proto")]
pub(crate) use backend_minecraft_protocol as backend;

#[cfg(any(test, feature = "steven_proto"))]
mod backend_stevenarella;
#[cfg(feature = "steven_proto")]
pub(crate) use backend_stevenarella as backend;

pub use plugin::ProtocolBackendPlugin;
