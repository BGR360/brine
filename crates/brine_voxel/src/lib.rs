#![doc = include_str!("../README.md")]

pub(crate) type IndexTy = u8;

mod axis;
mod view;

pub mod meshing;

pub use axis::{Axis, AxisSign, Direction};
pub use meshing::{Mesh, Mesher, MeshingView, Quad};
pub use view::VoxelView;
