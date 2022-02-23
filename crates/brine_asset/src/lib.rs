#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]

pub mod api;
pub mod bakery;

pub use api::{BlockFace, MinecraftAssets};
pub use bakery::{
    block_states::BakedBlockStateTable,
    models::{BakedModel, BakedModelKey, BakedModelTable, BakedQuad},
    textures::{TextureKey, TextureTable},
};
