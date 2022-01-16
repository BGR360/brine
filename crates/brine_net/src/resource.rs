//! Resources exposed by this crate.

use std::{any::Any, fmt::Debug, marker::PhantomData};

use async_channel::{unbounded, Receiver, Sender};
use async_codec::{Decode, Encode, Framed, ReadFrameError, WriteFrameError};
use async_net::TcpStream;
use bevy::{
    ecs::event::EventWriter,
    log,
    tasks::{Task, TaskPool},
};
use futures::{SinkExt, StreamExt};

use crate::event::{NetworkError, NetworkEvent};

pub struct NetworkResource<Codec: Decode + Encode> {
    task_pool: TaskPool,
    connection_task: Option<Task<()>>,
    network_event_sender: Sender<NetworkEvent<Codec>>,
    network_event_receiver: Receiver<NetworkEvent<Codec>>,
    serverbound_packet_sender: Sender<<Codec as Encode>::Item>,
    serverbound_packet_receiver: Receiver<<Codec as Encode>::Item>,
    clientbound_packet_sender: Sender<<Codec as Decode>::Item>,
    clientbound_packet_receiver: Receiver<<Codec as Decode>::Item>,
    _phantom: PhantomData<Codec>,
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
        let (serverbound_packet_sender, serverbound_packet_receiver) = unbounded();
        let (clientbound_packet_sender, clientbound_packet_receiver) = unbounded();

        Self {
            task_pool,
            connection_task: None,
            network_event_sender,
            network_event_receiver,
            serverbound_packet_sender,
            serverbound_packet_receiver,
            clientbound_packet_sender,
            clientbound_packet_receiver,
            _phantom: PhantomData,
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
            let task_pool = self.task_pool.clone();
            let event_sender = self.network_event_sender.clone();
            let serverbound_receiver = self.serverbound_packet_receiver.clone();
            let clientbound_sender = self.clientbound_packet_sender.clone();

            self.connection_task = Some(self.task_pool.spawn(async move {
                client_connection(
                    task_pool,
                    server_addr,
                    event_sender,
                    serverbound_receiver,
                    clientbound_sender,
                )
                .await
            }));
        }
    }

    // TODO: switch to events (encoding only needs reference to packet)
    pub fn send_packet(&mut self, packet: <Codec as Encode>::Item) {
        let sender = &mut self.serverbound_packet_sender;
        self.task_pool.scope(|scope| {
            scope.spawn(async move {
                sender.send(packet).await.unwrap();
            });
        });
    }

    pub fn try_recv_packet(&mut self) -> Option<<Codec as Decode>::Item> {
        match self.clientbound_packet_receiver.try_recv() {
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
    pub(crate) fn send_network_events(
        &mut self,
        mut event_writer: EventWriter<NetworkEvent<Codec>>,
    ) {
        while let Ok(event) = self.network_event_receiver.try_recv() {
            event_writer.send(event);
        }
    }
}

// TODO: move this into a class, put all the trait bounds on the impl block,
// and split this into more functions.
async fn client_connection<Codec>(
    task_pool: TaskPool,
    server_addr: String,
    event_sender: Sender<NetworkEvent<Codec>>,
    serverbound_packet_receiver: Receiver<<Codec as Encode>::Item>,
    clientbound_packet_sender: Sender<<Codec as Decode>::Item>,
) where
    Codec: Decode + Encode + Default + Unpin + Send + 'static,
    <Codec as Decode>::Item: Debug + Send + 'static,
    <Codec as Encode>::Item: Debug + Send + 'static,
    <Codec as Decode>::Error: Debug + Send + 'static,
    <Codec as Encode>::Error: Debug + Send + 'static,
{
    log::info!("Connecting to {} ...", &server_addr);

    let tcp_stream = match TcpStream::connect(server_addr.clone()).await {
        Ok(stream) => stream,
        Err(err) => {
            event_sender
                .send(NetworkEvent::Error(NetworkError::ConnectFailed(err)))
                .await
                .unwrap();
            return;
        }
    };

    log::info!("Connected to {}", &server_addr);

    event_sender.send(NetworkEvent::Connected).await.unwrap();

    // Spawn serverbound writer task.
    let tcp_stream_for_writer = tcp_stream.clone();
    let event_sender_for_writer = event_sender.clone();
    task_pool
        .spawn(async move {
            log::trace!("Serverbound writer task: starting");

            let mut codec_writer = Framed::new(tcp_stream_for_writer, Codec::default());

            loop {
                let serverbound_packet = serverbound_packet_receiver.recv().await.unwrap();

                log::trace!("Serverbound writer task: {:?}", &serverbound_packet);

                match codec_writer.send(serverbound_packet).await {
                    Ok(_) => codec_writer.flush().await.unwrap(),
                    Err(WriteFrameError::Io(err)) => {
                        event_sender_for_writer
                            .send(NetworkEvent::Error(NetworkError::TransportError(err)))
                            .await
                            .unwrap();
                    }
                    Err(err) => log::error!("Codec error: {:?}", err),
                }
            }
        })
        .detach();

    // Spawn clientbound reader task.
    let tcp_stream_for_reader = tcp_stream.clone();
    let event_sender_for_reader = event_sender.clone();
    task_pool
        .spawn(async move {
            log::trace!("Clientbound reader task: starting");

            let mut codec_reader = Framed::new(tcp_stream_for_reader, Codec::default());

            loop {
                let clientbound_packet = codec_reader.next().await;

                log::trace!("Clientbound reader task: {:?}", &clientbound_packet);

                if let Some(packet) = clientbound_packet {
                    match packet {
                        Ok(packet) => clientbound_packet_sender.send(packet).await.unwrap(),
                        Err(ReadFrameError::Io(err)) => event_sender_for_reader
                            .send(NetworkEvent::Error(NetworkError::TransportError(err)))
                            .await
                            .unwrap(),
                        Err(err) => log::error!("Codec error: {:?}", err),
                    }
                } else {
                    event_sender_for_reader
                        .send(NetworkEvent::Disconnected)
                        .await
                        .unwrap();
                    return;
                }
            }
        })
        .detach();
}
