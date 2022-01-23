use std::{
    io::{self, Cursor, Write},
    ops::Deref,
};

use bevy_log as log;
use steven_protocol::protocol::{self, State, VarInt};
pub use steven_protocol::protocol::{packet, Direction, Error, PacketType, Serializable};

use brine_net::{Decode, DecodeResult, Encode, EncodeResult};

use crate::codec::{
    IntoDecodeResult, IntoEncodeResult, MinecraftClientCodec, MinecraftProtocolState,
    UnknownPacket, HANDSHAKE_LOGIN_NEXT, HANDSHAKE_STATUS_NEXT,
};

/// Packet representation used by this implementation of the protocol codec.
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

/// Implementation of the Minecraft protocol using the [`steven_protocol`] crate.
///
/// [`steven_protocol`]: <https://github.com/iceiix/stevenarella/tree/master/protocol>
#[derive(Debug)]
pub struct MinecraftCodec;

pub type ProtocolCodec = MinecraftClientCodec<MinecraftCodec>;

impl MinecraftCodec {
    pub fn decode_packet(
        protocol_version: i32,
        protocol_state: MinecraftProtocolState,
        direction: Direction,
        buf: impl AsRef<[u8]>,
    ) -> Result<(usize, Packet), Error> {
        let buf = buf.as_ref();

        // Use a cursor so we can track how many bytes we've read
        // (VarInts have variable length).
        let mut cursor = Cursor::new(buf);

        // First field is the packet length in bytes. Note that this number does
        // **not** include the bytes used for the length field.
        let length = VarInt::read_from(&mut cursor)?.0 as usize;
        // Take note of how many bytes the `length` field took up.
        let length_length = cursor.position() as usize;

        // Ensure that there's enough data in the buffer to read the rest of the packet.
        let total_packet_bytes = length_length + length;
        if buf.len() < total_packet_bytes {
            return Err(Error::IOError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes in buffer",
            )));
        }

        // Next field is the packet id.
        let id = VarInt::read_from(&mut cursor)?.0;
        // Take note of how many bytes the `id` field took up.
        let id_length = cursor.position() as usize - length_length;

        // The rest of the packet is the actual packet data.
        let data_start = cursor.position() as usize;
        let data_length = length - id_length;
        let data_slice = &buf[data_start..data_start + data_length];

        let packet = Self::decode_packet_with_id(
            protocol_version,
            protocol_state,
            direction,
            id,
            data_slice,
        )?;

        Ok((total_packet_bytes, packet))
    }

    /// Decodes packet contents from a byte slice. Byte slice must be exactly
    /// the right size.
    pub fn decode_packet_with_id(
        protocol_version: i32,
        protocol_state: MinecraftProtocolState,
        direction: Direction,
        packet_id: i32,
        buf: impl AsRef<[u8]>,
    ) -> Result<Packet, Error> {
        let buf = buf.as_ref();

        let mut cursor = Cursor::new(buf);

        let packet = packet::packet_by_id(
            protocol_version,
            protocol_state.into(),
            direction,
            packet_id,
            &mut cursor,
        )
        .map(|maybe_packet| match maybe_packet {
            Some(packet) => Packet::Known(packet),
            None => Packet::Unknown(UnknownPacket {
                packet_id,
                body: Vec::from(buf),
            }),
        })?;

        // All of the data should have been read.
        assert_eq!(cursor.position() as usize, buf.len());

        Ok(packet)
    }

    pub fn encode_packet(
        protocol_version: i32,
        packet: &Packet,
        mut buf: impl AsMut<[u8]>,
    ) -> Result<usize, Error> {
        match packet {
            Packet::Known(packet) => {
                let mut cursor = Cursor::new(buf.as_mut());

                let mut id_and_data = Vec::new();
                Self::encode_packet_id_and_data(protocol_version, packet, &mut id_and_data)?;
                let length = id_and_data.len();

                VarInt(length as i32).write_to(&mut cursor)?;
                let length_length = cursor.position() as usize;

                let total_packet_bytes = length_length + length;
                if cursor.get_ref().len() < total_packet_bytes {
                    return Err(Error::IOError(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Not enough bytes in buffer",
                    )));
                }

                cursor.write_all(&id_and_data[..])?;

                assert_eq!(cursor.position() as usize, total_packet_bytes);

                Ok(total_packet_bytes)
            }
            Packet::Unknown(packet) => Err(Error::Err(format!(
                "Attempted to encode unknown packet: {:?}",
                packet
            ))),
        }
    }

    pub fn encode_packet_id_and_data(
        protocol_version: i32,
        packet: &packet::Packet,
        buf: &mut impl Write,
    ) -> Result<(), Error> {
        let id = VarInt(packet.packet_id(protocol_version));
        id.write_to(buf)?;

        Self::encode_packet_data(packet, buf)
    }

    pub fn encode_packet_data(packet: &packet::Packet, buf: &mut impl Write) -> Result<(), Error> {
        packet.write(buf)
    }

    /// Extracts the server's protocol version from a StatusResponse packet.
    /// See https://wiki.vg/Server_List_Ping#Response
    pub fn get_server_protocol_version(
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

    fn into_encode_result(self, buflen: usize) -> EncodeResult<Self::Error> {
        match self {
            Ok(length) => EncodeResult::Ok(length),
            Err(Error::IOError(io_error)) if io_error.kind() == io::ErrorKind::UnexpectedEof => {
                EncodeResult::Overflow(buflen * 2)
            }
            Err(err) => EncodeResult::Err(err),
        }
    }
}

impl MinecraftClientCodec<MinecraftCodec> {
    pub fn set_protocol_version(&self, protocol_version: i32) {
        log::debug!("Setting codec protocol version to {}", protocol_version);
        protocol::set_current_protocol_version(protocol_version);
        self.deref().set_protocol_version(protocol_version);
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
                    self.set_protocol_state(next_state);
                }
            }

            // On a StatusResponse packet, set the protocol version to that of
            // the server.
            Packet::Known(packet::Packet::StatusResponse(status_response)) => {
                let protocol_version =
                    match MinecraftCodec::get_server_protocol_version(&*status_response) {
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
                self.set_protocol_state(MinecraftProtocolState::Play);
            }

            _ => {}
        }
    }
}

impl Decode for MinecraftClientCodec<MinecraftCodec> {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, buf: &mut [u8]) -> (usize, DecodeResult<Packet, Error>) {
        let result = MinecraftCodec::decode_packet(
            self.protocol_version(),
            self.protocol_state(),
            Direction::Clientbound,
            buf,
        );

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

        let len = buf.len();

        MinecraftCodec::encode_packet(self.protocol_version(), packet, buf).into_encode_result(len)
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
