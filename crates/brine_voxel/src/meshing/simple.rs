use glam::Vec3;

use crate::{Direction, IndexTy};

use super::{Mesh, Mesher, MeshingView, Quad, QuadPositions};

#[derive(Debug, Default)]
pub struct SimpleMesher;

impl Mesher for SimpleMesher {
    fn generate_mesh<V>(&mut self, view: V) -> Mesh
    where
        V: MeshingView,
    {
        let mut mesh = Mesh::default();

        let mut context = SimpleMesherContext {
            view,
            mesh: &mut mesh,
        };

        context.generate_mesh();

        mesh
    }
}

pub struct SimpleMesherContext<'a, V> {
    view: V,
    mesh: &'a mut Mesh,
}

impl<'a, V: MeshingView> SimpleMesherContext<'a, V> {
    pub fn generate_mesh(&mut self) {
        for y in 0..self.view.size_y() {
            for z in 0..self.view.size_z() {
                for x in 0..self.view.size_x() {
                    if !self.view.is_empty(x, y, z) {
                        self.mesh_voxel(x, y, z);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn mesh_voxel(&mut self, x: IndexTy, y: IndexTy, z: IndexTy) {
        if self.view.is_full_cube(x, y, z) {
            self.mesh_full_cube(x, y, z);
        } else {
            self.mesh_voxel_using_view(x, y, z);
        }
    }

    #[inline]
    pub fn mesh_full_cube(&mut self, x: IndexTy, y: IndexTy, z: IndexTy) {
        let minimum = Vec3::new(x as f32, y as f32, z as f32);
        for face in Direction::values() {
            if !self.view.is_face_occluded(x, y, z, face) {
                let quad = Quad {
                    positions: Self::full_face_quad(minimum, face),
                    voxel: [x, y, z],
                    face: Some(face),
                };
                self.mesh.quads.push(quad);
            }
        }
    }

    #[inline]
    pub fn mesh_voxel_using_view(&mut self, x: IndexTy, y: IndexTy, z: IndexTy) {
        for face in Direction::values() {
            if !self.view.is_face_occluded(x, y, z, face) {
                for positions in self.view.face_quads(x, y, z, face).into_iter() {
                    let quad = Quad {
                        positions,
                        voxel: [x, y, z],
                        face: Some(face),
                    };
                    self.mesh.quads.push(quad);
                }
            }
        }

        for positions in self.view.non_face_quads(x, y, z).into_iter() {
            let quad = Quad {
                positions,
                voxel: [x, y, z],
                face: None,
            };
            self.mesh.quads.push(quad);
        }
    }

    #[inline]
    pub fn full_face_quad(voxel_pos: Vec3, face: Direction) -> QuadPositions {
        /*
               +y
               |
               |
               |_______ +x
              /
             /
           +z
        */
        const POSITIONS_XNEG: QuadPositions = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
        ];
        const POSITIONS_XPOS: QuadPositions = [
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ];
        const POSITIONS_YNEG: QuadPositions = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
        ];
        const POSITIONS_YPOS: QuadPositions = [
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        const POSITIONS_ZNEG: QuadPositions = [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        const POSITIONS_ZPOS: QuadPositions = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ];

        let base_positions = match face {
            Direction::XNeg => POSITIONS_XNEG,
            Direction::XPos => POSITIONS_XPOS,
            Direction::YNeg => POSITIONS_YNEG,
            Direction::YPos => POSITIONS_YPOS,
            Direction::ZNeg => POSITIONS_ZNEG,
            Direction::ZPos => POSITIONS_ZPOS,
        };

        base_positions.map(|base| (Vec3::from(base) + voxel_pos).into())
    }
}
