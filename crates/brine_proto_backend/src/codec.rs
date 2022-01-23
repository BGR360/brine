//! Backend-independent definitions for the Minecraft protocol codec.

use std::{
    fmt,
    marker::PhantomData,
    ops::Deref,
    sync::{
        atomic::{AtomicI32, AtomicU8, Ordering},
        Arc,
    },
};

use brine_net::{DecodeResult, EncodeResult};

use crate::version::get_protocol_version;

// Possible values for the `next` field in the Handshake packet.
pub const HANDSHAKE_STATUS_NEXT: i32 = 1;
pub const HANDSHAKE_LOGIN_NEXT: i32 = 2;

/// A protocol version has to be sent in the Handshake packet, even when
/// attempting to discover the protocol version of the server. This is the value
/// the backend should send when it does that.
const DEFAULT_PROTOCOL_VERSION_STRING: &str = "1.14.4";

/// The states of the Minecraft protocol.
///
/// See <https://wiki.vg/Protocol#Definitions>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinecraftProtocolState {
    Handshaking,
    Status,
    Login,
    Play,
}

/// Thin wrapper around some concrete implementation of the Minecraft protocol.
pub struct MinecraftClientCodec<Backend> {
    /// See note in [`brine_net`] docs to see why this needs to be an Arc.
    state: Arc<CodecState>,

    _phantom: PhantomData<Backend>,
}

impl<Backend> Deref for MinecraftClientCodec<Backend> {
    type Target = CodecState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<Backend> Default for MinecraftClientCodec<Backend> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<Backend> Clone for MinecraftClientCodec<Backend> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            _phantom: PhantomData,
        }
    }
}

/// Internal state common to all Minecraft codec implementations.
pub struct CodecState {
    /// See note in [`brine_net`] docs to see why this needs to be atomic.
    protocol_state: AtomicU8,
    /// See note in [`brine_net`] docs to see why this needs to be atomic.
    protocol_version: AtomicI32,
}

impl Default for CodecState {
    fn default() -> Self {
        Self {
            protocol_state: AtomicU8::new(Self::LOGIN),
            protocol_version: AtomicI32::new(
                get_protocol_version(DEFAULT_PROTOCOL_VERSION_STRING).unwrap(),
            ),
        }
    }
}

impl CodecState {
    // Protocol states.
    const HANDSHAKING: u8 = 0;
    const STATUS: u8 = 1;
    const LOGIN: u8 = 2;
    const PLAY: u8 = 3;

    pub fn protocol_state(&self) -> MinecraftProtocolState {
        match self.protocol_state.load(Ordering::Relaxed) {
            Self::HANDSHAKING => MinecraftProtocolState::Handshaking,
            Self::STATUS => MinecraftProtocolState::Status,
            Self::LOGIN => MinecraftProtocolState::Login,
            Self::PLAY => MinecraftProtocolState::Play,
            _ => unreachable!(),
        }
    }

    pub fn set_protocol_state(&self, state: MinecraftProtocolState) {
        let as_int = match state {
            MinecraftProtocolState::Handshaking => Self::HANDSHAKING,
            MinecraftProtocolState::Status => Self::STATUS,
            MinecraftProtocolState::Login => Self::LOGIN,
            MinecraftProtocolState::Play => Self::PLAY,
        };
        self.protocol_state.store(as_int, Ordering::Relaxed);
    }

    pub fn protocol_version(&self) -> i32 {
        self.protocol_version.load(Ordering::Relaxed)
    }

    pub fn set_protocol_version(&self, protocol_version: i32) {
        self.protocol_version
            .store(protocol_version, Ordering::Relaxed)
    }
}

#[cfg(test)]
impl<Backend> MinecraftClientCodec<Backend> {
    pub(crate) fn new(state: MinecraftProtocolState) -> Self {
        let codec_state = CodecState::default();
        codec_state.set_protocol_state(state);
        Self {
            state: Arc::new(codec_state),
            _phantom: PhantomData,
        }
    }
}

pub trait IntoDecodeResult {
    type Item;
    type Error;
    fn into_decode_result(self) -> (usize, DecodeResult<Self::Item, Self::Error>);
}

pub trait IntoEncodeResult {
    type Error;
    fn into_encode_result(self, buflen: usize) -> EncodeResult<Self::Error>;
}

#[derive(Clone, PartialEq, Eq)]
pub struct UnknownPacket {
    pub packet_id: i32,
    pub body: Vec<u8>,
}

impl fmt::Debug for UnknownPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnknownPacket")
            .field("packet_id", &self.packet_id)
            .field("body", &hex_dump(&self.body))
            .finish()
    }
}

fn hex_dump(bytes: &impl AsRef<[u8]>) -> String {
    const CONFIG: pretty_hex::HexConfig = pretty_hex::HexConfig {
        // Do not print a title.
        title: false,
        // Print all bytes on one line.
        width: 0,
        // Do not group the bytes.
        group: 0,
        // Do not split bytes into chunks.
        chunk: 0,
        // Include an ASCII representation at the end.
        ascii: true,
    };
    pretty_hex::config_hex(bytes, CONFIG)
}
