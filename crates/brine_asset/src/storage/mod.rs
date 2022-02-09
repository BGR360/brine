//! Data structures for quick runtime access to asset data.

use std::fmt;

mod block_state_table;
mod cuboid_table;
mod model_table;
mod quad_table;
mod texture_table;

pub use block_state_table::{BlockState, BlockStateGrabBag, BlockStateModel, BlockStateTable};
pub use cuboid_table::{Axis, Cuboid, CuboidKey, CuboidRotation, CuboidTable, DiscreteAngle};
pub use model_table::{Model, ModelKey, ModelTable};
pub use quad_table::{Quad, QuadKey, QuadTable};
pub use texture_table::{Texture, TextureKey, TextureTable};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum QuarterRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Default for QuarterRotation {
    fn default() -> Self {
        Self::Deg0
    }
}

impl From<u32> for QuarterRotation {
    fn from(deg: u32) -> Self {
        match deg {
            0 => Self::Deg0,
            90 => Self::Deg90,
            180 => Self::Deg180,
            270 => Self::Deg270,
            _ => panic!("Invalid quad rotation: {}", deg),
        }
    }
}

impl fmt::Debug for QuarterRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deg0 => write!(f, "0"),
            Self::Deg90 => write!(f, "90"),
            Self::Deg180 => write!(f, "180"),
            Self::Deg270 => write!(f, "270"),
        }
    }
}
