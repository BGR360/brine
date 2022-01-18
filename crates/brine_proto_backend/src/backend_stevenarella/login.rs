//! Implementation of the Minecraft protocol login handshake.
//!
//! This is driven by only a single message from the user's point of view:
//! [`ServerboundEvent::Login`]. These systems handle all of the login logic.
//!
//! # The Login Process
//!
//! See these pages for reference:
//!
//! * <https://wiki.vg/Protocol#Handshaking>
//! * <https://wiki.vg/Protocol#Login>
//! * <https://wiki.vg/Protocol_FAQ#What.27s_the_normal_login_sequence_for_a_client.3F>

use std::{io::Write, str::FromStr};

use bevy::prelude::*;
use brine_net::{CodecReader, CodecWriter, NetworkError, NetworkEvent, NetworkResource};
use brine_proto::{event, ClientboundEvent, ServerboundEvent};
use steven_protocol::protocol::{Serializable, VarInt};

use super::codec::{packet, Packet, ProtocolCodec, PROTOCOL_VERSION};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum LoginState {
    NotStarted,
    Connecting,
    HandshakeAndLoginStartSent,
    LoggedIn,
}

struct LoginResource {
    username: String,
}

pub(crate) fn build(app: &mut App) {
    app.add_state(LoginState::NotStarted);

    app.add_system_set(SystemSet::on_update(LoginState::NotStarted).with_system(connect_to_server));
    app.add_system_set(
        SystemSet::on_update(LoginState::Connecting)
            .with_system(handle_connection_error)
            .with_system(send_handshake_and_login_start),
    );
    app.add_system_set(
        SystemSet::on_update(LoginState::HandshakeAndLoginStartSent)
            .with_system(await_login_success),
    );
}

/// System that listents for a Login event and intiates a connection to the server.
fn connect_to_server(
    mut event_reader: EventReader<ServerboundEvent>,
    mut login_state: ResMut<State<LoginState>>,
    mut net_resource: ResMut<NetworkResource<ProtocolCodec>>,
    mut commands: Commands,
) {
    for event in event_reader.iter() {
        match event {
            ServerboundEvent::Login(login) => {
                info!("Connecting to server");
                net_resource.connect(login.server.clone());

                commands.insert_resource(LoginResource {
                    username: login.username.clone(),
                });

                login_state.set(LoginState::Connecting).unwrap();
                break;
            }
            _ => {
                warn!("Unexpected serverbound event when no connection exists.");
                debug!("{:#?}", event);
            }
        }
    }
}

/// System that listens for any connection failure event after the connection
/// has started forming, and emits it as a LoginFailure event.
fn handle_connection_error(
    mut network_event_reader: EventReader<NetworkEvent<ProtocolCodec>>,
    mut event_writer: EventWriter<ClientboundEvent>,
    mut login_state: ResMut<State<LoginState>>,
) {
    for event in network_event_reader.iter() {
        if let NetworkEvent::Error(NetworkError::ConnectFailed(io_error)) = event {
            error!("Connection failed: {}", io_error);

            event_writer.send(ClientboundEvent::LoginFailure(
                event::clientbound::LoginFailure {
                    reason: format!("Connection failed: {}", io_error),
                },
            ));

            login_state.set(LoginState::NotStarted).unwrap();
            break;
        }
    }
}

/// System that listens for a successful connection event and then sends the
/// first two packets of the login handshake.
fn send_handshake_and_login_start(
    mut network_event_reader: EventReader<NetworkEvent<ProtocolCodec>>,
    mut codec_writer: CodecWriter<ProtocolCodec>,
    mut login_state: ResMut<State<LoginState>>,
    login_resource: Res<LoginResource>,
) {
    for event in network_event_reader.iter() {
        if let NetworkEvent::Connected = event {
            info!("Connection established. Logging in...");

            debug!("Sending Handshake and LoginStart packets.");

            let handshake = make_handshake_packet(PROTOCOL_VERSION);
            trace!("{:#?}", &handshake);
            codec_writer.send(handshake);

            let login_start =
                make_login_start_packet(PROTOCOL_VERSION, login_resource.username.clone());
            trace!("{:#?}", &login_start);
            codec_writer.send(login_start);

            login_state
                .set(LoginState::HandshakeAndLoginStartSent)
                .unwrap();
            break;
        }
    }
}

/// System that listens for either a LoginSuccess or LoginDisconnect packet and
/// emits the proper event in response.
fn await_login_success(
    mut codec_reader: CodecReader<ProtocolCodec>,
    mut event_writer: EventWriter<ClientboundEvent>,
    mut login_state: ResMut<State<LoginState>>,
) {
    let mut on_login_success = |username: String, uuid: event::Uuid| {
        info!("Successfully logged in to server.");

        event_writer.send(ClientboundEvent::LoginSuccess(
            event::clientbound::LoginSuccess { username, uuid },
        ));

        login_state.set(LoginState::LoggedIn).unwrap();
    };

    for packet in codec_reader.iter() {
        match packet {
            Packet::Known(packet::Packet::LoginSuccess_String(login_success)) => {
                on_login_success(
                    login_success.username.clone(),
                    event::Uuid::from_str(&login_success.uuid).unwrap(),
                );
                break;
            }
            Packet::Known(packet::Packet::LoginSuccess_UUID(login_success)) => {
                // Grr, Steven, y u no make fields public!
                let mut uuid_bytes = Vec::with_capacity(16);
                login_success.uuid.write_to(&mut uuid_bytes).unwrap();
                let uuid = event::Uuid::from_bytes(uuid_bytes.try_into().unwrap());

                on_login_success(login_success.username.clone(), uuid);
                break;
            }

            Packet::Known(packet::Packet::LoginDisconnect(login_disconnect)) => {
                let message = format!("Login disconnect: {}", login_disconnect.reason);
                error!("{}", &message);

                let event = ClientboundEvent::LoginFailure(event::clientbound::LoginFailure {
                    reason: message,
                });
                event_writer.send(event);

                login_state.set(LoginState::NotStarted).unwrap();
                break;
            }

            _ => {}
        }
    }
}

fn make_handshake_packet(protocol_version: i32) -> Packet {
    Packet::Known(packet::Packet::Handshake(Box::new(
        packet::handshake::serverbound::Handshake {
            protocol_version: VarInt(protocol_version),
            // Next state to go to (1 for status, 2 for login)
            next: VarInt(2),
            ..Default::default()
        },
    )))
}

fn make_login_start_packet(_protocol_version: i32, username: String) -> Packet {
    Packet::Known(packet::Packet::LoginStart(Box::new(
        packet::login::serverbound::LoginStart { username },
    )))
}
