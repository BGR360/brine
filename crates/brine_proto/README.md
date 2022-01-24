# brine_proto

This crate is the glue between the main Brine application and whatever backend
is implementing the client-server communication protocol.

Its intention is to provide a high-level abstraction of the Minecraft game logic
that is not specific to any one game version, thereby allowing the same client
application to be reused with a different backend (say, for Bedrock Edition).

It is **far** from complete at this time.

## Map

The Brine protocol is defined by a set of clientbound and serverbound events in
[`event.rs`](src/event.rs).

The `ProtocolPlugin` that registers these event types is defined in
[`plugin/protocol.rs`](src/plugin/protocol.rs).