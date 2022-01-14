use crate::event::{ClientboundEvent, ServerboundEvent};

use bevy::prelude::*;

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
        app.add_event::<ClientboundEvent>();
        app.add_event::<ServerboundEvent>();
    }
}
