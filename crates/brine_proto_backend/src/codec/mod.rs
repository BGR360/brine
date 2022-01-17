//! Defining the `brine_net` codec for the Minecraft protocol.

use brine_net::{Decode, DecodeResult, Encode, EncodeResult};

mod r#impl;

use r#impl::MinecraftCodec;
pub use r#impl::{ClientboundPacket, DecodeError, EncodeError, ServerboundPacket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MinecraftProtocolState {
    Handshaking,
    _Status,
    Login,
    _Play,
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

#[derive(Clone)]
struct CodecState {
    state: MinecraftProtocolState,
}

impl Default for CodecState {
    fn default() -> Self {
        Self {
            state: MinecraftProtocolState::Login,
        }
    }
}

impl CodecState {
    pub(crate) fn state(&self) -> MinecraftProtocolState {
        self.state
    }

    pub(crate) fn set_state(&mut self, state: MinecraftProtocolState) {
        self.state = state;
    }
}

#[derive(Default, Clone)]
pub struct MinecraftClientCodec {
    state: CodecState,
}

impl MinecraftClientCodec {
    pub(crate) fn new(state: MinecraftProtocolState) -> Self {
        Self {
            state: CodecState { state },
        }
    }
}

impl Decode for MinecraftClientCodec {
    type Item = ClientboundPacket;
    type Error = DecodeError;

    fn decode(&mut self, buf: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        MinecraftCodec::new(self.state.state())
            .decode_clientbound_packet(buf as &[u8])
            .into_decode_result()
    }
}

impl Encode for MinecraftClientCodec {
    type Item = ServerboundPacket;
    type Error = EncodeError;

    fn encode(&mut self, item: &Self::Item, buf: &mut [u8]) -> EncodeResult<Self::Error> {
        MinecraftCodec::new(self.state.state())
            .encode_serverbound_packet(item, buf)
            .into_encode_result()
    }
}
