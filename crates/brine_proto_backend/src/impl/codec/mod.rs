use bevy::log;

use brine_net::{Decode, DecodeResult, Encode, EncodeResult};

use crate::codec::{
    IntoDecodeResult, IntoEncodeResult, MinecraftClientCodec, MinecraftProtocolState,
};

mod decode;
mod encode;

pub use decode::{ClientboundPacket, DecodeError};
pub use encode::{EncodeError, ServerboundPacket};

pub(crate) use minecraft_protocol::version::v1_14_4 as proto;

#[derive(Debug)]
pub struct MinecraftCodec {
    state: MinecraftProtocolState,
}

impl MinecraftCodec {
    pub(crate) fn new(state: MinecraftProtocolState) -> Self {
        Self { state }
    }
}

impl Decode for MinecraftClientCodec {
    type Item = ClientboundPacket;
    type Error = DecodeError;

    fn decode(&mut self, buf: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        let result =
            MinecraftCodec::new(self.state.state()).decode_clientbound_packet(buf as &[u8]);

        // Advance to state Play if the packet we just decoded was LoginSuccess.
        if let Ok((
            _,
            ClientboundPacket::Login(proto::login::LoginClientBoundPacket::LoginSuccess(_)),
        )) = result
        {
            log::debug!("Codec advancing to state Play");
            self.state.set_state(MinecraftProtocolState::Play);
        }

        result.into_decode_result()
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

#[cfg(test)]
mod test {
    use std::io::Write;

    use async_codec::Framed;
    use futures::{sink::SinkExt, stream::StreamExt};
    use minecraft_protocol::encoder::EncoderWriteExt;
    use uuid::Uuid;

    use crate::codec::MinecraftClientCodec;

    use super::*;

    fn encode_packet_from_file(id: u8, body_bytes: &[u8]) -> Vec<u8> {
        let mut vec = Vec::new();
        let packet_length = 1 + body_bytes.len();
        vec.write_var_i32(packet_length as i32).unwrap();
        vec.write_var_i32(id as i32).unwrap();
        vec.write_all(body_bytes).unwrap();
        vec
    }

    async fn do_packet_encode_test(bytes: &[u8], packet: ServerboundPacket) {
        let expected = encode_packet_from_file(packet.get_type_id(), bytes);
        let mut actual = Vec::<u8>::new();

        let protocol_state = match packet {
            ServerboundPacket::Handshake(_) => MinecraftProtocolState::Handshaking,
            ServerboundPacket::Login(_) => MinecraftProtocolState::Login,
        };
        let codec = MinecraftClientCodec::new(protocol_state);
        let mut framed = Framed::new(&mut actual, codec);

        framed.send(packet).await.unwrap();
        assert_eq!(actual, expected);
    }

    async fn do_packet_decode_test(bytes: &[u8], expected: ClientboundPacket) {
        let reader = encode_packet_from_file(expected.get_type_id(), bytes);

        let protocol_state = match expected {
            ClientboundPacket::Login(_) => MinecraftProtocolState::Login,
            ClientboundPacket::Play(_) => MinecraftProtocolState::Play,
            ClientboundPacket::Unknown(_) => panic!("not allowed"),
        };
        let codec = MinecraftClientCodec::new(protocol_state);
        let mut framed = Framed::new(&reader[..], codec);

        let actual = framed.next().await.unwrap().unwrap();
        assert_eq!(actual, expected);
    }

    #[async_std::test]
    async fn test_login_start() {
        do_packet_encode_test(
            include_bytes!(
                "../../../test/minecraft-protocol/protocol/test/packet/login/login_start.dat"
            ),
            ServerboundPacket::Login(proto::login::LoginServerBoundPacket::LoginStart(
                proto::login::LoginStart {
                    name: String::from("Username"),
                },
            )),
        )
        .await;
    }

    #[async_std::test]
    async fn test_login_success() {
        do_packet_decode_test(
            include_bytes!(
                "../../../test/minecraft-protocol/protocol/test/packet/login/login_success.dat"
            ),
            ClientboundPacket::Login(proto::login::LoginClientBoundPacket::LoginSuccess(
                proto::login::LoginSuccess {
                    uuid: Uuid::parse_str("35ee313b-d89a-41b8-b25e-d32e8aff0389").unwrap(),
                    username: String::from("Username"),
                },
            )),
        )
        .await
    }
}
