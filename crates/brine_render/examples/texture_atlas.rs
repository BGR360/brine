use bevy::prelude::*;
use minecraft_assets::api::{ResourceIdentifier, ResourcePath};

use brine_asset::storage::TextureKey;
use brine_render::texture::{TextureAtlas, TextureManager, TextureManagerPlugin};

fn get_a_few_textures(
    asset_server: &AssetServer,
) -> impl Iterator<Item = (TextureKey, Handle<Image>)> + '_ {
    const TEXTURES: &[&str] = &[
        "block/water_still.png",
        "block/campfire_fire.png",
        "block/stone.png",
    ];

    TEXTURES.iter().enumerate().map(|(index, name)| {
        let key = TextureKey(index);
        let loc = ResourceIdentifier::texture(name);
        let path = ResourcePath::for_resource("1.14.4", &loc);
        let handle = asset_server.load(path.as_ref());
        (key, handle)
    })
}

// fn get_all_textures(
//     asset_server: &AssetServer,
// ) -> impl Iterator<Item = (TextureKey, Handle<Image>)> + '_ {
//     let resource_provider = FileSystemResourceProvider::new("assets/1.14.4");

//     resource_provider
//         .enumerate_resources("minecraft", ResourceKind::Texture)
//         .unwrap()
//         .into_iter()
//         .enumerate()
//         .filter_map(|(index, resource_location)| {
//             println!("{index}: {resource_location:?}");
//             if resource_location.path().starts_with("block/")
//                 || resource_location.path().starts_with("effect/")
//                 || resource_location.path().starts_with("item/")
//                 || resource_location.path().starts_with("mob_effect/")
//                 || resource_location.path().starts_with("painting/")
//                 || resource_location.path().starts_with("particle/")
//             {
//                 let key = TextureKey(index);
//                 let path = ResourcePath::for_resource("1.14.4", &resource_location);
//                 let handle = asset_server.load(path.as_ref());
//                 Some((key, handle))
//             } else {
//                 None
//             }
//         })
// }

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<TheAtlas>()
        .add_plugin(TextureManagerPlugin)
        .add_state(AtlasState::Idle)
        .add_system_set(SystemSet::on_update(AtlasState::Idle).with_system(setup))
        .add_system_set(SystemSet::on_update(AtlasState::LoadingTextures).with_system(spawn_sprite))
        .run();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum AtlasState {
    Idle,
    LoadingTextures,
    Stitched,
}

#[derive(Default)]
struct TheAtlas {
    handle: Handle<TextureAtlas>,
}

fn setup(
    asset_server: Res<AssetServer>,
    mut texture_manager: ResMut<TextureManager>,
    mut the_atlas: ResMut<TheAtlas>,
    mut commands: Commands,
    mut state: ResMut<State<AtlasState>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_keys_and_handles = get_a_few_textures(&*asset_server);

    let atlas_handle = texture_manager.create_atlas(&*asset_server, texture_keys_and_handles);

    the_atlas.handle = atlas_handle;

    state.set(AtlasState::LoadingTextures).unwrap();
}

fn spawn_sprite(
    atlases: Res<Assets<TextureAtlas>>,
    the_atlas: Res<TheAtlas>,
    mut commands: Commands,
    mut state: ResMut<State<AtlasState>>,
) {
    if let Some(atlas) = atlases.get(&the_atlas.handle) {
        println!("Atlas stitched. Spawning sprite.");

        commands.spawn().insert_bundle(SpriteBundle {
            texture: atlas.texture.clone(),
            // transform: Transform::from_scale(Vec3::ONE * 0.5),
            ..Default::default()
        });

        state.set(AtlasState::Stitched).unwrap();
    }
}
