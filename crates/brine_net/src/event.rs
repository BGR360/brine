//! Events exposed by this crate.

use std::{io, marker::PhantomData};

#[derive(Debug)]
pub enum NetworkEvent<Codec> {
    Connected,
    Disconnected,
    Error(NetworkError),

    #[allow(unused)]
    #[doc(hidden)]
    _Unused(PhantomData<Codec>),
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("there is already a connection established")]
    AlreadyConnected,

    #[error("failed to connect to server")]
    ConnectFailed(io::Error),

    #[error("an error occurred during transport")]
    TransportError(io::Error),
}
