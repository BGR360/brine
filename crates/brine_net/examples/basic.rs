use async_codec::{Decode, DecodeResult, Encode, EncodeResult};
use bevy::{
    log::{Level, LogPlugin, LogSettings},
    prelude::*,
};

use brine_net::{NetworkEvent, NetworkPlugin, NetworkResource};

const SERVER: &str = "google.com:80";

#[derive(Debug)]
struct DummyCodec;

impl Encode for DummyCodec {
    type Item = ();
    type Error = ();

    fn encode(&mut self, _item: &Self::Item, _buf: &mut [u8]) -> EncodeResult<Self::Error> {
        EncodeResult::Ok(0)
    }
}

impl Decode for DummyCodec {
    type Item = ();
    type Error = ();

    fn decode(&mut self, _buffer: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        (0, DecodeResult::Ok(()))
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        })
        .add_plugin(LogPlugin)
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
        info!("NetworkEvent: {:?}", event);
    }
}
