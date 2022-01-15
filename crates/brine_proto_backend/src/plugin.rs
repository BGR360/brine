//! Plugins exported by this crate.

use bevy::prelude::*;
//use minecraft_protocol::version::v1_14_4 as minecraft_proto;

use brine_proto::{ClientboundEvent, ServerboundEvent};

/// A plugin that responds immediately with success to the first login request.
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
                .with_system(send_serverbound_packets)
                .with_system(recv_clientbound_packets),
        );
    }
}

struct Connection;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConnectionState {
    NotConnected,
    Connecting,
    Connected,
}

fn connect_to_server(
    mut rx: EventReader<ServerboundEvent>,
    mut connection_state: ResMut<State<ConnectionState>>,
    mut commands: Commands,
) {
    for event in rx.iter() {
        match event {
            ServerboundEvent::Login(_) => {
                info!("Connecting to server");
                commands.insert_resource(Connection);
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

fn send_serverbound_packets(mut rx: EventReader<ServerboundEvent>, _tx: ResMut<Connection>) {
    for event in rx.iter() {
        match event {
            ServerboundEvent::Login(_) => {}
        }
    }
}

fn recv_clientbound_packets(mut _rx: ResMut<Connection>, mut _tx: EventWriter<ClientboundEvent>) {}
