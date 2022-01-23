use bevy::{
    log::{Level, LogPlugin, LogSettings},
    prelude::*,
};

mod chunk;

const SERVER: &str = "localhost:25565";
const USERNAME: &str = "bgr360";

fn main() {
    App::new()
        .insert_resource(LogSettings {
            level: Level::INFO,
            ..Default::default()
        })
        // .add_plugins(MinimalPlugins)
        .add_plugins(DefaultPlugins)
        //.add_plugin(LogPlugin)
        .add_plugin(brine_proto::ProtocolPlugin)
        .add_plugin(brine_proto_backend::ProtocolBackendPlugin)
        .add_plugin(brine::login::LoginPlugin::new(
            SERVER.to_string(),
            USERNAME.to_string(),
        ))
        .run();
}
