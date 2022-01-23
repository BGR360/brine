//! Low-level client-server protocol implementation.

pub mod codec;
mod plugin;
pub mod version;

pub mod backend_stevenarella;

pub(crate) use backend_stevenarella as backend;

pub use plugin::ProtocolBackendPlugin;
