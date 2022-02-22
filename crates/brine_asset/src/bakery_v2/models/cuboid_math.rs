use std::fmt;

use glam::{const_vec3a, Affine3A, Vec3A};
use minecraft_assets::schemas::models::{Axis, BlockFace, ElementRotation};

use crate::bakery_v2::models::BakedQuad;

/*
   .aMMMb  dMP dMP dMMMMb  .aMMMb  dMP dMMMMb
  dMP"VMP dMP dMP dMP"dMP dMP"dMP amr dMP VMP
 dMP     dMP dMP dMMMMK" dMP dMP dMP dMP dMP
dMP.aMP dMP.aMP dMP.aMF dMP.aMP dMP dMP.aMP
VMMMP"  VMMMP" dMMMMP"  VMMMP" dMP dMMMMP"

FIGLET: Cuboid
*/

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cuboid {
    /// The eight vertices of the cuboid, in the following order:
    ///
    /// ```txt
    ///         4-------7          +y               U
    ///        /       /           | -z             | N
    ///       5-------6      -x____|/____+x   W ____|/____ E
    ///                           /|               /|
    ///         0-------1       +z |              S |
    ///        /       /           -y               D
    ///       3-------2
    /// ```
    pub vertices: [Vec3A; 8],
}

impl Cuboid {
    #[inline(always)]
    pub fn new<T: Into<Vec3A>>(min: T, max: T) -> Self {
        let min = min.into();
        let max = max.into();
        Self {
            vertices: [
                Vec3A::new(min.x, min.y, min.z),
                Vec3A::new(max.x, min.y, min.z),
                Vec3A::new(max.x, min.y, max.z),
                Vec3A::new(min.x, min.y, max.z),
                Vec3A::new(min.x, max.y, min.z),
                Vec3A::new(min.x, max.y, max.z),
                Vec3A::new(max.x, max.y, max.z),
                Vec3A::new(max.x, max.y, min.z),
            ],
        }
    }

    /// Gets the vertices of one of the faces of the cuboid, in the following order:
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
    pub fn get_face(&self, face: BlockFace) -> [Vec3A; 4] {
        let verts = match face {
            BlockFace::Down => [0, 1, 3, 2],
            BlockFace::Up => [5, 6, 4, 7],
            BlockFace::North => [1, 0, 7, 4],
            BlockFace::South => [3, 2, 5, 6],
            BlockFace::West => [0, 3, 4, 5],
            BlockFace::East => [2, 1, 6, 7],
        };

        [
            self.vertices[verts[0]],
            self.vertices[verts[1]],
            self.vertices[verts[2]],
            self.vertices[verts[3]],
        ]
    }

    #[inline(always)]
    pub const fn get_indices(face: BlockFace) -> [u8; 6] {
        //   4-------7          +y               U
        //  /       /           | -z             | N
        // 5-------6      -x____|/____+x   W ____|/____ E
        //                     /|               /|
        //   0-------1       +z |              S |
        //  /       /           -y               D
        // 3-------2
        match face {
            BlockFace::Down => [0, 1, 3, 0, 3, 2],  // [0, 1, 2, 0, 2, 3]
            BlockFace::Up => [0, 1, 3, 0, 3, 2],    // [5, 6, 7, 5, 7, 4]
            BlockFace::North => [0, 1, 2, 1, 3, 2], // [1, 0, 7, 0, 4, 7]
            BlockFace::South => [0, 1, 2, 1, 3, 2], // [3, 2, 5, 2, 6, 5]
            BlockFace::West => [0, 1, 3, 0, 3, 2],  // [0, 3, 5, 0, 5, 4]
            BlockFace::East => [0, 1, 3, 0, 3, 2],  // [2, 1, 7, 2, 7, 6]
        }
    }

    #[inline(always)]
    pub const fn get_normal(face: BlockFace) -> Vec3A {
        match face {
            BlockFace::Down => const_vec3a!([0.0, -1.0, 0.0]),
            BlockFace::Up => const_vec3a!([0.0, 1.0, 0.0]),
            BlockFace::North => const_vec3a!([0.0, 0.0, -1.0]),
            BlockFace::South => const_vec3a!([0.0, 0.0, 1.0]),
            BlockFace::West => const_vec3a!([-1.0, 0.0, 0.0]),
            BlockFace::East => const_vec3a!([1.0, 0.0, 0.0]),
        }
    }

