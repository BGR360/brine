//! Events exposed by this crate.

use std::{fmt::Debug, io};

use async_codec::{Decode, Encode};

#[derive(Debug)]
pub enum NetworkEvent<Codec: Decode + Encode>
where
    <Codec as Decode>::Error: Debug,
    <Codec as Encode>::Error: Debug,
{
    Connected,
    Disconnected,
    Error(NetworkError<Codec>),
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError<Codec: Decode + Encode>
where
    <Codec as Decode>::Error: Debug,
    <Codec as Encode>::Error: Debug,
{
    #[error("there is already a connection established")]
    AlreadyConnected,

    #[error("failed to connect to server: {0}")]
    ConnectFailed(io::Error),

    #[error("an error occurred during transport: {0}")]
    TransportError(io::Error),

    #[error("an error occurred while encoding a packet: {0:?}")]
    EncodeError(<Codec as Encode>::Error),

    #[error("an error occured while decoding a packet: {0:?}")]
    DecodeError(<Codec as Decode>::Error),
}
