use bevy::prelude::*;

use brine_asset::{storage::TextureKey, MinecraftAssets};

use crate::texture::{TextureAtlas, TextureManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinecraftTexturesState {
    Loading,
    Loaded,
}

pub struct MinecraftTexturesPlugin;

impl Plugin for MinecraftTexturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(MinecraftTexturesState::Loading);
        app.init_resource::<TheAtlas>();
        // app.add_startup_system(setup);
        app.add_system_set(SystemSet::on_enter(MinecraftTexturesState::Loading).with_system(setup));
        app.add_system_set(
            SystemSet::on_update(MinecraftTexturesState::Loading).with_system(await_loaded),
        );
    }
}

#[derive(Default)]
struct TheAtlas {
    handle: Handle<TextureAtlas>,
}

fn get_all_textures<'a>(
    mc_assets: &'a MinecraftAssets,
    asset_server: &'a AssetServer,
) -> impl Iterator<Item = (TextureKey, Handle<Image>)> + 'a {
    mc_assets
        .textures()
        .iter()
        .filter_map(|(texture_key, texture_id)| {
            trace!("{texture_key:?}: {texture_id:?}");

            if texture_id.path().starts_with("block/")
                || texture_id.path().starts_with("effect/")
                || texture_id.path().starts_with("item/")
                || texture_id.path().starts_with("mob_effect/")
                || texture_id.path().starts_with("painting/")
                || texture_id.path().starts_with("particle/")
            {
                let path = mc_assets.get_texture_path(texture_key).unwrap();
                let handle = asset_server.load(path);
                Some((texture_key, handle))
            } else {
                None
            }
        })
}

/// This system kicks off the creation of the texture atlas(es).
fn setup(
    mc_assets: Res<MinecraftAssets>,
    asset_server: Res<AssetServer>,
    mut the_atlas: ResMut<TheAtlas>,
    mut texture_manager: ResMut<TextureManager>,
) {
    let textures = get_all_textures(&*mc_assets, &*asset_server);

    let atlas_handle = texture_manager.create_atlas(&*asset_server, textures);
    the_atlas.handle = atlas_handle;
}

/// This system advances the state to `Loaded` once the texture atlas(es) is/are available.
fn await_loaded(
    the_atlas: Res<TheAtlas>,
    atlases: Res<Assets<TextureAtlas>>,
    mut state: ResMut<State<MinecraftTexturesState>>,
) {
    if atlases.contains(&the_atlas.handle) {
        state.set(MinecraftTexturesState::Loaded).unwrap();
    }
}
