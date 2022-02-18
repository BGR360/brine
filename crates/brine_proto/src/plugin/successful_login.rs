use bevy::prelude::*;

use crate::event::{clientbound::LoginSuccess, serverbound::Login, Uuid};

/// A plugin that responds immediately with success to the first login request.
///
/// # Events
///
/// The plugin does not register any events.
///
/// The plugin acts on the following events:
///
/// * [`Login`]
///
/// The plugin sends the following events:
///
/// * [`LoginSuccess`]
///
/// # Resources
///
/// The plugin does not register any resources.
///
/// The plugin does not expect any resources to exist.
pub struct AlwaysSuccessfulLoginPlugin;

impl Plugin for AlwaysSuccessfulLoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(ServerState::Login);
        app.add_system_set(SystemSet::on_update(ServerState::Login).with_system(handle_login));
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ServerState {
    Login,
    Play,
}

fn handle_login(
    mut state: ResMut<State<ServerState>>,
    mut rx: EventReader<Login>,
    mut tx: EventWriter<LoginSuccess>,
) {
    if let Some(login) = rx.iter().last() {
        debug!("Dummy server advancing to state Play");
        state.set(ServerState::Play).unwrap();

        tx.send(LoginSuccess {
            uuid: Uuid::new_v4(),
            username: login.username.clone(),
        });
    }
}
