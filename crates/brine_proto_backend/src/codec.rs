//! Defining the `brine_net` codec for the Minecraft protocol.

use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use brine_net::{DecodeResult, EncodeResult};

pub(crate) use crate::r#impl::codec::proto;
pub use crate::r#impl::codec::{ClientboundPacket, DecodeError, EncodeError, ServerboundPacket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MinecraftProtocolState {
    Handshaking,
    Status,
    Login,
    Play,
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

#[derive(Default, Clone)]
pub struct MinecraftClientCodec {
    /// See note in [`brine_net`] docs to see why this needs to be an Arc.
    pub(crate) state: Arc<CodecState>,
}

impl MinecraftClientCodec {
    #[cfg(test)]
    pub(crate) fn new(state: MinecraftProtocolState) -> Self {
        let codec_state = CodecState::default();
        codec_state.set_state(state);
        Self {
            state: Arc::new(codec_state),
        }
    }
}
