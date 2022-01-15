use bevy::prelude::*;

use brine_net::{codec::DummyCodec, NetworkEvent, NetworkPlugin, NetworkResource};

const SERVER: &str = "google.com:80";

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(NetworkPlugin::<DummyCodec>::default())
        .add_startup_system(connect)
        .add_system(read_net_events)
        .run();
}

fn connect(mut net_resource: ResMut<NetworkResource<DummyCodec>>) {
    net_resource.connect(SERVER.to_string());
}

fn read_net_events(mut reader: EventReader<NetworkEvent<DummyCodec>>) {
    for event in reader.iter() {
        println!("NetworkEvent: {:?}", event);
    }
}
