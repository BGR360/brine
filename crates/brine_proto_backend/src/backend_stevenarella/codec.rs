use std::io::{self, Cursor, Read, Write};

use bevy::log;
use steven_protocol::protocol::{self, Direction, PacketType, Serializable, State, VarInt};
pub(crate) use steven_protocol::protocol::{packet, Error};

use brine_net::{Decode, DecodeResult, Encode, EncodeResult};

use crate::{
    codec::{
        IntoDecodeResult, IntoEncodeResult, MinecraftClientCodec, MinecraftProtocolState,
        UnknownPacket,
    },
    version::get_protocol_version,
};

pub(crate) const VERSION: &str = "1.14.4";

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Known(packet::Packet),
    Unknown(UnknownPacket),
}

impl From<packet::Packet> for Packet {
    fn from(packet: packet::Packet) -> Self {
        Self::Known(packet)
    }
}

impl From<MinecraftProtocolState> for State {
    fn from(state: MinecraftProtocolState) -> Self {
        match state {
            MinecraftProtocolState::Handshaking => State::Handshaking,
            MinecraftProtocolState::Status => State::Status,
            MinecraftProtocolState::Login => State::Login,
            MinecraftProtocolState::Play => State::Play,
        }
    }
}

impl From<State> for MinecraftProtocolState {
    fn from(state: State) -> Self {
        match state {
            State::Handshaking => MinecraftProtocolState::Handshaking,
            State::Status => MinecraftProtocolState::Status,
            State::Login => MinecraftProtocolState::Login,
            State::Play => MinecraftProtocolState::Play,
        }
    }
}

#[derive(Debug)]
pub struct MinecraftCodec {
    state: MinecraftProtocolState,
    protocol_version: i32,
    direction: Direction,
}

pub type ProtocolCodec = MinecraftClientCodec<MinecraftCodec>;

impl MinecraftCodec {
    pub(crate) fn new(
        state: MinecraftProtocolState,
        protocol_version: i32,
        direction: Direction,
    ) -> Self {
        protocol::set_current_protocol_version(protocol_version);
        Self {
            state,
            protocol_version,
            direction,
        }
    }

    fn decode_packet(&self, buf: impl AsRef<[u8]>) -> Result<(usize, Packet), Error> {
        let buf = buf.as_ref();
        let mut cursor = Cursor::new(buf);

        let length = VarInt::read_from(&mut cursor)?.0 as usize;
        let length_length = cursor.position() as usize;

        let total_bytes_needed = length_length + length;
        if buf.len() < total_bytes_needed {
            return Err(Error::IOError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes in buffer",
            )));
        }

        let mut cursor = Cursor::new(&buf[length_length..total_bytes_needed]);

        let id = VarInt::read_from(&mut cursor)?.0;

        let mut cursor_clone = cursor.clone();

        let packet = packet::packet_by_id(
            self.protocol_version,
            self.state.into(),
            self.direction,
            id,
            &mut cursor,
        )
        .map(|maybe_packet| match maybe_packet {
            Some(packet) => Packet::Known(packet),
            None => {
                let mut body = Vec::new();
                cursor_clone.read_to_end(&mut body).unwrap();
                Packet::Unknown(UnknownPacket {
                    packet_id: id,
                    body,
                })
            }
        })?;

        assert_eq!(cursor.position() as usize, length);
        Ok((total_bytes_needed, packet))
    }

    fn encode_packet(&self, packet: &Packet, mut buf: impl AsMut<[u8]>) -> Result<usize, Error> {
        match packet {
            Packet::Known(packet) => {
                let buf = buf.as_mut();

                let payload = self.encode_packet_id_and_body(packet)?;
                let length = payload.len();

                let mut cursor = Cursor::new(buf);

                VarInt(length as i32).write_to(&mut cursor)?;
                let length_length = cursor.position() as usize;

                let total_bytes_needed = length_length + length;
                if cursor.get_ref().len() < total_bytes_needed {
                    return Err(Error::IOError(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Not enough bytes in buffer",
                    )));
                }

                cursor.write_all(&payload[..])?;

                assert_eq!(cursor.position() as usize, total_bytes_needed);
                Ok(total_bytes_needed)
            }
            Packet::Unknown(packet) => Err(Error::Err(format!(
                "Attempted to encode unknown packet: {:?}",
                packet
            ))),
        }
    }

    fn encode_packet_id_and_body(&self, packet: &packet::Packet) -> Result<Vec<u8>, Error> {
        let mut packet_id_and_body = Vec::new();

        let id = VarInt(packet.packet_id(self.protocol_version));
        id.write_to(&mut packet_id_and_body)?;

        packet.write(&mut packet_id_and_body)?;

        Ok(packet_id_and_body)
    }
}

