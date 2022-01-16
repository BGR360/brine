//! Plugins exposed by this crate.

use std::{any::Any, fmt::Debug, marker::PhantomData};

use async_codec::{Decode, Encode};
use bevy::{
    app::{App, Plugin},
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
    Codec: Decode + Encode + Default + Unpin + Any + Send + Sync,
    <Codec as Decode>::Item: Debug + Send,
    <Codec as Encode>::Item: Debug + Send,
    <Codec as Decode>::Error: Debug + Send,
    <Codec as Encode>::Error: Debug + Send,
{
    fn build(&self, app: &mut App) {
        app.add_event::<NetworkEvent<Codec>>();

        let task_pool = app.world.get_resource::<IoTaskPool>().unwrap().clone();
        let net_resource = NetworkResource::<Codec>::new(task_pool.0);
        app.insert_resource(net_resource);

        app.add_system(NetworkResource::<Codec>::send_network_events);
    }
}
