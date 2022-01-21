//! Implementation of the Minecraft protocol login handshake.
//!
//! This is driven by only a single message from the user's point of view:
//! [`Login`]. These systems handle all of the login logic.
//!
//! # The Login Process
//!
//! The login process consists of three phases:
//!
//! * Protocol Discovery
//!   1. Client connects
//!   1. C -> S: Handshake with Next State set to 1 (Status)
//!   2. C -> S: Status Request
//!   3. S -> C: Status Response (includes server's protocol version)
//!   4. C -> S: Status Ping
//!   5. S -> C: Status Pong
//!   6. Server disconnects
//!
//! * Login (unauthenticated)
//!   1. Client connects
//!   2. C -> S: Handshake with Next State set to 2 (Login)
//!   3. C -> S: Login Start
//!   4. S -> C: Login Success
//!
//! * Play
//!   * Periodic KeepAlive packets
//!   * Other play packets
//!
//! See these pages for reference:
//!
//! * <https://wiki.vg/Protocol#Handshaking>
//! * <https://wiki.vg/Protocol#Login>
//! * <https://wiki.vg/Protocol_FAQ#What.27s_the_normal_login_sequence_for_a_client.3F>

use std::str::FromStr;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::prelude::*;
use steven_protocol::protocol::{Serializable, VarInt};

use brine_net::{CodecReader, CodecWriter, NetworkError, NetworkEvent, NetworkResource};
use brine_proto::event::{
    clientbound::{Disconnect, LoginSuccess},
    serverbound::Login,
    Uuid,
};

use crate::codec::{HANDSHAKE_LOGIN_NEXT, HANDSHAKE_STATUS_NEXT};

use super::codec::{packet, Packet, ProtocolCodec};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum LoginState {
    Idle,

    // Phase 1
    StatusAwaitingConnect,
    StatusAwaitingResponse,
    StatusAwaitingDisconnect,

    // Phase 2
    LoginAwaitingConnect,
    LoginAwaitingSuccess,

    Play,
}

/// Keeps data around that is needed by systems occurring later in the state machine.
struct LoginResource {
    username: String,
    server_addr: String,
}

pub(crate) fn build(app: &mut App) {
    app.add_state(LoginState::Idle);

    protocol_discovery::build(app);
    login::build(app);
    play::build(app);
}

fn make_handshake_packet(protocol_version: i32, next_state: i32) -> Packet {
    Packet::Known(packet::Packet::Handshake(Box::new(
        packet::handshake::serverbound::Handshake {
            protocol_version: VarInt(protocol_version),
            // Next state to go to (1 for status, 2 for login)
            next: VarInt(next_state),
            ..Default::default()
        },
    )))
}

/// System that listens for any connection failure event and emits a LoginFailure event.
fn handle_connection_error(
    mut network_events: EventReader<NetworkEvent<ProtocolCodec>>,
    mut login_failure_events: EventWriter<Disconnect>,
    mut login_state: ResMut<State<LoginState>>,
) {
    for event in network_events.iter() {
        if let NetworkEvent::Error(NetworkError::ConnectFailed(io_error)) = event {
            error!("Connection failed: {}", io_error);

            login_failure_events.send(Disconnect {
                reason: format!("Connection failed: {}", io_error),
            });

            login_state.set(LoginState::Idle).unwrap();
            break;
        }
    }
}

mod protocol_discovery {
    use super::*;

