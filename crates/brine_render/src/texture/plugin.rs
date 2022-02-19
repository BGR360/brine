use bevy::prelude::*;

use crate::texture::{TextureAtlas, TextureManager};

pub struct TextureManagerPlugin;

impl Plugin for TextureManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextureManager>();
        app.add_asset::<TextureAtlas>();
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
