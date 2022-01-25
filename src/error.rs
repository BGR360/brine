use std::fmt;

use bevy::{app::AppExit, prelude::*};

/// System that can be chained onto the end of another system to log any errors.
pub fn log_error<T, E: fmt::Display>(In(result): In<Result<T, E>>) {
    if let Err(e) = result {
        error!("{}", e);
    }
}

pub fn exit_on_error<T, E: fmt::Display>(
    In(result): In<Result<T, E>>,
    mut app_exit: EventWriter<AppExit>,
) {
    if let Err(e) = result {
        error!("{}", e);
        app_exit.send(AppExit);
    }
}