    pub(crate) fn build(app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(LoginState::Idle).with_system(await_login_event_then_connect),
        );
        app.add_system_set(
            SystemSet::on_update(LoginState::StatusAwaitingConnect)
                .with_system(handle_connection_error)
                .with_system(await_connect_then_send_handshake_and_status_request),
        );
        app.add_system_set(
            SystemSet::on_update(LoginState::StatusAwaitingResponse)
                .with_system(await_response_then_send_status_ping),
        );
        app.add_system_set(
            SystemSet::on_update(LoginState::StatusAwaitingDisconnect)
                .with_system(await_disconnect_then_connect_for_login),
        );
    }

    fn await_login_event_then_connect(
        mut login_events: EventReader<Login>,
        mut login_state: ResMut<State<LoginState>>,
        mut net_resource: ResMut<NetworkResource<ProtocolCodec>>,
        mut commands: Commands,
    ) {
        if let Some(login) = login_events.iter().last() {
            info!("Logging in to server {}", login.server);

            debug!("Connecting to server for protocol discovery.");
            net_resource.connect(login.server.clone());

            commands.insert_resource(LoginResource {
                username: login.username.clone(),
                server_addr: login.server.clone(),
            });

            login_state.set(LoginState::StatusAwaitingConnect).unwrap();
        }
    }

    fn await_connect_then_send_handshake_and_status_request(
        mut network_events: EventReader<NetworkEvent<ProtocolCodec>>,
        mut packet_writer: CodecWriter<ProtocolCodec>,
        mut login_state: ResMut<State<LoginState>>,
        net_resource: Res<NetworkResource<ProtocolCodec>>,
    ) {
        for event in network_events.iter() {
            if let NetworkEvent::Connected = event {
                debug!("Connection established. Sending Handshake and StatusRequest packets.");

                let handshake = make_handshake_packet(
                    net_resource.codec().protocol_version(),
                    HANDSHAKE_STATUS_NEXT,
                );
                trace!("{:#?}", &handshake);
                packet_writer.send(handshake);

                let status_request = Packet::Known(packet::Packet::StatusRequest(Box::new(
                    packet::status::serverbound::StatusRequest::default(),
                )));
                packet_writer.send(status_request);

                login_state.set(LoginState::StatusAwaitingResponse).unwrap();
                break;
            }
        }
    }

    fn await_response_then_send_status_ping(
        mut packet_reader: CodecReader<ProtocolCodec>,
        mut packet_writer: CodecWriter<ProtocolCodec>,
        mut login_state: ResMut<State<LoginState>>,
        net_resource: Res<NetworkResource<ProtocolCodec>>,
    ) {
        for packet in packet_reader.iter() {
            if let Packet::Known(packet::Packet::StatusResponse(_)) = packet {
                // The codec will have already switched its internal protocol
                // version in response to decoding the StatusResponse packet,
                // so just read it from there.
                let protocol_version = net_resource.codec().protocol_version();

                debug!(
                    "StatusResponse received. Server protocol version = {}",
                    protocol_version
                );

                debug!("Sending StatusPing.");
                let status_ping = Packet::Known(packet::Packet::StatusPing(Box::new(
                    packet::status::serverbound::StatusPing::default(),
                )));
                packet_writer.send(status_ping);

                login_state
                    .set(LoginState::StatusAwaitingDisconnect)
                    .unwrap();
                break;
            }
        }
    }

    fn await_disconnect_then_connect_for_login(
        mut network_events: EventReader<NetworkEvent<ProtocolCodec>>,
        mut login_state: ResMut<State<LoginState>>,
        mut net_resource: ResMut<NetworkResource<ProtocolCodec>>,
        login_resource: Res<LoginResource>,
    ) {
        for event in network_events.iter() {
            if let NetworkEvent::Disconnected = event {
                debug!("Server disconnected as expected.");
                debug!("Connecting to server for login.");
                net_resource.connect(login_resource.server_addr.clone());

                login_state.set(LoginState::LoginAwaitingConnect).unwrap();
            }
        }
    }
}

#[allow(clippy::module_inception)]
mod login {
    use super::*;

