use bevy::prelude::*;

use brine_asset::MinecraftAssets;
use brine_data::MinecraftData;
use brine_render::texture::{
    MinecraftTexturesPlugin, MinecraftTexturesState, TextureAtlas, TextureManager,
    TextureManagerPlugin,
};

fn main() {
    let mc_data = MinecraftData::for_version("1.14.4");

    println!("Loading asset metadata");
    let mc_assets = MinecraftAssets::new("assets/1.14.4", &mc_data).unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(mc_assets)
        .add_plugin(TextureManagerPlugin)
        .add_plugin(MinecraftTexturesPlugin)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(MinecraftTexturesState::Loaded).with_system(spawn_sprite),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_sprite(
    texture_manager: Res<TextureManager>,
    atlases: Res<Assets<TextureAtlas>>,
    mut commands: Commands,
) {
    println!("Atlas stitched. Spawning sprite.");

    let atlas_handle = texture_manager.atlases().next().unwrap();

    let atlas = atlases.get(atlas_handle).unwrap();

    commands.spawn().insert_bundle(SpriteBundle {
        texture: atlas.texture.clone(),
        transform: Transform::from_scale(Vec3::ONE * 0.5),
        ..Default::default()
    });
}
