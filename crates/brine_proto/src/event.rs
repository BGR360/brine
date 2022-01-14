//! Events exported from this crate.

#[derive(Debug)]
pub enum ServerboundEvent {
    Login(serverbound::Login),
}

#[derive(Debug)]
pub enum ClientboundEvent {
    LoginSuccess(clientbound::LoginSuccess),
}

pub mod serverbound {
    #[derive(Debug)]
    pub struct Login;
}

pub mod clientbound {
    #[derive(Debug)]
    pub struct LoginSuccess;
}
