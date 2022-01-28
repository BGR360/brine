use std::path::PathBuf;

use bevy::{app::AppExit, prelude::*};

use brine_net::CodecReader;
use brine_proto::{event::clientbound::Disconnect, ProtocolPlugin};
use brine_proto_backend::{backend_stevenarella::codec::ProtocolCodec, ProtocolBackendPlugin};

use brine::{chunk::save_packet_if_has_chunk_data, login::LoginPlugin};

/// Reads chunk packets from a server and saves them to files.
///
/// Each ChunkData packet received will be saved to a pair of files in the
/// specified output directory.
///
/// Files will be named `chunk_{X}_{Z}.dump` and `chunk_{X}_{Z}.meta`.
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
        if let Ok(Some(path)) = save_packet_if_has_chunk_data(packet, &args.output)
            .map_err(|e| println!("Error writing file: {}", e))
        {
            *chunks_saved += 1;
            println!(
                "Saved chunk #{} to {}",
                *chunks_saved,
                path.to_string_lossy()
            )
        }

        if let Some(limit) = args.limit {
            if *chunks_saved >= limit {
                println!("Limit reached, terminating.");
                app_exit.send(AppExit);
                break;
            }
        }
    }
}
