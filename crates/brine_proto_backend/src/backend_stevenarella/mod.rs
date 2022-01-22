//! Implementation of the Minecraft codec using stevenarella's protocol crate as
//! the backend.

mod chunks;
mod codec;
mod login;

pub(crate) use codec::ProtocolCodec;

pub(crate) fn build(app: &mut bevy_app::App) {
    chunks::build(app);
    login::build(app);
}
