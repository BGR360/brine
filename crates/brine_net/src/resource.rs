//! Resources exposed by this crate.

use std::{any::Any, marker::PhantomData};

use async_net::TcpStream;
use bevy::{
    ecs::event::EventWriter,
    log,
    tasks::{Task, TaskPool},
};
use crossbeam_channel::{Receiver, Sender};

use crate::event::{NetworkError, NetworkEvent};

pub struct NetworkResource<Codec> {
    task_pool: TaskPool,
    connection_task: Option<Task<()>>,
    network_event_sender: Sender<NetworkEvent<Codec>>,
    network_event_receiver: Receiver<NetworkEvent<Codec>>,
    _phantom: PhantomData<Codec>,
}

impl<Codec> NetworkResource<Codec>
where
    Codec: Send + 'static,
{
    /// Establish a connection with a server that speaks this codec.
    ///
    /// The server address argument can be a hostname or an IP address.
    ///
    /// If any error occurs in the process of establishing the connection, this
    /// will be delivered as a [`NetworkEvent`][crate::NetworkEvent].
    pub fn connect(&mut self, server_addr: String) {
        if self.connection_task.is_some() {
            self.network_event_sender
                .send(NetworkEvent::Error(NetworkError::AlreadyConnected))
                .unwrap();
        } else {
            let event_sender = self.network_event_sender.clone();
            self.connection_task = Some(self.task_pool.spawn(async move {
                client_connection(server_addr, event_sender.clone())
                    .await
                    .map_err(|err| event_sender.send(NetworkEvent::Error(err)).unwrap())
                    .ok();
            }));
        }
    }
}

impl<Codec> NetworkResource<Codec>
where
    Codec: Any + Send + Sync,
{
    pub(crate) fn new(task_pool: TaskPool) -> Self {
        let (network_event_sender, network_event_receiver) = crossbeam_channel::unbounded();
        Self {
            task_pool,
            connection_task: None,
            network_event_sender,
            network_event_receiver,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn send_network_events(
        &mut self,
        mut event_writer: EventWriter<NetworkEvent<Codec>>,
    ) {
        let events = self.network_event_receiver.try_iter();
        event_writer.send_batch(events);
    }
}

async fn client_connection<Codec>(
    server_addr: String,
    event_sender: Sender<NetworkEvent<Codec>>,
) -> Result<(), NetworkError> {
    log::info!("Connecting to {}...", &server_addr);

    let _stream = TcpStream::connect(server_addr.clone())
        .await
        .map_err(NetworkError::ConnectFailed)?;

    log::info!("Connected to {}.", &server_addr,);

    event_sender.send(NetworkEvent::Connected).unwrap();

    Ok(())
}
