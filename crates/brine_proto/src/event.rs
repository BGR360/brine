//! Events exported from this crate.
//!
//! These events define a high-level API for a Minecraft client that is
//! compatible with multiple different server versions.
//!
//! The protocol defined by these events is similar but far from identical to
//! the actual Minecraft protocol defined at <https://wiki.vg/Protocol>. This
//! API is much more high-level, and the "back-end" is concerned with speaking
//! the actual protocol and converting to and from this higher-level API.

pub use uuid::Uuid;

pub mod serverbound {
    //! Definitions for all variants of [`ServerboundEvent`][super::ServerboundEvent].

    #[allow(unused)]
    use super::clientbound;

    /// Initiates login for the given user on the given server.
    ///
    /// The protocol backend handles the entire login exchange.
    ///
    /// # See also
    ///
    /// * [`clientbound::LoginSuccess`]
    /// * [`clientbound::Disconnect`]
    #[derive(Debug, Clone, PartialEq)]
    pub struct Login {
        /// Hostname or IP address of the server.
        pub server: String,

        /// Username being used to join the game.
        pub username: String,
    }

    pub(crate) fn add_events(app: &mut bevy_app::App) {
        app.add_event::<Login>();
    }
}

pub mod clientbound {
    //! Definitions for all variants of [`ClientboundEvent`][super::ClientboundEvent].

    #[allow(unused)]
    use super::serverbound;

    /// Notifies the client that they have successfully logged in to the server.
    ///
    /// # See also
    ///
    /// * [`serverbound::Login`]
    #[derive(Debug, Clone, PartialEq)]
    pub struct LoginSuccess {
        /// UUID assigned by the server to this client.
        pub uuid: uuid::Uuid,

        /// Username that was used to join the game.
        pub username: String,
    }

    /// Notifies the client they have been disconnected from the server.
    ///
    /// This could happen for a number of reasons:
    /// * Login failure.
    /// * User is kicked from the server.
    /// * Backend fails to keep the connection alive.
    /// * Generic networking error.
    /// * etc...
    #[derive(Debug, Clone, PartialEq)]
    pub struct Disconnect {
        /// Human-readable reason for why the disconnect occurred.
        pub reason: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ChunkData {
        pub chunk_data: brine_chunk::Chunk,
    }

    pub(crate) fn add_events(app: &mut bevy_app::App) {
        app.add_event::<LoginSuccess>();
        app.add_event::<Disconnect>();
        app.add_event::<ChunkData>();
    }
}
