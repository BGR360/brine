use std::{any::Any, fmt::Debug};

use async_channel::{Receiver, Sender};
use async_codec::{Decode, Encode, Framed, ReadFrameError, WriteFrameError};
use async_net::TcpStream;
use bevy::log;
use futures::{FutureExt, SinkExt, StreamExt};

use crate::{event::NetworkError, resource::NetworkResource, NetworkEvent};

/// Internal utility struct responsible for running
pub(crate) struct Connection<Codec: Decode + Encode> {
    network_event_sender: Sender<NetworkEvent<Codec>>,
    peerbound_packet_receiver: Receiver<<Codec as Encode>::Item>,
    selfbound_packet_sender: Sender<<Codec as Decode>::Item>,
}

impl<Codec> Connection<Codec>
where
    Codec: Decode + Encode + Default + Any + Unpin + Send + 'static,
    <Codec as Decode>::Item: Debug + Send + 'static,
    <Codec as Encode>::Item: Debug + Send + 'static,
    <Codec as Decode>::Error: Debug + Send + 'static,
    <Codec as Encode>::Error: Debug + Send + 'static,
{
    pub(crate) fn new(net_resource: &NetworkResource<Codec>) -> Self {
        Self {
            network_event_sender: net_resource.network_event_sender.clone(),
            peerbound_packet_receiver: net_resource.peerbound_packet_receiver.clone(),
            selfbound_packet_sender: net_resource.selfbound_packet_sender.clone(),
        }
    }

    async fn send_event(&self, event: NetworkEvent<Codec>) {
        self.network_event_sender.send(event).await.unwrap();
    }

    async fn send_error(&self, error: NetworkError) {
        self.send_event(NetworkEvent::Error(error)).await;
    }

    /// Connects to a remote host and runs two background tasks to encode and
    /// decode network packets.
    pub(crate) async fn connect_and_run(self, peer_addr: String) {
        log::info!("Connecting to {} ...", &peer_addr);

        let tcp_stream = match TcpStream::connect(peer_addr.clone()).await {
            Ok(stream) => stream,
            Err(err) => {
                self.send_error(NetworkError::ConnectFailed(err)).await;
                return;
            }
        };

        log::info!("Connected to {}", &peer_addr);

        self.send_event(NetworkEvent::Connected).await;

        let peerbound_future = self.run_peerbound(tcp_stream.clone()).fuse();
        let selfbound_future = self.run_selfbound(tcp_stream).fuse();

        futures::pin_mut!(peerbound_future, selfbound_future);
        futures::select! {
            _ = peerbound_future => {
                log::info!("");
            }
            _ = selfbound_future => {
                log::info!("");
            }
        };

        log::info!("Disconnected from {}", &peer_addr);

        self.send_event(NetworkEvent::Disconnected).await;
    }

    /// Run the half of the connection that encodes packets destined for the
    /// remote host.
    async fn run_peerbound(&self, tcp_stream: TcpStream) {
        log::trace!("peerbound writer task: starting");

        let mut codec_writer = Framed::new(tcp_stream, Codec::default());

        loop {
            let peerbound_packet = self.peerbound_packet_receiver.recv().await.unwrap();

            log::trace!("peerbound writer task: {:?}", &peerbound_packet);

            match codec_writer.send(peerbound_packet).await {
                Ok(_) => codec_writer.flush().await.unwrap(),
                Err(WriteFrameError::Io(err)) => {
                    self.send_error(NetworkError::TransportError(err)).await;
                }
                Err(err) => log::error!("Codec error: {:?}", err),
            }
        }
    }

    /// Runs the half of the connection that decodes packets destined for the
    /// local host.
    async fn run_selfbound(&self, tcp_stream: TcpStream) {
        log::trace!("selfbound reader task: starting");

        let mut codec_reader = Framed::new(tcp_stream.clone(), Codec::default());

        loop {
            let selfbound_packet = codec_reader.next().await;

            log::trace!("selfbound reader task: {:?}", &selfbound_packet);

            if let Some(packet) = selfbound_packet {
                match packet {
                    Ok(packet) => self.selfbound_packet_sender.send(packet).await.unwrap(),
                    Err(ReadFrameError::Io(err)) => {
                        self.send_error(NetworkError::TransportError(err)).await
                    }
                    Err(err) => log::error!("Codec error: {:?}", err),
                }
            } else {
                log::info!("Remote host terminated the connection.");
                return;
            }
        }
    }
}
