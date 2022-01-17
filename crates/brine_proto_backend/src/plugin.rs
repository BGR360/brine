//! Plugins exported by this crate.

use bevy::prelude::*;

use brine_net::{CodecReader, CodecWriter, NetworkEvent, NetworkPlugin};
use brine_proto::{ClientboundEvent, ServerboundEvent};

use crate::codec::MinecraftClientCodec;
use crate::convert::{ToEvent, ToPacket};

pub(crate) type ProtocolCodec = MinecraftClientCodec;

/// Minecraft protocol implementation plugin.
///
/// # Events
///
/// The plugin does not register any events.
///
/// The plugin acts on the following events:
///
/// * [`brine_proto::ServerboundEvent`]
///
/// The plugin sends the following events:
///
/// * [`brine_proto::ClientboundEvent`]
///
/// # Resources
///
/// The plugin does not register any resources.
///
/// The plugin does not expect any resources to exist.
pub struct ProtocolBackendPlugin;

impl Plugin for ProtocolBackendPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NetworkPlugin::<ProtocolCodec>::default());

        app.add_system(log_network_errors);

        Self::build_login(app);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConnectionState {
    NotConnected,
    Connecting,
    Connected,
}

fn log_network_errors(mut event_reader: EventReader<NetworkEvent<ProtocolCodec>>) {
    for event in event_reader.iter() {
        if let NetworkEvent::Error(network_error) = event {
            warn!("Network error: {}", network_error);
        }
    }
}

fn process_serverbound_packets(
    mut event_reader: EventReader<ServerboundEvent>,
    mut packet_writer: CodecWriter<ProtocolCodec>,
) {
    for event in event_reader.iter() {
        match event {
            ServerboundEvent::Login(login) => {
                warn!("Unexpected login event when connection already established.");
                debug!("{:#?}", login);
            }
            _ => {}
        }

        if let Some(packet) = event.to_packet() {
            packet_writer.send(packet);
        }
    }
}

fn process_clientbound_packets(
    mut packet_reader: CodecReader<ProtocolCodec>,
    mut event_writer: EventWriter<ClientboundEvent>,
) {
    for packet in packet_reader.iter() {
        if let Some(event) = packet.to_event() {
            event_writer.send(event);
        }
    }
}
