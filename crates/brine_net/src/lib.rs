//! Customizable two-way TCP networking for Bevy projects.
//!
//! This crate does not depend on any additional async runtime like Tokio; it
//! uses the same runtime provided by [`bevy::tasks`].
//!
//! It is based heavily on the [`async_codec`] crate.
//!
//! # Usage
//!
//! Using this crate starts with defining your **codec**, or how your protocol
//! is encoded and decoded through the network. Do this by defining a type that
//! implements [`async_codec::Encode`] and [`async_codec::Decode`]. The
//! [`async_codec` docs][`async_codec`] provide a good example of this.
//!
//! # Example
//!
//! ```no_run
//! use bevy::prelude::*;
//!
//! use brine_net::{CodecReader, CodecWriter, NetworkEvent, NetworkPlugin, NetworkResource};
//!
//! // `StringCodec` is a simple codec provided by this crate that sends and
//! // receives length-prefixed UTF-8 strings as its packets.
//! use brine_net::codec::StringCodec;
//!
//! // For the purpose of this example, assume we connect to a simple echo server.
//! const SERVER: &str = "my.echo.server:8000";
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(MinimalPlugins)
//!         .add_plugin(NetworkPlugin::<StringCodec>::default())
//!         .add_startup_system(connect)
//!         .add_system(wait_for_connect)
//!         .add_system(read_packets)
//!         .run();
//! }
//!
//! fn connect(
//!     // Use the `NetworkResource` to establish the connection.
//!     // A `NetworkEvent` will fire when the connection has been established.
//!     mut net_resource: ResMut<NetworkResource<StringCodec>>
//! ) {
//!     println!("Connecting to {} ...", SERVER);
//!     net_resource.connect(SERVER.to_string());
//! }
//!
//! fn wait_for_connect(
//!     // Non-packet events are sent as `NetworkEvents` and read using a normal
//!     // Bevy `EventReader`.
//!     mut event_reader: EventReader<NetworkEvent<StringCodec>>,
//!     // Packets can be sent using the `CodecWriter`.
//!     mut codec_writer: CodecWriter<StringCodec>,
//! ) {
//!     for event in event_reader.iter() {
//!         // Let's send a single string once the connection is established.
//!         if let NetworkEvent::Connected = event {
//!             println!("Connection established!");
//!
//!             let packet = String::from("hello world!");
//!
//!             println!("Client sending packet: {}", &packet);
//!             codec_writer.send(packet);
//!         }
//!     }
//! }
//!
//! fn read_packets(
//!     // Packets can be read using the `CodecReader`
//!     mut codec_reader: CodecReader<StringCodec>,
//! ) {
//!     for packet in codec_reader.iter() {
//!         println!("Client received packet: {}", packet);
//!     }
//! }
//! ```
//!
//! If the above app connects to an echo server, then we would see the following
//! output:
//!
//! ```txt
//! Connecting to my.echo.server:8000 ...
//! Connection established!
//! Client sending packet: hello world!
//! Client received packet: hello world!
//! ```

mod connection;
mod event;
mod plugin;
mod resource;
mod system_param;

pub mod codec;

pub use event::NetworkEvent;
pub use plugin::{CodecReader, CodecWriter, NetworkPlugin};
pub use resource::NetworkResource;