    #[inline(always)]
    pub fn scaled(self, scale_factor: f32) -> Self {
        let vertices = self.vertices.map(|vertex| vertex * scale_factor);

        Self { vertices }
    }
}

/*
   .aMMMb  dMP dMP dMMMMb  .aMMMb  dMP dMMMMb
  dMP"VMP dMP dMP dMP"dMP dMP"dMP amr dMP VMP
 dMP     dMP dMP dMMMMK" dMP dMP dMP dMP dMP
dMP.aMP dMP.aMP dMP.aMF dMP.aMP dMP dMP.aMP
VMMMP"  VMMMP" dMMMMP"  VMMMP" dMP dMMMMP"

    dMMMMb  .aMMMb dMMMMMMP .aMMMb dMMMMMMP dMP .aMMMb  dMMMMb
   dMP.dMP dMP"dMP   dMP   dMP"dMP   dMP   amr dMP"dMP dMP dMP
  dMMMMK" dMP dMP   dMP   dMMMMMP   dMP   dMP dMP dMP dMP dMP
 dMP"AMF dMP.aMP   dMP   dMP dMP   dMP   dMP dMP.aMP dMP dMP
dMP dMP  VMMMP"   dMP   dMP dMP   dMP   dMP  VMMMP" dMP dMP


FIGLET: CuboidRotation
*/

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CuboidRotation {
    pub origin: [f32; 3],
    pub axis: Axis,
    pub angle: EighthRotation,
    // TODO!
    pub rescale: bool,
}

impl CuboidRotation {
    #[inline(always)]
    pub fn rotate_vector(&self, vec: Vec3A) -> Vec3A {
        let transform = match self.axis {
            Axis::X => Affine3A::from_rotation_x(f32::from(self.angle).to_radians()),
            Axis::Y => Affine3A::from_rotation_y(f32::from(self.angle).to_radians()),
            Axis::Z => Affine3A::from_rotation_z(f32::from(self.angle).to_radians()),
        };

        transform.transform_vector3a(vec)
    }

    #[inline(always)]
    pub fn rotate_point(&self, point: Vec3A) -> Vec3A {
        let origin = Vec3A::from(self.origin);
        let from_origin = point - origin;

        let from_origin = self.rotate_vector(from_origin);

        origin + from_origin
    }

    #[inline(always)]
    pub fn rotate_cuboid(&self, cuboid: Cuboid) -> Cuboid {
        let vertices = cuboid.vertices.map(|vertex| self.rotate_point(vertex));

        Cuboid { vertices }
    }
}

impl Default for CuboidRotation {
    fn default() -> Self {
        Self {
            origin: Default::default(),
            axis: Axis::X,
            angle: Default::default(),
            rescale: false,
        }
    }
}

impl From<ElementRotation> for CuboidRotation {
    #[inline(always)]
    fn from(
        ElementRotation {
            origin,
            axis,
            angle,
            rescale,
        }: ElementRotation,
    ) -> Self {
        let angle = EighthRotation::from(angle);
        Self {
            origin,
            axis,
            angle,
            rescale,
        }
    }
}

/*
   .aMMMb  dMP dMP .aMMMb  dMMMMb
  dMP"dMP dMP dMP dMP"dMP dMP VMP
 dMP.dMP dMP dMP dMMMMMP dMP dMP
dMP.MMP dMP.aMP dMP dMP dMP.aMP
VMMP"MP VMMMP" dMP dMP dMMMMP"

    dMMMMb  .aMMMb dMMMMMMP .aMMMb dMMMMMMP dMP .aMMMb  dMMMMb
   dMP.dMP dMP"dMP   dMP   dMP"dMP   dMP   amr dMP"dMP dMP dMP
  dMMMMK" dMP dMP   dMP   dMMMMMP   dMP   dMP dMP dMP dMP dMP
 dMP"AMF dMP.aMP   dMP   dMP dMP   dMP   dMP dMP.aMP dMP dMP
dMP dMP  VMMMP"   dMP   dMP dMP   dMP   dMP  VMMMP" dMP dMP


FIGLET: QuadRotation
*/

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadRotation {
    pub x: QuarterRotation,
    pub y: QuarterRotation,
}

