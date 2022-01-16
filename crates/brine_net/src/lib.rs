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

mod connection;
mod event;
mod plugin;
mod resource;

pub mod codec;

pub use event::NetworkEvent;
pub use plugin::NetworkPlugin;
pub use resource::NetworkResource;
