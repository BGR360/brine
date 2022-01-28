//! Loading chunk data for testing.

use std::{fs, io, path::Path};

use brine_chunk::{decode::Error as ChunkError, Chunk};
use brine_proto_backend::{
    backend_stevenarella::{
        chunks::get_chunk_from_packet,
        codec::{Direction, Error as PacketError, MinecraftCodec},
    },
    codec::MinecraftProtocolState,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the loaded file does not contain valid chunk data")]
    NotAChunk,

    #[error(transparent)]
    Packet(#[from] PacketError),

    #[error(transparent)]
    Chunk(#[from] ChunkError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Loads chunk data from a file.
pub fn load_chunk(path: impl AsRef<Path>) -> Result<Chunk> {
    let file_bytes = fs::read(path.as_ref())?;
    decode_chunk(&file_bytes)
}

/// Decodes chunk data from a reader.
pub fn decode_chunk(reader: &[u8]) -> Result<Chunk> {
    const CHUNK_DATA_PACKET_ID: i32 = 0x21;

    let packet = MinecraftCodec::decode_packet_with_id(
        498,
        MinecraftProtocolState::Play,
        Direction::Clientbound,
        CHUNK_DATA_PACKET_ID,
        reader,
    )?;

    let chunk = get_chunk_from_packet(&packet)?;

    match chunk {
        Some(chunk) => Ok(chunk),
        None => Err(Error::NotAChunk),
    }
}
