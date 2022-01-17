use brine_proto::{ClientboundEvent, ServerboundEvent};

use crate::codec::{ClientboundPacket, ServerboundPacket};

pub trait ToPacket<T> {
    fn to_packet(&self) -> Option<T>;
}

pub trait ToEvent<T> {
    fn to_event(&self) -> Option<T>;
}

impl ToPacket<ServerboundPacket> for ServerboundEvent {
    fn to_packet(&self) -> Option<ServerboundPacket> {
        None
    }
}

impl ToEvent<ClientboundEvent> for ClientboundPacket {
    fn to_event(&self) -> Option<ClientboundEvent> {
        None
    }
}
