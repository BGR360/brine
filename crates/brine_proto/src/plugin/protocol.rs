use bevy_app::prelude::*;

use crate::event;

/// Protocol "front-end" plugin.
///
/// This plugin does not perform any actual protocol communication with a server.
/// It needs to be paired with a
///
/// # Events
///
/// The plugin registers the following event types:
///
/// * [`ClientboundEvent`]
/// * [`ServerboundEvent`]
///
/// The plugin does not react to any events.
///
/// The plugin does not send any events.
///
/// # Resources
///
/// The plugin registers no resources.
///
/// The plugin expects no resources to exist.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        event::serverbound::add_events(app);
        event::clientbound::add_events(app);
    }
}