impl QuadRotation {
    #[inline(always)]
    pub fn new<T: Into<QuarterRotation>>(x: T, y: T) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    #[inline(always)]
    pub fn rotate_quad(&self, quad: &mut BakedQuad) {
        let vertices = quad.positions;
        let vertices = vertices.map(|vertex| vertex.map(|coord| coord - 0.5));
        let vertices = vertices.map(|vertex| self.rotate_point(vertex));
        let vertices = vertices.map(|vertex| vertex.map(|coord| coord + 0.5));

        quad.positions = vertices;
    }

    #[inline(always)]
    pub fn rotate_point(&self, [x, y, z]: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = Self::rotate_x([x, y, z], self.x);
        let [x, y, z] = Self::rotate_y([x, y, z], self.y);

        [x, y, z]
    }

    #[inline(always)]
    fn rotate_x([x, y, z]: [f32; 3], rotation: QuarterRotation) -> [f32; 3] {
        match rotation {
            QuarterRotation::Deg0 => [x, y, z],
            QuarterRotation::Deg90 => [x, -z, y],
            QuarterRotation::Deg180 => [x, -y, -z],
            QuarterRotation::Deg270 => [x, z, -y],
        }
    }

    #[inline(always)]
    fn rotate_y([x, y, z]: [f32; 3], rotation: QuarterRotation) -> [f32; 3] {
        match rotation {
            QuarterRotation::Deg0 => [x, y, z],
            QuarterRotation::Deg90 => [z, y, -x],
            QuarterRotation::Deg180 => [-x, y, -z],
            QuarterRotation::Deg270 => [-z, y, x],
        }
    }
}

/*
   .aMMMb  dMP dMP .aMMMb  dMMMMb dMMMMMMP dMMMMMP dMMMMb
  dMP"dMP dMP dMP dMP"dMP dMP.dMP   dMP   dMP     dMP.dMP
 dMP.dMP dMP dMP dMMMMMP dMMMMK"   dMP   dMMMP   dMMMMK"
dMP.MMP dMP.aMP dMP dMP dMP"AMF   dMP   dMP     dMP"AMF
VMMP"MP VMMMP" dMP dMP dMP dMP   dMP   dMMMMMP dMP dMP

    dMMMMb  .aMMMb dMMMMMMP .aMMMb dMMMMMMP dMP .aMMMb  dMMMMb
   dMP.dMP dMP"dMP   dMP   dMP"dMP   dMP   amr dMP"dMP dMP dMP
  dMMMMK" dMP dMP   dMP   dMMMMMP   dMP   dMP dMP dMP dMP dMP
 dMP"AMF dMP.aMP   dMP   dMP dMP   dMP   dMP dMP.aMP dMP dMP
dMP dMP  VMMMP"   dMP   dMP dMP   dMP   dMP  VMMMP" dMP dMP


FIGLET: QuarterRotation
*/

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum QuarterRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Default for QuarterRotation {
    fn default() -> Self {
        Self::Deg0
    }
}

impl From<u32> for QuarterRotation {
    fn from(deg: u32) -> Self {
        match deg {
            0 => Self::Deg0,
            90 => Self::Deg90,
            180 => Self::Deg180,
            270 => Self::Deg270,
            _ => panic!("Invalid quad rotation: {}", deg),
        }
    }
}

impl From<i32> for QuarterRotation {
    fn from(deg: i32) -> Self {
        match deg {
            -270 => Self::Deg90,
            -180 => Self::Deg180,
            -90 => Self::Deg270,
            0 => Self::Deg0,
            90 => Self::Deg90,
            180 => Self::Deg180,
            270 => Self::Deg270,
            _ => panic!("Invalid quad rotation: {}", deg),
        }
    }
}

impl From<QuarterRotation> for f32 {
    fn from(rot: QuarterRotation) -> Self {
        match rot {
            QuarterRotation::Deg0 => 0.0,
            QuarterRotation::Deg90 => 90.0,
            QuarterRotation::Deg180 => 180.0,
            QuarterRotation::Deg270 => 270.0,
        }
    }
}

