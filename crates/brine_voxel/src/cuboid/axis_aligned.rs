use glam::{const_vec3a, Vec3A};

use crate::Direction;

/// An axis-aligned cuboid.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AaCuboid {
    /// The negative-most [x, y, z] point of the cuboid.
    pub min: Vec3A,

    /// The positive-most [x, y, z] point of the cuboid.
    pub max: Vec3A,
}

impl AaCuboid {
    /// Returns a new cuboid spanning the given endpoints.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// let x = AaCuboid::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    /// let y = AaCuboid::new([1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
    /// assert_eq!(x, y);
    /// ```
    #[inline]
    pub fn new<T: Into<Vec3A>>(from: T, to: T) -> Self {
        let from: Vec3A = from.into();
        let to: Vec3A = to.into();

        let (min, max) = if from.cmplt(to).any() {
            (from, to)
        } else {
            (to, from)
        };

        Self { min, max }
    }

    /// Returns the eight vertices of the cuboid, in the following order:
    ///
    /// ```txt
    ///           4--------7          +y      
    ///         / |      / |          | -z    
    ///       5--------6   |    -x____|/____+x
    ///       |   |    |   |         /|       
    ///       |   0----|---1       +z |       
    ///       | /      | /            -y      
    ///       3--------2                  
    /// ```
    #[inline(always)]
    pub fn vertices(&self) -> [Vec3A; 8] {
        [
            Vec3A::new(self.min.x, self.min.y, self.min.z),
            Vec3A::new(self.max.x, self.min.y, self.min.z),
            Vec3A::new(self.max.x, self.min.y, self.max.z),
            Vec3A::new(self.min.x, self.min.y, self.max.z),
            Vec3A::new(self.min.x, self.max.y, self.min.z),
            Vec3A::new(self.min.x, self.max.y, self.max.z),
            Vec3A::new(self.max.x, self.max.y, self.max.z),
            Vec3A::new(self.max.x, self.max.y, self.min.z),
        ]
    }

    /// Gets the vertices of one of the cuboid's faces, in the following order:
    ///
    /// ```txt
    ///         2 ----> 3
    ///           ^
    ///     ^       \
    ///     |         \
    ///  +v |   0 ----> 1
    ///     |
    ///      -------->
    ///        +u
    /// ```
    #[inline(always)]
    pub fn get_face(&self, face: Direction) -> [Vec3A; 4] {
        let vertices = self.vertices();

        let vert_indices = match face {
            Direction::XNeg => [0, 3, 4, 5],
            Direction::XPos => [2, 1, 6, 7],
            Direction::YNeg => [0, 1, 3, 2],
            Direction::YPos => [5, 6, 4, 7],
            Direction::ZNeg => [1, 0, 7, 4],
            Direction::ZPos => [3, 2, 5, 6],
        };

        [
            vertices[vert_indices[0]],
            vertices[vert_indices[1]],
            vertices[vert_indices[2]],
            vertices[vert_indices[3]],
        ]
    }

    #[inline(always)]
    pub const fn get_normal(face: Direction) -> Vec3A {
        match face {
            Direction::XNeg => const_vec3a!([-1.0, 0.0, 0.0]),
            Direction::XPos => const_vec3a!([1.0, 0.0, 0.0]),
            Direction::YNeg => const_vec3a!([0.0, -1.0, 0.0]),
            Direction::YPos => const_vec3a!([0.0, 1.0, 0.0]),
            Direction::ZNeg => const_vec3a!([0.0, 0.0, -1.0]),
            Direction::ZPos => const_vec3a!([0.0, 0.0, 1.0]),
        }
    }
}
