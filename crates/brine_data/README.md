# brine_data

Provides access to Minecraft data for any version, tailored for use in games.
Data is baked into the library and does not require Internet or file system
access.

This crate uses the
[`minecraft-data-rs`](https://github.com/Trivernis/minecraft-data-rs) crate as
the source of Minecraft data, and re-exports it in an API suitable for
high-performance applications such as games.