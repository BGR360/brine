//! High-level client-server API definition.

pub mod event;
mod plugin;

pub use plugin::{AlwaysSuccessfulLoginPlugin, ProtocolPlugin};
