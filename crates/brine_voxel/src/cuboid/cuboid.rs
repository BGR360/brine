use std::cmp::Ordering;

use glam::Vec3A;

use crate::{AaCuboid, CuboidTransform, Direction};

/// A cuboid with some transformation applied to it.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Cuboid {
    pub original: AaCuboid,
    pub transform: CuboidTransform,
}

impl Cuboid {
    #[inline]
    pub fn new(original: AaCuboid, transform: CuboidTransform) -> Self {
        Self {
            original,
            transform,
        }
    }

    #[inline]
    pub fn axis_aligned<T: Into<Vec3A>>(from: T, to: T) -> Self {
        Self {
            original: AaCuboid::new(from, to),
            ..Default::default()
        }
    }

    /// Returns the transformed positions of the original cuboid's vertices, in
    /// the same order as they would have been returned by
    /// [`AaCuboid::vertices`].
    #[inline]
    pub fn vertices(&self) -> [Vec3A; 8] {
        self.original
            .vertices()
            .map(|vertex| self.transform.transform_point(vertex))
    }

    /// Returns the minimum point of the bounding box of this cuboid.
    #[inline]
    pub fn min(&self) -> Vec3A {
        let verts = self.vertices();

        let float_cmp = |a: &f32, b: &f32| {
            if a <= b {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        };

        let x = verts
            .map(|vert| vert.x)
            .into_iter()
            .min_by(float_cmp)
            .unwrap();
        let y = verts
            .map(|vert| vert.y)
            .into_iter()
            .min_by(float_cmp)
            .unwrap();
        let z = verts
            .map(|vert| vert.z)
            .into_iter()
            .min_by(float_cmp)
            .unwrap();

        Vec3A::new(x, y, z)
    }

    /// Returns the vertex with the maximum [x, y, z] value.
    #[inline]
    pub fn max(&self) -> Vec3A {
        let verts = self.vertices();

        let float_cmp = |a: &f32, b: &f32| {
            if a >= b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        };

        let x = verts
            .map(|vert| vert.x)
            .into_iter()
            .max_by(float_cmp)
            .unwrap();
        let y = verts
            .map(|vert| vert.y)
            .into_iter()
            .max_by(float_cmp)
            .unwrap();
        let z = verts
            .map(|vert| vert.z)
            .into_iter()
            .max_by(float_cmp)
            .unwrap();

        Vec3A::new(x, y, z)
    }

    /// Returns the axis-aligned bounding cuboid for this cuboid.
    #[inline]
    pub fn bounding_box(&self) -> AaCuboid {
        AaCuboid {
            min: self.min(),
            max: self.max(),
        }
    }

    /// Returns the transformed vertices of one of the original cuboid's faces,
    /// in the same order as they would have been returned by
    /// [`AaCuboid::get_face`].
    #[inline]
    pub fn get_face(&self, face: Direction) -> [Vec3A; 4] {
        self.original
            .get_face(face)
            .map(|vertex| self.transform.transform_point(vertex))
    }

    #[inline]
    pub fn get_normal(&self, face: Direction) -> Vec3A {
        self.transform.transform_vector(AaCuboid::get_normal(face))
    }
}

impl From<AaCuboid> for Cuboid {
    fn from(original: AaCuboid) -> Self {
        Self {
            original,
            ..Default::default()
        }
    }
}