    pub(crate) fn build(app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(LoginState::LoginAwaitingConnect)
                .with_system(handle_connection_error)
                .with_system(await_connect_then_send_handshake_and_login_start),
        );
        app.add_system_set(
            SystemSet::on_update(LoginState::LoginAwaitingSuccess).with_system(await_login_success),
        );
    }

    fn make_login_start_packet(_protocol_version: i32, username: String) -> Packet {
        Packet::Known(packet::Packet::LoginStart(Box::new(
            packet::login::serverbound::LoginStart { username },
        )))
    }

    /// System that listens for a successful connection event and then sends the
    /// first two packets of the login exchange.
    fn await_connect_then_send_handshake_and_login_start(
        mut network_events: EventReader<NetworkEvent<ProtocolCodec>>,
        mut packet_writer: CodecWriter<ProtocolCodec>,
        mut login_state: ResMut<State<LoginState>>,
        login_resource: Res<LoginResource>,
        net_resource: Res<NetworkResource<ProtocolCodec>>,
    ) {
        for event in network_events.iter() {
            if let NetworkEvent::Connected = event {
                debug!("Connection established. Sending Handshake and LoginStart packets.");

                let protocol_version = net_resource.codec().protocol_version();

                let handshake = make_handshake_packet(protocol_version, HANDSHAKE_LOGIN_NEXT);
                trace!("{:#?}", &handshake);
                packet_writer.send(handshake);

                let login_start =
                    make_login_start_packet(protocol_version, login_resource.username.clone());
                trace!("{:#?}", &login_start);
                packet_writer.send(login_start);

                login_state.set(LoginState::LoginAwaitingSuccess).unwrap();
                break;
            }
        }
    }

    /// System that listens for either a LoginSuccess or LoginDisconnect packet and
    /// emits the proper event in response.
    fn await_login_success(
        mut packet_reader: CodecReader<ProtocolCodec>,
        mut login_success_events: EventWriter<LoginSuccess>,
        mut disconnect_events: EventWriter<Disconnect>,
        mut login_state: ResMut<State<LoginState>>,
    ) {
        let mut on_login_success = |username: String, uuid: Uuid| {
            info!("Successfully logged in to server.");

            login_success_events.send(LoginSuccess { username, uuid });

            login_state.set(LoginState::Play).unwrap();
        };

        for packet in packet_reader.iter() {
            match packet {
                Packet::Known(packet::Packet::LoginSuccess_String(login_success)) => {
                    on_login_success(
                        login_success.username.clone(),
                        Uuid::from_str(&login_success.uuid).unwrap(),
                    );
                    break;
                }
                Packet::Known(packet::Packet::LoginSuccess_UUID(login_success)) => {
                    // Grr, Steven, y u no make fields public!
                    let mut uuid_bytes = Vec::with_capacity(16);
                    login_success.uuid.write_to(&mut uuid_bytes).unwrap();
                    let uuid = Uuid::from_bytes(uuid_bytes.try_into().unwrap());

                    on_login_success(login_success.username.clone(), uuid);
                    break;
                }

                Packet::Known(packet::Packet::LoginDisconnect(login_disconnect)) => {
                    let message = format!("Login disconnect: {}", login_disconnect.reason);
                    error!("{}", &message);

                    disconnect_events.send(Disconnect { reason: message });

                    login_state.set(LoginState::Idle).unwrap();
                    break;
                }

                _ => {}
            }
        }
    }
}

mod play {
    use super::*;

    pub(crate) fn build(app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(LoginState::Play)
                .with_system(respond_to_keep_alive_packets)
                .with_system(handle_disconnect),
        );
    }

    fn respond_to_keep_alive_packets(
        mut packet_reader: CodecReader<ProtocolCodec>,
        mut packet_writer: CodecWriter<ProtocolCodec>,
    ) {
        for packet in packet_reader.iter() {
            let response = match packet {
                Packet::Known(packet::Packet::KeepAliveClientbound_VarInt(keep_alive)) => {
                    Packet::Known(packet::Packet::KeepAliveServerbound_VarInt(Box::new(
                        packet::play::serverbound::KeepAliveServerbound_VarInt {
                            id: keep_alive.id,
                        },
                    )))
                }
                Packet::Known(packet::Packet::KeepAliveClientbound_i32(keep_alive)) => {
                    Packet::Known(packet::Packet::KeepAliveServerbound_i32(Box::new(
                        packet::play::serverbound::KeepAliveServerbound_i32 { id: keep_alive.id },
                    )))
                }
                Packet::Known(packet::Packet::KeepAliveClientbound_i64(keep_alive)) => {
                    Packet::Known(packet::Packet::KeepAliveServerbound_i64(Box::new(
                        packet::play::serverbound::KeepAliveServerbound_i64 { id: keep_alive.id },
                    )))
                }

                _ => continue,
            };

            debug!("KeepAlive");
            packet_writer.send(response);
            break;
        }
    }

    fn handle_disconnect(
        mut packet_reader: CodecReader<ProtocolCodec>,
        mut disconnect_events: EventWriter<Disconnect>,
    ) {
        for packet in packet_reader.iter() {
            if let Packet::Known(packet::Packet::Disconnect(disconnect)) = packet {
                let reason = disconnect.reason.to_string();
                disconnect_events.send(Disconnect { reason });
            }
        }
    }
}
