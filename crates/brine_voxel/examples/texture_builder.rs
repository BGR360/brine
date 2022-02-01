use std::path::PathBuf;

use bevy::prelude::*;

use minecraft_assets::{
    api::{AssetPack, ModelResolver, ResourceLocation},
    schemas::BlockStates,
};

use brine_data::{block::BlockStateId, MinecraftData};
use brine_voxel::texture::{BlockTextures, TextureBuilderPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MinecraftData::for_version("1.14.4"))
        .add_plugin(TextureBuilderPlugin)
        .add_state(AppState::Loading)
        .init_resource::<Atlas>()
        .add_startup_system(load_atlas)
        .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(load_atlas))
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(check_atlas))
        .add_system_set(SystemSet::on_enter(AppState::Finished).with_system(setup))
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Loading,
    Finished,
}

#[derive(Default)]
struct Atlas {
    handle: Option<Handle<TextureAtlas>>,
}

fn get_texture_path(mc_data: &MinecraftData, block_state_id: BlockStateId) -> Option<PathBuf> {
    let assets = AssetPack::at_path("assets/1.14.4");

    let block = mc_data.blocks.get_block_by_state_id(block_state_id)?;
    let name = &block.name;
    let blockstates = assets.load_blockstates(name).ok()?;

    let first_variant = match blockstates {
        BlockStates::Variants { ref variants } => variants.values().next().unwrap(),
        BlockStates::Multipart { ref cases } => &cases[0].apply,
    };

    let model_name = &first_variant.models()[0].model;

    if model_name.contains("water") || model_name.contains("lava") || model_name.contains("fire") {
        return None;
    }

    let models = assets.load_block_model_recursive(model_name).ok()?;

    let model = ModelResolver::resolve_model(models.iter());

    let texture = &model.textures.as_ref()?.values().next()?.0;

    let path = assets.get_resource_path(&ResourceLocation::Texture(texture.into()));

    Some(path.strip_prefix("assets").unwrap().into())
}

fn load_atlas(
    mc_data: Res<MinecraftData>,
    asset_server: ResMut<AssetServer>,
    mut block_textures: ResMut<BlockTextures>,
    mut atlas: ResMut<Atlas>,
) {
    let block_states = (1..1000).map(BlockStateId);

    let atlas_handle = block_textures.create_texture_atlas(block_states, &asset_server, |b| {
        get_texture_path(&*mc_data, b)
    });

    atlas.handle = Some(atlas_handle);
}

fn check_atlas(
    atlas: Res<Atlas>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut app_state: ResMut<State<AppState>>,
) {
    if texture_atlases.contains(atlas.handle.as_ref().unwrap()) {
        app_state.set(AppState::Finished).unwrap();
    }
}

fn setup(atlas: Res<Atlas>, texture_atlases: Res<Assets<TextureAtlas>>, mut commands: Commands) {
    let texture_atlas = texture_atlases.get(atlas.handle.as_ref().unwrap()).unwrap();

    let texture_atlas_texture = texture_atlas.texture.clone();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(SpriteBundle {
        texture: texture_atlas_texture,
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE * 2.0),
        ..Default::default()
    });
}
