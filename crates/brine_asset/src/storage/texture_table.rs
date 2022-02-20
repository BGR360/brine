use indexmap::IndexSet;
use minecraft_assets::api::ResourceIdentifier;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureKey(pub usize);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextureTable {
    textures: IndexSet<ResourceIdentifier<'static>>,
}

impl TextureTable {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (TextureKey, &ResourceIdentifier<'static>)> {
        self.textures
            .iter()
            .enumerate()
            .map(|(index, id)| (TextureKey(index), id))
    }

    #[inline]
    pub fn insert(&mut self, id: ResourceIdentifier<'static>) -> TextureKey {
        let (index, _) = self.textures.insert_full(id);

        TextureKey(index)
    }

    #[inline]
    pub fn get_by_key(&self, key: TextureKey) -> Option<&ResourceIdentifier<'_>> {
        self.textures.get_index(key.0)
    }

    #[inline]
    pub fn get_key(&self, name: &ResourceIdentifier) -> Option<TextureKey> {
        self.textures.get_index_of(name).map(TextureKey)
    }
}
