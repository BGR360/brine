use crate::event::{clientbound::LoginSuccess, ClientboundEvent, ServerboundEvent};

use bevy::prelude::*;

/// A plugin that responds immediately with success to the first login request.
///
/// # Events
///
/// The plugin does not register any events.
///
/// The plugin acts on the following events:
///
/// * [`ServerboundEvent::Login`]
///
/// The plugin sends the following events:
///
/// * [`ClientboundEvent::LoginSuccess`]
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
    mut rx: EventReader<ServerboundEvent>,
    mut tx: EventWriter<ClientboundEvent>,
) {
    for event in rx.iter() {
        if let ServerboundEvent::Login(login) = event {
            debug!("Dummy server advancing to state Play");
            state.set(ServerState::Play).unwrap();

            tx.send(ClientboundEvent::LoginSuccess(LoginSuccess {
                uuid: uuid::Uuid::new_v4(),
                username: login.username.clone(),
            }));
            break;
        }
    }
}
