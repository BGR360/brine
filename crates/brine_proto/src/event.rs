//! Events exported from this crate.
//!
//! The [`ServerboundEvent`] and [`ClientboundEvent`] enums define an API that
//! plugins use to respond to Minecraft server packets and send client packets.
//!
//! The protocol defined by these events is similar but not identical to the
//! actual Minecraft protocol defined at <https://wiki.vg/Protocol>. This API
//! is a little more high-level, and the "back-end" is concerned with speaking
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

    /// Notifies the client that they failed to log in to the server.
    ///
    /// # See also
    ///
    /// * [`serverbound::Login`]
    #[derive(Debug, Clone, PartialEq)]
    pub struct LoginFailure {
        /// Human-readable reason for why the login failed.
        pub reason: String,
    }

    pub(crate) fn add_events(app: &mut bevy_app::App) {
        app.add_event::<LoginSuccess>();
        app.add_event::<LoginFailure>();
    }
}
