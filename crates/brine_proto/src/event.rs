//! Events exported from this crate.

#[derive(Debug)]
pub enum ServerboundEvent {
    Handshake(serverbound::Handshake),
    LoginStart(serverbound::LoginStart),
}

#[derive(Debug)]
pub enum ClientboundEvent {
    LoginSuccess(clientbound::LoginSuccess),
}

pub mod serverbound {
    #[derive(Debug)]
    pub struct Handshake;

    #[derive(Debug)]
    pub struct LoginStart;
}

pub mod clientbound {
    #[derive(Debug)]
    pub struct LoginSuccess;
}
