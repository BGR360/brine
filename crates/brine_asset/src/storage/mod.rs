//! Data structures for quick runtime access to asset data.

mod model_table;
mod texture_table;

pub use model_table::{Model, ModelKey, ModelTable};
pub use texture_table::{Texture, TextureKey, TextureTable};
