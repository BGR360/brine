//! Implementation of the Minecraft codec using stevenarella's protocol crate as
//! the backend.

mod codec;
mod login;

pub(crate) use codec::ProtocolCodec;

pub(crate) fn build(app: &mut bevy::app::App) {
    login::build(app);
}
