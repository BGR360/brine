//! Resources exposed by this crate.

use std::fmt::Debug;

use async_channel::{unbounded, Receiver, Sender};
use async_codec::{Decode, Encode};
use bevy_tasks::{Task, TaskPool};

use crate::{
    connection::Connection,
    event::{NetworkError, NetworkEvent},
};

/// Resource that provides a TCP connection that encodes and decodes
/// packets as specified by the given codec.
pub struct NetworkResource<Codec: Decode + Encode>
where
    <Codec as Decode>::Error: Debug,
    <Codec as Encode>::Error: Debug,
{
    pub(crate) codec: Codec,
    pub(crate) task_pool: TaskPool,
    pub(crate) connection_task: Option<Task<()>>,

    /// Used by background tasks to produce [`NetworkEvent`]s.
    pub(crate) network_event_sender: Sender<NetworkEvent<Codec>>,

    /// Used by the plugin to forward [`NetworkEvent`]s through an
    /// [`EventWriter`][bevy::ecs::event::EventWriter].
    pub(crate) network_event_receiver: Receiver<NetworkEvent<Codec>>,

    /// Used by the [`CodecWriter`][crate::system_param::CodecWriter] to produce
    /// packets destined for the remote host.
    pub(crate) peerbound_packet_sender: Sender<<Codec as Encode>::Item>,

    /// Used by background tasks to consume and encode packets destined for the
    /// remote host.
    pub(crate) peerbound_packet_receiver: Receiver<<Codec as Encode>::Item>,

    /// Used by background tasks to produce packets destined for the local host.
    pub(crate) selfbound_packet_sender: Sender<<Codec as Decode>::Item>,

    /// Used by the plugin to forward packets to the
    /// [`CodecReader`][crate::system_param::CodecReader].
    pub(crate) selfbound_packet_receiver: Receiver<<Codec as Decode>::Item>,
}

impl<Codec> NetworkResource<Codec>
where
    Codec: Decode + Encode + Default + Clone + Unpin + Send + 'static,
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
            codec: Default::default(),
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

    /// Returns a reference to the network resource's codec.
    ///
    /// Can be used to alter parameters of the codec.
    pub fn codec(&self) -> &Codec {
        &self.codec
    }

    /// Establish a connection with a server that speaks this codec.
    ///
    /// The server address argument can be a `<hostname>:<port>` pair or an
    /// `<ip_addr>:<port>` pair (or anything that can be successfully resolved
    /// to one or more IP addresses with
    /// [`ToSocketAddrs`][std::net::ToSocketAddrs]).
    ///
    /// If any error occurs in the process of establishing the connection or
    /// while the connection is active, it will be delivered as a
    /// [`NetworkEvent`][crate::NetworkEvent].
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

            let codec = self.codec.clone();
            self.connection_task = Some(self.task_pool.spawn(async move {
                connection.connect_and_run(server_addr, codec).await;
            }));
        }
    }
}
