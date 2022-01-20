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

use bevy::prelude::*;
use brine_net::{CodecReader, CodecWriter, NetworkError, NetworkEvent, NetworkResource};
use brine_proto::event::{
    clientbound::{LoginFailure, LoginSuccess},
    serverbound::Login,
    Uuid,
};
use minecraft_protocol::data::chat;

use super::codec::{proto, ClientboundPacket, ProtocolCodec, ServerboundPacket};

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
    mut login_events: EventReader<Login>,
    mut login_state: ResMut<State<LoginState>>,
    mut net_resource: ResMut<NetworkResource<ProtocolCodec>>,
    mut commands: Commands,
) {
    if let Some(login) = login_events.iter().last() {
        info!("Connecting to server");
        net_resource.connect(login.server.clone());

        commands.insert_resource(LoginResource {
            username: login.username.clone(),
        });

        login_state.set(LoginState::Connecting).unwrap();
    }
}

/// System that listens for any connection failure event after the connection
/// has started forming, and emits it as a LoginFailure event.
fn handle_connection_error(
    mut network_events: EventReader<NetworkEvent<ProtocolCodec>>,
    mut login_failure_events: EventWriter<LoginFailure>,
    mut login_state: ResMut<State<LoginState>>,
) {
    for event in network_events.iter() {
        if let NetworkEvent::Error(NetworkError::ConnectFailed(io_error)) = event {
            error!("Connection failed: {}", io_error);

            login_failure_events.send(LoginFailure {
                reason: format!("Connection failed: {}", io_error),
            });

            login_state.set(LoginState::NotStarted).unwrap();
            break;
        }
    }
}

/// System that listens for a successful connection event and then sends the
/// first two packets of the login handshake.
fn send_handshake_and_login_start(
    mut network_events: EventReader<NetworkEvent<ProtocolCodec>>,
    mut packet_writer: CodecWriter<ProtocolCodec>,
    mut login_state: ResMut<State<LoginState>>,
    login_resource: Res<LoginResource>,
) {
    for event in network_events.iter() {
        if let NetworkEvent::Connected = event {
            info!("Connection established. Logging in...");

            debug!("Sending Handshake and LoginStart packets.");

            let handshake = ServerboundPacket::Handshake(
                proto::handshake::HandshakeServerBoundPacket::Handshake(
                    proto::handshake::Handshake {
                        protocol_version: 498,
                        server_addr: "".to_string(),
                        server_port: 0,
                        next_state: 2,
                    },
                ),
            );
            trace!("{:#?}", &handshake);
            packet_writer.send(handshake);

            let login_start = ServerboundPacket::Login(
                proto::login::LoginServerBoundPacket::LoginStart(proto::login::LoginStart {
                    name: login_resource.username.clone(),
                }),
            );
            trace!("{:#?}", &login_start);
            packet_writer.send(login_start);

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
    mut packet_reader: CodecReader<ProtocolCodec>,
    mut login_success_events: EventWriter<LoginSuccess>,
    mut login_failure_events: EventWriter<LoginFailure>,
    mut login_state: ResMut<State<LoginState>>,
) {
    for packet in packet_reader.iter() {
        match packet {
            ClientboundPacket::Login(proto::login::LoginClientBoundPacket::LoginSuccess(
                login_success,
            )) => {
                info!("Successfully logged in to server.");

                login_success_events.send(LoginSuccess {
                    username: login_success.username.clone(),
                    uuid: Uuid::from_bytes(*login_success.uuid.as_bytes()),
                });

                login_state.set(LoginState::LoggedIn).unwrap();
                break;
            }

            ClientboundPacket::Login(proto::login::LoginClientBoundPacket::LoginDisconnect(
                login_disconnect,
            )) => {
                let message = match login_disconnect.reason.payload {
                    chat::Payload::Text { ref text } => format!("Login disconnect: {}", text),
                    _ => "Login disconnect: unknown reason".to_string(),
                };
                error!("{}", &message);
                debug!("{:?}", &login_disconnect.reason);

                login_failure_events.send(LoginFailure { reason: message });

                login_state.set(LoginState::NotStarted).unwrap();
                break;
            }

            _ => {}
        }
    }
}
