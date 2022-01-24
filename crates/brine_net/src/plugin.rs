//! Plugins exposed by this crate.

use std::{any::Any, fmt::Debug, marker::PhantomData};

use async_codec::{Decode, Encode};
use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::{
    event::{EventWriter, Events},
    system::{Res, ResMut},
};
use bevy_tasks::IoTaskPool;

use crate::{
    event::NetworkEvent,
    resource::NetworkResource,
    system_param::{self, Read, Write},
};

pub type CodecReader<'w, 's, Codec> =
    system_param::CodecReader<'w, 's, <Codec as Decode>::Item, Codec>;

pub type CodecWriter<'w, 's, Codec> =
    system_param::CodecWriter<'w, 's, <Codec as Encode>::Item, Codec>;

/// Plugin that implements the provided network codec.
///
/// # Events
///
/// Use the following event processors to interact with the plugin:
///
/// * `EventReader<NetworkEvent<Codec>>`
///
///   * Vanilla Bevy [`EventReader`] of [`NetworkEvent<Codec>`]s.
///
///   * These events provide information about the status of the network
///     connection (e.g., connected, disconnected, errors).
///
/// * `CodecReader<Codec>`
///
///   * [`CodecReader`] provides packets that have been received and decoded
///     from the remote host.
///
///   * Packet reception and decoding happens asynchronously in the background
///     between frames.
///
/// * `CodecWriter<Codec>`
///
///   * [`CodecWriter`] allows packets to be encoded and sent to the remote
///     host.
///
///   * Packet encoding and transmission happens asynchronously in the
///     background between frames.
///
/// # Resources
///
/// The plugin registers the following resources:
/// * [`NetworkResource<Codec>`]
///   * Use [`connect()`][NetworkResource::connect] to establish a connection.
///
/// The plugin expects no resources to exist.
///
/// [`EventReader`]: bevy_ecs::event::EventReader
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

type CodecReadEvent<Codec> = Read<<Codec as Decode>::Item, Codec>;
type CodecWriteEvent<Codec> = Write<<Codec as Encode>::Item, Codec>;

impl<Codec> Plugin for NetworkPlugin<Codec>
where
    Codec: Decode + Encode + Default + Clone + Unpin + Any + Send + Sync,
    <Codec as Decode>::Item: Debug + Send + Sync,
    <Codec as Encode>::Item: Debug + Send + Sync,
    <Codec as Decode>::Error: Debug + Send + Sync,
    <Codec as Encode>::Error: Debug + Send + Sync,
{
    fn build(&self, app: &mut App) {
        app.add_event::<NetworkEvent<Codec>>();
        app.add_event::<CodecReadEvent<Codec>>();
        app.add_event::<CodecWriteEvent<Codec>>();

        let task_pool = app.world.get_resource::<IoTaskPool>().unwrap().clone();
        let net_resource = NetworkResource::<Codec>::new(task_pool.0);
        app.insert_resource(net_resource);

        app.add_system_to_stage(CoreStage::PreUpdate, Self::send_network_events);
        app.add_system_to_stage(CoreStage::PreUpdate, Self::send_packets_to_codec_reader);
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            Self::receive_packets_from_codec_writer,
        );
    }
}

impl<Codec> NetworkPlugin<Codec>
where
    Codec: Decode + Encode + Any + Send + Sync,
    <Codec as Decode>::Item: Send + Sync,
    <Codec as Encode>::Item: Send + Sync,
    <Codec as Decode>::Error: Debug + Send + Sync,
    <Codec as Encode>::Error: Debug + Send + Sync,
{
    /// System that pulls [`NetworkEvent`]s from the internal channel and
    /// forwards them through an [`EventWriter`] so they can be read by the
    /// appropriate [`EventReader`][bevy::ecs::event::EventReader].
    fn send_network_events(
        mut net_resource: ResMut<NetworkResource<Codec>>,
        mut event_writer: EventWriter<NetworkEvent<Codec>>,
    ) {
        while let Ok(event) = net_resource.network_event_receiver.try_recv() {
            // Clear the connection task if the connection has terminated,
            // thus allowing a new connection to form in the future.
            if let NetworkEvent::Disconnected = event {
                net_resource.connection_task = None;
            }

            event_writer.send(event);
        }
    }

    /// System that pulls decoded packets from the internal channel and forwards
    /// them through an [`EventWriter`] so they can be read by the
    /// appropriate [`CodecReader`].
    fn send_packets_to_codec_reader(
        net_resource: Res<NetworkResource<Codec>>,
        mut event_writer: EventWriter<CodecReadEvent<Codec>>,
    ) {
        while let Ok(packet) = net_resource.selfbound_packet_receiver.try_recv() {
            event_writer.send(Read(packet, PhantomData));
        }
    }

    /// System that pulls packets written by the appropriate [`CodecWriter`] and
    /// forwards them to the internal channel to be encoded and sent to the
    /// remote host.
    fn receive_packets_from_codec_writer(
        net_resource: Res<NetworkResource<Codec>>,
        mut events: ResMut<Events<CodecWriteEvent<Codec>>>,
    ) {
        net_resource.task_pool.scope(|scope| {
            scope.spawn(async {
                for packet in events.drain() {
                    net_resource
                        .peerbound_packet_sender
                        .send(packet.0)
                        .await
                        .unwrap();
                }
            });
        });
    }
}
