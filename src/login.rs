use bevy::prelude::*;

use brine_proto::event::{
    clientbound::{Disconnect, LoginSuccess},
    serverbound::Login,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Idle,
    Login,
    Play,
}

#[derive(Debug, Clone)]
struct LoginInfo {
    server: String,
    username: String,
}

/// Simple plugin that initiates login to a Minecraft server on app startup.
pub struct LoginPlugin {
    info: LoginInfo,
}

impl LoginPlugin {
    pub fn new(server: String, username: String) -> Self {
        Self {
            info: LoginInfo { server, username },
        }
    }
}

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.info.clone())
            .add_state(GameState::Idle)
            .add_startup_system(initiate_login)
            .add_system_set(SystemSet::on_update(GameState::Login).with_system(await_success))
            .add_system_set(SystemSet::on_update(GameState::Play).with_system(handle_disconnect));
    }
}

fn initiate_login(
    login_info: Res<LoginInfo>,
    mut login_events: EventWriter<Login>,
    mut app_state: ResMut<State<GameState>>,
) {
    info!("Initiating login");
    login_events.send(Login {
        server: login_info.server.clone(),
        username: login_info.username.clone(),
    });
    app_state.set(GameState::Login).unwrap();
}

fn await_success(
    mut login_success_events: EventReader<LoginSuccess>,
    mut app_state: ResMut<State<GameState>>,
) {
    if login_success_events.iter().last().is_some() {
        info!("Login successful, advancing to state Play");
        app_state.set(GameState::Play).unwrap();
    }
}

fn handle_disconnect(
    mut disconnect_events: EventReader<Disconnect>,
    mut app_state: ResMut<State<GameState>>,
) {
    if let Some(disconnect) = disconnect_events.iter().last() {
        info!("Disconnected from server. Reason: {}", disconnect.reason);
        app_state.set(GameState::Idle).unwrap();
    }
}
