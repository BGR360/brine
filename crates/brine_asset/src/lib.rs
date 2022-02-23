#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]

pub mod api;
pub mod bakery_v2;

pub use api::{BlockFace, MinecraftAssets};
pub use bakery_v2::{
    block_states::BakedBlockStateTable,
    models::{BakedModel, BakedModelKey, BakedModelTable, BakedQuad},
    textures::{TextureKey, TextureTable},
};
