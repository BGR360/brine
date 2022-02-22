use std::path::{Path, PathBuf};

use minecraft_assets::api::AssetPack;
use tracing::*;

use brine_asset::{
    bakery,
    bakery_v2::{self, models::ModelBakery},
};
use brine_data::MinecraftData;

fn cargo_workspace_relative_path(relative: impl AsRef<Path>) -> PathBuf {
    let manifest_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut path = PathBuf::from(manifest_path);
    path.push(relative);

    path
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_default())
        .init();

    let mc_data = MinecraftData::for_version("1.14.4");
    let asset_pack = AssetPack::at_path(cargo_workspace_relative_path("../../assets/1.14.4"));

    let baked_assets = bakery_v2::bake_all(&mc_data, &asset_pack);

    // println!("{:#?}", baked_assets);
}

fn print_a_few(mc_data: &MinecraftData, asset_pack: &AssetPack) {
    info!("Loading textures");
    let texture_table = bakery::textures::load_texture_ids(&asset_pack).unwrap();

    info!("Loading unbaked modlels");
    let unbaked_models = bakery_v2::models::load_unbaked_block_models(&asset_pack).unwrap();

    trace!(
        "Unbaked models: {:#?}",
        unbaked_models
            .keys()
            .map(|id| id.to_canonical())
            .collect::<Vec<_>>()
    );

    info!("Loading unbaked block states");
    let unbaked_block_states = bakery_v2::block_states::load_unbaked_block_states(&asset_pack);

    let model_bakery = ModelBakery::new(&unbaked_models, &texture_table);

    // print_baked_block(&model_bakery, "stone");
    // print_baked_block(&model_bakery, "grass_block");
    print_baked_block(&model_bakery, "oak_stairs");
}

fn print_baked_block(model_bakery: &ModelBakery, block_name: &str) {
    let baked = model_bakery.bake_model(block_name, false).unwrap();

    info!("{:#?}", baked);
}
