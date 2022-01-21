use brine_proto::event::{
    clientbound::{Disconnect, LoginSuccess},
    serverbound::Login,
};

use bevy::{
    log::{Level, LogPlugin, LogSettings},
    prelude::*,
};

const SERVER: &str = "localhost:25565";
const USERNAME: &str = "bgr360";

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        })
        .add_plugin(LogPlugin)
        .add_plugin(brine_proto::ProtocolPlugin)
        .add_plugin(brine_proto_backend::ProtocolBackendPlugin)
        .add_startup_system(initiate_login)
        .add_state(AppState::Idle)
        .add_system_set(SystemSet::on_update(AppState::Login).with_system(advance_to_play))
        .add_system_set(SystemSet::on_update(AppState::Play).with_system(handle_disconnect))
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Idle,
    Login,
    Play,
}

fn initiate_login(mut login_events: EventWriter<Login>, mut app_state: ResMut<State<AppState>>) {
    info!("Initiating login");
    login_events.send(Login {
        server: SERVER.to_string(),
        username: USERNAME.to_string(),
    });
    app_state.set(AppState::Login).unwrap();
}

/// System that advances to the Play state when a LoginSuccess event is received.
fn advance_to_play(
    mut login_success_events: EventReader<LoginSuccess>,
    mut app_state: ResMut<State<AppState>>,
) {
    if login_success_events.iter().last().is_some() {
        info!("Login successful, advancing to state Play");
        app_state.set(AppState::Play).unwrap();
    }
}

fn handle_disconnect(
    mut disconnect_events: EventReader<Disconnect>,
    mut app_state: ResMut<State<AppState>>,
) {
    if let Some(disconnect) = disconnect_events.iter().last() {
        info!("Disconnected from server. Reason: {}", disconnect.reason);
        app_state.set(AppState::Idle).unwrap();
    }
}
