use brine_proto::{
    event::serverbound::{Handshake, LoginStart},
    ClientboundEvent, ServerboundEvent,
};

use bevy::{
    log::{Level, LogSettings},
    prelude::*,
};

fn main() {
    App::new()
        .insert_resource(LogSettings {
            level: Level::INFO,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(brine_proto::ProtocolPlugin)
        .add_plugin(brine_proto::AlwaysSuccessfulLoginPlugin)
        .add_state(AppState::Login)
        .add_system_set(
            SystemSet::on_update(AppState::Login)
                .with_system(send_handshake_and_login)
                .with_system(advance_to_play),
        )
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Login,
    Play,
}

/// System that sends a handshake and login event when the space key is pressed.
fn send_handshake_and_login(
    mut tx: EventWriter<ServerboundEvent>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        info!("Initiating login");
        tx.send(ServerboundEvent::Handshake(Handshake));
        tx.send(ServerboundEvent::LoginStart(LoginStart));
    }
}

/// System that advances to the Play state when a LoginSuccess event is received.
fn advance_to_play(mut app_state: ResMut<State<AppState>>, mut rx: EventReader<ClientboundEvent>) {
    for event in rx.iter() {
        if let ClientboundEvent::LoginSuccess(_) = event {
            info!("Login successful, advancing to state Play");
            app_state.set(AppState::Play).unwrap();
        }
    }
}
