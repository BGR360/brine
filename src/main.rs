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
        .add_state(AppState::Login)
        .add_system_set(SystemSet::on_update(AppState::Login).with_system(advance_to_play))
        .add_system(print_state)
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Login,
    Play,
}

/// System that prints the current `AppState` every frame.
fn print_state(app_state: Res<State<AppState>>) {
    debug!("Current AppState: {:?}", app_state.current());
}

/// System that runs in the `AppState::Login` state and advances to
/// `AppState::Play` when the space key is pressed.
fn advance_to_play(mut app_state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        info!("Advancing to state Play");
        app_state.set(AppState::Play).unwrap();
    }
}
