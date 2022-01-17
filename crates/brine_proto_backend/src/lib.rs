//! Low-level client-server protocol implementation.

mod codec;
mod convert;
mod plugin;

pub use plugin::ProtocolBackendPlugin;
