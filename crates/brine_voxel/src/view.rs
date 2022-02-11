use crate::IndexTy;

/// A view into a cuboid chunk of voxels.
///
/// Views are used to interpret the data of a voxel chunk in one or more ways
/// and provide information to the various data generators and services in this
/// library.
///
/// For example, a [`MeshingView`] is used with a [`Mesher`] to generate mesh
/// geometry for a voxel chunk. Multiple [`MeshingViews`] can be used in
/// separate passes to generate geometry for different rendering layers of the
/// same chunk.
///
/// [`MeshingView`]: crate::meshing::MeshingView
/// [`MeshingViews`]: crate::meshing::MeshingView
/// [`Mesher`]: crate::meshing::Mesher
pub trait VoxelView {
    /// The size of the chunk on the x axis, in voxels.
    fn size_x(&self) -> IndexTy;

    /// The size of the chunk on the y axis, in voxels.
    fn size_y(&self) -> IndexTy;

    /// The size of the chunk on the z axis, in voxels.
    fn size_z(&self) -> IndexTy;
}
