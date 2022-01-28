use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::prelude::*;

use brine_chunk::{decode::Result, BlockState, Chunk, Palette};
use brine_net::CodecReader;
use brine_proto::event;

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

fn handle_chunk_data(
    mut packet_reader: CodecReader<ProtocolCodec>,
    mut chunk_events: EventWriter<event::clientbound::ChunkData>,
) {
    for packet in packet_reader.iter() {
        match get_chunk_from_packet(packet) {
            Ok(Some(chunk_data)) => {
                trace!("Chunk: {:?}", chunk_data);
                chunk_events.send(event::clientbound::ChunkData { chunk_data });
            }
            Err(e) => error!("{}", e),
            _ => {}
        }
    }
}

pub fn get_chunk_from_packet(packet: &Packet) -> Result<Option<Chunk>> {
    if let Packet::Known(packet::Packet::ChunkData_HeightMap(chunk_data)) = packet {
        Ok(Some(Chunk::decode(
            chunk_data.chunk_x,
            chunk_data.chunk_z,
            chunk_data.new,
            chunk_data.bitmask.0 as u16,
            &DummyPalette,
            &mut &chunk_data.data.data[..],
        )?))
    } else {
        Ok(None)
    }
}
