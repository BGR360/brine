//! Plugins exported by this crate.

use bevy::prelude::*;

use brine_net::{NetworkEvent, NetworkPlugin};

use crate::codec::MinecraftClientCodec;

pub(crate) type ProtocolCodec = MinecraftClientCodec;

/// Minecraft protocol implementation plugin.
///
/// # Events
///
/// The plugin does not register any events.
///
/// The plugin acts on the following events:
///
/// * [`brine_proto::ServerboundEvent`]
///
/// The plugin sends the following events:
///
/// * [`brine_proto::ClientboundEvent`]
///
/// # Resources
///
/// The plugin does not register any resources.
///
/// The plugin does not expect any resources to exist.
pub struct ProtocolBackendPlugin;

impl Plugin for ProtocolBackendPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NetworkPlugin::<ProtocolCodec>::default());

        app.add_system(log_network_errors);

        crate::r#impl::login::build(app);
    }
}

fn log_network_errors(mut event_reader: EventReader<NetworkEvent<ProtocolCodec>>) {
    for event in event_reader.iter() {
        if let NetworkEvent::Error(network_error) = event {
            warn!("Network error: {}", network_error);
        }
    }
}
