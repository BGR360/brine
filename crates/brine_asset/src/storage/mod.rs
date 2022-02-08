//! Data structures for quick runtime access to asset data.

mod cuboid_table;
mod model_table;
mod quad_table;
mod texture_table;

pub use cuboid_table::{Axis, Cuboid, CuboidKey, CuboidRotation, CuboidTable, DiscreteAngle};
pub use model_table::{Model, ModelKey, ModelTable};
pub use quad_table::{Quad, QuadKey, QuadRotation, QuadTable};
pub use texture_table::{Texture, TextureKey, TextureTable};
