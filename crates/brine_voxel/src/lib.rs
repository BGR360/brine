//! Library for rendering Minecraft worlds.
//!
//! Currently all that is implemented is two different [chunk builders]
//! (["visible faces"] and ["naive blocks"]) that generate meshes from chunk
//! data. The former is implemented using the [`block-mesh`] crate.
//!
//! [chunk builders]: ChunkBuilder
//! ["visible faces"]: VisibleFacesChunkBuilder
//! ["naive blocks"]: NaiveBlocksChunkBuilder
//! [`block-mesh`]: <https://github.com/bonsairobo/block-mesh-rs>

pub mod chunk_builder;
pub mod mesh;
pub mod texture;

pub use chunk_builder::{
    AddToWorld, ChunkBuilder, ChunkBuilderPlugin, NaiveBlocksChunkBuilder, VisibleFacesChunkBuilder,
};
