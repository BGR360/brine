//! Low-level client-server protocol implementation.

mod codec;
mod plugin;

mod r#impl;

pub use plugin::ProtocolBackendPlugin;
