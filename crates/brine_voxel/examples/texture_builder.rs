use bevy::prelude::*;

use brine_data::{block::BlockStateId, MinecraftData};
use brine_voxel::texture::{get_texture_path, BlockTextures, TextureBuilderPlugin};

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

fn load_atlas(
    mc_data: Res<MinecraftData>,
    mut asset_server: ResMut<AssetServer>,
    mut block_textures: ResMut<BlockTextures>,
    mut atlas: ResMut<Atlas>,
) {
    let block_states = (1..500).map(BlockStateId);

    let atlas_handle =
        block_textures.create_texture_atlas(block_states, &mut asset_server, |block_state| {
            get_texture_path(block_state, &mc_data.blocks)
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
