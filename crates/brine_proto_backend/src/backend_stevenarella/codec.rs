use std::io::{self, Cursor, Read, Write};

use bevy::log;
use steven_protocol::protocol::{self, Direction, PacketType, Serializable, State, VarInt};
pub(crate) use steven_protocol::protocol::{packet, Error};

use brine_net::{Decode, DecodeResult, Encode, EncodeResult};

use crate::codec::{
    IntoDecodeResult, IntoEncodeResult, MinecraftClientCodec, MinecraftProtocolState,
    UnknownPacket, HANDSHAKE_LOGIN_NEXT, HANDSHAKE_STATUS_NEXT,
};

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

impl MinecraftClientCodec<MinecraftCodec> {
    pub fn protocol_version(&self) -> i32 {
        self.state.protocol_version()
    }

    pub fn set_protocol_version(&self, protocol_version: i32) {
        log::debug!("Setting codec protocol version to {}", protocol_version);
        protocol::set_current_protocol_version(protocol_version);
        self.state.set_protocol_version(protocol_version);
    }

    /// Makes any necessary adjustments to the codec state in response to
    /// certain outbound or inbound packets.
    fn react_to_packet(&self, packet: &Packet) {
        match packet {
            // On a Handshake packet, set the protocol state to whatever is
            // specified by the `next` field.
            Packet::Known(packet::Packet::Handshake(handshake)) => {
                if let Some(next_state) = match handshake.next.0 {
                    HANDSHAKE_STATUS_NEXT => Some(MinecraftProtocolState::Status),
                    HANDSHAKE_LOGIN_NEXT => Some(MinecraftProtocolState::Login),
                    i => {
                        log::error!("Invalid next state in Handshake packet: {}", i);
                        None
                    }
                } {
                    log::debug!("Codec advancing to state {:?}", next_state);
                    self.state.set_state(next_state);
                }
            }

            // On a StatusResponse packet, set the protocol version to that of
            // the server.
            Packet::Known(packet::Packet::StatusResponse(status_response)) => {
                let protocol_version = match self.get_server_protocol_version(&*status_response) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                };

                self.set_protocol_version(protocol_version);
            }

            // On a LoginSuccess packet, advance to state Play.
            Packet::Known(
                packet::Packet::LoginSuccess_String(_) | packet::Packet::LoginSuccess_UUID(_),
            ) => {
                log::debug!("Codec advancing to state Play");
                self.state.set_state(MinecraftProtocolState::Play);
            }

            _ => {}
        }
    }

    /// Extracts the server's protocol version from a StatusResponse packet.
    /// See https://wiki.vg/Server_List_Ping#Response
    fn get_server_protocol_version(
        &self,
        status_response: &packet::status::clientbound::StatusResponse,
    ) -> Result<i32, String> {
        use serde_json::Value;
        let status: Value =
            serde_json::from_str(&status_response.status).map_err(|e| e.to_string())?;

        let invalid_status =
            || format!("Malformed StatusResponse json: {}", &status_response.status);

        let version = status.get("version").ok_or_else(invalid_status)?;
        let protocol_version = version
            .get("protocol")
            .and_then(Value::as_i64)
            .ok_or_else(invalid_status)?;

        Ok(protocol_version as i32)
    }
}

impl Decode for MinecraftClientCodec<MinecraftCodec> {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, buf: &mut [u8]) -> (usize, DecodeResult<Packet, Error>) {
        let result = MinecraftCodec::new(
            self.state.state(),
            self.state.protocol_version(),
            Direction::Clientbound,
        )
        .decode_packet(buf);

        if let Ok((_, ref packet)) = result {
            self.react_to_packet(packet);
        }

        result.into_decode_result()
    }
}

impl Encode for MinecraftClientCodec<MinecraftCodec> {
    type Item = Packet;
    type Error = Error;

    fn encode(&mut self, packet: &Packet, buf: &mut [u8]) -> EncodeResult<Error> {
        self.react_to_packet(packet);

        MinecraftCodec::new(
            self.state.state(),
            self.state.protocol_version(),
            Direction::Serverbound,
        )
        .encode_packet(packet, buf)
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
