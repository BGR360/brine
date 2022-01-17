use std::{
    fmt,
    io::{self, Cursor, Read},
};

use minecraft_protocol::decoder::DecoderReadExt;
pub use minecraft_protocol::error::DecodeError;

use crate::codec::{
    r#impl::{proto, MinecraftCodec},
    IntoDecodeResult, MinecraftProtocolState,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum ClientboundPacket {
    Login(proto::login::LoginClientBoundPacket),
    Play(proto::game::GameClientBoundPacket),
    Unknown(UnknownPacket),
}

impl PartialEq for ClientboundPacket {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Login(l0), Self::Login(r0)) => l0 == r0,
            (Self::Unknown(l0), Self::Unknown(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl ClientboundPacket {
    #[cfg(test)]
    pub fn get_type_id(&self) -> u8 {
        match self {
            Self::Login(p) => p.get_type_id(),
            Self::Play(p) => p.get_type_id(),
            Self::Unknown(p) => p.packet_id,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UnknownPacket {
    packet_id: u8,
    body: Vec<u8>,
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

type DecodeResult<T> = Result<T, DecodeError>;

impl MinecraftCodec {
    pub(crate) fn decode_clientbound_packet(
        &self,
        buf: impl AsRef<[u8]>,
    ) -> DecodeResult<(usize, ClientboundPacket)> {
        let buf = buf.as_ref();
        let mut cursor = Cursor::new(buf);

        let packet_length = self.decode_packet_length(&mut cursor)?;
        let length_length = cursor.position() as usize;

        let total_packet_bytes = length_length + packet_length;
        if buf.len() < total_packet_bytes {
            return Err(DecodeError::IOError {
                io_error: io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Not enough bytes in buffer",
                ),
            });
        }

        let id = self.decode_packet_id(&mut cursor)?;
        let id_length = (cursor.position() as usize) - length_length;

        let mut cursor_clone = cursor.clone();

        self.decode_packet_body(id, &mut cursor)
            .map(|packet| (total_packet_bytes, packet))
            .or_else(|err| match err {
                DecodeError::UnknownPacketType { type_id } => {
                    let body_length = packet_length - id_length;
                    let mut body_bytes = vec![0u8; body_length];
                    cursor_clone.read_exact(&mut body_bytes).unwrap();

                    let unknown_packet = ClientboundPacket::Unknown(UnknownPacket {
                        packet_id: type_id,
                        body: body_bytes,
                    });

                    Ok((total_packet_bytes, unknown_packet))
                }
                _ => Err(err),
            })
    }

    fn decode_packet_length(&self, buf: &mut impl Read) -> DecodeResult<usize> {
        self.decode_varint_into(buf)
    }

    fn decode_packet_id(&self, buf: &mut impl Read) -> DecodeResult<u8> {
        self.decode_varint_into(buf)
    }

    fn decode_varint_into<T: TryFrom<i32>>(&self, buf: &mut impl Read) -> DecodeResult<T> {
        buf.read_var_i32()?
            .try_into()
            .map_err(|_| DecodeError::IOError {
                io_error: io::Error::new(io::ErrorKind::Other, "Failed to convert integer"),
            })
    }

    fn decode_packet_body(
        &self,
        packet_id: u8,
        buf: &mut impl Read,
    ) -> DecodeResult<ClientboundPacket> {
        let packet = match self.state {
            MinecraftProtocolState::Login => ClientboundPacket::Login(
                proto::login::LoginClientBoundPacket::decode(packet_id, buf)?,
            ),
            MinecraftProtocolState::Play => {
                ClientboundPacket::Play(proto::game::GameClientBoundPacket::decode(packet_id, buf)?)
            }
            _ => return Err(DecodeError::UnknownPacketType { type_id: 0 }),
        };

        Ok(packet)
    }
}

impl<T> IntoDecodeResult for DecodeResult<(usize, T)> {
    type Item = T;
    type Error = DecodeError;

    fn into_decode_result(self) -> (usize, brine_net::DecodeResult<Self::Item, Self::Error>) {
        match self {
            Ok((length, packet)) => (length, brine_net::DecodeResult::Ok(packet)),
            Err(DecodeError::IOError { io_error })
                if io_error.kind() == io::ErrorKind::UnexpectedEof =>
            {
                (0, brine_net::DecodeResult::UnexpectedEnd)
            }
            Err(err) => (0, brine_net::DecodeResult::Err(err)),
        }
    }
}
