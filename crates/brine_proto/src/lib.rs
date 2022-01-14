//! High-level client-server API definition.

pub mod event;
mod plugin;

pub use event::{ClientboundEvent, ServerboundEvent};
pub use plugin::{AlwaysSuccessfulLoginPlugin, ProtocolPlugin};
