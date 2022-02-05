#![doc = include_str!("../README.md")]

pub(crate) use minecraft_data_rs::api::Api;

pub mod blocks;

mod data;
mod version;

pub use blocks::Blocks;
pub use data::MinecraftData;
pub use version::Version;
