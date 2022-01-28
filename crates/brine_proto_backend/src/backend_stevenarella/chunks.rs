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

/// Common representation of the different versions of ChunkData packets.
pub struct ChunkData<T> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub full_chunk: bool,
    pub bitmask: u16,
    pub data: T,
}

impl<'d> ChunkData<&'d [u8]> {
    pub fn from_packet(packet: &'d Packet) -> Option<Self> {
        let (chunk_x, chunk_z, full_chunk, bitmask, data) = match packet {
            /*Packet::Known(packet::Packet::ChunkData_Biomes3D_Bitmasks(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                ??,
                ??,
                &chunk_data.data[..],
            ),*/
            Packet::Known(packet::Packet::ChunkData_Biomes3D_VarInt(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &chunk_data.data.data[..],
            ),
            Packet::Known(packet::Packet::ChunkData_Biomes3D_bool(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &chunk_data.data.data[..],
            ),
            Packet::Known(packet::Packet::ChunkData_Biomes3D(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &chunk_data.data.data[..],
            ),
            Packet::Known(packet::Packet::ChunkData_HeightMap(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &chunk_data.data.data[..],
            ),
            Packet::Known(packet::Packet::ChunkData(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &chunk_data.data.data[..],
            ),
            Packet::Known(packet::Packet::ChunkData_NoEntities(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask.0 as u16,
                &chunk_data.data.data[..],
            ),
            /*Packet::Known(packet::Packet::ChunkData_NoEntities_u16(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask,
                &chunk_data.data.data[..] ?? varints,
            ),*/
            /*Packet::Known(packet::Packet::ChunkData_17(chunk_data)) => (
                chunk_data.chunk_x,
                chunk_data.chunk_z,
                chunk_data.new,
                chunk_data.bitmask,
                &chunk_data.data.data[..] ?? compressed data,
            ),*/
            _ => return None,
        };

        Some(ChunkData {
            chunk_x,
            chunk_z,
            bitmask,
            full_chunk,
            data,
        })
    }
}

impl<T: AsRef<[u8]>> ChunkData<T> {
    pub fn decode(&self) -> Result<Chunk> {
        let mut buf = self.data.as_ref();
        Chunk::decode(
            self.chunk_x,
            self.chunk_z,
            self.full_chunk,
            self.bitmask,
            &DummyPalette,
            &mut buf,
        )
    }
}

pub fn get_chunk_from_packet(packet: &Packet) -> Result<Option<Chunk>> {
    if let Some(chunk_data) = ChunkData::from_packet(packet) {
        Ok(Some(chunk_data.decode()?))
    } else {
        Ok(None)
    }
}

pub(crate) fn build(app: &mut App) {
    app.add_system(handle_chunk_data);
}

/// System that listens for ChunkData packets and sends ChunkData events to the
/// client application.
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