impl<T> IntoDecodeResult for Result<(usize, T), Error> {
    type Item = T;
    type Error = Error;

    fn into_decode_result(self) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        match self {
            Ok((length, item)) => (length, DecodeResult::Ok(item)),
            Err(Error::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => {
                (0, DecodeResult::UnexpectedEnd)
            }
            Err(err) => (0, DecodeResult::Err(err)),
        }
    }
}

impl IntoEncodeResult for Result<usize, Error> {
    type Error = Error;

    fn into_encode_result(self) -> EncodeResult<Self::Error> {
        match self {
            Ok(length) => EncodeResult::Ok(length),
            Err(Error::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => {
                EncodeResult::Overflow(0)
            }
            Err(err) => EncodeResult::Err(err),
        }
    }
}

impl Decode for MinecraftClientCodec<MinecraftCodec> {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, buf: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        let result = MinecraftCodec::new(
            self.state.state(),
            get_protocol_version(VERSION).unwrap(),
            Direction::Clientbound,
        )
        .decode_packet(buf);

        // Advance to state Play if the packet we just decoded was LoginSuccess.
        if let Ok((
            _,
            Packet::Known(
                packet::Packet::LoginSuccess_String(_) | packet::Packet::LoginSuccess_UUID(_),
            ),
        )) = result
        {
            log::debug!("Codec advancing to state Play");
            self.state.set_state(MinecraftProtocolState::Play);
        }

        result.into_decode_result()
    }
}

impl Encode for MinecraftClientCodec<MinecraftCodec> {
    type Item = Packet;
    type Error = Error;

    fn encode(&mut self, item: &Self::Item, buf: &mut [u8]) -> EncodeResult<Self::Error> {
        MinecraftCodec::new(
            self.state.state(),
            get_protocol_version(VERSION).unwrap(),
            Direction::Serverbound,
        )
        .encode_packet(item, buf)
        .into_encode_result()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Write;

    use async_codec::Framed;
    use futures::{sink::SinkExt, stream::StreamExt};

    use crate::codec::MinecraftClientCodec;

    const PROTOCOL_VERSION: i32 = 498;

    fn encode_packet_from_file(id: u8, body_bytes: &[u8]) -> Vec<u8> {
        let mut vec = Vec::new();
        let packet_length = 1 + body_bytes.len();
        VarInt(packet_length as i32).write_to(&mut vec).unwrap();
        VarInt(id as i32).write_to(&mut vec).unwrap();
        vec.write_all(body_bytes).unwrap();
        vec
    }

    async fn do_packet_encode_test(
        codec: MinecraftClientCodec<MinecraftCodec>,
        packet: packet::Packet,
        bytes: &[u8],
    ) {
        let expected = encode_packet_from_file(packet.packet_id(PROTOCOL_VERSION) as u8, bytes);
        let mut actual = Vec::<u8>::new();

        let mut framed = Framed::new(&mut actual, codec);

        framed.send(Packet::from(packet)).await.unwrap();
        assert_eq!(actual, expected);
    }

    async fn do_packet_decode_test(
        codec: MinecraftClientCodec<MinecraftCodec>,
        expected: packet::Packet,
        bytes: &[u8],
    ) {
        let reader = encode_packet_from_file(expected.packet_id(498) as u8, bytes);

        let mut framed = Framed::new(&reader[..], codec);

        let actual = framed.next().await.unwrap().unwrap();
        assert_eq!(actual, Packet::from(expected));
    }

    #[async_std::test]
    async fn test_login_start() {
        do_packet_encode_test(
            MinecraftClientCodec::new(MinecraftProtocolState::Login),
            packet::Packet::LoginStart(Box::new(packet::login::serverbound::LoginStart {
                username: String::from("Username"),
            })),
            include_bytes!("../../test/packet-data/login/login_start.dat"),
        )
        .await;
    }

    #[async_std::test]
    async fn test_login_success() {
        do_packet_decode_test(
            MinecraftClientCodec::new(MinecraftProtocolState::Login),
            packet::Packet::LoginSuccess_String(Box::new(
                packet::login::clientbound::LoginSuccess_String {
                    uuid: String::from("35ee313b-d89a-41b8-b25e-d32e8aff0389"),
                    username: String::from("Username"),
                },
            )),
            include_bytes!("../../test/packet-data/login/login_success.dat"),
        )
        .await
    }

    #[test]
    fn packet_size() {
        assert_eq!(std::mem::size_of::<packet::Packet>(), 16);
    }

    #[test]
    fn metadata_size() {
        assert_eq!(std::mem::size_of::<steven_protocol::types::Metadata>(), 48);
    }
}
