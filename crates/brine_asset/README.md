# brine_asset

Provides access to Minecraft assets and resource packs for any version, tailored
for use in games.

This crate uses the
[`minecraft-assets`](https://github.com/BGR360/minecraft-assets-rs) crate as the
mechanism for parsing the data files in the `assets/` directory or in a resource
pack, and re-exports this information in an API suitable for high-performance
applications such as games.
