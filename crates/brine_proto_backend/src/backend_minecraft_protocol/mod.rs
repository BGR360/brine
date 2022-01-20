pub(crate) mod codec;
pub(crate) mod login;

pub(crate) use codec::ProtocolCodec;

pub(crate) fn build(app: &mut bevy_app::App) {
    login::build(app);
}
