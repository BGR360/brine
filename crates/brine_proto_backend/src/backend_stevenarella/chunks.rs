use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::prelude::*;

use brine_chunk::{BlockState, Chunk, Palette};
use brine_net::CodecReader;

use super::codec::{packet, Packet, ProtocolCodec};

/// A dummy palette for testing that performs no translation.
pub struct DummyPalette;

impl Palette for DummyPalette {
    fn id_to_block_state(&self, id: u32) -> Option<brine_chunk::BlockState> {
        Some(BlockState(id))
    }
}

pub(crate) fn build(app: &mut App) {
    app.add_system(handle_chunk_data);
}

fn handle_chunk_data(mut packet_reader: CodecReader<ProtocolCodec>) {
    for packet in packet_reader.iter() {
        if let Packet::Known(packet::Packet::ChunkData_HeightMap(chunk_data)) = packet {
            info!("Chunk time!");
            match Chunk::decode(
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &DummyPalette,
                &mut &chunk_data.data.data[..],
            ) {
                Ok(chunk) => {
                    debug!("Chunk: {:?}", chunk);
                }
                Err(e) => error!("{}", e),
            }
        }
    }
}
