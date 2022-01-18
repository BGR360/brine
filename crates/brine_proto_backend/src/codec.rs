//! Backend-independent definitions for the Minecraft protocol codec.

use std::{
    fmt,
    marker::PhantomData,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

use brine_net::{DecodeResult, EncodeResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MinecraftProtocolState {
    Handshaking,
    Status,
    Login,
    Play,
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

pub(crate) trait IntoDecodeResult {
    type Item;
    type Error;
    fn into_decode_result(self) -> (usize, DecodeResult<Self::Item, Self::Error>);
}

pub(crate) trait IntoEncodeResult {
    type Error;
    fn into_encode_result(self) -> EncodeResult<Self::Error>;
}

const HANDSHAKING: u8 = 0;
const STATUS: u8 = 1;
const LOGIN: u8 = 2;
const PLAY: u8 = 3;

pub(crate) struct CodecState {
    /// See note in [`brine_net`] docs to see why this needs to be atomic.
    state: AtomicU8,
}

impl Default for CodecState {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(LOGIN),
        }
    }
}

impl CodecState {
    pub(crate) fn state(&self) -> MinecraftProtocolState {
        match self.state.load(Ordering::Relaxed) {
            HANDSHAKING => MinecraftProtocolState::Handshaking,
            STATUS => MinecraftProtocolState::Status,
            LOGIN => MinecraftProtocolState::Login,
            PLAY => MinecraftProtocolState::Play,
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_state(&self, state: MinecraftProtocolState) {
        let as_int = match state {
            MinecraftProtocolState::Handshaking => HANDSHAKING,
            MinecraftProtocolState::Status => STATUS,
            MinecraftProtocolState::Login => LOGIN,
            MinecraftProtocolState::Play => PLAY,
        };
        self.state.store(as_int, Ordering::Relaxed);
    }
}

pub struct MinecraftClientCodec<Backend> {
    /// See note in [`brine_net`] docs to see why this needs to be an Arc.
    pub(crate) state: Arc<CodecState>,

    _phantom: PhantomData<Backend>,
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

impl<Backend> MinecraftClientCodec<Backend> {
    #[cfg(test)]
    pub(crate) fn new(state: MinecraftProtocolState) -> Self {
        let codec_state = CodecState::default();
        codec_state.set_state(state);
        Self {
            state: Arc::new(codec_state),
            _phantom: PhantomData,
        }
    }
}
