use std::io::{self, Cursor, Write};

use minecraft_protocol::encoder::{Encoder, EncoderWriteExt};
pub use minecraft_protocol::error::EncodeError;

use crate::codec::IntoEncodeResult;

use super::{proto, MinecraftCodec};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerboundPacket {
    Handshake(proto::handshake::HandshakeServerBoundPacket),
    Login(proto::login::LoginServerBoundPacket),
}

impl ServerboundPacket {
    #[cfg(test)]
    pub fn get_type_id(&self) -> u8 {
        match self {
            Self::Handshake(p) => p.get_type_id(),
            Self::Login(p) => p.get_type_id(),
        }
    }
}

type EncodeResult<T> = Result<T, EncodeError>;

macro_rules! encode_packet_body {
    (
        $packet_var:expr,
        $writer_var:expr,
        {
            $(
            $state_variant:path => {
                $(
                $packet_variant:path
                ),*
            }
            )*
        }
    ) => {
        match $packet_var {
            $(
            $state_variant (p) => {
                match p {
                    $(
                    $packet_variant (p) => {
                        p.encode($writer_var)?;
                    }
                    )*
                }
            }
            )*
        }
    };
}

impl MinecraftCodec {
    pub(crate) fn encode_serverbound_packet(
        &self,
        packet: &ServerboundPacket,
        mut buf: impl AsMut<[u8]>,
    ) -> EncodeResult<usize> {
        // First, encode the packet id and body into a separate buffer so we can
        // find out its length.
        let mut packet_id_and_body = Vec::<u8>::new();
        self.encode_serverbound_packet_id_and_body(packet, &mut packet_id_and_body)?;

        // Then, write the payload length followed by the payload.
        let mut buf = Cursor::new(buf.as_mut());
        buf.write_var_i32(packet_id_and_body.len() as i32)?;
        buf.write_all(&packet_id_and_body)?;

        let bytes_written = buf.position() as usize;
        Ok(bytes_written)
    }

    fn encode_serverbound_packet_id_and_body(
        &self,
        packet: &ServerboundPacket,
        buf: &mut impl Write,
    ) -> EncodeResult<()> {
        let packet_id = match packet {
            ServerboundPacket::Handshake(p) => p.get_type_id(),
            ServerboundPacket::Login(p) => p.get_type_id(),
        };
        buf.write_var_i32(packet_id as i32)?;

        use proto::handshake::HandshakeServerBoundPacket;
        use proto::login::LoginServerBoundPacket;

        encode_packet_body!(packet, buf, {
            ServerboundPacket::Handshake => {
                HandshakeServerBoundPacket::Handshake
            }
            ServerboundPacket::Login => {
                LoginServerBoundPacket::LoginStart,
                LoginServerBoundPacket::EncryptionResponse,
                LoginServerBoundPacket::LoginPluginResponse
            }
        });

        Ok(())
    }
}

impl IntoEncodeResult for EncodeResult<usize> {
    type Error = EncodeError;

    fn into_encode_result(self) -> brine_net::EncodeResult<Self::Error> {
        match self {
            Ok(length) => brine_net::EncodeResult::Ok(length),
            Err(EncodeError::IOError { io_error })
                if io_error.kind() == io::ErrorKind::UnexpectedEof =>
            {
                brine_net::EncodeResult::Overflow(0)
            }
            Err(err) => brine_net::EncodeResult::Err(err),
        }
    }
}
