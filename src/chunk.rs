//! Saving and loading chunk data for testing.
//!
//! File format based on
//! <https://github.com/PrismarineJS/prismarine-chunk/tree/master/test>, i.e.
//! binary blob stored in `{file}.dump` and extra information stored as JSON in
//! `{file}.meta`.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use brine_chunk::{decode::Error as ChunkError, Chunk};
use brine_proto_backend::backend_stevenarella::{chunks::ChunkData, codec::Packet};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    //#[error(transparent)]
    //Packet(#[from] PacketError),
    #[error(transparent)]
    Chunk(#[from] ChunkError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// JSON data stored in the `{file}.meta` file.
#[derive(Deserialize, Serialize)]
pub struct ChunkMeta {
    #[serde(rename = "x")]
    pub chunk_x: i32,
    #[serde(rename = "z")]
    pub chunk_z: i32,
    #[serde(rename = "bitMap")]
    pub bitmask: u16,
}

/// Loads **undecoded** chunk data from a pair of `.dump` and `.meta` files.
pub fn load_chunk_data(path: impl AsRef<Path>) -> Result<ChunkData<Vec<u8>>> {
    let path = path.as_ref();
    let dump_path = path.with_extension("dump");
    let meta_path = path.with_extension("meta");

    let ChunkMeta {
        chunk_x,
        chunk_z,
        bitmask,
    } = serde_json::from_reader(fs::File::open(meta_path)?)?;

    let data = fs::read(dump_path)?;

    Ok(ChunkData {
        chunk_x,
        chunk_z,
        bitmask,
        full_chunk: true,
        data,
    })
}

/// Loads a chunk from a pair of `.dump` and `.meta` files.
pub fn load_chunk(path: impl AsRef<Path>) -> Result<Chunk> {
    let chunk = load_chunk_data(path)?.decode()?;

    Ok(chunk)
}

/// Saves a chunk packet to a pair of `chunk_{X}_{Z}.dump` and
/// `chunk_{X}_{Z}.meta` files in the directory pointed to by `path`.
pub fn save_packet_if_has_chunk_data(
    packet: &Packet,
    path: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    if let Some(ChunkData {
        chunk_x,
        chunk_z,
        bitmask,
        full_chunk: true,
        data,
    }) = ChunkData::from_packet(packet)
    {
        let mut path = PathBuf::from(path.as_ref());
        path.push(format!("chunk_{}_{}.dump", chunk_x, chunk_z));

        let dump_path = path.with_extension("dump");
        let meta_path = path.with_extension("meta");

        let meta = ChunkMeta {
            chunk_x,
            chunk_z,
            bitmask,
        };
        serde_json::to_writer(fs::File::create(meta_path)?, &meta)?;

        fs::write(&dump_path, data)?;

        Ok(Some(dump_path))
    } else {
        Ok(None)
    }
}
