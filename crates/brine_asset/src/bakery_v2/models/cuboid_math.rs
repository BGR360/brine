use std::fmt;

use glam::{const_vec3a, Affine3A, Vec3A};
use minecraft_assets::schemas::models::{Axis, BlockFace, ElementRotation};

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
                Vec3A::new(max.x, max.y, min.z),
                Vec3A::new(max.x, max.y, max.z),
            ],
        }
    }

    #[inline(always)]
    pub fn get_face(&self, face: BlockFace) -> [Vec3A; 4] {
        // Note: Triangles are always generated with the following index permutation:
        // [0, 1, 3, 1, 2, 3]
        let verts = match face {
            BlockFace::Down => [0, 1, 2, 3],
            BlockFace::Up => [4, 5, 6, 7],
            BlockFace::North => [4, 7, 1, 0],
            BlockFace::South => [3, 2, 6, 5],
            BlockFace::West => [0, 3, 5, 4],
            BlockFace::East => [6, 2, 1, 7],
        };

        [
            self.vertices[verts[0]],
            self.vertices[verts[1]],
            self.vertices[verts[2]],
            self.vertices[verts[3]],
        ]
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
            Axis::X => Affine3A::from_rotation_x(self.angle.into()),
            Axis::Y => Affine3A::from_rotation_y(self.angle.into()),
            Axis::Z => Affine3A::from_rotation_z(self.angle.into()),
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
