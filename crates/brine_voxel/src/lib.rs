#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]

pub(crate) type IndexTy = u8;

mod axis;
mod cuboid;
mod direction;
mod view;

pub mod meshing;

pub use axis::{Axis, AxisSign};
pub use cuboid::{AaCuboid, Cuboid, CuboidTransform};
pub use direction::Direction;
pub use meshing::{Mesh, Mesher, MeshingView, SimpleMesher};
pub use view::VoxelView;
