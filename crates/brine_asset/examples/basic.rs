use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use brine_asset::MinecraftAssets;
use brine_data::MinecraftData;

fn workspace_relative_path(relative_path: impl AsRef<Path>) -> PathBuf {
    let manifest_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut path = PathBuf::from(manifest_path);
    path.push(relative_path);

    path
}

fn main() {
    let data = Arc::new(MinecraftData::for_version("1.14.4"));

    let path = workspace_relative_path("../../assets/1.14.4");
    println!("{}", path.to_string_lossy());

    let _assets = MinecraftAssets::new(path, &data).unwrap();
}
