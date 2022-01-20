use brine_proto::event::{clientbound::LoginSuccess, serverbound::Login};

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};

const SERVER: &str = "localhost:25565";
const USERNAME: &str = "bgr360";

fn main() {
    App::new()
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(brine_proto::ProtocolPlugin)
        .add_plugin(brine_proto_backend::ProtocolBackendPlugin)
        .add_startup_system(initiate_login)
        .add_state(AppState::Login)
        .add_system_set(SystemSet::on_update(AppState::Login).with_system(advance_to_play))
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Login,
    Play,
}

fn initiate_login(mut login_events: EventWriter<Login>) {
    info!("Initiating login");
    login_events.send(Login {
        server: SERVER.to_string(),
        username: USERNAME.to_string(),
    });
}

/// System that advances to the Play state when a LoginSuccess event is received.
fn advance_to_play(
    mut app_state: ResMut<State<AppState>>,
    mut login_success_events: EventReader<LoginSuccess>,
) {
    if login_success_events.iter().last().is_some() {
        info!("Login successful, advancing to state Play");
        app_state.set(AppState::Play).unwrap();
    }
}
