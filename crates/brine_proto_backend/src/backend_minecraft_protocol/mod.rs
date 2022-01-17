pub(crate) mod codec;
pub(crate) mod convert;
pub(crate) mod login;

pub(crate) use codec::ProtocolCodec;

pub(crate) fn build(app: &mut bevy::app::App) {
    login::build(app);
}
