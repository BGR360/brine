//! A Work-in-Progress Minecraft client written in Rust using the Bevy game engine.
//!
//! This library houses code that is common to the main Brine binary and other
//! utility binaries in `src/bin/`.

pub mod chunk;
pub mod error;
pub mod login;
pub mod server;

pub const DEFAULT_LOG_FILTER: &str = "wgpu_core=warn,naga=warn";
