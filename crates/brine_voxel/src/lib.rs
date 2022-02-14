#![doc = include_str!("../README.md")]

pub(crate) type IndexTy = u8;

mod axis;
mod direction;
mod view;

pub mod meshing;

pub use axis::{Axis, AxisSign};
pub use direction::Direction;
pub use meshing::{Mesh, Mesher, MeshingView, SimpleMesher};
pub use view::VoxelView;
