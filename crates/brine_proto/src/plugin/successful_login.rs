use crate::event::{clientbound::LoginSuccess, ClientboundEvent, ServerboundEvent};

use bevy::{ecs::schedule::StateError, prelude::*};

/// A plugin that responds immediately with success to the first login request.
///
/// # Events
///
/// The plugin does not register any events.
///
/// The plugin acts on the following events:
///
/// * [`ServerboundEvent::Handshake`]
/// * [`ServerboundEvent::LoginStart`]
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
        app.add_state(ServerState::Handshake);
        app.add_system_set(
            SystemSet::on_update(ServerState::Handshake).with_system(handle_handshake),
        );
        app.add_system_set(
            SystemSet::on_update(ServerState::Login).with_system(handle_login_start),
        );
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum ServerState {
    Handshake,
    Login,
    Play,
}

fn handle_handshake(mut state: ResMut<State<ServerState>>, mut rx: EventReader<ServerboundEvent>) {
    for event in rx.iter() {
        if let ServerboundEvent::Handshake(_) = event {
            advance(&mut state, ServerState::Login).unwrap();
            break;
        }
    }
}

fn handle_login_start(
    mut state: ResMut<State<ServerState>>,
    mut rx: EventReader<ServerboundEvent>,
    mut tx: EventWriter<ClientboundEvent>,
) {
    for event in rx.iter() {
        if let ServerboundEvent::LoginStart(_) = event {
            advance(&mut state, ServerState::Play).unwrap();
            tx.send(ClientboundEvent::LoginSuccess(LoginSuccess));
            break;
        }
    }
}

fn advance(
    state: &mut ResMut<State<ServerState>>,
    new_state: ServerState,
) -> Result<(), StateError> {
    debug!("Dummy server advancing to state {:?}", new_state);
    state.set(new_state)
}
