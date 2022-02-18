//! Plugins exported by this crate.

use bevy::prelude::*;

use brine_net::{NetworkEvent, NetworkPlugin};

use crate::backend::{self, ProtocolCodec};

/// Minecraft protocol implementation plugin.
///
/// # Events
///
/// The plugin does not register any events.
///
/// The plugin acts on the following events:
///
/// * [`brine_proto::event::serverbound::*`][brine_proto::event::serverbound]
///
/// The plugin sends the following events:
///
/// * [`brine_proto::event::clientbound::*`][brine_proto::event::clientbound]
///
/// # Resources
///
/// The plugin registers a [`NetworkPlugin`] which provides things. See its
/// documentation.
pub struct ProtocolBackendPlugin;

impl Plugin for ProtocolBackendPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NetworkPlugin::<ProtocolCodec>::default());

        app.add_system(log_network_errors);

        backend::build(app);
    }
}

fn log_network_errors(mut event_reader: EventReader<NetworkEvent<ProtocolCodec>>) {
    for event in event_reader.iter() {
        if let NetworkEvent::Error(network_error) = event {
            warn!("Network error: {}", network_error);
        }
    }
}
