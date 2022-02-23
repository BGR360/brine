use smallvec::SmallVec;

use brine_asset::{BakedModel, BlockFace, MinecraftAssets};
use brine_chunk::{ChunkSection, SECTION_HEIGHT, SECTION_WIDTH};
use brine_data::{blocks::Block, BlockStateId, MinecraftData};
use brine_voxel::{meshing::QuadPositions, Direction, MeshingView, VoxelView};

pub struct ChunkView<'a> {
    mc_data: &'a MinecraftData,
    mc_assets: &'a MinecraftAssets,
    chunk: &'a ChunkSection,
}

impl<'a> ChunkView<'a> {
    const MAX_X: u8 = (SECTION_WIDTH as u8) - 1;
    const MAX_Y: u8 = (SECTION_HEIGHT as u8) - 1;
    const MAX_Z: u8 = (SECTION_WIDTH as u8) - 1;

    pub fn new(
        mc_data: &'a MinecraftData,
        mc_assets: &'a MinecraftAssets,
        chunk: &'a ChunkSection,
    ) -> Self {
        Self {
            mc_data,
            mc_assets,
            chunk,
        }
    }

    #[inline]
    pub fn get_block_state_id(&self, x: u8, y: u8, z: u8) -> BlockStateId {
        let block_state = self.chunk.get_block((x, y, z)).unwrap();
        BlockStateId(block_state.0 as u16)
    }

    #[inline]
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> Option<Block<'a>> {
        let block_state_id = self.get_block_state_id(x, y, z);
        self.mc_data.blocks().get_by_state_id(block_state_id)
    }

    #[inline]
    pub fn get_block_model(&self, x: u8, y: u8, z: u8) -> Option<&'a BakedModel> {
        let block_state_id = self.get_block_state_id(x, y, z);
        let baked_block_state = self.mc_assets.block_states().get_by_key(block_state_id)?;
        let model_key = baked_block_state.get_first_model()?;
        self.mc_assets.models().get_by_key(model_key)
    }

    #[inline]
    pub fn is_air(&self, x: u8, y: u8, z: u8) -> bool {
        self.get_block(x, y, z)
            .map_or(false, |block| block.is_air())
    }

    #[inline]
    fn get_quads_for_block_face(
        &self,
        x: u8,
        y: u8,
        z: u8,
        face: Option<Direction>,
    ) -> SmallVec<[QuadPositions; 6]> {
        self.get_block_model(x, y, z)
            .map_or(Default::default(), |model| {
                let face = face.map(|direction| match direction {
                    Direction::XNeg => BlockFace::West,
                    Direction::XPos => BlockFace::East,
                    Direction::YNeg => BlockFace::Down,
                    Direction::YPos => BlockFace::Up,
                    Direction::ZNeg => BlockFace::North,
                    Direction::ZPos => BlockFace::South,
                });

                model
                    .quads
                    .iter()
                    .filter(|quad| quad.cull_face == face)
                    .map(|quad| {
                        quad.positions
                            .map(|[x0, y0, z0]| [x0 + x as f32, y0 + y as f32, z0 + z as f32])
                    })
                    .collect()
            })
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
    type Quads = SmallVec<[QuadPositions; 6]>;

    #[inline]
    fn is_empty(&self, x: u8, y: u8, z: u8) -> bool {
        self.is_air(x, y, z)
    }

    #[inline]
    fn is_full_cube(&self, x: u8, y: u8, z: u8) -> bool {
        self.get_block_model(x, y, z)
            .map_or(false, |model| model.is_full_cube)
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

            _ => {
                let [x, y, z] = face.translate_pos([x, y, z], 1).unwrap();
                !self.is_empty(x, y, z) && self.is_full_cube(x, y, z)
            }
        }
    }

    #[inline]
    fn face_quads(&self, x: u8, y: u8, z: u8, face: Direction) -> Self::Quads {
        self.get_quads_for_block_face(x, y, z, Some(face))
    }

    #[inline]
    fn non_face_quads(&self, x: u8, y: u8, z: u8) -> Self::Quads {
        self.get_quads_for_block_face(x, y, z, None)
    }
}
