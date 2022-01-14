//! Plugins exported from this crate.

mod protocol;
mod successful_login;

pub use protocol::ProtocolPlugin;
pub use successful_login::AlwaysSuccessfulLoginPlugin;