impl fmt::Debug for QuarterRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deg0 => write!(f, "0"),
            Self::Deg90 => write!(f, "90"),
            Self::Deg180 => write!(f, "180"),
            Self::Deg270 => write!(f, "270"),
        }
    }
}

/*
    dMMMMMP dMP .aMMMMP dMP dMP dMMMMMMP dMP dMP
   dMP     amr dMP"    dMP dMP    dMP   dMP dMP
  dMMMP   dMP dMP MMP"dMMMMMP    dMP   dMMMMMP
 dMP     dMP dMP.dMP dMP dMP    dMP   dMP dMP
dMMMMMP dMP  VMMMP" dMP dMP    dMP   dMP dMP

    dMMMMb  .aMMMb dMMMMMMP .aMMMb dMMMMMMP dMP .aMMMb  dMMMMb
   dMP.dMP dMP"dMP   dMP   dMP"dMP   dMP   amr dMP"dMP dMP dMP
  dMMMMK" dMP dMP   dMP   dMMMMMP   dMP   dMP dMP dMP dMP dMP
 dMP"AMF dMP.aMP   dMP   dMP dMP   dMP   dMP dMP.aMP dMP dMP
dMP dMP  VMMMP"   dMP   dMP dMP   dMP   dMP  VMMMP" dMP dMP


FIGLET: EighthRotation
*/

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EighthRotation {
    Neg45,
    Neg22_5,
    Zero,
    Pos22_5,
    Pos45,
}

impl From<f32> for EighthRotation {
    #[inline(always)]
    fn from(f: f32) -> Self {
        match f {
            f if f == -45.0 => Self::Neg45,
            f if f == -22.5 => Self::Neg22_5,
            f if f == 0.0 => Self::Zero,
            f if f == 22.5 => Self::Pos22_5,
            f if f == 45.0 => Self::Pos45,
            _ => panic!("Invalid model element rotation value: {}", f),
        }
    }
}

impl From<EighthRotation> for f32 {
    #[inline(always)]
    fn from(a: EighthRotation) -> Self {
        match a {
            EighthRotation::Neg45 => -45.0,
            EighthRotation::Neg22_5 => -22.5,
            EighthRotation::Zero => 0.0,
            EighthRotation::Pos22_5 => 22.5,
            EighthRotation::Pos45 => 45.0,
        }
    }
}

impl Default for EighthRotation {
    fn default() -> Self {
        Self::Zero
    }
}

impl fmt::Debug for EighthRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", f32::from(*self))
    }
}

/*
 dMMMMMMP dMMMMMP .dMMMb dMMMMMMP .dMMMb
   dMP   dMP     dMP" VP   dMP   dMP" VP
  dMP   dMMMP    VMMMb    dMP    VMMMb
 dMP   dMP     dP .dMP   dMP   dP .dMP
dMP   dMMMMMP  VMMMP"   dMP    VMMMP"


FIGLET: Tests
*/

#[cfg(test)]
mod tests {
    use super::*;

    use glam::Quat;

    fn quad_rotation_to_quat(rotation: QuadRotation) -> Quat {
        let x = f32::from(rotation.x).to_radians();
        let y = f32::from(rotation.y).to_radians();
        let rot_x = Quat::from_rotation_x(x);
        let rot_y = Quat::from_rotation_y(y);

        rot_y.mul_quat(rot_x)
    }

    fn assert_rotation_is_correct(point: [f32; 3], rotation: QuadRotation) {
        let rotated_point = rotation.rotate_point(point);

        let expected = quad_rotation_to_quat(rotation).mul_vec3a(Vec3A::from(point));
        let actual = Vec3A::from(rotated_point);

        assert!(
            (actual - expected).distance(Vec3A::ZERO) <= 0.0001,
            "point: {:?}, rotation: {:?}, actual: {:?}, expected: {:?}",
            point,
            rotation,
            actual,
            expected
        );
    }

    fn do_quad_rotation_test(point: [f32; 3]) {
        for x in [0, 90, 180, 270] {
            for y in [0, 90, 180, 270] {
                assert_rotation_is_correct(point, QuadRotation::new(x, y));
            }
        }
    }

    #[test]
    fn quad_rotation() {
        for x in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            for y in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                for z in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                    do_quad_rotation_test([x, y, z]);
                }
            }
        }
    }
}
