//! Plugins exported by this crate.

use bevy::prelude::*;

use brine_net::{CodecReader, CodecWriter, NetworkResource};
use brine_proto::{ClientboundEvent, ServerboundEvent};

use crate::codec::MinecraftClientCodec;
use crate::convert::{ToEvent, ToPacket};

type ProtocolCodec = MinecraftClientCodec;

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
        app.add_state(ConnectionState::NotConnected);
        app.add_system_set(
            SystemSet::on_update(ConnectionState::NotConnected).with_system(connect_to_server),
        );
        app.add_system_set(
            SystemSet::on_enter(ConnectionState::Connecting).with_system(on_connection_established),
        );
        app.add_system_set(
            SystemSet::on_update(ConnectionState::Connected)
                .with_system(process_serverbound_packets)
                .with_system(process_clientbound_packets),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConnectionState {
    NotConnected,
    Connecting,
    Connected,
}

fn connect_to_server(
    mut event_reader: EventReader<ServerboundEvent>,
    mut connection_state: ResMut<State<ConnectionState>>,
    mut net_resource: ResMut<NetworkResource<ProtocolCodec>>,
) {
    for event in event_reader.iter() {
        match event {
            ServerboundEvent::Login(login) => {
                info!("Connecting to server");
                net_resource.connect(login.server.clone());
                connection_state.set(ConnectionState::Connecting).unwrap();
                break;
            }
            _ => {
                warn!("Unexpected serverbound event when no connection exists.");
                debug!("{:#?}", event);
            }
        }
    }
}

fn on_connection_established(mut connection_state: ResMut<State<ConnectionState>>) {
    info!("Connection established.");
    connection_state.set(ConnectionState::Connected).unwrap();
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
