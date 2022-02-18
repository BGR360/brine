use bevy::{app::prelude::*, asset::prelude::*, ecs::prelude::*, render::texture::Image};

use crate::texture::{TextureAtlas, TextureManager};

const PLACEHOLDER_PATH: &str = "placeholder.png";

pub struct TextureManagerPlugin;

impl Plugin for TextureManagerPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world.get_resource::<AssetServer>().unwrap();
        let placeholder_texture = asset_server.load(PLACEHOLDER_PATH);

        let manager = TextureManager::new(placeholder_texture);
        app.insert_resource(manager);

        app.add_system(stitch_pending_atlases);
    }
}

fn stitch_pending_atlases(
    mut manager: ResMut<TextureManager>,
    mut textures: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    manager.try_stitch_pending_atlases(&mut *textures, &mut *atlases);
}
