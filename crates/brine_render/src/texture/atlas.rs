use bevy::{
    asset::{Assets, Handle},
    log::*,
    math::Vec2,
    reflect::TypeUuid,
    render::texture::Image,
    sprite::Rect,
    utils::HashMap,
};

use brine_asset::storage::TextureKey;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "3e8bc6e9-b91f-4f11-81ef-105ec53fa370"]
pub struct TextureAtlas {
    /// The handle to the stitched texture atlas.
    pub texture: Handle<Image>,

    /// Mapping from texture key to UV coordinate within the atlas (`0.0` to
    /// `1.0` scale).
    pub regions: HashMap<TextureKey, Rect>,

    /// The texture atlas will always contain a placeholder texture in one of
    /// the regions. This stores that region.
    pub placeholder_region: Rect,
}

impl TextureAtlas {
    /// Returns the UV coordinates within the stitched atlas at which the given
    /// texture can be found.
    ///
    /// If the given texture is not in the atlas, the UV coordinates will
    /// correspond to some placeholder texture in the atlas.
    pub fn get_uv(&self, texture: TextureKey) -> Rect {
        self.regions
            .get(&texture)
            .copied()
            .unwrap_or(self.placeholder_region)
    }

    pub fn stitch<'a, T>(
        assets: &mut Assets<Image>,
        textures: T,
        placeholder_texture: &Handle<Image>,
    ) -> Self
    where
        T: IntoIterator<Item = (TextureKey, &'a Handle<Image>)>,
    {
        let mut textures: Vec<(TextureKey, &Handle<Image>)> = textures.into_iter().collect();

        debug!("Stitching texture atlas with {} textures", textures.len());

        trace!("Sorting textures by size");

        // Sort textures by their longest side, to optimize placement.
        textures.sort_by_cached_key(|(_, handle)| {
            let image = assets.get(*handle).expect("All textures must be loaded");
            let size = image.texture_descriptor.size;
            std::cmp::max(size.width, size.height)
        });

        let mut builder = bevy::sprite::TextureAtlasBuilder::default();

        trace!("Adding textures to builder");

        // Place the largest textures first.
        for (_, handle) in textures.iter().rev() {
            let image = assets.get(*handle).unwrap();
            builder.add_texture(handle.clone_weak(), image);
        }

        trace!("Stitching atlas");

        let bevy_atlas = builder.finish(assets).unwrap();

        trace!("Mapping keys to regions");

        let atlas_image = assets.get(&bevy_atlas.texture).unwrap();
        let atlas_size = atlas_image.texture_descriptor.size;
        let atlas_size = Vec2::new(atlas_size.width as f32, atlas_size.height as f32);

        let handle_to_uv = |handle: &Handle<Image>| {
            let index = bevy_atlas.get_texture_index(handle).unwrap();
            let pixel_rect = bevy_atlas.textures[index];
            Rect {
                min: pixel_rect.min / atlas_size,
                max: pixel_rect.max / atlas_size,
            }
        };

        let key_to_uv = textures
            .iter()
            .map(|(key, handle)| (*key, handle_to_uv(handle)))
            .collect();

        let placeholder_uv = handle_to_uv(placeholder_texture);

        debug!(
            "Done. Final atlas size: {} x {}",
            atlas_size.x as u32, atlas_size.y as u32
        );

        Self {
            texture: bevy_atlas.texture,
            regions: key_to_uv,
            placeholder_region: placeholder_uv,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PendingAtlas {
    /// Strong handle to each texture that will eventually be added to the atlas.
    pub textures: Vec<(TextureKey, Handle<Image>)>,

    /// Strong handle that we will eventually populate with a built atlas.
    // TODO: should be weak?
    pub handle: Handle<TextureAtlas>,
}

impl PendingAtlas {
    pub fn all_textures_loaded(&self, assets: &Assets<Image>) -> bool {
        self.textures
            .iter()
            .all(|(_, handle)| assets.contains(handle))
    }
}
