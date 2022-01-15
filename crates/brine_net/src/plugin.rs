//! Plugins exposed by this crate.

use std::any::Any;
use std::marker::PhantomData;

use async_codec::{Decode, Encode};
use bevy::{
    app::{App, Plugin},
    ecs::prelude::*,
    tasks::IoTaskPool,
};

use crate::{event::NetworkEvent, resource::NetworkResource};

/// Plugin that implements the provided network codec.
pub struct NetworkPlugin<Codec> {
    _phantom: PhantomData<Codec>,
}

impl<Codec> Default for NetworkPlugin<Codec> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Codec> Plugin for NetworkPlugin<Codec>
where
    Codec: Decode + Encode + Any + Send + Sync,
{
    fn build(&self, app: &mut App) {
        app.add_event::<NetworkEvent<Codec>>();

        let task_pool = app.world.get_resource::<IoTaskPool>().unwrap().clone();
        let net_resource = NetworkResource::<Codec>::new(task_pool.0);
        app.insert_resource(net_resource);

        app.add_system(send_network_events::<Codec>);
    }
}

fn send_network_events<Codec>(
    mut net_resource: ResMut<NetworkResource<Codec>>,
    event_writer: EventWriter<NetworkEvent<Codec>>,
) where
    Codec: Any + Send + Sync,
{
    net_resource.send_network_events(event_writer);
}
