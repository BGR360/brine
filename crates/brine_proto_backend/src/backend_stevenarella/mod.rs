//! Implementation of the Minecraft codec using stevenarella's protocol crate as
//! the backend.

pub mod chunks;
pub mod codec;
mod login;

pub use codec::ProtocolCodec;

pub(crate) fn build(app: &mut bevy::app::App) {
    chunks::build(app);
    login::build(app);
}
