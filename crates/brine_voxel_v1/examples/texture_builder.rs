use bevy::prelude::*;

use brine_asset::{BlockFace, MinecraftAssets};

use brine_data::{blocks::BlockStateId, MinecraftData};
use brine_voxel_v1::texture::{BlockTextures, TextureBuilderPlugin};

fn main() {
    let mc_data = MinecraftData::for_version("1.14.4");
    let mc_assets = MinecraftAssets::new("assets/1.14.4", &mc_data).unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(mc_data)
        .insert_resource(mc_assets)
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
    mc_assets: Res<MinecraftAssets>,
    asset_server: ResMut<AssetServer>,
    mut block_textures: ResMut<BlockTextures>,
    mut atlas: ResMut<Atlas>,
) {
    let block_states = (1..500).map(BlockStateId);

    let atlas_handle = block_textures.create_texture_atlas(block_states, &asset_server, |b| {
        mc_assets
            .textures()
            .get_texture_path_for_block_state_and_face(b, BlockFace::South)
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
