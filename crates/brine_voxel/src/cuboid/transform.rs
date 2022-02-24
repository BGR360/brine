use std::ops::Mul;

use glam::{Affine3A, Quat, Vec3, Vec3A};

use crate::{AaCuboid, Cuboid};

/// A transform applied to an [`AaCuboid`].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CuboidTransform {
    pub affine3: Affine3A,
}

impl CuboidTransform {
    #[inline]
    pub fn from_rotation_about_origin<T>(rotation: Quat, origin: T) -> Self
    where
        T: Copy + Into<Vec3>,
    {
        Self::default().with_rotation(rotation).with_origin(origin)
    }

    #[inline]
    pub fn with_rotation(self, rotation: Quat) -> Self {
        Self {
            affine3: Affine3A::from_quat(rotation) * self.affine3,
        }
    }

    #[inline]
    pub fn with_rotation_x(self, radians: f32) -> Self {
        Self {
            affine3: Affine3A::from_rotation_x(radians) * self.affine3,
        }
    }

    #[inline]
    pub fn with_rotation_y(self, radians: f32) -> Self {
        Self {
            affine3: Affine3A::from_rotation_y(radians) * self.affine3,
        }
    }

    #[inline]
    pub fn with_rotation_z(self, radians: f32) -> Self {
        Self {
            affine3: Affine3A::from_rotation_z(radians) * self.affine3,
        }
    }

    #[inline]
    pub fn with_scale(self, scale_factor: f32) -> Self {
        Self {
            affine3: Affine3A::from_scale(Vec3::ONE * scale_factor) * self.affine3,
        }
    }

    #[inline]
    pub fn with_origin<T>(self, origin: T) -> Self
    where
        T: Copy + Into<Vec3>,
    {
        let shift = Affine3A::from_translation(-origin.into());
        let shift_back = Affine3A::from_translation(origin.into());

        Self {
            affine3: shift_back * self.affine3 * shift,
        }
    }

    #[inline]
    pub fn transform_aa_cuboid(&self, aa_cuboid: AaCuboid) -> Cuboid {
        *self * aa_cuboid
    }

    #[inline]
    pub fn transform_cuboid(&self, cuboid: Cuboid) -> Cuboid {
        *self * cuboid
    }

    #[inline]
    pub fn transform_point(&self, point: Vec3A) -> Vec3A {
        self.affine3.transform_point3a(point)
    }

    #[inline]
    pub fn transform_vector(&self, vector: Vec3A) -> Vec3A {
        self.affine3.transform_vector3a(vector)
    }
}

impl Mul for CuboidTransform {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            affine3: self.affine3 * rhs.affine3,
        }
    }
}

impl Mul<AaCuboid> for CuboidTransform {
    type Output = Cuboid;

    #[inline(always)]
    fn mul(self, rhs: AaCuboid) -> Self::Output {
        Cuboid {
            original: rhs,
            transform: self,
        }
    }
}

impl Mul<Cuboid> for CuboidTransform {
    type Output = Cuboid;

    #[inline]
    fn mul(self, rhs: Cuboid) -> Self::Output {
        let original = rhs.original;
        let transform = self * rhs.transform;

        Cuboid {
            original,
            transform,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use glam::const_vec3a;

    use crate::Direction;

    use super::*;

    trait PartialEqEps<T = Self, E = T> {
        fn eq_eps(&self, rhs: &T, eps: &E) -> bool;
    }

    impl PartialEqEps for f32 {
        fn eq_eps(&self, rhs: &Self, eps: &Self) -> bool {
            (self - rhs).abs() <= *eps
        }
    }

    impl PartialEqEps for Vec3A {
        fn eq_eps(&self, rhs: &Self, eps: &Self) -> bool {
            (*self - *rhs).abs().cmple(*eps).all()
        }
    }

    impl<const N: usize> PartialEqEps<Self, Vec3A> for [Vec3A; N] {
        fn eq_eps(&self, rhs: &Self, eps: &Vec3A) -> bool {
            self.iter()
                .zip(rhs.iter())
                .all(|(lhs, rhs)| lhs.eq_eps(rhs, eps))
        }
    }

    const EPS_F32: f32 = 0.00001;
    const EPS_VEC3A: Vec3A = const_vec3a!([EPS_F32, EPS_F32, EPS_F32]);

    fn assert_eq_epsilon<T, U, E>(lhs: T, rhs: U, eps: E)
    where
        T: PartialEqEps<U, E> + fmt::Debug,
        U: fmt::Debug,
    {
        assert!(
            lhs.eq_eps(&rhs, &eps),
            "Values differ by too much: left={:?}, right={:?}",
            lhs,
            rhs,
        );
    }

    #[test]
    fn rotation_about_origin_90_degrees() {
        let do_test = |transform: CuboidTransform| {
            let original = AaCuboid::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
            let transformed = transform * Cuboid::from(original);

            assert_eq_epsilon(transformed.min(), original.min, EPS_VEC3A);
        };

        do_test(CuboidTransform::default().with_rotation_x(0f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_x(90f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_x(180f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_x(270f32.to_radians()));

        do_test(CuboidTransform::default().with_rotation_y(0f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_y(90f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_y(180f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_y(270f32.to_radians()));

        do_test(CuboidTransform::default().with_rotation_z(0f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_z(90f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_z(180f32.to_radians()));
        do_test(CuboidTransform::default().with_rotation_z(270f32.to_radians()));
    }

    #[test]
    fn transformed_face() {
        let original = AaCuboid::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

        let transform = CuboidTransform::default()
            .with_rotation_y(180f32.to_radians())
            .with_origin([0.5, 0.5, 0.5]);

        let transformed = transform * Cuboid::from(original);

        assert_eq_epsilon(
            original.get_face(Direction::XNeg),
            transformed.get_face(Direction::XPos),
            EPS_VEC3A,
        );
        assert_eq_epsilon(
            original.get_face(Direction::ZNeg),
            transformed.get_face(Direction::ZPos),
            EPS_VEC3A,
        );

        let transform = CuboidTransform::default()
            .with_rotation_y(90f32.to_radians())
            .with_origin([0.5, 0.5, 0.5]);

        let transformed = transform * Cuboid::from(original);

        assert_eq_epsilon(
            original.get_face(Direction::XNeg),
            transformed.get_face(Direction::ZNeg),
            EPS_VEC3A,
        );
        assert_eq_epsilon(
            original.get_face(Direction::ZNeg),
            transformed.get_face(Direction::XPos),
            EPS_VEC3A,
        );
    }
}
