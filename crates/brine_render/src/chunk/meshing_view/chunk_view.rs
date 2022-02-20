use brine_chunk::{ChunkSection, SECTION_HEIGHT, SECTION_WIDTH};
use brine_data::{BlockStateId, MinecraftData};
use brine_voxel::{meshing::QuadPositions, Direction, MeshingView, VoxelView};

pub struct ChunkView<'a> {
    mc_data: &'a MinecraftData,
    chunk: &'a ChunkSection,
}

impl<'a> ChunkView<'a> {
    const MAX_X: u8 = (SECTION_WIDTH as u8) - 1;
    const MAX_Y: u8 = (SECTION_HEIGHT as u8) - 1;
    const MAX_Z: u8 = (SECTION_WIDTH as u8) - 1;

    pub fn new(mc_data: &'a MinecraftData, chunk: &'a ChunkSection) -> Self {
        Self { mc_data, chunk }
    }

    #[inline]
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> BlockStateId {
        let block_state = self.chunk.get_block((x, y, z)).unwrap();
        BlockStateId(block_state.0 as u16)
    }
}

impl<'a> VoxelView for ChunkView<'a> {
    #[inline(always)]
    fn size_x(&self) -> u8 {
        SECTION_WIDTH as u8
    }

    #[inline(always)]
    fn size_y(&self) -> u8 {
        SECTION_HEIGHT as u8
    }

    #[inline(always)]
    fn size_z(&self) -> u8 {
        SECTION_WIDTH as u8
    }
}

impl<'a> MeshingView for ChunkView<'a> {
    type Quads = Vec<QuadPositions>;

    #[inline]
    fn is_empty(&self, x: u8, y: u8, z: u8) -> bool {
        let block_state_id = self.get_block(x, y, z);
        self.mc_data.blocks().is_air(block_state_id).unwrap()
    }

    #[inline(always)]
    fn is_full_cube(&self, _x: u8, _y: u8, _z: u8) -> bool {
        true
    }

    #[inline]
    fn is_face_occluded(&self, x: u8, y: u8, z: u8, face: Direction) -> bool {
        match (face, x, y, z) {
            // Faces on the edge of the chunk are always visible.
            (Direction::XNeg, 0, _, _)
            | (Direction::YNeg, _, 0, _)
            | (Direction::ZNeg, _, _, 0)
            | (Direction::XPos, Self::MAX_X.., _, _)
            | (Direction::YPos, _, Self::MAX_Y.., _)
            | (Direction::ZPos, _, _, Self::MAX_Z..) => false,

            // Otherwise the face is occluded if the adjacent block is not empty.
            _ => {
                let [x, y, z] = face.translate_pos([x, y, z], 1).unwrap();
                !self.is_empty(x, y, z)
            }
        }
    }

    fn face_quads(&self, _x: u8, _y: u8, _z: u8, _face: Direction) -> Self::Quads {
        todo!()
    }

    fn non_face_quads(&self, _x: u8, _y: u8, _z: u8) -> Self::Quads {
        todo!()
    }
}
