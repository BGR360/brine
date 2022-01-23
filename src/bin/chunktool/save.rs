use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bevy::{app::AppExit, prelude::*};

use brine_net::CodecReader;
use brine_proto::{event::clientbound::Disconnect, ProtocolPlugin};
use brine_proto_backend::{
    backend_stevenarella::codec::{packet, Error, Packet, PacketType, ProtocolCodec},
    ProtocolBackendPlugin,
};

use brine::login::LoginPlugin;

/// Reads chunk packets from a server and saves them to files.
///
/// Each ChunkData packet received will be saved to a separate file in the
/// specified output directory.
///
/// Files will be named `chunk.{X}.{Z}.dat`
#[derive(clap::Args)]
pub struct Args {
    /// Output directory.
    #[clap(short, long, value_name = "DIR")]
    output: PathBuf,

    /// Server hostname or IP address.
    #[clap(short, long, value_name = "HOST", default_value = "localhost")]
    server: String,

    /// Server port.
    #[clap(short, long, default_value = "25565")]
    port: u16,

    /// Username to login with.
    #[clap(short, long, default_value = "Herobrine")]
    username: String,

    /// Exit after saving this many chunks.
    #[clap(short, long)]
    limit: Option<usize>,
}

pub fn main(args: Args) {
    let server_addr = format!("{}:{}", args.server, args.port);

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(ProtocolPlugin)
        .add_plugin(ProtocolBackendPlugin)
        .add_plugin(LoginPlugin::new(server_addr, args.username.clone()))
        .insert_resource(args)
        .add_system(receive_chunks)
        .add_system(handle_disconnect)
        .run();
}

fn handle_disconnect(
    mut disconnect_events: EventReader<Disconnect>,
    mut app_exit: EventWriter<AppExit>,
) {
    if let Some(disconnect) = disconnect_events.iter().last() {
        println!("Disconnected from server. Reason: {}", disconnect.reason);
        app_exit.send(AppExit);
    }
}

fn receive_chunks(
    args: Res<Args>,
    mut chunks_saved: Local<usize>,
    mut packet_reader: CodecReader<ProtocolCodec>,
    mut app_exit: EventWriter<AppExit>,
) {
    for packet in packet_reader.iter() {
        let (chunk_x, chunk_z, packet) = match packet {
            Packet::Known(packet @ packet::Packet::ChunkData_Biomes3D_Bitmasks(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_Biomes3D_VarInt(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_Biomes3D_bool(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_Biomes3D(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_HeightMap(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_NoEntities(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_NoEntities_u16(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }
            Packet::Known(packet @ packet::Packet::ChunkData_17(chunk_data)) => {
                (chunk_data.chunk_x, chunk_data.chunk_z, &*packet)
            }

            _ => continue,
        };

        *chunks_saved += 1;

        save_chunk(&args.output, chunk_x, chunk_z, packet)
            .map(|path| {
                println!(
                    "Saved chunk #{} to {}",
                    *chunks_saved,
                    path.to_string_lossy()
                )
            })
            .map_err(|e| println!("Error writing file: {}", e))
            .ok();

        if let Some(limit) = args.limit {
            if *chunks_saved >= limit {
                println!("Limit reached, terminating.");
                app_exit.send(AppExit);
                break;
            }
        }
    }
}

fn save_chunk(
    output_dir: &Path,
    chunk_x: i32,
    chunk_z: i32,
    packet: &packet::Packet,
) -> io::Result<PathBuf> {
    let mut path = PathBuf::from(output_dir);

    let filename = format!("chunk.{}.{}.dat", chunk_x, chunk_z);
    path.push(filename);

    let mut file = fs::File::create(&path)?;
    packet.write(&mut file).map_err(|err| match err {
        Error::IOError(e) => e,
        _ => panic!("Unexpected error: {}", err),
    })?;

    Ok(path)
}
