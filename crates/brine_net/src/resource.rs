//! Resources exposed by this crate.

use std::{any::Any, fmt::Debug};

use async_channel::{unbounded, Receiver, Sender};
use async_codec::{Decode, Encode};
use bevy::{
    ecs::{event::EventWriter, system::Res},
    tasks::{Task, TaskPool},
};

use crate::{
    connection::Connection,
    event::{NetworkError, NetworkEvent},
};

/// Resource that provides a TCP connection that encodes and decodes
/// packets as specified by the given codec.
pub struct NetworkResource<Codec: Decode + Encode> {
    pub(crate) task_pool: TaskPool,
    pub(crate) connection_task: Option<Task<()>>,

    /// Used by background tasks to produce [`NetworkEvent`]s.
    pub(crate) network_event_sender: Sender<NetworkEvent<Codec>>,

    /// Used to forward [`NetworkEvent`]s through an [`EventWriter`].
    pub(crate) network_event_receiver: Receiver<NetworkEvent<Codec>>,

    /// Used by the main Bevy app loop to produce packets destined for the
    /// remote host.
    pub(crate) peerbound_packet_sender: Sender<<Codec as Encode>::Item>,

    /// Used by background tasks to forward packets to the remote host.
    pub(crate) peerbound_packet_receiver: Receiver<<Codec as Encode>::Item>,

    /// Used by background tasks to produce packets destined for the local host.
    pub(crate) selfbound_packet_sender: Sender<<Codec as Decode>::Item>,

    /// Used by the main Bevy app loop to forward packets through an [`EventWriter`].
    pub(crate) selfbound_packet_receiver: Receiver<<Codec as Decode>::Item>,
}

impl<Codec> NetworkResource<Codec>
where
    Codec: Decode + Encode + Default + Unpin + Send + 'static,
    <Codec as Decode>::Item: Debug + Send + 'static,
    <Codec as Encode>::Item: Debug + Send + 'static,
    <Codec as Decode>::Error: Debug + Send + 'static,
    <Codec as Encode>::Error: Debug + Send + 'static,
{
    pub(crate) fn new(task_pool: TaskPool) -> Self {
        let (network_event_sender, network_event_receiver) = unbounded();
        let (peerbound_packet_sender, peerbound_packet_receiver) = unbounded();
        let (selfbound_packet_sender, selfbound_packet_receiver) = unbounded();

        Self {
            task_pool,
            connection_task: None,
            network_event_sender,
            network_event_receiver,
            peerbound_packet_sender,
            peerbound_packet_receiver,
            selfbound_packet_sender,
            selfbound_packet_receiver,
        }
    }

    /// Establish a connection with a server that speaks this codec.
    ///
    /// The server address argument can be a hostname or an IP address.
    ///
    /// If any error occurs in the process of establishing the connection, this
    /// will be delivered as a [`NetworkEvent`][crate::NetworkEvent].
    pub fn connect(&mut self, server_addr: String) {
        if self.connection_task.is_some() {
            self.task_pool.scope(|scope| {
                scope.spawn(async {
                    self.network_event_sender
                        .send(NetworkEvent::Error(NetworkError::AlreadyConnected))
                        .await
                        .unwrap();
                });
            });
        } else {
            let connection = Connection::new(self);

            self.connection_task = Some(self.task_pool.spawn(async move {
                connection.connect_and_run(server_addr).await;
            }));
        }
    }

    // TODO: switch to events (encoding only needs reference to packet)
    pub fn send_packet(&self, packet: <Codec as Encode>::Item) {
        self.task_pool.scope(|scope| {
            scope.spawn(async move {
                self.peerbound_packet_sender.send(packet).await.unwrap();
            });
        });
    }

    pub fn try_recv_packet(&self) -> Option<<Codec as Decode>::Item> {
        match self.selfbound_packet_receiver.try_recv() {
            Ok(packet) => Some(packet),
            Err(_) => None,
        }
    }
}

impl<Codec> NetworkResource<Codec>
where
    Codec: Decode + Encode + Any + Send + Sync,
    <Codec as Decode>::Item: Send,
    <Codec as Encode>::Item: Send,
{
    /// System that pulls [`NetworkEvent`]s from the internal channel and
    /// forwards them through an [`EventWriter`].
    pub(crate) fn send_network_events(
        net_resource: Res<NetworkResource<Codec>>,
        mut event_writer: EventWriter<NetworkEvent<Codec>>,
    ) {
        while let Ok(event) = net_resource.network_event_receiver.try_recv() {
            event_writer.send(event);
        }
    }
}
