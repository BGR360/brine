use crate::{Direction, IndexTy};

/// Contains a list of [`Quads`] representing the geometry of a voxel chunk.
///
/// [`Quads`]: Quad
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mesh {
    pub quads: Vec<Quad>,
}

pub type QuadPositions = [[f32; 3]; 4];
pub type QuadNormals = [[f32; 3]; 4];
pub type QuadTexCoords = [[f32; 2]; 4];
pub type QuadIndices = [u8; 6];

/// A single quad in a [`Mesh`].
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Quad {
    /// The `[x, y, z]` **positions** of the quad's vertices in 3D space.
    ///
    /// The vertices will be in the same order as they provided by the
    /// [`MeshingView`] that was used to generate the mesh.
    ///
    /// [`MeshingView`]: super::MeshingView
    pub positions: QuadPositions,

    /// The `[x, y, z]` **index** of the voxel that produced this quad.
    ///
    /// This will be relative to the origin of the [`MeshingView`] that was used
    /// to generate the mesh.
    ///
    /// [`MeshingView`]: super::MeshingView
    pub voxel: [IndexTy; 3],

    /// Which of the six cube sides this quad belongs to.
    ///
    /// This is [`None`] if the quad does not belong to any specific face. See
    /// the [`MeshingView::non_face_quads`] documentation for more info.
    ///
    /// [`MeshingView::non_face_quads`]: super::MeshingView::non_face_quads
    pub face: Option<Direction>,
}

impl Quad {
    #[inline(always)]
    pub fn get_indices(&self) -> QuadIndices {
        [0, 1, 2, 1, 3, 2]
    }

    #[inline(always)]
    pub fn get_normals(&self) -> QuadNormals {
        let normal = match self.face {
            Some(Direction::XNeg) => [-1.0, 0.0, 0.0],
            Some(Direction::XPos) => [1.0, 0.0, 0.0],
            Some(Direction::YNeg) => [0.0, -1.0, 0.0],
            Some(Direction::YPos) => [0.0, 1.0, 0.0],
            Some(Direction::ZNeg) => [0.0, 0.0, -1.0],
            Some(Direction::ZPos) => [0.0, 0.0, 1.0],
            None => unimplemented!(),
        };

        [normal; 4]
    }

    #[inline(always)]
    pub fn get_tex_coords(&self) -> QuadTexCoords {
        [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]]
    }
}
