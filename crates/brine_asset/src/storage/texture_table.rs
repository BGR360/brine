use std::path::PathBuf;

use indexmap::IndexMap;
use minecraft_assets::api::ResourceIdentifier;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Texture {
    pub path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TextureKey(usize);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextureTable {
    textures: IndexMap<ResourceIdentifier<'static>, Texture>,
}

impl TextureTable {
    #[inline]
    pub fn insert(&mut self, name: &ResourceIdentifier, texture: Texture) -> TextureKey {
        let (index, _) = self.textures.insert_full(name.into_owned(), texture);

        TextureKey(index)
    }

    #[inline]
    pub fn get_by_key(&self, key: TextureKey) -> Option<&Texture> {
        self.textures
            .get_index(key.0)
            .map(|(_name, texture)| texture)
    }

    #[inline]
    pub fn get_key(&self, name: &ResourceIdentifier) -> Option<TextureKey> {
        self.textures.get_index_of(name).map(TextureKey)
    }
}
