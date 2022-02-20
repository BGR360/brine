use crate::{Direction, IndexTy, VoxelView};

/// A [`VoxelView`] that can be used with a [`Mesher`] to generate a [`Mesh`] for a
/// cuboid chunk of voxels.
///
/// [`Mesher`]: super::Mesher
/// [`Mesh`]: super::Mesh
pub trait MeshingView: VoxelView {
    type Quads: IntoIterator<Item = [[f32; 3]; 4]>;

    /// Returns true if the voxel at index `[x, y, z]` has no geometry to
    /// provide to the mesh.
    ///
    /// **Note:** [`is_empty`] and [`is_face_occluded`] are independent of each
    /// other. In other words, non-empty voxels can still be occluded by empty
    /// ones. An "empty" voxel simply means that no geometry should be produced
    /// for that voxel in this view.
    ///
    /// This makes it possible to render a chunk in multiple separate layers
    /// without generating any redundant geometry. To do this, use a different
    /// [`MeshingView`] for each layer, and use [`is_empty`] to signify which
    /// voxels should be included in each layer's mesh.
    ///
    /// [`is_empty`]: MeshingView::is_empty
    /// [`is_face_occluded`]: MeshingView::is_face_occluded
    fn is_empty(&self, x: IndexTy, y: IndexTy, z: IndexTy) -> bool;

    /// Returns true if the voxel at index `[x, y, z]` is a cube that occupies
    /// the entire voxel's volume.
    ///
    /// Returning `true` from this method is a promise that the voxel's geometry
    /// consists of exactly 6 quads, each covering the entirety of one of the
    /// voxel's faces.
    ///
    /// As such, returning `true` from this method means that [`face_quads`] and
    /// [`non_face_quads`] will not be called for the given voxel. Instead the
    /// mesher will generate the face quads itself.
    ///
    /// [`face_quads`]: MeshingView::face_quads
    /// [`non_face_quads`]: MeshingView::non_face_quads
    fn is_full_cube(&self, x: IndexTy, y: IndexTy, z: IndexTy) -> bool;

    /// Returns true if the given face of the voxel at index `[x, y, z`] is
    /// fully occluded by its neighbor in the same direction.
    ///
    /// If [`is_empty`] is `true` for the given voxel, then the mesher will not
    /// make this query for any of the voxel's faces.
    ///
    /// Otherwise, if this method returns `false`, then the mesher will call
    /// [`face_quads`] for that face.
    ///
    /// [`is_empty`]: MeshingView::is_empty
    /// [`face_quads`]: MeshingView::face_quads
    fn is_face_occluded(&self, x: IndexTy, y: IndexTy, z: IndexTy, face: Direction) -> bool;

    /// Returns quads that should be rendered for the voxel at index `[x, y, z]`
    /// but can be skipped if the given face is occluded.
    ///
    /// If [`is_empty`] is `true` for the given voxel, then the mesher will not
    /// call this method.
    ///
    /// Otherwise, if  [`is_face_occluded`] is `false` for the given voxel face,
    /// then the mesher will request that face's quads from this method.
    ///
    /// [`is_empty`]: MeshingView::is_empty
    /// [`is_face_occluded`]: MeshingView::is_face_occluded
    fn face_quads(&self, x: IndexTy, y: IndexTy, z: IndexTy, face: Direction) -> Self::Quads;

    /// Returns quads that should be rendered for the voxel at index `[x, y, z]`
    /// and cannot be skipped unless all the voxel's faces are occluded.
    ///
    /// If [`is_empty`] is `true` for the given voxel, then the mesher will not
    /// request these quads.
    ///
    /// Otherwise, if [`is_face_occluded`] is `false` for any of the voxel's
    /// faces, then the mesher will request quads from this method.
    ///
    /// [`is_empty`]: MeshingView::is_empty
    /// [`is_face_occluded`]: MeshingView::is_face_occluded
    fn non_face_quads(&self, x: IndexTy, y: IndexTy, z: IndexTy) -> Self::Quads;
}

/// A trait that makes it possible to implement composable meshing views that
/// delegate by default to some inner [`MeshingView`].
pub trait DelegatingMeshingView: VoxelView {
    type Delegate: MeshingView;

    fn delegate(&self) -> &Self::Delegate;

    #[inline(always)]
    fn is_empty(&self, x: u8, y: u8, z: u8) -> bool {
        self.delegate().is_empty(x, y, z)
    }

    #[inline(always)]
    fn is_full_cube(&self, x: u8, y: u8, z: u8) -> bool {
        self.delegate().is_full_cube(x, y, z)
    }

    #[inline(always)]
    fn is_face_occluded(&self, x: u8, y: u8, z: u8, face: Direction) -> bool {
        self.delegate().is_face_occluded(x, y, z, face)
    }

    #[inline(always)]
    fn face_quads(
        &self,
        x: u8,
        y: u8,
        z: u8,
        face: Direction,
    ) -> <Self::Delegate as MeshingView>::Quads {
        self.delegate().face_quads(x, y, z, face)
    }

    #[inline(always)]
    fn non_face_quads(&self, x: u8, y: u8, z: u8) -> <Self::Delegate as MeshingView>::Quads {
        self.delegate().non_face_quads(x, y, z)
    }
}

impl<T: DelegatingMeshingView> VoxelView for T {
    #[inline(always)]
    fn size_x(&self) -> IndexTy {
        self.delegate().size_x()
    }

    #[inline(always)]
    fn size_y(&self) -> IndexTy {
        self.delegate().size_y()
    }

    #[inline(always)]
    fn size_z(&self) -> IndexTy {
        self.delegate().size_z()
    }
}

impl<T: DelegatingMeshingView> MeshingView for T {
    type Quads = <T::Delegate as MeshingView>::Quads;

    #[inline(always)]
    fn is_empty(&self, x: u8, y: u8, z: u8) -> bool {
        DelegatingMeshingView::is_empty(self, x, y, z)
    }

    #[inline(always)]
    fn is_full_cube(&self, x: u8, y: u8, z: u8) -> bool {
        DelegatingMeshingView::is_full_cube(self, x, y, z)
    }

    #[inline(always)]
    fn is_face_occluded(&self, x: u8, y: u8, z: u8, face: Direction) -> bool {
        DelegatingMeshingView::is_face_occluded(self, x, y, z, face)
    }

    #[inline(always)]
    fn face_quads(&self, x: u8, y: u8, z: u8, face: Direction) -> Self::Quads {
        DelegatingMeshingView::face_quads(self, x, y, z, face)
    }

    #[inline(always)]
    fn non_face_quads(&self, x: u8, y: u8, z: u8) -> Self::Quads {
        DelegatingMeshingView::non_face_quads(self, x, y, z)
    }
}
